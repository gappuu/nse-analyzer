use super::processor;
use super::MCXClient;
use super::config;
use super::mcx_api_server;
use super::rules;

use anyhow::Result;
use colored::Colorize;
use std::sync::Arc;

/// MCX Command Handler - encapsulates all MCX-related operations
pub struct MCXCommands;

impl MCXCommands {
    /// Run batch fetch for all MCX tickers (nearest future expiry per symbol only)
    pub async fn run_batch() -> Result<()> {
        println!("{}", "=".repeat(60).blue());
        println!("{}", "MCX Batch Processor (Nearest Future Expiry Only)".green().bold());
        println!("{}", "=".repeat(60).blue());
        println!();

        let client = Arc::new(MCXClient::new()?);

        // Step 1: Fetch all MCX tickers from web scraping
        println!("{}", "Step 1: Scraping MCX tickers from option chain page...".cyan());
        let all_tickers = client.fetch_ticker_list().await?;
        println!("{} Found {} total tickers", "✓".green(), all_tickers.len());
        
        let unique_symbols = MCXClient::get_unique_symbols(&all_tickers);
        println!("{} Unique symbols: {}", "ℹ".blue(), unique_symbols.len());
        println!("{} Symbols: {}", "ℹ".blue(), unique_symbols.join(", ").yellow());
        println!();

        // Step 1.5: Filter to only nearest future expiry per symbol
        println!("{}", "Step 1.5: Filtering to nearest future expiry per symbol...".cyan());
        let tickers = MCXClient::filter_latest_expiry_per_symbol(all_tickers.clone());
        let reduction_percent = ((all_tickers.len() - tickers.len()) as f64 / all_tickers.len() as f64) * 100.0;
        
        println!("{} Filtered to {} tickers (nearest future expiry only)", "✓".green(), tickers.len());
        println!("{} Reduction: {:.1}% fewer API calls", "📉".yellow(), reduction_percent);
        println!("{} Processing {} symbols instead of {} total combinations", 
                 "ℹ".blue(), tickers.len(), all_tickers.len());
        println!();

        // Step 2: Bulk process filtered tickers with timeout handling
        println!("{}", "Step 2: Processing nearest future expiry tickers only...".cyan());
        
        let max_concurrent = if config::is_ci_environment() {
            println!("{} CI Mode: Using lower concurrency ({})", "ℹ".blue(), config::CI_MAX_CONCURRENT);
            config::CI_MAX_CONCURRENT
        } else {
            println!("{} Max concurrent requests: {}", "ℹ".blue(), config::DEFAULT_MAX_CONCURRENT);
            config::DEFAULT_MAX_CONCURRENT
        };
        
        println!();

        let start_time = std::time::Instant::now();
        
        // Wrap the batch processing with a timeout for CI environments
        let results = if config::is_ci_environment() {
            println!("{} CI timeout enabled: {} seconds", "⏱".yellow(), config::GITHUB_ACTIONS_TIMEOUT_SECS);
            
            let timeout_duration = std::time::Duration::from_secs(config::GITHUB_ACTIONS_TIMEOUT_SECS);
            
            match tokio::time::timeout(
                timeout_duration,
                client.fetch_all_option_chains(tickers.clone(), max_concurrent)
            ).await {
                Ok(results) => results,
                Err(_) => {
                    println!("{} Timeout reached after {} seconds - stopping analysis", "⚠".red(), config::GITHUB_ACTIONS_TIMEOUT_SECS);
                    println!("{} This may indicate MCX API issues or network problems", "ℹ".blue());
                    
                    // Create empty results vector matching the tickers count
                    tickers.iter().map(|_| Err(anyhow::anyhow!("Timeout"))).collect()
                }
            }
        } else {
            // No timeout for local development
            client.fetch_all_option_chains(tickers.clone(), max_concurrent).await
        };

        let elapsed = start_time.elapsed();
        
        // Step 3: Process results
        let mut successful = Vec::new();
        let mut failed = Vec::new();
        let mut timeout_count = 0;

        for (ticker, result) in tickers.iter().zip(results.iter()) {
            match result {
                Ok((_, chain)) => {
                    successful.push((ticker.clone(), chain.clone()));
                    print!("{}", ".".green());
                }
                Err(e) => {
                    if e.to_string().contains("Timeout") {
                        timeout_count += 1;
                        print!("{}", "⏱".yellow());
                    } else {
                        failed.push((ticker.symbol.clone(), e.to_string()));
                        print!("{}", "✗".red());
                    }
                }
            }
        }
        
        println!("\n");

        // Step 4: Display summary
        Self::display_batch_summary(&successful, &failed, timeout_count, elapsed, &tickers, &all_tickers);

        // Step 5: Process data and run rules (similar to NSE)
        Self::process_batch_data_and_rules(successful).await?;

        println!();
        println!("{}", "=".repeat(60).blue());
        println!("{}", "Done!".green().bold());
        println!("{}", "=".repeat(60).blue());

        Ok(())
    }

    /// Run single ticker fetch (for API endpoints only - not used in GitHub Actions)
    pub async fn run_single(symbol: &str, expiry: &str) -> Result<()> {
        println!("{}", "=".repeat(60).blue());
        println!("{}", "MCX Single Ticker Fetch".green().bold());
        println!("{}", "=".repeat(60).blue());
        println!();

        let client = MCXClient::new()?;

        println!("{} Fetching option chain for {}...", "→".cyan(), symbol.yellow());
        println!("{} Expiry: {}", "→".cyan(), expiry.yellow());
        println!();

        let chain = client.fetch_option_chain(symbol, expiry).await?;

        // Display results
        Self::display_single_results(symbol, &chain, expiry);

        println!("{}", "=".repeat(60).blue());

        Ok(())
    }

    /// Run API server mode
    pub async fn run_server(port: u16) -> Result<()> {
        println!("{}", "=".repeat(60).blue());
        println!("{}", "MCX API Server".green().bold());
        println!("{}", "=".repeat(60).blue());
        println!();

        mcx_api_server::start_mcx_server(port).await
    }

    /// Display batch processing summary with filtering information
    fn display_batch_summary(
        successful: &[(super::models::Ticker, super::models::OptionChainResponse)],
        failed: &[(String, String)],
        timeout_count: i32,
        elapsed: std::time::Duration,
        filtered_tickers: &[super::models::Ticker],
        all_tickers: &[super::models::Ticker],
    ) {
        println!("{}", "=".repeat(60).blue());
        println!("{}", "Summary".cyan().bold());
        println!("{}", "=".repeat(60).blue());
        
        // Filtering summary
        println!("{} Total tickers available: {}", "📊".blue(), all_tickers.len());
        println!("{} Nearest future expiry filtered: {}", "🔍".blue(), filtered_tickers.len());
        let reduction_percent = ((all_tickers.len() - filtered_tickers.len()) as f64 / all_tickers.len() as f64) * 100.0;
        println!("{} API call reduction: {:.1}%", "📉".green(), reduction_percent);
        println!();
        
        // Processing summary
        println!("{} Successful: {}/{}", "✓".green(), successful.len(), filtered_tickers.len());
        println!("{} Failed: {}/{}", "✗".red(), failed.len(), filtered_tickers.len());
        
        if timeout_count > 0 {
            println!("{} Timed out: {} (due to {} second limit)", "⏱".yellow(), timeout_count, config::GITHUB_ACTIONS_TIMEOUT_SECS);
        }
        
        println!("{} Time taken: {:.2}s", "⏱".yellow(), elapsed.as_secs_f64());
        
        if filtered_tickers.len() > 0 {
            println!("{} Avg time per ticker: {:.2}s", "⏱".yellow(), elapsed.as_secs_f64() / filtered_tickers.len() as f64);
        }
        
        println!();

        // Show failed tickers
        if !failed.is_empty() {
            println!("{}", "Failed Tickers:".red());
            for (symbol, error) in failed.iter().take(10) {
                println!("  {} {} → {}", "✗".red(), symbol.yellow(), error.chars().take(80).collect::<String>());
            }
            if failed.len() > 10 {
                println!("  ... and {} more", failed.len() - 10);
            }
            println!();
        }
    }

    /// Display single ticker fetch results
    fn display_single_results(symbol: &str, chain: &super::models::OptionChainResponse, expiry: &str) {
        println!("{}", "=".repeat(60).blue());
        println!("{}", "Results".cyan().bold());
        println!("{}", "=".repeat(60).blue());
        println!("{} Symbol: {}", "✓".green(), symbol.yellow());
        println!("{} Expiry: {}", "✓".green(), expiry);
        
        if let Some(as_on) = &chain.d.summary.as_on {
            println!("{} As On: {}", "✓".green(), as_on);
        }
        
        if let Some(count) = chain.d.summary.count {
            println!("{} Total strikes: {}", "✓".green(), count);
        }
        
        println!();
        println!("{} Data points processed: {}", "✓".green(), chain.d.data.len());
        println!();
    }

    /// Process batch data and apply rules (similar to NSE implementation)
    async fn process_batch_data_and_rules(
        successful: Vec<(super::models::Ticker, super::models::OptionChainResponse)>
    ) -> Result<()> {
        println!("{}", "Processing data and applying rules...".cyan());
        
        // Process each ticker's data through the processor and rules
        let mut batch_for_rules = Vec::new();
        
        for (ticker, chain) in successful.iter() {
            // Get underlying value from first available data point
            let underlying_value = chain.d.data.iter()
                .find_map(|d| d.underlying_value)
                .unwrap_or(0.0);
            
            // Process through the MCX processor
            match processor::process_mcx_option_data(
                chain.d.data.clone(),
                underlying_value,
                &ticker.expiry_date,
            ) {
                Ok((processed_data, spread, _days_to_expiry, _ce_oi, _pe_oi)) => {
                    // Store for rules processing
                    batch_for_rules.push((
                        ticker.symbol.clone(),
                        processor::convert_mcx_timestamp(&chain.d.summary.as_on.clone().unwrap_or_else(|| "".to_string())),
                        underlying_value,
                        processed_data,
                        spread,
                    ));
                }
                Err(e) => {
                    println!("{} Failed to process {}: {}", "⚠".yellow(), ticker.symbol, e);
                }
            }
        }
        
        // Run rules on all processed securities
        let rules_outputs = rules::run_mcx_batch_rules(batch_for_rules);
        
        // Save only the rules output (alerts) - similar to NSE
        if !rules_outputs.is_empty() {
            std::fs::write(
                "mcx_batch_results.json",
                serde_json::to_string_pretty(&rules_outputs)?,
            )?;
            
            let total_alerts: usize = rules_outputs.iter()
                .map(|r| r.alerts.len())
                .sum();
            
            println!("{} Saved alerts to mcx_batch_results.json", "✓".green());
            println!("{} Securities with alerts: {}", "ℹ".blue(), rules_outputs.len());
            println!("{} Total alerts: {}", "ℹ".blue(), total_alerts);
        } else {
            // Create empty file for consistency
            std::fs::write("mcx_batch_results.json", "[]")?;
            println!("{} No alerts found across all securities", "ℹ".blue());
            println!("{} Created empty results file: mcx_batch_results.json", "✓".green());
        }

        Ok(())
    }

    /// Print usage instructions
    pub fn print_usage() {
        eprintln!("Set MCX_MODE environment variable to control execution mode");
        eprintln!("Examples:");
        eprintln!("  MCX_MODE=server MCX_PORT=3002 cargo run   # Start MCX API server on port 3002");
        eprintln!("  MCX_MODE=batch cargo run                   # Run MCX batch analysis (nearest future expiry only)");
        eprintln!("Note: GitHub Actions supports 'batch' mode");
    }

    /// Handle CI environment mode switching
    pub fn handle_ci_mode_override(mode: &str) -> bool {
        if config::is_ci_environment() && (mode == "server") {
            println!("{} GitHub Actions only supports batch mode, running batch instead", "ℹ".blue());
            true
        } else {
            false
        }
    }
}