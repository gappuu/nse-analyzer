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

// -----------------------------------------------
// CLIENT WRAPPER WITH SESSION STATE AND EXPIRY CACHE
// -----------------------------------------------
pub struct NSEClient {
    client: Client,
    warmed_up: Arc<RwLock<bool>>,
    cached_equity_expiry: Arc<RwLock<Option<String>>>, // NEW: Cache for equity expiry
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
            cached_equity_expiry: Arc::new(RwLock::new(None)), // NEW: Initialize cache
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
            let _ = self.client
                .get(config::NSE_BASE_URL)
                .header("Accept", config::HEADER_ACCEPT_HTML)
                .send()
                .await
                .context("Failed to warm up NSE session")?;
            
            tokio::time::sleep(Duration::from_millis(config::WARMUP_DELAY_MS)).await;
            *warmed = true;
        }
        
        Ok(())
    }

    /// Generic retry fetch with better error handling
    async fn fetch_json(&self, url: &str) -> Result<String> {
        self.warmup_if_needed().await?;

        let backoff = ExponentialBackoff::from_millis(config::RETRY_BASE_DELAY_MS)
            .factor(config::RETRY_FACTOR)
            .max_delay(Duration::from_secs(config::RETRY_MAX_DELAY_SECS))
            .take(config::RETRY_MAX_ATTEMPTS);

        Retry::spawn(backoff, || async {
            let res = self.client
                .get(url)
                .header("Referer", config::HEADER_REFERER)
                .header("X-Requested-With", config::HEADER_X_REQUESTED_WITH)
                .send()
                .await
                .context("Request send failed")?;

            let status = res.status();
            
            // ============================================
            // HTTP Response Logging
            // ============================================
            if config::is_ci_environment() {
                println!("\n{}", "=".repeat(80));
                // println!("{} HTTP Response Log", "🌐".to_string());
                // println!("{}", "=".repeat(80));
                println!("{} URL: {}", "→", url);
                println!("{} Status: {} {}", "→", status.as_u16(), status.canonical_reason().unwrap_or("Unknown"));
                
                // Log response headers
                // println!("\n{} Response Headers:", "📋");
                // for (name, value) in res.headers() {
                //     if let Ok(val_str) = value.to_str() {
                //         println!("  {}: {}", name, val_str);
                //     }
                // }
                // println!("{}", "=".repeat(80));
            }

            // Handle different status codes
            if status.is_success() {
                let text = res.text().await.context("Failed to read body")?;

                // ============================================
                // NEW: Log response body preview in CI
                // ============================================
                // if config::is_ci_environment() {
                //     let preview: String = text.chars().take(500).collect();
                //     println!("\n{} Response Body Preview (first 500 chars):", "📄");
                //     println!("{}", preview);
                //     println!("\n{} Response Length: {} bytes", "📊", text.len());
                //     println!("{}", "=".repeat(80));
                //     println!();
                // }
                // ============================================

                // Validate JSON
                let trimmed = text.trim();
                if !trimmed.starts_with('{') && !trimmed.starts_with('[') {
                    let preview: String = text.chars().take(200).collect();
                    
                    if config::is_ci_environment() {
                        println!("{} Non-JSON response detected!", "❌");
                        println!("{} Full response body:", "⚠️");
                        println!("{}", text);
                        println!("{}", "=".repeat(80));
                    }
                    
                    anyhow::bail!("Non-JSON response: {}", preview);
                }

                Ok(text)
            } else if status == StatusCode::TOO_MANY_REQUESTS || status.is_server_error() {
                let body = res.text().await.unwrap_or_default();
                
                if config::is_ci_environment() {
                    println!("\n{} Error Response Body:", "❌");
                    println!("{}", body);
                    println!("{}", "=".repeat(80));
                    println!();
                }
                
                anyhow::bail!("Retryable error: {}", status)
            } else {
                let body = res.text().await.unwrap_or_default();
                let preview: String = body.chars().take(200).collect();
                
                if config::is_ci_environment() {
                    println!("\n{} Client Error Response:", "❌");
                    println!("{} Status: {}", "→", status);
                    println!("{} Full Body:", "→");
                    println!("{}", body);
                    println!("{}", "=".repeat(80));
                    println!();
                }
                
                anyhow::bail!("Client error {}: {}", status, preview)
            }
        })
        .await
    }

    // -----------------------------------------------
    // STEP 1: FETCH FNO LIST
    // -----------------------------------------------
    pub async fn fetch_fno_list(&self) -> Result<Vec<Security>> {
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
        let url = config::nse_contract_info_url(symbol);
        let text = self.fetch_json(&url).await?;
        let info: ContractInfo = serde_json::from_str(&text)
            .context("Failed to parse contract info")?;
        
        Ok(info)
    }

    // -----------------------------------------------
    // NEW: FETCH CONTRACT INFO WITH CACHING FOR EQUITIES
    // -----------------------------------------------
    async fn fetch_contract_info_with_cache(&self, security: &Security) -> Result<String> {
        match security.security_type {
            SecurityType::Indices => {
                // Indices always fetch fresh contract info (different expiry schedules)
                if config::is_ci_environment() {
                    println!("{} {} (Index): Fetching fresh contract info", "🔍".to_string(), security.symbol);
                }
                let contract_info = self.fetch_contract_info(&security.symbol).await?;
                let expiry = select_expiry(&contract_info.expiry_dates)?;
                Ok(expiry.clone())
            }
            SecurityType::Equity => {
                // Check if we have cached equity expiry
                let cached = self.cached_equity_expiry.read().await;
                
                if let Some(expiry) = cached.as_ref() {
                    // Use cached expiry (saves 1 API call per equity!)
                    if config::is_ci_environment() {
                        println!("{} {} (Equity): Using cached expiry: {}", "⚡".to_string(), security.symbol, expiry);
                    }
                    Ok(expiry.clone())
                } else {
                    // First equity - fetch and cache
                    drop(cached); // Release read lock
                    
                    if config::is_ci_environment() {
                        println!("{} {} (First Equity): Fetching and caching expiry", "🔍".to_string(), security.symbol);
                    }
                    
                    let contract_info = self.fetch_contract_info(&security.symbol).await?;
                    let expiry = select_expiry(&contract_info.expiry_dates)?;
                    
                    // Cache the expiry for future equities
                    let mut cache = self.cached_equity_expiry.write().await;
                    *cache = Some(expiry.clone());
                    
                    if config::is_ci_environment() {
                        println!("{} Cached equity expiry: {} (will be reused for all equities)", "✓".to_string(), expiry);
                    }
                    
                    Ok(expiry.clone())
                }
            }
        }
    }

    // -----------------------------------------------
    // STEP 3: FETCH OPTION CHAIN
    // -----------------------------------------------
    pub async fn fetch_option_chain(
        &self,
        security: &Security,
        expiry: &str,
    ) -> Result<OptionChain> {
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
    // API A: FETCH FUTURES DATA
    // -----------------------------------------------
    pub async fn fetch_futures_data(
        &self,
        symbol: &str,
        expiry: &str,
    ) -> Result<Value> {
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
    // API B: FETCH DERIVATIVES HISTORICAL DATA
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
    // OPTIMIZED BATCH FETCH WITH EXPIRY CACHING
    // -----------------------------------------------
    pub async fn fetch_all_option_chains(
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

                // NEW: Use cached expiry for equities, fresh fetch for indices
                let expiry = client.fetch_contract_info_with_cache(&security).await?;

                // Get option chain
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