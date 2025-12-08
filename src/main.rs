mod models;
mod nse_client;

use anyhow::Result;
use colored::Colorize;
use nse_client::NSEClient;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    println!("{}", "=".repeat(60).blue());
    println!("{}", "NSE Data Fetcher - Clean & Simple".green().bold());
    println!("{}", "=".repeat(60).blue());
    println!();

    // Build NSE client (warmup handled automatically)
    let client = Arc::new(NSEClient::new()?);

    // Example 1: Fetch FNO list
    println!("{}", "Step 1: Fetching FNO Securities...".cyan());
    let securities = client.fetch_fno_list().await?;
    println!("{} Found {} securities", "✓".green(), securities.len());
    println!();

    // Example 2: Fetch contract info for one symbol
    println!("{}", "Step 2: Fetching contract info for NIFTY...".cyan());
    let contract_info = client.fetch_contract_info("NIFTY").await?;
    println!("{} Expiries: {:?}", "✓".green(), &contract_info.expiry_dates[..3.min(contract_info.expiry_dates.len())]);
    println!();

    // Example 3: Fetch option chain
    println!("{}", "Step 3: Fetching option chain for NIFTY...".cyan());
    let nifty = models::Security::index("NIFTY".to_string());
    let expiry = &contract_info.expiry_dates[0]; // Nearest expiry
    let chain = client.fetch_option_chain(&nifty, expiry).await?;
    println!("{} Underlying: {}", "✓".green(), chain.records.underlying_value);
    println!("{} Total strikes: {}", "✓".green(), chain.records.data.len());
    println!();

    // Example 4: Batch fetch multiple symbols
    println!("{}", "Batch: Fetching multiple securities...".cyan());
    let test_securities = vec![
        models::Security::equity("RELIANCE".to_string()),
        models::Security::equity("TCS".to_string()),
        models::Security::index("BANKNIFTY".to_string()),
    ];

    let results = client.fetch_all_option_chains(
        test_securities,
        3, // max 3 concurrent
    )
    .await;

    for result in results {
        match result {
            Ok((security, chain)) => {
                println!(
                    "{} {} → {} strikes, underlying: {}",
                    "✓".green(),
                    security.symbol.yellow(),
                    chain.records.data.len(),
                    chain.records.underlying_value
                );
            }
            Err(e) => {
                println!("{} Error: {}", "✗".red(), e);
            }
        }
    }

    println!();
    println!("{}", "=".repeat(60).blue());
    println!("{}", "Done!".green().bold());
    Ok(())
}