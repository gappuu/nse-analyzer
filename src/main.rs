mod config;
mod models;
mod nse_client;
mod processor;
mod rules;

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

    // Step 2: Bulk process all securities
    println!("{}", "Step 2: Processing all securities...".cyan());
    println!("{} Max concurrent requests: {}", "ℹ".blue(), config::DEFAULT_MAX_CONCURRENT);
    println!();

    let start_time = std::time::Instant::now();
    
    let results = client.fetch_all_option_chains(
        securities.clone(),
        config::DEFAULT_MAX_CONCURRENT,
    )
    .await;

    let elapsed = start_time.elapsed();
    
    // Step 3: Process results
    let mut successful = Vec::new();
    let mut failed = Vec::new();

    for (security, result) in securities.iter().zip(results.iter()) {
        match result {
            Ok((_, chain)) => {
                successful.push((security.clone(), chain.clone()));
                print!("{}", ".".green());
            }
            Err(e) => {
                failed.push((security.symbol.clone(), e.to_string()));
                print!("{}", "✗".red());
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
    println!("{} Time taken: {:.2}s", "⏱".yellow(), elapsed.as_secs_f64());
    println!("{} Avg time per security: {:.2}s", "⏱".yellow(), elapsed.as_secs_f64() / securities.len() as f64);
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
        
        // Store for JSON output
        processed_batch.push(serde_json::json!({
            "record": {
                "symbol": security.symbol,
                "timestamp": chain.records.timestamp,
                "underlying_value": chain.records.underlying_value,
                "spread": spread,
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
    
    // Save processed data
    let output = serde_json::json!(processed_batch);
    std::fs::write(
        "output.json",
        serde_json::to_string_pretty(&output)?,
    )?;
    println!("{} Saved {} securities to output.json", "✓".green(), successful.len());
    
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
        println!("{} No alerts found across all securities", "ℹ".blue());
    }
    
    println!("{} Saved rules to batch_rules.json", "✓".green());
    // println!("{} Total alerts across all securities: {}", "ℹ".blue(), total_alerts);
    
    println!();
    println!("{}", "=".repeat(60).blue());
    println!("{}", "Done!".green().bold());
    println!("{}", "=".repeat(60).blue());

    Ok(())
}

/// Run single security fetch
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

    // Save to JSON
    let data_single = serde_json::json!({
        "record": {
            "timestamp": chain.records.timestamp,
            "underlying_value": chain.records.underlying_value,
            "spread": spread,
            "expiry": expiry,
            "symbol": symbol,
            "ce_oi": chain.filtered.ce_totals.total_oi,
            "pe_oi": chain.filtered.pe_totals.total_oi,
        },
        "data": processed_data,
    });

    std::fs::write(
        "single_output.json",
        serde_json::to_string_pretty(&data_single)?,
    )?;
    
    println!("{} Data saved to single_output.json", "✓".green());
    
    // Run rules on processed data - now pass spread parameter
    let rules_output = rules::run_rules(
        &processed_data,
        symbol.to_string(),
        chain.records.timestamp.clone(),
        chain.records.underlying_value,
        spread,  // Pass the spread value
    );
    
    if let Some(output) = rules_output {
        std::fs::write(
            "single_rules.json",
            serde_json::to_string_pretty(&output)?,
        )?;
        
        println!("{} Saved rules to single_rules.json", "✓".green());
        println!("{} Total alerts: {}", "ℹ".blue(), output.alerts.len());
    } else {
        println!("{} No alerts found", "ℹ".blue());
    }
    println!("{}", "=".repeat(60).blue());

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // ========================================
    // CONFIGURATION - EDIT THIS SECTION
    // ========================================
    
    let mode = "batch"; // Change to "batch" or "single"
    
    // For single mode:
    let symbol = "COALINDIA";
    let expiry = "30-Dec-2025";
    
    // ========================================
    
    match mode {
        "batch" => run_batch().await?,
        "single" => run_single(symbol, expiry).await?,
        _ => {
            eprintln!("Invalid mode. Use 'batch' or 'single'");
            std::process::exit(1);
        }
    }

    Ok(())
}