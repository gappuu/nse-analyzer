# NSE Data Fetcher

A robust Rust-based NSE (National Stock Exchange) data fetcher following the official NSE API flow with proper error handling, retries, rate limiting, and logging.

## Features

✅ **Cookie/Session Management** - Automatic handling of NSE session cookies  
✅ **Rate Limiting** - Respects NSE API limits (2 requests/second)  
✅ **Automatic Retries** - Exponential backoff with jitter (max 3 retries)  
✅ **Comprehensive Logging** - File and console logging with timestamps  
✅ **Async/Concurrent** - Built on Tokio for efficient I/O  
✅ **Batch Processing** - Fetch multiple symbols with concurrency control  
✅ **3-Step API Flow** - Follows official NSE API structure  

## NSE API Flow

### Step 1: Get FNO Securities List
```
GET https://www.nseindia.com/api/master-quote
Returns: ["360ONE", "ABB", "ABCAPITAL", ...]
+ Add indices: NIFTY, BANKNIFTY, FINNIFTY
```

### Step 2: Get Contract Info
```
GET https://www.nseindia.com/api/option-chain-contract-info?symbol={ticker}
Returns: {
  "expiryDates": ["30-Dec-2025", "27-Jan-2026"],
  "strikePrice": ["1680", "1720", ...]
}
```

### Step 3: Get Option Chain
```
GET https://www.nseindia.com/api/option-chain-v3?type={type}&symbol={ticker}&expiry={expiry}
type = "Equity" or "Indices"
expiry = "30-Dec-2025" format
```

## Quick Start

```bash
# Build and run
cargo run

# With debug logs
RUST_LOG=debug cargo run
```

## Basic Usage

```rust
use nse_analyzer::{init_logging, NSEClient, Security};

#[tokio::main]
async fn main() {
    init_logging();
    let client = NSEClient::new().unwrap();
    
    // Step 1: Get securities
    let securities = client.fetch_fno_list().await.unwrap();
    
    // Step 2: Get contract info
    let info = client.fetch_contract_info("NIFTY").await.unwrap();
    
    // Step 3: Get option chain
    let security = Security::new_index("NIFTY".to_string());
    let chain = client.fetch_option_chain(&security, &info.expiry_dates[0]).await.unwrap();
}
```

## Batch Processing

```rust
let securities = client.fetch_fno_list().await?;
let results = client.fetch_option_chains_batch(securities, 3).await;
// Uses latest expiry automatically
```

See QUICKSTART.md for detailed examples.