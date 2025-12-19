use std::time::Duration;

// -----------------------------------------------
// MCX API ENDPOINTS
// -----------------------------------------------
pub const MCX_BASE_URL: &str = "https://www.mcxindia.com";
pub const MCX_OPTION_CHAIN_PAGE: &str = "https://www.mcxindia.com/market-data/option-chain";
pub const MCX_OPTION_CHAIN_API: &str = "https://www.mcxindia.com/backpage.aspx/GetOptionChain";

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

pub const HTTP_TIMEOUT: Duration = Duration::from_secs(30);

// -----------------------------------------------
// SESSION WARMUP
// -----------------------------------------------
pub const WARMUP_DELAY_MS: u64 = 300;

// -----------------------------------------------
// RETRY CONFIG
// -----------------------------------------------
pub const RETRY_BASE_DELAY_MS: u64 = 300;
pub const RETRY_FACTOR: u64 = 2;
pub const RETRY_MAX_DELAY_SECS: u64 = 10;
pub const RETRY_MAX_ATTEMPTS: usize = 3;

// -----------------------------------------------
// GITHUB ACTIONS TIMEOUT CONFIG
// -----------------------------------------------
pub const GITHUB_ACTIONS_TIMEOUT_SECS: u64 = 300;  // 5 minute timeout for CI

// -----------------------------------------------
// CONCURRENCY LIMITS
// -----------------------------------------------
pub const DEFAULT_MAX_CONCURRENT: usize = 3;
pub const CI_MAX_CONCURRENT: usize = 2;

// -----------------------------------------------
// HTTP HEADERS
// -----------------------------------------------
pub const HEADER_REFERER: &str = "https://www.mcxindia.com/";
pub const HEADER_X_REQUESTED_WITH: &str = "XMLHttpRequest";
pub const HEADER_ACCEPT_HTML: &str = "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8";
pub const HEADER_CONTENT_TYPE: &str = "application/json; charset=utf-8";

// -----------------------------------------------
// REGEX PATTERNS FOR SCRAPING
// -----------------------------------------------
pub const VTICK_PATTERN: &str = r"var\s+vTick\s*=\s*(\[.*?\]);";

// -----------------------------------------------
// RUNTIME CONFIGURATION
// -----------------------------------------------

/// Check if running in CI/automated environment
pub fn is_ci_environment() -> bool {
    std::env::var("CI").is_ok() || std::env::var("GITHUB_ACTIONS").is_ok()
}

/// Get symbol for single mode execution
pub fn get_single_symbol() -> String {
    std::env::var("MCX_SYMBOL").unwrap_or_else(|_| "COPPER".to_string())
}

/// Get expiry for single mode execution  
pub fn get_single_expiry() -> String {
    std::env::var("MCX_EXPIRY").unwrap_or_else(|_| "23DEC2025".to_string())
}