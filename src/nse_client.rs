use crate::models::{ContractInfo, OptionChain, Security, SecurityType};
use anyhow::{Context, Result};
use rand::{seq::SliceRandom, thread_rng};
use reqwest::{header, Client, StatusCode};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Semaphore, RwLock};
use tokio_retry::strategy::ExponentialBackoff;
use tokio_retry::Retry;

// -----------------------------------------------
// CLIENT WRAPPER WITH SESSION STATE
// -----------------------------------------------
pub struct NSEClient {
    client: Client,
    warmed_up: Arc<RwLock<bool>>,
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
            let _ = self.client
                .get("https://www.nseindia.com")
                .header("Accept", "text/html")
                .send()
                .await
                .context("Failed to warm up NSE session")?;
            
            tokio::time::sleep(Duration::from_millis(200)).await;
            *warmed = true;
        }
        
        Ok(())
    }

    /// Generic retry fetch with better error handling
    async fn fetch_json(&self, url: &str) -> Result<String> {
        self.warmup_if_needed().await?;

        let backoff = ExponentialBackoff::from_millis(200)
            .factor(3)
            .max_delay(Duration::from_secs(5))
            .take(5);

        Retry::spawn(backoff, || async {
            let res = self.client
                .get(url)
                .header("Referer", "https://www.nseindia.com/")
                .header("X-Requested-With", "XMLHttpRequest")
                .send()
                .await
                .context("Request send failed")?;

            let status = res.status();

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
        })
        .await
    }

    // -----------------------------------------------
    // STEP 1: FETCH FNO LIST
    // -----------------------------------------------
    pub async fn fetch_fno_list(&self) -> Result<Vec<Security>> {
        let url = "https://www.nseindia.com/api/master-quote";
        let text = self.fetch_json(url).await?;
        
        let symbols: Vec<String> = serde_json::from_str(&text)
            .context("Failed to parse FNO list")?;
        
        let mut securities: Vec<Security> = symbols
            .into_iter()
            .map(Security::equity)
            .collect();
        
        // Add indices
        securities.push(Security::index("NIFTY".to_string()));
        securities.push(Security::index("BANKNIFTY".to_string()));
        securities.push(Security::index("FINNIFTY".to_string()));
        
        Ok(securities)
    }

    // -----------------------------------------------
    // STEP 2: FETCH CONTRACT INFO
    // -----------------------------------------------
    pub async fn fetch_contract_info(&self, symbol: &str) -> Result<ContractInfo> {
        let url = format!(
            "https://www.nseindia.com/api/option-chain-contract-info?symbol={}",
            symbol
        );
        
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
        let typ = match security.security_type {
            SecurityType::Equity => "Equity",
            SecurityType::Indices => "Indices",
        };
        
        let url = format!(
            "https://www.nseindia.com/api/option-chain-v3?type={}&symbol={}&expiry={}",
            typ, security.symbol, expiry
        );
        
        let text = self.fetch_json(&url).await?;
        let chain: OptionChain = serde_json::from_str(&text)
            .context("Failed to parse option chain")?;
        
        Ok(chain)
    }

    // -----------------------------------------------
    // BATCH FETCH WITH CONCURRENCY CONTROL
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
                // Acquire permit - handle error properly
                let _permit = sem.acquire_owned().await
                    .map_err(|e| anyhow::anyhow!("Semaphore error: {}", e))?;

                // Get contract info
                let contract_info = client.fetch_contract_info(&security.symbol).await?;
                
                // Use nearest (first) expiry
                let expiry = contract_info
                    .expiry_dates
                    .first()
                    .context("No expiry dates found")?;

                // Get option chain
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
    
    // Rotating Accept-Language headers (fingerprint avoidance)
    let langs = ["en-US,en;q=0.9", "en-GB,en;q=0.8", "en-IN,en;q=0.9"];
    let lang = langs.choose(&mut thread_rng()).unwrap();
    headers.insert(
        header::ACCEPT_LANGUAGE, 
        header::HeaderValue::from_str(lang)?
    );
    headers.insert(header::ACCEPT, header::HeaderValue::from_static("*/*"));

    Ok(Client::builder()
        .default_headers(headers)
        .cookie_store(true) // crucial for NSE
        .user_agent(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) \
             AppleWebKit/537.36 (KHTML, like Gecko) \
             Chrome/131.0.0.0 Safari/537.36",
        )
        .timeout(Duration::from_secs(20))
        .build()
        .context("Failed to build HTTP client")?)
}