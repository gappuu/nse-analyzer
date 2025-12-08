use std::time::Duration;

// -----------------------------------------------
// NSE API ENDPOINTS
// -----------------------------------------------
pub const NSE_BASE_URL: &str = "https://www.nseindia.com";
pub const NSE_API_MASTER_QUOTE: &str = "https://www.nseindia.com/api/master-quote";

pub fn nse_contract_info_url(symbol: &str) -> String {
    format!(
        "{}/api/option-chain-contract-info?symbol={}",
        NSE_BASE_URL,
        urlencoding::encode(symbol) // URL-encode the symbol
    )
}

pub fn nse_option_chain_url(typ: &str, symbol: &str, expiry: &str) -> String {
    format!(
        "{}/api/option-chain-v3?type={}&symbol={}&expiry={}",
        NSE_BASE_URL,
        typ,
        urlencoding::encode(symbol), // URL-encode the symbol
        urlencoding::encode(expiry)  // URL-encode the expiry (just in case)
    )
}

// -----------------------------------------------
// INDICES TO ADD
// -----------------------------------------------
pub const NSE_INDICES: &[&str] = &["NIFTY", "BANKNIFTY", "FINNIFTY"];

// -----------------------------------------------
// HTTP CLIENT CONFIG
// -----------------------------------------------
pub const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) \
                               AppleWebKit/537.36 (KHTML, like Gecko) \
                               Chrome/131.0.0.0 Safari/537.36";

pub const ACCEPT_LANGUAGES: &[&str] = &[
    "en-US,en;q=0.9",
    "en-GB,en;q=0.8",
    "en-IN,en;q=0.9",
];

pub const HTTP_TIMEOUT: Duration = Duration::from_secs(20);

// -----------------------------------------------
// SESSION WARMUP
// -----------------------------------------------
pub const WARMUP_DELAY_MS: u64 = 200;

// -----------------------------------------------
// RETRY CONFIG
// -----------------------------------------------
pub const RETRY_BASE_DELAY_MS: u64 = 200;
pub const RETRY_FACTOR: u64 = 3;
pub const RETRY_MAX_DELAY_SECS: u64 = 5;
pub const RETRY_MAX_ATTEMPTS: usize = 5;

// -----------------------------------------------
// CONCURRENCY LIMITS
// -----------------------------------------------
pub const DEFAULT_MAX_CONCURRENT: usize = 5;

// -----------------------------------------------
// RATE LIMITING (if needed)
// -----------------------------------------------
// Uncomment and adjust if you add rate limiting
// pub const RATE_LIMIT_PER_SECOND: u32 = 2;

// -----------------------------------------------
// HTTP HEADERS
// -----------------------------------------------
pub const HEADER_REFERER: &str = "https://www.nseindia.com/";
pub const HEADER_X_REQUESTED_WITH: &str = "XMLHttpRequest";
pub const HEADER_ACCEPT_HTML: &str = "text/html";