use super::processor;
use super::NSEClient;
use super::config;
use super::models;
use super::rules;
use super::nse_api_server;

use anyhow::{Result, Context};
use colored::Colorize;
use std::sync::Arc;
use crate::utility::{Timer, AggregateTimer};

/// NSE Command Handler - encapsulates all NSE-related operations
pub struct NSECommands;

impl NSECommands {
    /// Run batch fetch for all FNO securities
    pub async fn run_batch() -> Result<()> {
        let _total_timer = Timer::start("Total Batch Processing");
        
        println!("{}", "=".repeat(60).blue());
        println!("{}", "NSE Batch Processor".green().bold());
        println!("{}", "=".repeat(60).blue());
        println!();

        let client = Arc::new(NSEClient::new()?);

        // Step 1: Fetch all FNO securities
        let securities = {
            let _step1_timer = Timer::start("Step 1: Fetch FNO List");
            println!("{}", "Step 1: Fetching all FNO securities...".cyan());
            let securities = client.fetch_fno_list().await?;
            println!("{} Found {} securities", "✓".green(), securities.len());
            println!();
            securities
        };

        // Step 2: Bulk process all securities
        println!("{}", "Step 2: Processing all securities...".cyan());
        
        let max_concurrent = if config::is_ci_environment() {
            println!("{} CI Mode: Using concurrency ({})", "ℹ".blue(), config::CI_MAX_CONCURRENT);
            config::CI_MAX_CONCURRENT
        } else {
            println!("{} Max concurrent requests: {}", "ℹ".blue(), config::DEFAULT_MAX_CONCURRENT);
            config::DEFAULT_MAX_CONCURRENT
        };
        
        println!();

        let (results, step2_elapsed) = {
            let step2_timer = Timer::silent("Step 2");
            
        let results = if config::is_ci_environment() {
            println!("{} CI timeout enabled: {} seconds", "⏱".yellow(), config::GITHUB_ACTIONS_TIMEOUT_SECS);
            
            let timeout_duration = std::time::Duration::from_secs(config::GITHUB_ACTIONS_TIMEOUT_SECS);
            
            match tokio::time::timeout(
                timeout_duration,
                client.fetch_all_option_chains(securities.clone(), max_concurrent)
            ).await {
                Ok(results) => results,
                Err(_) => {
                    println!("{} Timeout reached after {} seconds - stopping analysis", "⚠".red(), config::GITHUB_ACTIONS_TIMEOUT_SECS);
                    println!("{} This may indicate NSE API issues or network problems", "ℹ".blue());
                    securities.iter().map(|_| Err(anyhow::anyhow!("Timeout"))).collect()
                }
            }
        } else {
            client.fetch_all_option_chains(securities.clone(), max_concurrent).await
        };
        
        let elapsed = step2_timer.elapsed();
        println!("⏱️  Step 2: Fetch All Option Chains - {:.2}s", elapsed.as_secs_f64());
        (results, elapsed)
    };
    
    // Step 3: Process results
    let (successful, failed, timeout_count) = {
        let _step3_timer = Timer::start("Step 3: Collect Results");
        let mut successful = Vec::new();
        let mut failed = Vec::new();
        let mut timeout_count = 0;

        for (security, result) in securities.iter().zip(results.iter()) {
            match result {
                Ok((_, chain)) => {
                    successful.push((security.clone(), chain.clone()));
                    // print!("{}", ".".green());
                }
                Err(e) => {
                    if e.to_string().contains("Timeout") {
                        timeout_count += 1;
                        print!("{}", "⏱".yellow());
                    } else {
                        failed.push((security.symbol.clone(), e.to_string()));
                        print!("{}", "✗".red());
                    }
                }
            }
        }
        println!("\n");
        (successful, failed, timeout_count)
    };

        // Step 4: Display summary
        Self::display_batch_summary(&successful, &failed, timeout_count, step2_elapsed, &securities);

        // Step 5: Process data and run rules
        Self::process_batch_data_and_rules(successful).await?;

        println!();
        println!("{}", "=".repeat(60).blue());
        println!("{}", "Done!".green().bold());
        println!("{}", "=".repeat(60).blue());

        Ok(())
    }
    /// Run single security fetch (for API endpoints only - not used in GitHub Actions)
    pub async fn run_single(symbol: &str, expiry: &str) -> Result<()> {
        let _total_timer = Timer::start(format!("Single Security: {}", symbol));
        
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

        let chain = {
            let _fetch_timer = Timer::start("Fetch Option Chain");
            client.fetch_option_chain(&security, expiry).await?
        };

        Self::display_single_results(symbol, &chain, expiry);

        let (processed_data, spread) = {
            let _process_timer = Timer::start("Process Option Data");
            processor::process_option_data(
                chain.filtered.data.clone(),
                chain.records.underlying_value
            )
        };

        let days_to_expiry = processed_data.first()
            .map(|opt| opt.days_to_expiry)
            .unwrap_or(0);

        println!("{} Days to expiry: {}", "ℹ".blue(), days_to_expiry);
        
        let rules_output = {
            let _rules_timer = Timer::start("Run Rules Engine");
            rules::run_rules(
                &processed_data,
            symbol.to_string(),
            chain.records.timestamp.clone(),
            chain.records.underlying_value,
            spread,
            )
        };
        
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

    /// Display batch processing summary
    fn display_batch_summary(
        successful: &[(models::Security, models::OptionChain)],
        failed: &[(String, String)],
        timeout_count: i32,
        elapsed: std::time::Duration,
        _securities: &[models::Security],
    ) {
        println!("{}", "=".repeat(60).blue());
        println!("{}", "Summary".cyan().bold());
        println!("{}", "=".repeat(60).blue());
        println!("{} Successful: {}", "✓".green(), successful.len());
        println!("{} Failed: {}", "✗".red(), failed.len());
        
        if timeout_count > 0 {
            println!("{} Timed out: {} (due to {} second limit)", "⏱".yellow(), timeout_count, config::GITHUB_ACTIONS_TIMEOUT_SECS);
        }
        
        println!("{} Time taken: {:.2}s", "⏱".yellow(), elapsed.as_secs_f64());
        
        if successful.len() > 0 {
            println!("{} Avg time per security: {:.2}s", "📊".to_string(), elapsed.as_secs_f64() / successful.len() as f64);
            let requests_per_sec = successful.len() as f64 / elapsed.as_secs_f64();
            println!("{} Throughput: {:.2} securities/sec", "⚡".to_string(), requests_per_sec);
        }
        
        println!();

        // Show failed securities
        if !failed.is_empty() {
            println!("{}", "Failed Securities:".red());
            for (symbol, error) in failed.iter().take(10) {
                println!("  {} {} → {}", "✗".red(), symbol.yellow(), error.chars().take(80).collect::<String>());
            }
            if failed.len() > 10 {
                println!("  ... and {} more", failed.len() - 10);
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

    /// Process batch data and apply rules
    async fn process_batch_data_and_rules(
        successful: Vec<(models::Security, models::OptionChain)>
    ) -> Result<()> {
        let _total_timer = Timer::start("Step 4: Process Data & Apply Rules");
        println!("{}", "Processing data and applying rules...".cyan());
        
        let mut process_timer = AggregateTimer::new("Option Data Processing");
        let mut write_timer = AggregateTimer::new("File Write Operations");
        
        // Create output directory for individual ticker files
        let output_dir = std::path::Path::new("processed_data");
        if !output_dir.exists() {
            std::fs::create_dir_all(output_dir)
                .context("Failed to create processed_data directory")?;
        }
        
        let mut processed_batch = Vec::new();
        let mut batch_for_rules = Vec::new();
        
        for (security, chain) in successful.iter() {
            let item_timer = Timer::silent("process_item");
            
            let (processed_data, spread) = processor::process_option_data(
                chain.filtered.data.clone(),
                chain.records.underlying_value
            );
            
            process_timer.record(item_timer.elapsed());
            
            let days_to_expiry = processed_data.first()
                .map(|opt| opt.days_to_expiry)
                .unwrap_or(0);
            
            // Create record structure
            let record = serde_json::json!({
                "record": {
                    "symbol": security.symbol,
                    // "security_type": match security.security_type {
                    //     models::SecurityType::Equity => "Equity",
                    //     models::SecurityType::Indices => "Index",
                    // },
                    "timestamp": chain.records.timestamp,
                    "underlying_value": chain.records.underlying_value,
                    "spread": spread,
                    "days_to_expiry": days_to_expiry,
                    "ce_oi": chain.filtered.ce_totals.total_oi,
                    "pe_oi": chain.filtered.pe_totals.total_oi,
                    // "ce_change_in_oi": chain.filtered.ce_totals.total_change_in_oi,
                    // "pe_change_in_oi": chain.filtered.pe_totals.total_change_in_oi,
                },
                "data": processed_data.clone(),
            });
            
            // Write individual ticker file
            let write_item_timer = Timer::silent("write_file");
            let filename = format!("{}.json", security.symbol);
            let filepath = output_dir.join(&filename);
            
            std::fs::write(
                &filepath,
                serde_json::to_string_pretty(&record)?,
            ).with_context(|| format!("Failed to write {}", filename))?;
            
            write_timer.record(write_item_timer.elapsed());
            
            // Store for batch processing
            processed_batch.push(record);
            
            // Store for rules processing
            batch_for_rules.push((
                security.symbol.clone(),
                chain.records.timestamp.clone(),
                chain.records.underlying_value,
                processed_data,
                spread,
            ));
        }
        
        println!();
        process_timer.summary();
        write_timer.summary();
        
        println!("{} Written {} ticker files to {}/", 
            "✓".green(), 
            successful.len(), 
            output_dir.display().to_string().yellow()
        );
        println!();
        
        let rules_outputs = {
            let _rules_timer = Timer::start("Run Rules Engine");
            rules::run_batch_rules(batch_for_rules)
        };
        
        {
            let _save_timer = Timer::start("Save Rules Output");
        if !rules_outputs.is_empty() {
            std::fs::write(
                "batch_rules.json",
                serde_json::to_string_pretty(&rules_outputs)?,
            )?;
            
            let total_alerts: usize = rules_outputs.iter()
                .map(|r| r.alerts.len())
                .sum();
            
            println!("{} Saved rules to batch_rules.json", "✓".green());
            println!("{} Securities with alerts: {}", "ℹ".blue(), rules_outputs.len());
            println!("{} Total alerts: {}", "ℹ".blue(), total_alerts);
        } else {
            std::fs::write("batch_rules.json", "[]")?;
            println!("{} No alerts found across all securities", "ℹ".blue());
            println!("{} Created empty rules file: batch_rules.json", "✓".green());
        }
        }

        Ok(())
    }
   
    /// Print usage instructions
    pub fn print_usage() {
        eprintln!("Set NSE_MODE environment variable to control execution mode");
        eprintln!("Examples:");
        eprintln!("  NSE_MODE=server NSE_PORT=3001 cargo run   # Start API server on port 3001");
        eprintln!("  NSE_MODE=batch cargo run                   # Run batch analysis");
        eprintln!("Note: GitHub Actions only supports 'batch' mode");
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