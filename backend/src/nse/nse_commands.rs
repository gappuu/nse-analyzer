use super::processor;
use super::NSEClient;
use super::config;
use super::models;
use super::rules;
use super::nse_api_server;

use anyhow::Result;
use colored::Colorize;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

/// NSE Command Handler - encapsulates all NSE-related operations
pub struct NSECommands;

impl NSECommands {
    /// OPTIMIZED: Run batch fetch with streaming and progress tracking
    pub async fn run_batch() -> Result<()> {
        println!("{}", "=".repeat(60).blue());
        println!("{}", "NSE Batch Processor (OPTIMIZED)".green().bold());
        println!("{}", "=".repeat(60).blue());
        println!();

        let client = Arc::new(NSEClient::new()?);

        // Step 1: Fetch all FNO securities
        println!("{}", "Step 1: Fetching all FNO securities...".cyan());
        let securities = client.fetch_fno_list().await?;
        println!("{} Found {} securities", "✓".green(), securities.len());
        println!();

        // Step 2: Determine concurrency based on environment
        let max_concurrent = config::get_max_concurrent();
        
        println!("{}", "Step 2: Processing all securities...".cyan());
        if config::is_ci_environment() {
            println!("{} CI Mode: AGGRESSIVE concurrency ({})", "🚀".to_string(), max_concurrent);
            println!("{} Timeout: {} seconds", "⏱".yellow(), config::GITHUB_ACTIONS_TIMEOUT_SECS);
        } else {
            println!("{} Max concurrent requests: {}", "ℹ".blue(), max_concurrent);
        }
        println!();

        let start_time = std::time::Instant::now();
        
        // OPTIMIZATION: Progress tracking
        let total_count = securities.len();
        let processed_count = Arc::new(AtomicUsize::new(0));
        let success_count = Arc::new(AtomicUsize::new(0));
        let failed_count = Arc::new(AtomicUsize::new(0));
        
        // Spawn progress reporter for CI
        let progress_handle = if config::is_ci_environment() {
            let processed = Arc::clone(&processed_count);
            let total = total_count;
            let start = start_time.clone();
            
            Some(tokio::spawn(async move {
                let mut last_reported = 0;
                loop {
                    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                    let current = processed.load(Ordering::Relaxed);
                    
                    if current > last_reported {
                        let elapsed = start.elapsed().as_secs_f64();
                        let rate = current as f64 / elapsed;
                        let remaining = total - current;
                        let eta = remaining as f64 / rate;
                        
                        println!("{} Progress: {}/{} ({:.1}%) | Rate: {:.1}/s | ETA: {:.0}s", 
                            "📊".to_string(), current, total, 
                            (current as f64 / total as f64) * 100.0,
                            rate, eta);
                        
                        last_reported = current;
                    }
                    
                    if current >= total {
                        break;
                    }
                }
            }))
        } else {
            None
        };
        
        // Wrap the batch processing with timeout for CI
        let results = if config::is_ci_environment() {
            let timeout_duration = std::time::Duration::from_secs(config::GITHUB_ACTIONS_TIMEOUT_SECS);
            
            match tokio::time::timeout(
                timeout_duration,
                client.fetch_all_option_chains(securities.clone(), max_concurrent)
            ).await {
                Ok(results) => results,
                Err(_) => {
                    println!("{} Timeout reached after {} seconds", "⚠".red(), config::GITHUB_ACTIONS_TIMEOUT_SECS);
                    
                    // Return partial results with timeout errors
                    securities.iter().map(|_| Err(anyhow::anyhow!("Timeout"))).collect()
                }
            }
        } else {
            client.fetch_all_option_chains(securities.clone(), max_concurrent).await
        };

        // Cancel progress reporter
        if let Some(handle) = progress_handle {
            handle.abort();
        }

        let elapsed = start_time.elapsed();
        
        // Step 3: Process results with real-time feedback
        let mut successful = Vec::new();
        let mut failed = Vec::new();
        let mut timeout_errors = 0;

        println!("\n{} Processing results...", "→".cyan());
        
        for (security, result) in securities.iter().zip(results.iter()) {
            processed_count.fetch_add(1, Ordering::Relaxed);
            
            match result {
                Ok((_, chain)) => {
                    successful.push((security.clone(), chain.clone()));
                    success_count.fetch_add(1, Ordering::Relaxed);
                    print!("{}", ".".green());
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    if error_msg.contains("Timeout") {
                        timeout_errors += 1;
                        print!("{}", "⏱".yellow());
                    } else {
                        failed.push((security.symbol.clone(), error_msg));
                        failed_count.fetch_add(1, Ordering::Relaxed);
                        print!("{}", "✗".red());
                    }
                }
            }
            
            // Print newline every 50 items for readability
            if processed_count.load(Ordering::Relaxed) % 50 == 0 {
                println!();
            }
        }
        
        println!("\n");

        // Step 4: Display summary
        Self::display_batch_summary(&successful, &failed, timeout_errors, elapsed, &securities);

        // Step 5: Process data and run rules
        if !successful.is_empty() {
            Self::process_batch_data_and_rules(successful).await?;
        } else {
            println!("{} No successful fetches to process", "⚠".yellow());
        }

        println!();
        println!("{}", "=".repeat(60).blue());
        println!("{}", "Done!".green().bold());
        println!("{}", "=".repeat(60).blue());

        Ok(())
    }

    /// Run single security fetch
    pub async fn run_single(symbol: &str, expiry: &str) -> Result<()> {
        println!("{}", "=".repeat(60).blue());
        println!("{}", "NSE Single Security Fetch".green().bold());
        println!("{}", "=".repeat(60).blue());
        println!();

        let client = NSEClient::new()?;

        let security = if config::NSE_INDICES.contains(&symbol) {
            models::Security::index(symbol.to_string())
        } else {
            models::Security::equity(symbol.to_string())
        };

        println!("{} Fetching option chain for {}...", "→".cyan(), symbol.yellow());
        println!("{} Expiry: {}", "→".cyan(), expiry.yellow());
        println!();

        let chain = client.fetch_option_chain(&security, expiry).await?;

        Self::display_single_results(symbol, &chain, expiry);

        let (processed_data, spread) = processor::process_option_data(
            chain.filtered.data.clone(),
            chain.records.underlying_value
        );

        let days_to_expiry = processed_data.first()
            .map(|opt| opt.days_to_expiry)
            .unwrap_or(0);

        println!("{} Days to expiry: {}", "ℹ".blue(), days_to_expiry);
        
        let rules_output = rules::run_rules(
            &processed_data,
            symbol.to_string(),
            chain.records.timestamp.clone(),
            chain.records.underlying_value,
            spread,
        );
        
        if let Some(output) = rules_output {
            println!("{} Total alerts: {}", "ℹ".blue(), output.alerts.len());
        } else {
            println!("{} No alerts found", "ℹ".blue());
        }
        println!("{}", "=".repeat(60).blue());

        Ok(())
    }

    /// Run API server mode
    pub async fn run_server(port: u16) -> Result<()> {
        println!("{}", "=".repeat(60).blue());
        println!("{}", "NSE API Server".green().bold());
        println!("{}", "=".repeat(60).blue());
        println!();

        nse_api_server::start_server(port).await
    }

    /// OPTIMIZED: Display batch summary with performance metrics
    fn display_batch_summary(
        successful: &[(models::Security, models::OptionChain)],
        failed: &[(String, String)],
        timeout_count: i32,
        elapsed: std::time::Duration,
        securities: &[models::Security],
    ) {
        println!("{}", "=".repeat(60).blue());
        println!("{}", "Summary".cyan().bold());
        println!("{}", "=".repeat(60).blue());
        
        let success_pct = (successful.len() as f64 / securities.len() as f64) * 100.0;
        
        println!("{} Successful: {} ({:.1}%)", "✓".green(), successful.len(), success_pct);
        println!("{} Failed: {}", "✗".red(), failed.len());
        
        if timeout_count > 0 {
            println!("{} Timed out: {}", "⏱".yellow(), timeout_count);
        }
        
        println!();
        println!("{} Performance Metrics:", "📊".to_string());
        println!("  • Total time: {:.2}s", elapsed.as_secs_f64());
        println!("  • Avg per security: {:.2}s", elapsed.as_secs_f64() / securities.len() as f64);
        println!("  • Throughput: {:.1} securities/s", securities.len() as f64 / elapsed.as_secs_f64());
        
        if !successful.is_empty() {
            println!("  • Success rate: {:.1}%", success_pct);
        }
        
        println!();

        // Show sample of failed securities
        if !failed.is_empty() {
            println!("{}", "Failed Securities (sample):".red());
            for (symbol, error) in failed.iter().take(5) {
                let short_error: String = error.chars().take(60).collect();
                println!("  {} {} → {}", "✗".red(), symbol.yellow(), short_error);
            }
            if failed.len() > 5 {
                println!("  ... and {} more", failed.len() - 5);
            }
            println!();
        }
    }

    /// Display single security fetch results
    fn display_single_results(symbol: &str, chain: &models::OptionChain, expiry: &str) {
        println!("{}", "=".repeat(60).blue());
        println!("{}", "Results".cyan().bold());
        println!("{}", "=".repeat(60).blue());
        println!("{} Symbol: {}", "✓".green(), symbol.yellow());
        println!("{} Timestamp: {}", "✓".green(), chain.records.timestamp);
        println!("{} Underlying: {:.2}", "✓".green(), chain.records.underlying_value);
        println!("{} Expiry: {}", "✓".green(), expiry);
        println!();
        println!("{} Total strikes processed: {}", "✓".green(), chain.filtered.data.len());
        println!();
    }

    /// OPTIMIZED: Process batch data with parallel file writing
    async fn process_batch_data_and_rules(
        successful: Vec<(models::Security, models::OptionChain)>
    ) -> Result<()> {
        println!("{}", "Processing data and applying rules...".cyan());
        
        let mut batch_for_rules = Vec::new();
        let mut all_processed_data = Vec::new();
        
        // OPTIMIZATION: Pre-allocate capacity
        batch_for_rules.reserve(successful.len());
        all_processed_data.reserve(successful.len());
        
        for (security, chain) in successful.iter() {
            let (processed_data, spread) = processor::process_option_data(
                chain.filtered.data.clone(),
                chain.records.underlying_value
            );
            
            let days_to_expiry = processed_data.first()
                .map(|opt| opt.days_to_expiry)
                .unwrap_or(0);
            
            let ticker_data = serde_json::json!({
                "record": {
                    "symbol": security.symbol,
                    "timestamp": chain.records.timestamp,
                    "underlying_value": chain.records.underlying_value,
                    "spread": spread,
                    "days_to_expiry": days_to_expiry,
                    "ce_oi": chain.filtered.ce_totals.total_oi,
                    "pe_oi": chain.filtered.pe_totals.total_oi,
                },
                "data": processed_data.clone(),
            });
            
            all_processed_data.push(ticker_data);
            
            batch_for_rules.push((
                security.symbol.clone(),
                chain.records.timestamp.clone(),
                chain.records.underlying_value,
                processed_data,
                spread,
            ));
        }
        
        // Write consolidated file
        let write_start = std::time::Instant::now();
        std::fs::write(
            "batch_processed.json",
            serde_json::to_string(&all_processed_data)?,
        )?;
        
        println!("{} Saved batch_processed.json ({} securities) in {:.2}s", 
                "✓".green(), all_processed_data.len(), write_start.elapsed().as_secs_f64());
        
        // Run rules
        let rules_start = std::time::Instant::now();
        let rules_outputs = rules::run_batch_rules(batch_for_rules);
        
        println!("{} Rules processed in {:.2}s", "✓".green(), rules_start.elapsed().as_secs_f64());
        
        if !rules_outputs.is_empty() {
            std::fs::write(
                "batch_rules.json",
                serde_json::to_string_pretty(&rules_outputs)?,
            )?;
            
            let total_alerts: usize = rules_outputs.iter()
                .map(|r| r.alerts.len())
                .sum();
            
            println!("{} Saved rules to batch_rules.json", "✓".green());
            println!("{} Securities with alerts: {}", "📋".to_string(), rules_outputs.len());
            println!("{} Total alerts: {}", "🔔".to_string(), total_alerts);
        } else {
            std::fs::write("batch_rules.json", "[]")?;
            println!("{} No alerts found", "ℹ".blue());
        }

        Ok(())
    }

    /// Split batch_processed.json into individual ticker files
    pub async fn split_batch_file() -> Result<()> {
        use rayon::prelude::*;
        use serde_json::Value;
        
        println!("{}", "=".repeat(60).blue());
        println!("{}", "Splitting batch file".green().bold());
        println!("{}", "=".repeat(60).blue());
        println!();
        
        let start = std::time::Instant::now();
        let content = std::fs::read_to_string("batch_processed.json")?;
        let data: Vec<Value> = serde_json::from_str(&content)?;
        
        println!("{} Loaded {} securities in {:.2}s", "✓".green(), data.len(), start.elapsed().as_secs_f64());
        
        let output_dir = std::path::Path::new("processed_data");
        if !output_dir.exists() {
            std::fs::create_dir_all(output_dir)?;
        }
        
        let split_start = std::time::Instant::now();
        let files_created: Result<usize> = data
            .par_iter()
            .map(|ticker_data| {
                let symbol = ticker_data["record"]["symbol"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("Missing symbol"))?;
                
                let filename = format!("processed_data/{}.json", symbol);
                std::fs::write(&filename, serde_json::to_string(ticker_data)?)?;
                
                Ok(1)
            })
            .try_fold(|| 0, |acc, result: Result<usize>| result.map(|v| acc + v))
            .try_reduce(|| 0, |a, b| Ok(a + b));
        
        let files_created = files_created?;
        
        println!("{} Created {} files in {:.2}s", "✓".green(), files_created, split_start.elapsed().as_secs_f64());
        println!("{} Total time: {:.2}s", "✓".green(), start.elapsed().as_secs_f64());
        println!();
        
        Ok(())
    }

    pub fn print_usage() {
        eprintln!("Set NSE_MODE environment variable to control execution mode");
        eprintln!("Examples:");
        eprintln!("  NSE_MODE=server NSE_PORT=3001 cargo run   # Start API server");
        eprintln!("  NSE_MODE=batch cargo run                   # Run batch analysis");
        eprintln!("  NSE_MODE=batch NSE_MAX_CONCURRENT=20 cargo run  # Custom concurrency");
    }

    pub fn handle_ci_mode_override(mode: &str) -> bool {
        if config::is_ci_environment() && (mode == "server") {
            println!("{} GitHub Actions: switching to batch mode", "ℹ".blue());
            true
        } else {
            false
        }
    }
}