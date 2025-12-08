mod config;
mod models;
mod nse_client;

use anyhow::Result;
use colored::Colorize;
use nse_client::NSEClient;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    println!("{}", "=".repeat(60).blue());
    println!("{}", "NSE Bulk Processor".green().bold());
    println!("{}", "=".repeat(60).blue());
    println!();

    // Build NSE client
    let client = Arc::new(NSEClient::new()?);

    // Step 1: Fetch all FNO securities
    println!("{}", "Step 1: Fetching all FNO securities...".cyan());
    let securities = client.fetch_fno_list().await?;
    println!("{} Found {} securities", "✓".green(), securities.len());
    println!();

    // Example 1
    // Step 2: Bulk process all securities
    println!("{}", "Step 2: Processing all securities (this may take a while)...".cyan());
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
                print!("{}", ".".green()); // Progress indicator
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
                chain.records.data.len(),
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

    // Step 5: Save to JSON (optional)
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
                "strikes_count": chain.records.data.len(),
                // Add your analysis here:
                // "pcr": calculate_pcr(chain),
                // "max_pain": calculate_max_pain(chain),
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

    // Example 2: Fetch contract info for one symbol
    // println!("{}", "Step 2: Fetching contract info for M&M...".cyan());
    // let contract_info = client.fetch_contract_info("M&M").await?;
    // println!("{} Expiries: {:?}", "✓".green(), &contract_info.expiry_dates[..3.min(contract_info.expiry_dates.len())]);
    // println!();

    // // Example 3: Fetch option chain for one symbol
    // println!("{}", "Step 3: Fetching option chain for M&M...".cyan());
    // let ticker = models::Security::index("M&M".to_string());
    // let expiry = &contract_info.expiry_dates[0]; // Nearest expiry
    // let chain = client.fetch_option_chain(&ticker, expiry).await?;
    // println!("{} Underlying: {}", "✓".green(), chain.records.underlying_value);
    // println!("{} Total strikes: {}", "✓".green(), chain.records.data.len());
    // println!();

    Ok(())
}