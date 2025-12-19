use super::config;
use super::models::{Ticker, OptionChainResponse};
use anyhow::{anyhow, Result};
use chrono::{Datelike, NaiveDate, Utc, Weekday};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tokio_retry::strategy::ExponentialBackoff;
use tokio_retry::Retry;

// -----------------------------------------------
// BHAVCOPY API STRUCTURES
// -----------------------------------------------

#[derive(Debug, Clone, Deserialize)]
struct BhavCopyResponse {
    d: BhavCopyData,
}

#[derive(Debug, Clone, Deserialize)]
struct BhavCopyData {
    #[serde(rename = "Summary")]
    summary: BhavCopySummary,
    #[serde(rename = "Data")]
    data: Vec<BhavCopyEntry>,
}

#[derive(Debug, Clone, Deserialize)]
struct BhavCopySummary {
    #[serde(rename = "AsOn")]
    as_on: String,
    #[serde(rename = "Count")]
    count: i32,
    // #[serde(rename = "Status")]
    // status: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct BhavCopyEntry {
    // #[serde(rename = "Date")]
    // date: String,
    #[serde(rename = "Symbol")]
    symbol: String,
    #[serde(rename = "ExpiryDate")]
    expiry_date: String,
    // #[serde(rename = "Open")]
    // open: f64,
    // #[serde(rename = "High")]
    // high: f64,
    // #[serde(rename = "Low")]
    // low: f64,
    // #[serde(rename = "Close")]
    // close: f64,
    // #[serde(rename = "PreviousClose")]
    // previous_close: f64,
    // #[serde(rename = "Volume")]
    // volume: i64,
    // #[serde(rename = "VolumeInThousands")]
    // volume_in_thousands: String,
    // #[serde(rename = "Value")]
    // value: f64,
    // #[serde(rename = "OpenInterest")]
    // open_interest: i64,
    // #[serde(rename = "DateDisplay")]
    // date_display: String,
    #[serde(rename = "InstrumentName")]
    instrument_name: String,
    // #[serde(rename = "StrikePrice")]
    // strike_price: f64,
    // #[serde(rename = "OptionType")]
    // option_type: String,
}

#[derive(Debug, Clone, Serialize)]
struct BhavCopyPayload {
    #[serde(rename = "Date")]
    date: String,
    #[serde(rename = "InstrumentName")]
    instrument_name: String,
}

// -----------------------------------------------
// MCX CLIENT USING OFFICIAL API
// -----------------------------------------------
pub struct MCXClient {
    client: Client,
}

impl MCXClient {
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .cookie_store(true)
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36")
            .timeout(Duration::from_secs(30))
            .redirect(reqwest::redirect::Policy::limited(10))
            .gzip(true)
            .build()?;
        
        Ok(Self { client })
    }

    /// Get the most recent weekday date for fetching data
    fn get_data_date() -> String {
        let today = Utc::now().date_naive();
        let mut check_date = today - chrono::Duration::days(1);
        
        // Find the most recent weekday
        while check_date.weekday() == Weekday::Sat || check_date.weekday() == Weekday::Sun {
            check_date = check_date - chrono::Duration::days(1);
        }
        
        check_date.format("%Y%m%d").to_string()
    }

    /// Parse epoch timestamp from MCX format to NaiveDate
    fn parse_epoch_to_date(epoch_str: &str) -> Option<NaiveDate> {
        // Extract epoch from format "/Date(1766128567354)/"
        if let Some(start) = epoch_str.find('(') {
            if let Some(end) = epoch_str.find(')') {
                if let Ok(epoch_ms) = epoch_str[start + 1..end].parse::<i64>() {
                    let epoch_secs = epoch_ms / 1000;
                    if let Some(datetime) = chrono::DateTime::from_timestamp(epoch_secs, 0) {
                        return Some(datetime.date_naive());
                    }
                }
            }
        }
        None
    }

    /// Get previous weekday from given date
    fn get_previous_weekday(date: NaiveDate) -> NaiveDate {
        let mut prev_date = date - chrono::Duration::days(1);
        while prev_date.weekday() == Weekday::Sat || prev_date.weekday() == Weekday::Sun {
            prev_date = prev_date - chrono::Duration::days(1);
        }
        prev_date
    }

    /// Fetch bhav copy data with automatic date fallback
    async fn fetch_bhav_copy_with_fallback(&self) -> Result<BhavCopyResponse> {
        let mut data_date = Self::get_data_date();
        let max_attempts = 5; // Don't go back more than 5 days
        
        for attempt in 0..max_attempts {
            println!("🔍 Trying to fetch MCX data for date: {}", data_date);
            
            let payload = BhavCopyPayload {
                date: data_date.clone(),
                instrument_name: "OPTFUT".to_string(),
            };

            let backoff = ExponentialBackoff::from_millis(config::RETRY_BASE_DELAY_MS)
                .factor(config::RETRY_FACTOR)
                .max_delay(Duration::from_secs(config::RETRY_MAX_DELAY_SECS))
                .take(config::RETRY_MAX_ATTEMPTS);

            let result = Retry::spawn(backoff, || async {
                let res = self.client
                    .post("https://www.mcxindia.com/backpage.aspx/GetDateWiseBhavCopy")
                    .header("Accept", "application/json, text/javascript, */*; q=0.01")
                    .header("Accept-Encoding", "gzip, deflate, br")
                    .header("Accept-Language", "en-US,en;q=0.9")
                    .header("Content-Type", "application/json; charset=utf-8")
                    .header("Cache-Control", "no-cache")
                    .header("Pragma", "no-cache")
                    .header("Referer", "https://www.mcxindia.com/market-data/bhavcopy")
                    .header("Sec-Ch-Ua", "\"Google Chrome\";v=\"131\", \"Chromium\";v=\"131\", \"Not_A Brand\";v=\"24\"")
                    .header("Sec-Ch-Ua-Mobile", "?0")
                    .header("Sec-Ch-Ua-Platform", "\"Windows\"")
                    .header("Sec-Fetch-Dest", "empty")
                    .header("Sec-Fetch-Mode", "cors")
                    .header("Sec-Fetch-Site", "same-origin")
                    .header("X-Requested-With", "XMLHttpRequest")
                    .json(&payload)
                    .send()
                    .await?;

                let status = res.status();
                if status.is_success() {
                    let text = res.text().await?;
                    if text.trim().is_empty() {
                        anyhow::bail!("Empty response from MCX BhavCopy API");
                    }
                    
                    let response: BhavCopyResponse = serde_json::from_str(&text)
                        .map_err(|e| anyhow::anyhow!("Failed to parse BhavCopy response: {}", e))?;
                    
                    Ok(response)
                } else if status == StatusCode::TOO_MANY_REQUESTS || status.is_server_error() {
                    anyhow::bail!("Retryable error: {}", status)
                } else {
                    let body = res.text().await.unwrap_or_default();
                    let preview: String = body.chars().take(200).collect();
                    anyhow::bail!("Client error {}: {}", status, preview)
                }
            }).await;

            match result {
                Ok(response) => {
                    if response.d.summary.count > 0 {
                        println!("✅ Found {} OPTFUT entries for date {}", response.d.summary.count, data_date);
                        return Ok(response);
                    } else {
                        println!("⚠️  No data found for date {} (count: 0), trying previous date...", data_date);
                        
                        // Use AsOn date if available, otherwise calculate previous weekday
                        if let Some(as_on_date) = Self::parse_epoch_to_date(&response.d.summary.as_on) {
                            let prev_date = Self::get_previous_weekday(as_on_date);
                            data_date = prev_date.format("%Y%m%d").to_string();
                        } else {
                            // Fallback to manual calculation
                            if let Ok(current_date) = NaiveDate::parse_from_str(&data_date, "%Y%m%d") {
                                let prev_date = Self::get_previous_weekday(current_date);
                                data_date = prev_date.format("%Y%m%d").to_string();
                            } else {
                                return Err(anyhow!("Failed to parse date format: {}", data_date));
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("❌ Failed to fetch data for date {}: {}", data_date, e);
                    if attempt == max_attempts - 1 {
                        return Err(anyhow!("Failed to fetch MCX data after {} attempts", max_attempts));
                    }
                    
                    // Try previous weekday
                    if let Ok(current_date) = NaiveDate::parse_from_str(&data_date, "%Y%m%d") {
                        let prev_date = Self::get_previous_weekday(current_date);
                        data_date = prev_date.format("%Y%m%d").to_string();
                    } else {
                        return Err(anyhow!("Failed to parse date format: {}", data_date));
                    }
                }
            }
        }
        
        Err(anyhow!("Exhausted all attempts to fetch MCX data"))
    }

    /// Fetch unique symbol-expiry combinations from bhav copy
    pub async fn fetch_ticker_list(&self) -> Result<Vec<Ticker>> {
        let bhav_copy = self.fetch_bhav_copy_with_fallback().await?;
        
        // Calculate total entries before moving the data
        let total_entries = bhav_copy.d.data.len();
        
        // Use HashSet to ensure uniqueness of Symbol-ExpiryDate combinations
        let mut unique_entries = HashSet::new();
        let mut tickers = Vec::new();
        
        for entry in bhav_copy.d.data {
            let symbol = entry.symbol.trim().to_string();
            let expiry = entry.expiry_date.trim().to_string();
            let key = (symbol.clone(), expiry.clone());
            
            if unique_entries.insert(key) {
                tickers.push(Ticker {
                    expiry_date: expiry,
                    instrument_name: entry.instrument_name,
                    symbol,
                    symbol_value: entry.symbol.trim().to_string(),
                    todays_traded: 1, // Indicates this is from live data
                });
            }
        }
        
        // Sort by symbol, then by expiry date
        tickers.sort_by(|a, b| {
            match a.symbol.cmp(&b.symbol) {
                std::cmp::Ordering::Equal => a.expiry_date.cmp(&b.expiry_date),
                other => other,
            }
        });
        
        println!("📊 Extracted {} unique symbol-expiry combinations from {} total entries", 
                 tickers.len(), total_entries);
        
        Ok(tickers)
    }

    /// Fetch option chain data (unchanged from previous implementation)
    pub async fn fetch_option_chain(
        &self,
        commodity: &str,
        expiry: &str,
    ) -> Result<OptionChainResponse> {
        let payload = serde_json::json!({
            "Commodity": commodity,
            "Expiry": expiry
        });
        
        let backoff = ExponentialBackoff::from_millis(config::RETRY_BASE_DELAY_MS)
            .factor(config::RETRY_FACTOR)
            .max_delay(Duration::from_secs(config::RETRY_MAX_DELAY_SECS))
            .take(config::RETRY_MAX_ATTEMPTS);

        let result = Retry::spawn(backoff, || async {
            let res = self.client
                .post("https://www.mcxindia.com/backpage.aspx/GetOptionChain")
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
                .json(&payload)
                .send()
                .await?;

            let status = res.status();
            if status.is_success() {
                let text = res.text().await?;
                if text.trim().is_empty() || text.contains("error") {
                    anyhow::bail!("Empty or error response from MCX option chain API");
                }
                
                let response: super::models::McxOptionChainResponse = serde_json::from_str(&text)?;
                
                // Convert to legacy format
                let legacy_response = OptionChainResponse {
                    d: super::models::OptionChainData {
                        type_name: response.d.type_name,
                        extension_data: response.d.extension_data,
                        data: response.d.data.into_iter().map(|d| super::models::OptionData {
                            extension_data: d.extension_data,
                            ce_absolute_change: d.ce_absolute_change,
                            ce_ask_price: d.ce_ask_price,
                            ce_ask_qty: d.ce_ask_qty,
                            ce_bid_price: d.ce_bid_price,
                            ce_bid_qty: d.ce_bid_qty,
                            ce_change_in_oi: d.ce_change_in_oi,
                            ce_ltp: d.ce_ltp,
                            ce_ltt: d.ce_ltt,
                            ce_net_change: d.ce_net_change,
                            ce_open_interest: d.ce_open_interest,
                            ce_strike_price: d.ce_strike_price,
                            ce_volume: d.ce_volume,
                            pe_absolute_change: d.pe_absolute_change,
                            pe_ask_price: d.pe_ask_price,
                            pe_ask_qty: d.pe_ask_qty,
                            pe_bid_price: d.pe_bid_price,
                            pe_bid_qty: d.pe_bid_qty,
                            pe_change_in_oi: d.pe_change_in_oi,
                            pe_ltp: d.pe_ltp,
                            pe_ltt: d.pe_ltt,
                            pe_net_change: d.pe_net_change,
                            pe_open_interest: d.pe_open_interest,
                            pe_volume: d.pe_volume,
                            expiry_date: d.expiry_date,
                            ltt: d.ltt,
                            symbol: d.symbol,
                            underlying_value: d.underlying_value,
                        }).collect(),
                        summary: super::models::OptionSummary {
                            extension_data: response.d.summary.extension_data,
                            as_on: response.d.summary.as_on,
                            count: response.d.summary.count,
                            status: response.d.summary.status,
                        },
                    },
                };
                
                Ok(legacy_response)
            } else {
                anyhow::bail!("HTTP error: {}", status)
            }
        }).await;
        
        match result {
            Ok(response) => {
                println!("✅ Successfully fetched option chain for {}", commodity);
                Ok(response)
            }
            Err(e) => {
                println!("❌ Failed to fetch option chain for {}: {} - {}", commodity,expiry, e);
                Err(e)
            }
        }
    }

    /// Batch fetch all option chains
    pub async fn fetch_all_option_chains(
        self: Arc<Self>,
        tickers: Vec<Ticker>,
        max_concurrent: usize,
    ) -> Vec<Result<(Ticker, OptionChainResponse)>> {
        println!("📈 Batch fetching {} option chains with {} max concurrent", 
                 tickers.len(), max_concurrent);
        
        let semaphore = Arc::new(Semaphore::new(max_concurrent));
        let mut handles = vec![];

        for ticker in tickers {
            let client = Arc::clone(&self);
            let sem = Arc::clone(&semaphore);

            let handle = tokio::spawn(async move {
                let _permit = sem.acquire_owned().await
                    .map_err(|e| anyhow::anyhow!("Semaphore error: {}", e))?;

                let chain = client.fetch_option_chain(&ticker.symbol, &ticker.expiry_date).await?;
                Ok((ticker, chain))
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

    /// Utility methods
    pub fn get_unique_symbols(tickers: &[Ticker]) -> Vec<String> {
        let mut symbols: Vec<String> = tickers
            .iter()
            .map(|t| t.symbol.clone())
            .collect();
        
        symbols.sort();
        symbols.dedup();
        symbols
    }

    pub fn get_expiries_for_symbol(tickers: &[Ticker], symbol: &str) -> Vec<String> {
        tickers
            .iter()
            .filter(|t| t.symbol == symbol)
            .map(|t| t.expiry_date.clone())
            .collect()
    }
}

// For development convenience
impl Default for MCXClient {
    fn default() -> Self {
        Self::new().unwrap()
    }
}