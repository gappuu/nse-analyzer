use crate::config;
use crate::models::*;
use anyhow::{anyhow, Context, Result};
use chrono::{Local, NaiveDate, NaiveTime};
use rand::{seq::SliceRandom, thread_rng};
use regex::Regex;
use reqwest::{header, Client, StatusCode};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Semaphore, RwLock};
use tokio_retry::strategy::ExponentialBackoff;
use tokio_retry::Retry;

// -----------------------------------------------
// MCX CLIENT WRAPPER WITH SESSION STATE
// -----------------------------------------------
pub struct McxClient {
    client: Client,
    warmed_up: Arc<RwLock<bool>>,
}

impl McxClient {
    pub fn new() -> Result<Self> {
        Ok(Self {
            client: build_mcx_client()?,
            warmed_up: Arc<RwLock::new(false)),
        })
    }

    /// Warmup MCX session (only once per client)
    async fn warmup_if_needed(&self) -> Result<()> {
        // Check if already warmed up
        if *self.warmed_up.read().await {
            return Ok(());
        }

        // Acquire write lock and warmup
        let mut warmed = self.warmed_up.write().await;
        if !*warmed {
            // First, visit the main page to establish session
            let _ = self.client
                .get("https://www.mcxindia.com")
                .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7")
                .header("Accept-Encoding", "gzip, deflate, br")
                .header("Accept-Language", "en-US,en;q=0.9")
                .header("Cache-Control", "no-cache")
                .header("Pragma", "no-cache")
                .header("Sec-Ch-Ua", "\"Google Chrome\";v=\"131\", \"Chromium\";v=\"131\", \"Not_A Brand\";v=\"24\"")
                .header("Sec-Ch-Ua-Mobile", "?0")
                .header("Sec-Ch-Ua-Platform", "\"Windows\"")
                .header("Sec-Fetch-Dest", "document")
                .header("Sec-Fetch-Mode", "navigate")
                .header("Sec-Fetch-Site", "none")
                .header("Sec-Fetch-User", "?1")
                .header("Upgrade-Insecure-Requests", "1")
                .send()
                .await
                .context("Failed to warm up MCX main page")?;
            
            tokio::time::sleep(Duration::from_millis(1000)).await;
            
            // Then visit the option chain page to get cookies/session
            let _ = self.client
                .get("https://www.mcxindia.com/market-data/option-chain")
                .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7")
                .header("Accept-Encoding", "gzip, deflate, br")
                .header("Accept-Language", "en-US,en;q=0.9")
                .header("Cache-Control", "no-cache")
                .header("Pragma", "no-cache")
                .header("Referer", "https://www.mcxindia.com/")
                .header("Sec-Ch-Ua", "\"Google Chrome\";v=\"131\", \"Chromium\";v=\"131\", \"Not_A Brand\";v=\"24\"")
                .header("Sec-Ch-Ua-Mobile", "?0")
                .header("Sec-Ch-Ua-Platform", "\"Windows\"")
                .header("Sec-Fetch-Dest", "document")
                .header("Sec-Fetch-Mode", "navigate")
                .header("Sec-Fetch-Site", "same-origin")
                .header("Sec-Fetch-User", "?1")
                .header("Upgrade-Insecure-Requests", "1")
                .send()
                .await
                .context("Failed to warm up MCX option chain page")?;
            
            tokio::time::sleep(Duration::from_millis(config::WARMUP_DELAY_MS)).await;
            *warmed = true;
        }
        
        Ok(())
    }

    /// Generic retry fetch with better error handling for MCX
    async fn fetch_text(&self, url: &str) -> Result<String> {
        self.warmup_if_needed().await?;

        let backoff = ExponentialBackoff::from_millis(config::RETRY_BASE_DELAY_MS)
            .factor(config::RETRY_FACTOR)
            .max_delay(Duration::from_secs(config::RETRY_MAX_DELAY_SECS))
            .take(config::RETRY_MAX_ATTEMPTS);

        Retry::spawn(backoff, || async {
            let res = self.client
                .get(url)
                .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7")
                .header("Accept-Encoding", "gzip, deflate, br")
                .header("Accept-Language", "en-US,en;q=0.9")
                .header("Cache-Control", "no-cache")
                .header("Pragma", "no-cache")
                .header("Referer", "https://www.mcxindia.com/")
                .header("Sec-Ch-Ua", "\"Google Chrome\";v=\"131\", \"Chromium\";v=\"131\", \"Not_A Brand\";v=\"24\"")
                .header("Sec-Ch-Ua-Mobile", "?0")
                .header("Sec-Ch-Ua-Platform", "\"Windows\"")
                .header("Sec-Fetch-Dest", "document")
                .header("Sec-Fetch-Mode", "navigate")
                .header("Sec-Fetch-Site", "same-origin")
                .header("Upgrade-Insecure-Requests", "1")
                .send()
                .await
                .context("Request send failed")?;

            let status = res.status();

            if status.is_success() {
                let text = res.text().await.context("Failed to read body")?;
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
        })
        .await
    }

    /// Post JSON data to MCX API
    async fn post_json(&self, url: &str, payload: &Value) -> Result<String> {
        self.warmup_if_needed().await?;

        let backoff = ExponentialBackoff::from_millis(config::RETRY_BASE_DELAY_MS)
            .factor(config::RETRY_FACTOR)
            .max_delay(Duration::from_secs(config::RETRY_MAX_DELAY_SECS))
            .take(config::RETRY_MAX_ATTEMPTS);

        Retry::spawn(backoff, || async {
            let res = self.client
                .post(url)
                .header("Accept", "application/json, text/javascript, */*; q=0.01")
                .header("Accept-Encoding", "gzip, deflate, br")
                .header("Accept-Language", "en-US,en;q=0.9")
                .header("Content-Type", "application/json; charset=utf-8")
                .header("Cache-Control", "no-cache")
                .header("Pragma", "no-cache")
                .header("Referer", "https://www.mcxindia.com/market-data/option-chain")
                .header("Sec-Ch-Ua", "\"Google Chrome\";v=\"131\", \"Chromium\";v=\"131\", \"Not_A Brand\";v=\"24\"")
                .header("Sec-Ch-Ua-Mobile", "?0")
                .header("Sec-Ch-Ua-Platform", "\"Windows\"")
                .header("Sec-Fetch-Dest", "empty")
                .header("Sec-Fetch-Mode", "cors")
                .header("Sec-Fetch-Site", "same-origin")
                .header("X-Requested-With", "XMLHttpRequest")
                .json(payload)
                .send()
                .await
                .context("Request send failed")?;

            let status = res.status();

            if status.is_success() {
                let text = res.text().await.context("Failed to read body")?;
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
        })
        .await
    }

    // -----------------------------------------------
    // STEP 1: FETCH MCX SYMBOLS AND CONTRACT INFO
    // -----------------------------------------------
    pub async fn fetch_mcx_symbols(&self) -> Result<Vec<McxSymbolData>> {
        let html = self.fetch_text("https://www.mcxindia.com/market-data/option-chain").await?;
        
        // Extract vTick data from script tag using regex
        let re = Regex::new(r"var vTick=(\[.*?\]);")
            .context("Failed to compile regex")?;
        
        let captures = re.captures(&html)
            .ok_or_else(|| anyhow!("vTick data not found in HTML"))?;
        
        let json_str = captures.get(1)
            .ok_or_else(|| anyhow!("Failed to extract vTick JSON"))?
            .as_str();
        
        let symbol_data: Vec<McxSymbolData> = serde_json::from_str(json_str)
            .context("Failed to parse vTick JSON data")?;
        
        Ok(symbol_data)
    }

    /// Get contract info for all symbols (group by symbol with expiry dates)
    pub async fn fetch_mcx_contract_list(&self) -> Result<Vec<McxContractInfo>> {
        let symbol_data = self.fetch_mcx_symbols().await?;
        
        // Group by symbol to collect all expiry dates
        let mut symbol_map: HashMap<String, McxContractInfo> = HashMap::new();
        
        for data in symbol_data {
            let entry = symbol_map
                .entry(data.symbol.clone())
                .or_insert_with(|| McxContractInfo {
                    symbol: data.symbol.clone(),
                    expiry_dates: Vec::new(),
                    instrument_name: data.instrument_name.clone(),
                    symbol_value: data.symbol_value.clone(),
                });
            
            if !entry.expiry_dates.contains(&data.expiry_date) {
                entry.expiry_dates.push(data.expiry_date);
            }
        }
        
        // Sort expiry dates for each symbol
        for contract in symbol_map.values_mut() {
            contract.expiry_dates.sort_by(|a, b| {
                let date_a = NaiveDate::parse_from_str(a, "%d%b%Y");
                let date_b = NaiveDate::parse_from_str(b, "%d%b%Y");
                match (date_a, date_b) {
                    (Ok(a), Ok(b)) => a.cmp(&b),
                    _ => a.cmp(b), // Fallback to string comparison
                }
            });
        }
        
        Ok(symbol_map.into_values().collect())
    }

    /// Get contract info for a specific symbol
    pub async fn fetch_mcx_contract_info(&self, symbol: &str) -> Result<McxContractInfo> {
        let symbol_data = self.fetch_mcx_symbols().await?;
        
        let mut expiry_dates = Vec::new();
        let mut instrument_name = String::new();
        let mut symbol_value = String::new();
        
        for data in symbol_data {
            if data.symbol.eq_ignore_ascii_case(symbol) {
                if !expiry_dates.contains(&data.expiry_date) {
                    expiry_dates.push(data.expiry_date);
                }
                if instrument_name.is_empty() {
                    instrument_name = data.instrument_name;
                    symbol_value = data.symbol_value;
                }
            }
        }
        
        if expiry_dates.is_empty() {
            return Err(anyhow!("Symbol '{}' not found", symbol));
        }
        
        // Sort expiry dates
        expiry_dates.sort_by(|a, b| {
            let date_a = NaiveDate::parse_from_str(a, "%d%b%Y");
            let date_b = NaiveDate::parse_from_str(b, "%d%b%Y");
            match (date_a, date_b) {
                (Ok(a), Ok(b)) => a.cmp(&b),
                _ => a.cmp(b), // Fallback to string comparison
            }
        });
        
        Ok(McxContractInfo {
            symbol: symbol.to_string(),
            expiry_dates,
            instrument_name,
            symbol_value,
        })
    }

    // -----------------------------------------------
    // STEP 2: FETCH OPTION CHAIN DATA (RAW)
    // -----------------------------------------------
    pub async fn fetch_mcx_option_chain_raw(
        &self,
        symbol: &str,
        expiry: &str,
    ) -> Result<McxOptionChainResponse> {
        let payload = serde_json::json!({
            "Commodity": symbol,
            "Expiry": expiry
        });
        
        let text = self.post_json(
            "https://www.mcxindia.com/backpage.aspx/GetOptionChain",
            &payload
        ).await?;
        
        let response: McxOptionChainResponse = serde_json::from_str(&text)
            .context("Failed to parse MCX option chain response")?;
        
        Ok(response)
    }

    // -----------------------------------------------
    // HELPER FUNCTION: SELECT APPROPRIATE EXPIRY
    // -----------------------------------------------
    fn select_expiry<'a>(expiry_dates: &'a [String]) -> Result<&'a String> {
        if expiry_dates.is_empty() {
            return Err(anyhow!("No expiry dates found"));
        }

        // Parse all dates and keep their original indices
        let mut parsed: Vec<(NaiveDate, usize)> = Vec::new();

        for (idx, s) in expiry_dates.iter().enumerate() {
            let d = NaiveDate::parse_from_str(s, "%d%b%Y")
                .with_context(|| format!("Failed to parse expiry date: {}", s))?;
            parsed.push((d, idx));
        }

        // Sort by date (earliest first)
        parsed.sort_by_key(|(d, _)| *d);

        // Get today's date and current time
        let now = Local::now();
        let today = now.date_naive();
        let current_time = now.time();
        let cutoff = NaiveTime::from_hms_opt(17, 0, 0).unwrap(); // 17:00 for MCX

        // Apply expiry selection rules
        for (date, idx) in parsed {
            if date < today {
                // Past date → skip, try next
                continue;
            }

            if date == today {
                // Today's expiry
                if current_time < cutoff {
                    // Before 17:00 → use today
                    return Ok(&expiry_dates[idx]);
                } else {
                    // After 17:00 → skip today, try next
                    continue;
                }
            }

            // Future date (> today) → use it
            if date > today {
                return Ok(&expiry_dates[idx]);
            }
        }

        // If we reach here, all expiries were invalid
        Err(anyhow!("No valid expiry found (all past or after cutoff)"))
    }

    // -----------------------------------------------
    // BATCH FETCH WITH CONCURRENCY CONTROL
    // -----------------------------------------------
    pub async fn fetch_all_mcx_option_chains(
        self: Arc<Self>,
        symbols: Vec<McxContractInfo>,
        max_concurrent: usize,
    ) -> Vec<Result<(McxContractInfo, McxOptionChainResponse)>> {
        let semaphore = Arc::new(Semaphore::new(max_concurrent));
        let mut handles = vec![];

        for contract in symbols {
            let client = Arc::clone(&self);
            let sem = Arc::clone(&semaphore);

            let handle = tokio::spawn(async move {
                // Acquire permit
                let _permit = sem.acquire_owned().await
                    .map_err(|e| anyhow::anyhow!("Semaphore error: {}", e))?;

                // Select appropriate expiry
                let expiry = Self::select_expiry(&contract.expiry_dates)?;

                // Get option chain (raw)
                let chain = client.fetch_mcx_option_chain_raw(&contract.symbol, expiry).await?;

                Ok((contract, chain))
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
// HTTP CLIENT BUILDER FOR MCX
// -----------------------------------------------
fn build_mcx_client() -> Result<Client> {
    let mut headers = header::HeaderMap::new();
    
    // More comprehensive browser headers to avoid detection
    headers.insert(
        header::ACCEPT, 
        header::HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7")
    );
    headers.insert(
        header::ACCEPT_LANGUAGE, 
        header::HeaderValue::from_static("en-US,en;q=0.9")
    );
    headers.insert(
        header::ACCEPT_ENCODING, 
        header::HeaderValue::from_static("gzip, deflate, br")
    );
    headers.insert(
        "sec-ch-ua", 
        header::HeaderValue::from_static("\"Google Chrome\";v=\"131\", \"Chromium\";v=\"131\", \"Not_A Brand\";v=\"24\"")
    );
    headers.insert(
        "sec-ch-ua-mobile", 
        header::HeaderValue::from_static("?0")
    );
    headers.insert(
        "sec-ch-ua-platform", 
        header::HeaderValue::from_static("\"Windows\"")
    );
    headers.insert(
        "sec-fetch-dest", 
        header::HeaderValue::from_static("document")
    );
    headers.insert(
        "sec-fetch-mode", 
        header::HeaderValue::from_static("navigate")
    );
    headers.insert(
        "sec-fetch-site", 
        header::HeaderValue::from_static("none")
    );
    headers.insert(
        "sec-fetch-user", 
        header::HeaderValue::from_static("?1")
    );
    headers.insert(
        "upgrade-insecure-requests", 
        header::HeaderValue::from_static("1")
    );

    Ok(Client::builder()
        .default_headers(headers)
        .cookie_store(true) // Critical for MCX session
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36")
        .timeout(Duration::from_secs(30))
        .redirect(reqwest::redirect::Policy::limited(10))
        .gzip(true)
        .build()
        .context("Failed to build MCX HTTP client")?)
}