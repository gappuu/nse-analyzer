use super::config;
use super::models::{ContractInfo, OptionChain, Security, SecurityType};
use anyhow::{anyhow, Context, Result};
use rand::{seq::SliceRandom, thread_rng};
use reqwest::{header, Client, StatusCode};
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Semaphore, RwLock};
use tokio_retry::strategy::ExponentialBackoff;
use tokio_retry::Retry;
use chrono::{NaiveDate, NaiveTime, Local};
use colored::Colorize;
use std::sync::atomic::{AtomicUsize, Ordering};


// Import timing utilities
use crate::utility::Timer;

// -----------------------------------------------
// CLIENT WRAPPER WITH SESSION STATE
// -----------------------------------------------
pub struct NSEClient {
    client: Client,
    warmed_up: Arc<RwLock<bool>>,
}

fn select_expiry<'a>(expiry_dates: &'a [String]) -> Result<&'a String> {
        if expiry_dates.is_empty() {
            return Err(anyhow!("No expiry dates found"));
        }

        // 1) Parse all dates and keep their original indices
        let mut parsed: Vec<(NaiveDate, usize)> = Vec::new();

        for (idx, s) in expiry_dates.iter().enumerate() {
            let d = NaiveDate::parse_from_str(s, "%d-%b-%Y")
                .with_context(|| format!("Failed to parse expiry date: {}", s))?;
            parsed.push((d, idx));
        }

        // 2) Sort by date (earliest first)
        parsed.sort_by_key(|(d, _)| *d);

        // 3) Get today's date and current time
        let now = Local::now();
        let today = now.date_naive();
        let current_time = now.time();
        let cutoff = NaiveTime::from_hms_opt(15, 30, 0).unwrap(); // 15:30

        // 4) Apply your rules while scanning sorted expiries
        for (date, idx) in parsed {
            if date < today {
                // Rule 3: past date → skip, try next
                continue;
            }

            if date == today {
                // Rule 1 & 4: today's expiry
                if current_time < cutoff {
                    // Before 15:30 → use today
                    return Ok(&expiry_dates[idx]);
                } else {
                    // After 15:30 → skip today, try next
                    continue;
                }
            }

            // Rule 2: future date (> today) → use it
            if date > today {
                return Ok(&expiry_dates[idx]);
            }
        }

        // If we reach here, all expiries were invalid (past or today after cutoff)
        Err(anyhow!("No valid expiry found (all past or after cutoff)"))
    }


impl NSEClient {
    pub fn new() -> Result<Self> {
        Ok(Self {
            client: build_client()?,
            warmed_up: Arc::new(RwLock::new(false)),
        })
    }

    /// Warmup NSE session (only once per client)
    async fn warmup_if_needed(&self) -> Result<()> {
        // Check if already warmed up
        if *self.warmed_up.read().await {
            return Ok(());
        }

        // Acquire write lock and warmup
        let mut warmed = self.warmed_up.write().await;
        if !*warmed {
            let timer = Timer::start("NSE Warmup");
            
            let response = self.client
                .get(config::NSE_BASE_URL)
                .header("Accept", config::HEADER_ACCEPT_HTML)
                .send()
                .await
                .context("Failed to warm up NSE session")?;
            
            let status = response.status();
            let elapsed = timer.elapsed_ms();
            
            println!(
                "🌐 {} | {} | {}ms",
                config::NSE_BASE_URL.bright_blue(),
                format_status(status),
                elapsed
            );
            
            tokio::time::sleep(Duration::from_millis(config::WARMUP_DELAY_MS)).await;
            *warmed = true;
        }
        
        Ok(())
    }

    
    /// Generic retry fetch with better error handling and timing
    async fn fetch_json(&self, url: &str) -> Result<String> {
        self.warmup_if_needed().await?;

        let backoff = ExponentialBackoff::from_millis(config::RETRY_BASE_DELAY_MS)
            .factor(config::RETRY_FACTOR)
            .max_delay(Duration::from_secs(config::RETRY_MAX_DELAY_SECS))
            .take(config::RETRY_MAX_ATTEMPTS);

        // Use Arc<AtomicUsize> for thread-safe attempt counting
        let attempt = Arc::new(AtomicUsize::new(0));
        let url_owned = url.to_string(); // Clone URL for closure
        
        Retry::spawn(backoff, || {
            let attempt = Arc::clone(&attempt);
            let url = url_owned.clone();
            let client = self.client.clone();
            
            async move {
                let current_attempt = attempt.fetch_add(1, Ordering::SeqCst) + 1;
                let timer = Timer::silent(format!("Fetch attempt {}", current_attempt));
                
                let res = client
                    .get(&url)
                    .header("Referer", config::HEADER_REFERER)
                    .header("X-Requested-With", config::HEADER_X_REQUESTED_WITH)
                    .send()
                    .await
                    .context("Request send failed")?;

                let status = res.status();
                let elapsed = timer.elapsed_ms();

                // Log request with status and timing
                let retry_indicator = if current_attempt > 1 {
                    format!(" (retry {})", current_attempt).yellow().to_string()
                } else {
                    String::new()
                };

                println!(
                    "🌐 {} | {} | {}ms{}",
                    truncate_url(&url, 80).bright_blue(),
                    format_status(status),
                    elapsed,
                    retry_indicator
                );

                // Handle different status codes
                if status.is_success() {
                    let text = res.text().await.context("Failed to read body")?;

                    // Validate JSON
                    let trimmed = text.trim();
                    if !trimmed.starts_with('{') && !trimmed.starts_with('[') {
                        let preview: String = text.chars().take(200).collect();
                        anyhow::bail!("Non-JSON response: {}", preview);
                    }

                    Ok(text)
                } else if status == StatusCode::TOO_MANY_REQUESTS || status.is_server_error() {
                    // Retry on server errors and rate limits
                    anyhow::bail!("Retryable error: {}", status)
                } else {
                    // Fail fast on client errors
                    let body = res.text().await.unwrap_or_default();
                    let preview: String = body.chars().take(200).collect();
                    anyhow::bail!("Client error {}: {}", status, preview)
                }
            }
        })
        .await
    }

    
    // -----------------------------------------------
    // STEP 1: FETCH FNO LIST
    // -----------------------------------------------
    pub async fn fetch_fno_list(&self) -> Result<Vec<Security>> {
        let _timer = Timer::start("1. Fetch FNO List");
        
        let text = self.fetch_json(config::NSE_API_MASTER_QUOTE).await?;
        
        let symbols: Vec<String> = serde_json::from_str(&text)
            .context("Failed to parse FNO list")?;
        
        let mut securities: Vec<Security> = symbols
            .into_iter()
            .map(Security::equity)
            .collect();
        
        // Add indices
        for index in config::NSE_INDICES {
            securities.push(Security::index(index.to_string()));
        }
        
        Ok(securities)
    }

    // -----------------------------------------------
    // STEP 2: FETCH CONTRACT INFO
    // -----------------------------------------------
    pub async fn fetch_contract_info(&self, symbol: &str) -> Result<ContractInfo> {
        // let _timer = Timer::start(format!("2. Fetch Contract Info: {}", symbol));
        
        let url = config::nse_contract_info_url(symbol);
        let text = self.fetch_json(&url).await?;
        let info: ContractInfo = serde_json::from_str(&text)
            .context("Failed to parse contract info")?;
        
        Ok(info)
    }

    // -----------------------------------------------
    // STEP 3: FETCH OPTION CHAIN
    // -----------------------------------------------
    pub async fn fetch_option_chain(
        &self,
        security: &Security,
        expiry: &str,
    ) -> Result<OptionChain> {
        // let _timer = Timer::start(format!("3. Fetch Option Chain: {} {}", security.symbol, expiry));
        
        let typ = match security.security_type {
            SecurityType::Equity => "Equity",
            SecurityType::Indices => "Indices",
        };
        
        let url = config::nse_option_chain_url(typ, &security.symbol, expiry);
        let text = self.fetch_json(&url).await?;
        let chain: OptionChain = serde_json::from_str(&text)
            .context("Failed to parse option chain")?;
        
        Ok(chain)
    }

    // -----------------------------------------------
    // NEW API A: FETCH FUTURES DATA
    // -----------------------------------------------
    pub async fn fetch_futures_data(
        &self,
        symbol: &str,
        expiry: &str,
    ) -> Result<Value> {
        let _timer = Timer::start(format!("Fetch Futures: {} {}", symbol, expiry));
        
        let url = format!(
            "{}/api/NextApi/apiClient/GetQuoteApi?functionName=getSymbolDerivativesData&symbol={}&instrumentType=FUT&expiryDt={}",
            config::NSE_BASE_URL,
            urlencoding::encode(symbol),
            urlencoding::encode(expiry)
        );
        
        let text = self.fetch_json(&url).await?;
        let data: Value = serde_json::from_str(&text)
            .context("Failed to parse futures data")?;
        
        Ok(data)
    }

    // -----------------------------------------------
    // NEW API B: FETCH DERIVATIVES HISTORICAL DATA
    // -----------------------------------------------
    pub async fn fetch_derivatives_historical_data(
        &self,
        symbol: &str,
        security_type: &SecurityType,
        instrument_type: &str,
        year: Option<&str>,
        expiry: &str,
        strike_price: Option<&str>,
        option_type: Option<&str>,
        from_date: &str,
        to_date: &str,
    ) -> Result<Value> {
        let _timer = Timer::start(format!("Fetch Historical: {} {}", symbol, instrument_type));
        
        // Determine instrument type based on security type and instrument
        let instype = match (security_type, instrument_type) {
            (SecurityType::Equity, "OPTIONS") => "OPTSTK",
            (SecurityType::Equity, "FUTURES") => "FUTSTK", 
            (SecurityType::Indices, "OPTIONS") => "OPTIDX",
            (SecurityType::Indices, "FUTURES") => "FUTIDX",
            _ => instrument_type,
        };

        let mut url = format!(
            "{}/api/NextApi/apiClient/GetQuoteApi?functionName=getDerivativesHistoricalData&symbol={}&instrumentType={}&expiryDate={}&fromDate={}&toDate={}",
            config::NSE_BASE_URL,
            urlencoding::encode(symbol),
            urlencoding::encode(instype),
            urlencoding::encode(expiry),
            urlencoding::encode(from_date),
            urlencoding::encode(to_date)
        );

        // Add optional parameters
        if let Some(year_val) = year {
            if !year_val.is_empty() {
                url.push_str(&format!("&year={}", urlencoding::encode(year_val)));
            }
        }

        if let Some(strike) = strike_price {
            if !strike.is_empty() {
                url.push_str(&format!("&strikePrice={}", urlencoding::encode(strike)));
            }
        }

        if let Some(opt_type) = option_type {
            if !opt_type.is_empty() {
                url.push_str(&format!("&optionType={}", urlencoding::encode(opt_type)));
            }
        }

        let text = self.fetch_json(&url).await?;
        let data: Value = serde_json::from_str(&text)
            .context("Failed to parse derivatives historical data")?;
        
        Ok(data)
    }

    // -----------------------------------------------
    // BATCH FETCH WITH CONCURRENCY CONTROL
    // -----------------------------------------------
    pub async fn fetch_all_option_chains(
        self: Arc<Self>,
        securities: Vec<Security>,
        max_concurrent: usize,
    ) -> Vec<Result<(Security, OptionChain)>> {
        let _timer = Timer::start(format!(
            "Batch Fetch {} Option Chains (concurrency: {})",
            securities.len(),
            max_concurrent
        ));
        
        // Separate equities from indices
        let (equities, indices): (Vec<_>, Vec<_>) = securities
            .into_iter()
            .partition(|s| matches!(s.security_type, SecurityType::Equity));

        println!("{} Equities: {}, Indices: {}", "ℹ".blue(), equities.len(), indices.len());

        // Step 1: Fetch equity expiry once (using any equity as representative)
        let equity_expiry = if !equities.is_empty() {
            let _expiry_timer = Timer::start("Fetch Equity Expiry (shared)");
            
            // Use first equity to get standard expiry dates
            let sample_symbol = &equities[0].symbol;
            match self.fetch_contract_info(sample_symbol).await {
                Ok(contract_info) => {
                    match select_expiry(&contract_info.expiry_dates) {
                        Ok(expiry) => {
                            println!("{} Using equity expiry: {} (applies to all {} equities)", 
                                "✓".green(), expiry.yellow(), equities.len());
                            Some(expiry.clone())
                        }
                        Err(e) => {
                            println!("{} Failed to select equity expiry: {}", "✗".red(), e);
                            None
                        }
                    }
                }
                Err(e) => {
                    println!("{} Failed to fetch equity contract info: {}", "✗".red(), e);
                    None
                }
            }
        } else {
            None
        };

        // Step 2: Process equities (no contract info fetch needed)
        let equity_results = if let Some(expiry) = equity_expiry {
            let _equity_timer = Timer::start(format!("Fetch {} Equity Chains", equities.len()));
            Arc::clone(&self).fetch_option_chains_with_expiry(equities, &expiry, max_concurrent).await
        } else if !equities.is_empty() {
            println!("{} Skipping equities - no valid expiry found", "⚠".yellow());
            equities.into_iter()
                .map(|_sec| Err(anyhow!("No valid equity expiry")))
                .collect()
        } else {
            Vec::new()
        };

        // Step 3: Process indices (each needs individual contract info)
        let index_results = if !indices.is_empty() {
            let _index_timer = Timer::start(format!("Fetch {} Index Chains", indices.len()));
            Arc::clone(&self).fetch_option_chains_with_contract_info(indices, max_concurrent).await
        } else {
            Vec::new()
        };

        // Combine results
        let mut all_results = Vec::new();
        all_results.extend(equity_results);
        all_results.extend(index_results);

        all_results
    }
    /// Fetch option chains when expiry is already known (equities)
    async fn fetch_option_chains_with_expiry(
        self: Arc<Self>,
        securities: Vec<Security>,
        expiry: &str,
        max_concurrent: usize,
    ) -> Vec<Result<(Security, OptionChain)>> {
        let semaphore = Arc::new(Semaphore::new(max_concurrent));
        let expiry = expiry.to_string();
        let mut handles = vec![];

        for security in securities {
            let client = Arc::clone(&self);
            let sem = Arc::clone(&semaphore);
            let expiry = expiry.clone();

            let handle = tokio::spawn(async move {
                let _permit = sem.acquire_owned().await
                    .map_err(|e| anyhow::anyhow!("Semaphore error: {}", e))?;

                // Direct fetch - no contract info needed
                let chain = client.fetch_option_chain(&security, &expiry).await?;

                Ok((security, chain))
            });

            handles.push(handle);
        }

        let mut results = vec![];
        for handle in handles {
            match handle.await {
                Ok(res) => results.push(res),
                Err(e) => results.push(Err(anyhow::anyhow!("Task error: {}", e))),
            }
        }

        results
    }

    /// Fetch option chains with individual contract info (indices)
    async fn fetch_option_chains_with_contract_info(
        self: Arc<Self>,
        securities: Vec<Security>,
        max_concurrent: usize,
    ) -> Vec<Result<(Security, OptionChain)>> {
        let semaphore = Arc::new(Semaphore::new(max_concurrent));
        let mut handles = vec![];

        for security in securities {
            let client = Arc::clone(&self);
            let sem = Arc::clone(&semaphore);

            let handle = tokio::spawn(async move {
                let _permit = sem.acquire_owned().await
                    .map_err(|e| anyhow::anyhow!("Semaphore error: {}", e))?;

                // Fetch contract info to get expiry
                let contract_info = client.fetch_contract_info(&security.symbol).await?;
                let expiry = select_expiry(&contract_info.expiry_dates)?;
                let chain = client.fetch_option_chain(&security, expiry).await?;

                Ok((security, chain))
            });

            handles.push(handle);
        }

        let mut results = vec![];
        for handle in handles {
            match handle.await {
                Ok(res) => results.push(res),
                Err(e) => results.push(Err(anyhow::anyhow!("Task error: {}", e))),
            }
        }

        results
    }
}

// -----------------------------------------------
// HTTP CLIENT BUILDER
// -----------------------------------------------
fn build_client() -> Result<Client> {
    let mut headers = header::HeaderMap::new();
    
    let lang = config::ACCEPT_LANGUAGES.choose(&mut thread_rng()).unwrap();
    headers.insert(
        header::ACCEPT_LANGUAGE, 
        header::HeaderValue::from_str(lang)?
    );
    headers.insert(header::ACCEPT, header::HeaderValue::from_static("*/*"));

    Ok(Client::builder()
        .default_headers(headers)
        .cookie_store(true)
        .user_agent(config::USER_AGENT)
        .timeout(config::HTTP_TIMEOUT)
        .build()
        .context("Failed to build HTTP client")?)
}

// -----------------------------------------------
// HELPER FUNCTIONS FOR DISPLAY
// -----------------------------------------------

/// Format HTTP status code with color
fn format_status(status: StatusCode) -> String {
    let code = status.as_u16();
    let status_str = format!("{}", code);
    
    if status.is_success() {
        status_str.green().to_string()
    } else if status.is_client_error() {
        status_str.yellow().to_string()
    } else if status.is_server_error() {
        status_str.red().to_string()
    } else {
        status_str.to_string()
    }
}

/// Truncate URL for display
fn truncate_url(url: &str, max_len: usize) -> String {
    let trimmed: String = url.chars().skip(24).collect();

    if trimmed.len() <= max_len {
        trimmed
    } else {
        format!("{}...", &trimmed[..max_len])
    }
}
