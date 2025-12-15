mod config;
mod models;
mod nse_client;
mod processor;
mod rules;
mod api_server_axum;  // Use the new Axum-based server

use anyhow::Result;
use colored::Colorize;
use nse_client::NSEClient;
use std::sync::Arc;

/// Run batch fetch for all FNO securities
async fn run_batch() -> Result<()> {
    println!("{}", "=".repeat(60).blue());
    println!("{}", "NSE Batch Processor".green().bold());
    println!("{}", "=".repeat(60).blue());
    println!();

    let client = Arc::new(NSEClient::new()?);

    // Step 1: Fetch all FNO securities
    println!("{}", "Step 1: Fetching all FNO securities...".cyan());
    let securities = client.fetch_fno_list().await?;
    println!("{} Found {} securities", "✓".green(), securities.len());
    println!();

    // Step 2: Bulk process all securities with timeout handling
    println!("{}", "Step 2: Processing all securities...".cyan());
    
    let max_concurrent = if config::is_ci_environment() {
        println!("{} CI Mode: Using higher concurrency ({})", "ℹ".blue(), config::CI_MAX_CONCURRENT);
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
            client.fetch_all_option_chains(securities.clone(), max_concurrent)
        ).await {
            Ok(results) => results,
            Err(_) => {
                println!("{} Timeout reached after {} seconds - stopping analysis", "⚠".red(), config::GITHUB_ACTIONS_TIMEOUT_SECS);
                println!("{} This may indicate NSE API issues or network problems", "ℹ".blue());
                
                // Create empty results vector matching the securities count
                securities.iter().map(|_| Err(anyhow::anyhow!("Timeout"))).collect()
            }
        }
    } else {
        // No timeout for local development
        client.fetch_all_option_chains(securities.clone(), max_concurrent).await
    };

    let elapsed = start_time.elapsed();
    
    // Step 3: Process results
    let mut successful = Vec::new();
    let mut failed = Vec::new();
    let mut timeout_count = 0;

    for (security, result) in securities.iter().zip(results.iter()) {
        match result {
            Ok((_, chain)) => {
                successful.push((security.clone(), chain.clone()));
                print!("{}", ".".green());
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

    // Step 4: Display summary
    println!("{}", "=".repeat(60).blue());
    println!("{}", "Summary".cyan().bold());
    println!("{}", "=".repeat(60).blue());
    println!("{} Successful: {}", "✓".green(), successful.len());
    println!("{} Failed: {}", "✗".red(), failed.len());
    if timeout_count > 0 {
        println!("{} Timed out: {} (due to {} second limit)", "⏱".yellow(), timeout_count, config::GITHUB_ACTIONS_TIMEOUT_SECS);
    }
    println!("{} Time taken: {:.2}s", "⏱".yellow(), elapsed.as_secs_f64());
    if securities.len() > 0 {
        println!("{} Avg time per security: {:.2}s", "⏱".yellow(), elapsed.as_secs_f64() / securities.len() as f64);
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

    // Step 5: Process data and run rules
    println!("{}", "Processing data and applying rules...".cyan());
    
    // Process each security's data
    let mut processed_batch = Vec::new();
    let mut batch_for_rules = Vec::new();
    
    for (security, chain) in successful.iter() {
        let (processed_data, spread) = processor::process_option_data(
            chain.filtered.data.clone(),
            chain.records.underlying_value
        );
        
        // Extract days_to_expiry from first processed option (they should all be the same)
        let days_to_expiry = processed_data.first()
            .map(|opt| opt.days_to_expiry)
            .unwrap_or(0);
        
        // Store for JSON output
        processed_batch.push(serde_json::json!({
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
        }));
        
        // Store for rules processing - now includes spread
        batch_for_rules.push((
            security.symbol.clone(),
            chain.records.timestamp.clone(),
            chain.records.underlying_value,
            processed_data,
            spread,  // Add spread to the tuple
        ));
    }
    
    // Run rules on all securities
    let rules_outputs = rules::run_batch_rules(batch_for_rules);
    
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
        // Create empty file for consistency
        std::fs::write("batch_rules.json", "[]")?;
        println!("{} No alerts found across all securities", "ℹ".blue());
        println!("{} Created empty rules file: batch_rules.json", "✓".green());
    }
    
    println!();
    println!("{}", "=".repeat(60).blue());
    println!("{}", "Done!".green().bold());
    println!("{}", "=".repeat(60).blue());

    Ok(())
}

/// Run single security fetch (for API endpoints only - not used in GitHub Actions)
async fn run_single(symbol: &str, expiry: &str) -> Result<()> {
    println!("{}", "=".repeat(60).blue());
    println!("{}", "NSE Single Security Fetch".green().bold());
    println!("{}", "=".repeat(60).blue());
    println!();

    let client = NSEClient::new()?;

    // Determine security type (you can modify this logic)
    let security = if config::NSE_INDICES.contains(&symbol) {
        models::Security::index(symbol.to_string())
    } else {
        models::Security::equity(symbol.to_string())
    };

    println!("{} Fetching option chain for {}...", "→".cyan(), symbol.yellow());
    println!("{} Expiry: {}", "→".cyan(), expiry.yellow());
    println!();

    let chain = client.fetch_option_chain(&security, expiry).await?;

    // Display results
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

    // Process the data
    let (processed_data, spread) = processor::process_option_data(
        chain.filtered.data.clone(),
        chain.records.underlying_value
    );

    // Extract days_to_expiry from first processed option
    let days_to_expiry = processed_data.first()
        .map(|opt| opt.days_to_expiry)
        .unwrap_or(0);

    // Print results (no file saving - this will be returned as JSON in API)
    println!("{} Days to expiry: {}", "ℹ".blue(), days_to_expiry);
    
    // Run rules on processed data
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

/// Test the new futures API
async fn test_futures(symbol: &str, expiry: &str) -> Result<()> {
    println!("{}", "=".repeat(60).blue());
    println!("{}", "NSE Futures Data Test".green().bold());
    println!("{}", "=".repeat(60).blue());
    println!();

    let client = NSEClient::new()?;

    println!("{} Fetching futures data for {}...", "→".cyan(), symbol.yellow());
    println!("{} Expiry: {}", "→".cyan(), expiry.yellow());
    println!();

    match client.fetch_futures_data(symbol, expiry).await {
        Ok(data) => {
            println!("{} Futures data fetched successfully", "✓".green());
            println!("{} Data preview:", "ℹ".blue());
            println!("{}", serde_json::to_string_pretty(&data)?);
        }
        Err(e) => {
            println!("{} Failed to fetch futures data: {}", "✗".red(), e);
        }
    }

    Ok(())
}

/// Test the new derivatives historical API
async fn test_derivatives_historical(
    symbol: &str,
    instrument_type: &str,
    expiry: &str,
    from_date: &str,
    to_date: &str,
) -> Result<()> {
    println!("{}", "=".repeat(60).blue());
    println!("{}", "NSE Derivatives Historical Data Test".green().bold());
    println!("{}", "=".repeat(60).blue());
    println!();

    let client = NSEClient::new()?;
    
    // Determine security type
    let security_type = if config::NSE_INDICES.contains(&symbol) {
        models::SecurityType::Indices
    } else {
        models::SecurityType::Equity
    };

    println!("{} Fetching derivatives historical data for {}...", "→".cyan(), symbol.yellow());
    println!("{} Instrument: {}", "→".cyan(), instrument_type.yellow());
    println!("{} Expiry: {}", "→".cyan(), expiry.yellow());
    println!("{} Date range: {} to {}", "→".cyan(), from_date.yellow(), to_date.yellow());
    println!();

    match client.fetch_derivatives_historical_data(
        symbol,
        &security_type,
        instrument_type,
        None, // year
        expiry,
        None, // strike_price
        None, // option_type
        from_date,
        to_date,
    ).await {
        Ok(data) => {
            println!("{} Historical data fetched successfully", "✓".green());
            println!("{} Data preview:", "ℹ".blue());
            println!("{}", serde_json::to_string_pretty(&data)?);
        }
        Err(e) => {
            println!("{} Failed to fetch historical data: {}", "✗".red(), e);
        }
    }

    Ok(())
}

/// Run API server mode
async fn run_server(port: u16) -> Result<()> {
    println!("{}", "=".repeat(60).blue());
    println!("{}", "NSE API Server".green().bold());
    println!("{}", "=".repeat(60).blue());
    println!();

    api_server_axum::start_server(port).await
}

#[tokio::main]
async fn main() -> Result<()> {
    // ========================================
    // CONFIGURATION - Now from environment
    // ========================================
    
    let mode = config::get_execution_mode();
    
    // For single mode:
    let symbol = config::get_single_symbol();
    let expiry = config::get_single_expiry();
    
    // For test modes (new):
    let test_from_date = std::env::var("NSE_FROM_DATE")
        .unwrap_or_else(|_| "06-11-2025".to_string());
    let test_to_date = std::env::var("NSE_TO_DATE")
        .unwrap_or_else(|_| "06-12-2025".to_string());
    let test_instrument = std::env::var("NSE_INSTRUMENT")
        .unwrap_or_else(|_| "FUTURES".to_string());
    
    // For server mode:
    let port = std::env::var("NSE_PORT")
        .unwrap_or_else(|_| "3001".to_string())
        .parse::<u16>()
        .unwrap_or(3001);
    
    // Log configuration for CI environments
    if config::is_ci_environment() {
        println!("{}", "Running in CI environment (GitHub Actions)".blue());
        println!("{} Mode: {}", "→".cyan(), mode.yellow());
        if mode == "single" || mode == "server" || mode == "test-futures" || mode == "test-historical" {
            println!("{} Test/Single/Server modes not supported in CI - switching to batch", "⚠".yellow());
        }
        println!();
    }
    
    // ========================================
    
    match mode.as_str() {
        "server" => {
            if config::is_ci_environment() {
                // Force batch mode in CI
                println!("{} GitHub Actions only supports batch mode, running batch instead", "ℹ".blue());
                run_batch().await?;
            } else {
                run_server(port).await?;
            }
        }
        "batch" => run_batch().await?,
        "single" => {
            if config::is_ci_environment() {
                // Force batch mode in CI
                println!("{} GitHub Actions only supports batch mode, running batch instead", "ℹ".blue());
                run_batch().await?;
            } else {
                run_single(&symbol, &expiry).await?;
            }
        }
        "test-futures" => {
            if config::is_ci_environment() {
                // Force batch mode in CI
                println!("{} GitHub Actions only supports batch mode, running batch instead", "ℹ".blue());
                run_batch().await?;
            } else {
                test_futures(&symbol, &expiry).await?;
            }
        }
        "test-historical" => {
            if config::is_ci_environment() {
                // Force batch mode in CI
                println!("{} GitHub Actions only supports batch mode, running batch instead", "ℹ".blue());
                run_batch().await?;
            } else {
                test_derivatives_historical(&symbol, &test_instrument, &expiry, &test_from_date, &test_to_date).await?;
            }
        }
        _ => {
            if config::is_ci_environment() {
                // Force batch mode in CI
                println!("{} GitHub Actions only supports batch mode, switching to batch", "ℹ".blue());
                run_batch().await?;
            } else {
                eprintln!("Invalid mode '{}'. Use 'batch', 'single', 'server', 'test-futures', or 'test-historical'", mode);
                eprintln!("Set NSE_MODE environment variable to control execution mode");
                eprintln!("Examples:");
                eprintln!("  NSE_MODE=server NSE_PORT=3001 cargo run   # Start API server on port 3001");
                eprintln!("  NSE_MODE=batch cargo run                   # Run batch analysis");
                eprintln!("  NSE_MODE=single NSE_SYMBOL=NIFTY NSE_EXPIRY=30-Dec-2025 cargo run");
                eprintln!("  NSE_MODE=test-futures NSE_SYMBOL=NIFTY NSE_EXPIRY=30-Dec-2025 cargo run");
                eprintln!("  NSE_MODE=test-historical NSE_SYMBOL=NIFTY NSE_EXPIRY=30-Dec-2025 NSE_INSTRUMENT=FUTURES NSE_FROM_DATE=06-11-2025 NSE_TO_DATE=06-12-2025 cargo run");
                eprintln!("Note: GitHub Actions only supports 'batch' mode");
                std::process::exit(1);
            }
        }
    }

    Ok(())
}