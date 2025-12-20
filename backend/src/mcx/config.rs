use std::time::Duration;
use reqwest::{ RequestBuilder};

// -----------------------------------------------
// MCX API ENDPOINTS
// -----------------------------------------------
pub const MCX_BASE_URL: &str = "https://www.mcxindia.com";
pub const MCX_OPTION_CHAIN_PAGE: &str = "https://www.mcxindia.com/market-data/option-chain";
pub const MCX_OPTION_CHAIN_API: &str = "https://www.mcxindia.com/backpage.aspx/GetOptionChain";
pub const MCX_BHAVCOPY_API: &str = "https://www.mcxindia.com/backpage.aspx/GetDateWiseBhavCopy";
pub const MCX_HISTORIC_DATA_API: &str = "https://www.mcxindia.com/backpage.aspx/GetCommoditywiseBhavCopy";
pub const MCX_FUTURE_QUOTE_API: &str = "https://www.mcxindia.com/BackPage.aspx/GetQuote";
pub const MCX_OPTION_QUOTE_API: &str = "https://www.mcxindia.com/BackPage.aspx/GetQuoteOption";
pub const MCX_FUTURE_SYMBOLS_API: &str = "https://www.mcxindia.com/api/ContractAvailableForTrading/StaggeredProductDetailsCurrent";

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
// STANDARD HTTP HEADERS FOR MCX API CALLS
// -----------------------------------------------
pub const HEADER_ACCEPT_JSON: &str = "application/json, text/javascript, */*; q=0.01";
pub const HEADER_ACCEPT_ENCODING: &str = "gzip, deflate, br";
pub const HEADER_ACCEPT_LANGUAGE: &str = "en-US,en;q=0.9";
pub const HEADER_CACHE_CONTROL: &str = "no-cache";
pub const HEADER_PRAGMA: &str = "no-cache";
pub const HEADER_SEC_CH_UA: &str = "\"Google Chrome\";v=\"131\", \"Chromium\";v=\"131\", \"Not_A Brand\";v=\"24\"";
pub const HEADER_SEC_CH_UA_MOBILE: &str = "?0";
pub const HEADER_SEC_CH_UA_PLATFORM: &str = "\"Windows\"";
pub const HEADER_SEC_FETCH_DEST: &str = "empty";
pub const HEADER_SEC_FETCH_MODE: &str = "cors";
pub const HEADER_SEC_FETCH_SITE: &str = "same-origin";

// -----------------------------------------------
// SPECIFIC REFERER URLS
// -----------------------------------------------
pub const REFERER_OPTION_CHAIN: &str = "https://www.mcxindia.com/market-data/option-chain";
pub const REFERER_BHAVCOPY: &str = "https://www.mcxindia.com/market-data/bhavcopy";

// -----------------------------------------------
// REGEX PATTERNS FOR SCRAPING
// -----------------------------------------------
// pub const VTICK_PATTERN: &str = r"var\s+vTick\s*=\s*(\[.*?\]);";

// -----------------------------------------------
// FUTURE SYMBOLS API QUERY PARAMETERS
// -----------------------------------------------
pub const FUTURE_SYMBOLS_QUERY_PARAMS: &[(&str, &str)] = &[
    ("instrumentName", "FUTCOM"),
    ("product", "ALL"),
    ("productMonth", "ALL"),
    ("OptionType", "ALL"),
    ("StrikePrice", "ALL"),
];

// -----------------------------------------------
// HELPER FUNCTIONS FOR HTTP REQUESTS
// -----------------------------------------------

/// Apply standard MCX API headers to a POST request builder
pub fn apply_standard_post_headers(builder: RequestBuilder, referer: &str) -> RequestBuilder {
    builder
        .header("Accept", HEADER_ACCEPT_JSON)
        .header("Accept-Encoding", HEADER_ACCEPT_ENCODING)
        .header("Accept-Language", HEADER_ACCEPT_LANGUAGE)
        .header("Content-Type", HEADER_CONTENT_TYPE)
        .header("Cache-Control", HEADER_CACHE_CONTROL)
        .header("Pragma", HEADER_PRAGMA)
        .header("Referer", referer)
        .header("Sec-Ch-Ua", HEADER_SEC_CH_UA)
        .header("Sec-Ch-Ua-Mobile", HEADER_SEC_CH_UA_MOBILE)
        .header("Sec-Ch-Ua-Platform", HEADER_SEC_CH_UA_PLATFORM)
        .header("Sec-Fetch-Dest", HEADER_SEC_FETCH_DEST)
        .header("Sec-Fetch-Mode", HEADER_SEC_FETCH_MODE)
        .header("Sec-Fetch-Site", HEADER_SEC_FETCH_SITE)
        .header("X-Requested-With", HEADER_X_REQUESTED_WITH)
}

/// Apply standard MCX API headers to a GET request builder
pub fn apply_standard_get_headers(builder: RequestBuilder, referer: &str) -> RequestBuilder {
    builder
        .header("Accept", "application/json, text/plain, */*")
        .header("Accept-Encoding", HEADER_ACCEPT_ENCODING)
        .header("Accept-Language", HEADER_ACCEPT_LANGUAGE)
        .header("Cache-Control", HEADER_CACHE_CONTROL)
        .header("Pragma", HEADER_PRAGMA)
        .header("Referer", referer)
        .header("Sec-Ch-Ua", HEADER_SEC_CH_UA)
        .header("Sec-Ch-Ua-Mobile", HEADER_SEC_CH_UA_MOBILE)
        .header("Sec-Ch-Ua-Platform", HEADER_SEC_CH_UA_PLATFORM)
        .header("Sec-Fetch-Dest", HEADER_SEC_FETCH_DEST)
        .header("Sec-Fetch-Mode", HEADER_SEC_FETCH_MODE)
        .header("Sec-Fetch-Site", HEADER_SEC_FETCH_SITE)
        .header("X-Requested-With", HEADER_X_REQUESTED_WITH)
}

/// Apply headers for session establishment (visiting main pages)
pub fn apply_session_headers(builder: RequestBuilder) -> RequestBuilder {
    builder
        .header("Accept", HEADER_ACCEPT_HTML)
        .header("Accept-Encoding", HEADER_ACCEPT_ENCODING)
        .header("Accept-Language", HEADER_ACCEPT_LANGUAGE)
        .header("Sec-Ch-Ua", HEADER_SEC_CH_UA)
        .header("Sec-Ch-Ua-Mobile", HEADER_SEC_CH_UA_MOBILE)
        .header("Sec-Ch-Ua-Platform", HEADER_SEC_CH_UA_PLATFORM)
        .header("Sec-Fetch-Dest", "document")
        .header("Sec-Fetch-Mode", "navigate")
        .header("Sec-Fetch-Site", "none")
        .header("Sec-Fetch-User", "?1")
        .header("Upgrade-Insecure-Requests", "1")
}

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