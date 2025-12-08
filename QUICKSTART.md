# Quick Start Guide

## Test the Fetcher

```bash
cd nse-analyzer

# Build
cargo build

# Run examples
cargo run

# Debug mode
RUST_LOG=debug cargo run
```

## The 3-Step NSE API Flow

### Step 1: Get All FNO Securities

```rust
let client = NSEClient::new()?;
let securities = client.fetch_fno_list().await?;

// Returns stocks from master-quote + indices (NIFTY, BANKNIFTY, FINNIFTY)
// Output: Vec<Security>
```

### Step 2: Get Contract Info

```rust
let contract_info = client.fetch_contract_info("NIFTY").await?;

println!("Expiry dates: {:?}", contract_info.expiry_dates);
println!("Strike prices: {:?}", contract_info.strike_prices);
```

### Step 3: Get Option Chain

```rust
let security = Security::new_index("NIFTY".to_string());
let expiry = &contract_info.expiry_dates[0]; // Latest expiry

let option_chain = client.fetch_option_chain(&security, expiry).await?;

println!("Underlying: {}", option_chain.records.underlying_value);
println!("Timestamp: {}", option_chain.records.timestamp);
```

## Access Option Data

```rust
for option in &option_chain.records.data {
    println!("Strike: {}", option.strike_price);
    
    // Call data
    if let Some(call) = &option.call {
        println!("  CALL OI: {}", call.open_interest);
        println!("  CALL Change: {}", call.change_in_open_interest);
        println!("  CALL Price: {}", call.last_price);
        println!("  CALL IV: {}", call.implied_volatility);
    }
    
    // Put data
    if let Some(put) = &option.put {
        println!("  PUT OI: {}", put.open_interest);
        println!("  PUT Change: {}", put.change_in_open_interest);
        println!("  PUT Price: {}", put.last_price);
        println!("  PUT IV: {}", put.implied_volatility);
    }
}
```

## Batch Processing (for GitHub Actions)

```rust
// Get all securities
let securities = client.fetch_fno_list().await?;

// Batch fetch with max 3 concurrent (automatically uses latest expiry)
let results = client.fetch_option_chains_batch(securities, 3).await;

for result in results {
    match result {
        Ok((security, option_chain)) => {
            // Apply your analysis rules here
            println!("{}: {} strikes", 
                     security.symbol, 
                     option_chain.records.data.len());
        }
        Err(e) => eprintln!("Error: {:?}", e),
    }
}
```

## Calculate PCR (Put-Call Ratio)

```rust
fn calculate_pcr(option_chain: &OptionChainResponse) -> f64 {
    let mut total_put_oi = 0.0;
    let mut total_call_oi = 0.0;
    
    for option in &option_chain.records.data {
        if let Some(call) = &option.call {
            total_call_oi += call.open_interest;
        }
        if let Some(put) = &option.put {
            total_put_oi += put.open_interest;
        }
    }
    
    total_put_oi / total_call_oi
}

let pcr = calculate_pcr(&option_chain);
println!("PCR: {:.2}", pcr);
```

## What's Logged

All operations are automatically logged:

**Console:**
```
INFO  Step 1: Fetching FNO Securities
INFO  Successfully fetched 180 securities
INFO  Step 2: Fetching contract info for: NIFTY
INFO  Step 3: Fetching option chain
```

**File** (`./logs/nse-analyzer.log`):
```json
{"timestamp":"2024-12-08T10:30:45Z","level":"INFO","message":"Fetching option chain for: NIFTY"}
```

## Next Steps

1. ✅ **Add Analysis Logic** - PCR, OI buildup, support/resistance
2. ✅ **Create Web API** - Axum + HTMX for user interface
3. ✅ **GitHub Actions** - Scheduled batch processing

Check the main README for full documentation.