mod config;
mod models;
mod nse_client;

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

    // Show sample successful results
    if !successful.is_empty() {
        println!("{}", "Sample Results (first 5):".cyan());
        for (security, chain) in successful.iter().take(5) {
            println!(
                "  {} {} → {} strikes, underlying: {:.2}",
                "✓".green(),
                security.symbol.yellow(),
                chain.filtered.data.len(),
                chain.records.underlying_value
            );
        }
        println!();
    }

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

    // Step 5: Save to JSON
    println!("{}", "Saving results to output.json...".cyan());
    let output: Vec<serde_json::Value> = successful
        .iter()
        .map(|(security, chain)| {
            serde_json::json!({
                "symbol": security.symbol,
                "type": match security.security_type {
                    models::SecurityType::Equity => "Equity",
                    models::SecurityType::Indices => "Indices",
                },
                "underlying": chain.records.underlying_value,
                "timestamp": chain.records.timestamp,
                "strikes_count": chain.filtered.data.len(),
                "data": chain.filtered.data,
            })
        })
        .collect();

    std::fs::write(
        "output.json",
        serde_json::to_string_pretty(&output)?,
    )?;
    println!("{} Saved {} securities to output.json", "✓".green(), successful.len());
    
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
    
    println!("{} Total strikes: {}", "✓".green(), chain.filtered.data.len());
    println!("{} Total CE OI: {:.0}", "✓".green(), chain.filtered.ce_totals.total_oi);
    println!("{} Total PE OI: {:.0}", "✓".green(), chain.filtered.pe_totals.total_oi);
    println!("{} Total CE Volume: {:.0}", "✓".green(), chain.filtered.ce_totals.total_volume);
    println!("{} Total PE Volume: {:.0}", "✓".green(), chain.filtered.pe_totals.total_volume);
    println!();

    // Save to JSON
    let output = serde_json::json!({
        "symbol": symbol,
        "type": match security.security_type {
            models::SecurityType::Equity => "Equity",
            models::SecurityType::Indices => "Indices",
        },
        "timestamp": chain.records.timestamp,
        "underlying_value": chain.records.underlying_value,
        "expiry": expiry,
        "totals": {
            "ce_oi": chain.filtered.ce_totals.total_oi,
            "pe_oi": chain.filtered.pe_totals.total_oi,
            "ce_volume": chain.filtered.ce_totals.total_volume,
            "pe_volume": chain.filtered.pe_totals.total_volume,
        },
        "data": chain.filtered.data,
    });

    std::fs::write(
        "single_output.json",
        serde_json::to_string_pretty(&output)?,
    )?;
    
    println!("{} Saved to single_output.json", "✓".green());
    println!("{}", "=".repeat(60).blue());

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // ========================================
    // CONFIGURATION - EDIT THIS SECTION
    // ========================================
    
    let mode = "single"; // Change to "batch" or "single"
    
    // For single mode:
    let symbol = "360ONE";
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