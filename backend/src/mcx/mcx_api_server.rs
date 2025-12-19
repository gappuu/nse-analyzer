use super::config;
use super::models::{Ticker, OptionChainResponse};
use super::mcx_client::MCXClient;
use anyhow::Result;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;

// -----------------------------------------------
// API REQUEST/RESPONSE MODELS
// -----------------------------------------------

#[derive(Debug, Deserialize)]
pub struct TickerListQuery {
    // No parameters needed for ticker list
}

#[derive(Debug, Deserialize)]
pub struct OptionChainQuery {
    pub commodity: String,
    pub expiry: String,
}

#[derive(Debug, Deserialize)]
pub struct OptionQuoteQuery {
    pub commodity: String,
    pub expiry: String,
}

#[derive(Debug, Deserialize)]
pub struct HistoricDataQuery {
    pub symbol: String,
    pub expiry: String,
    pub from_date: String,  // Format: YYYYMMDD
    pub to_date: String,    // Format: YYYYMMDD
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub processing_time_ms: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct TickerListResponse {
    pub tickers: Vec<Ticker>,
    pub unique_symbols: Vec<String>,
    pub total_tickers: usize,
}

#[derive(Debug, Serialize)]
pub struct BatchAnalysisResponse {
    pub summary: BatchSummary,
    pub results: Vec<BatchResult>,
}

#[derive(Debug, Serialize)]
pub struct BatchSummary {
    pub total_tickers: usize,
    pub total_unique_symbols: usize,
    pub filtered_latest_expiry: usize,
    pub successful: usize,
    pub failed: usize,
    pub processing_time_ms: u64,
}

#[derive(Debug, Serialize)]
pub struct BatchResult {
    pub ticker: Ticker,
    pub option_chain: Option<OptionChainResponse>,
    pub error: Option<String>,
}

// -----------------------------------------------
// APPLICATION STATE
// -----------------------------------------------

#[derive(Clone)]
pub struct AppState {
    client: Arc<MCXClient>,
    cache: Arc<RwLock<Cache>>,
}

#[derive(Default)]
struct Cache {
    ticker_list: Option<(Vec<Ticker>, Instant)>,
    option_chains: HashMap<String, (OptionChainResponse, Instant)>,
    future_quotes: HashMap<String, (serde_json::Value, Instant)>,
    future_symbols: Option<(serde_json::Value, Instant)>,
    historic_data: HashMap<String, (serde_json::Value, Instant)>,
}

const CACHE_DURATION: Duration = Duration::from_secs(300); // 5 minutes

impl AppState {
    pub fn new() -> Result<Self> {
        Ok(Self {
            client: Arc::new(MCXClient::new()?),
            cache: Arc::new(RwLock::new(Cache::default())),
        })
    }
}

// -----------------------------------------------
// API HANDLERS
// -----------------------------------------------

/// GET /health - Health check endpoint
async fn health() -> &'static str {
    "OK"
}

/// GET /api/mcx/tickers - Get all available MCX tickers
async fn get_ticker_list(State(app_state): State<AppState>) -> Result<Json<ApiResponse<TickerListResponse>>, StatusCode> {
    let start_time = Instant::now();

    // Check cache first
    {
        let cache = app_state.cache.read().await;
        if let Some((tickers, cached_at)) = &cache.ticker_list {
            if cached_at.elapsed() < CACHE_DURATION {
                let unique_symbols = MCXClient::get_unique_symbols(tickers);
                return Ok(Json(ApiResponse {
                    success: true,
                    data: Some(TickerListResponse {
                        total_tickers: tickers.len(),
                        unique_symbols: unique_symbols.clone(),
                        tickers: tickers.clone(),
                    }),
                    error: None,
                    processing_time_ms: Some(start_time.elapsed().as_millis() as u64),
                }));
            }
        }
    }

    // Fetch from web scraping
    match app_state.client.fetch_ticker_list().await {
        Ok(tickers) => {
            let unique_symbols = MCXClient::get_unique_symbols(&tickers);
            
            // Update cache
            {
                let mut cache = app_state.cache.write().await;
                cache.ticker_list = Some((tickers.clone(), Instant::now()));
            }

            Ok(Json(ApiResponse {
                success: true,
                data: Some(TickerListResponse {
                    total_tickers: tickers.len(),
                    unique_symbols,
                    tickers,
                }),
                error: None,
                processing_time_ms: Some(start_time.elapsed().as_millis() as u64),
            }))
        }
        Err(e) => Ok(Json(ApiResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
            processing_time_ms: Some(start_time.elapsed().as_millis() as u64),
        })),
    }
}

/// GET /api/mcx/option-chain?commodity=COPPER&expiry=23DEC2025 - Get option chain for specific commodity and expiry
async fn get_option_chain(
    Query(query): Query<OptionChainQuery>,
    State(app_state): State<AppState>,
) -> Result<Json<ApiResponse<OptionChainResponse>>, StatusCode> {
    let start_time = Instant::now();
    let cache_key = format!("{}_{}", query.commodity, query.expiry);

    // Check cache first
    {
        let cache = app_state.cache.read().await;
        if let Some((option_chain, cached_at)) = cache.option_chains.get(&cache_key) {
            if cached_at.elapsed() < CACHE_DURATION {
                return Ok(Json(ApiResponse {
                    success: true,
                    data: Some(option_chain.clone()),
                    error: None,
                    processing_time_ms: Some(start_time.elapsed().as_millis() as u64),
                }));
            }
        }
    }

    // Fetch from API
    match app_state.client.fetch_option_chain(&query.commodity, &query.expiry).await {
        Ok(option_chain) => {
            // Update cache
            {
                let mut cache = app_state.cache.write().await;
                cache.option_chains.insert(
                    cache_key,
                    (option_chain.clone(), Instant::now()),
                );
            }

            Ok(Json(ApiResponse {
                success: true,
                data: Some(option_chain),
                error: None,
                processing_time_ms: Some(start_time.elapsed().as_millis() as u64),
            }))
        }
        Err(e) => Ok(Json(ApiResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
            processing_time_ms: Some(start_time.elapsed().as_millis() as u64),
        })),
    }
}

/// POST /api/mcx/batch-analysis - Run batch analysis for latest expiry per symbol only
async fn run_batch_analysis(
    State(app_state): State<AppState>,
) -> Result<Json<ApiResponse<BatchAnalysisResponse>>, StatusCode> {
    let start_time = Instant::now();

    // Step 1: Fetch all MCX tickers
    let all_tickers = match app_state.client.fetch_ticker_list().await {
        Ok(tickers) => tickers,
        Err(e) => {
            return Ok(Json(ApiResponse {
                success: false,
                data: None,
                error: Some(format!("Failed to fetch ticker list: {}", e)),
                processing_time_ms: Some(start_time.elapsed().as_millis() as u64),
            }));
        }
    };

    let total_tickers = all_tickers.len();
    let unique_symbols = MCXClient::get_unique_symbols(&all_tickers);
    let total_unique_symbols = unique_symbols.len();

    // Step 2: Filter to only latest expiry per symbol
    let filtered_tickers = MCXClient::filter_latest_expiry_per_symbol(all_tickers);
    let filtered_count = filtered_tickers.len();

    println!("📊 Batch Analysis Summary:");
    println!("   Total tickers from MCX: {}", total_tickers);
    println!("   Unique symbols: {}", total_unique_symbols);
    println!("   Latest expiry filtered: {}", filtered_count);
    println!("   Reduction: {:.1}%", 
             (1.0 - (filtered_count as f64 / total_tickers as f64)) * 100.0);

    let max_concurrent = if config::is_ci_environment() {
        config::CI_MAX_CONCURRENT
    } else {
        config::DEFAULT_MAX_CONCURRENT
    };

    // Step 3: Bulk process filtered tickers (only latest expiry per symbol)
    let results = app_state.client.clone()
        .fetch_all_option_chains(filtered_tickers.clone(), max_concurrent)
        .await;

    // Step 4: Process results
    let mut batch_results = Vec::new();
    let mut successful_count = 0;
    let mut failed_count = 0;

    for (ticker, result) in filtered_tickers.iter().zip(results.iter()) {
        match result {
            Ok((_, chain)) => {
                successful_count += 1;
                batch_results.push(BatchResult {
                    ticker: ticker.clone(),
                    option_chain: Some(chain.clone()),
                    error: None,
                });
            }
            Err(e) => {
                failed_count += 1;
                batch_results.push(BatchResult {
                    ticker: ticker.clone(),
                    option_chain: None,
                    error: Some(e.to_string()),
                });
            }
        }
    }

    let summary = BatchSummary {
        total_tickers,
        total_unique_symbols,
        filtered_latest_expiry: filtered_count,
        successful: successful_count,
        failed: failed_count,
        processing_time_ms: start_time.elapsed().as_millis() as u64,
    };

    println!("✅ Batch analysis completed:");
    println!("   Successful: {}/{}", successful_count, filtered_count);
    println!("   Failed: {}/{}", failed_count, filtered_count);
    println!("   Processing time: {}ms", summary.processing_time_ms);

    Ok(Json(ApiResponse {
        success: true,
        data: Some(BatchAnalysisResponse {
            summary,
            results: batch_results,
        }),
        error: None,
        processing_time_ms: Some(start_time.elapsed().as_millis() as u64),
    }))
}

/// GET /api/mcx/future-quote?commodity=ALUMINI&expiry=31DEC2025 - Get option quote for specific commodity and expiry
async fn get_future_quote(
    Query(query): Query<OptionQuoteQuery>,
    State(app_state): State<AppState>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    let start_time = Instant::now();
    let cache_key = format!("quote_{}_{}", query.commodity, query.expiry);

    // Check cache first
    {
        let cache = app_state.cache.read().await;
        if let Some((quote_data, cached_at)) = cache.future_quotes.get(&cache_key) {
            if cached_at.elapsed() < CACHE_DURATION {
                return Ok(Json(ApiResponse {
                    success: true,
                    data: Some(quote_data.clone()),
                    error: None,
                    processing_time_ms: Some(start_time.elapsed().as_millis() as u64),
                }));
            }
        }
    }

    // Fetch from MCX API
    match app_state.client.fetch_future_quote(&query.commodity, &query.expiry).await {
        Ok(quote_data) => {
            // Update cache
            {
                let mut cache = app_state.cache.write().await;
                cache.future_quotes.insert(
                    cache_key,
                    (quote_data.clone(), Instant::now()),
                );
            }

            Ok(Json(ApiResponse {
                success: true,
                data: Some(quote_data),
                error: None,
                processing_time_ms: Some(start_time.elapsed().as_millis() as u64),
            }))
        }
        Err(e) => Ok(Json(ApiResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
            processing_time_ms: Some(start_time.elapsed().as_millis() as u64),
        })),
    }
}

/// GET /api/mcx/future-symbols - Get available future symbols and expiry dates
async fn get_future_symbols(State(app_state): State<AppState>) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    let start_time = Instant::now();

    // Check cache first
    {
        let cache = app_state.cache.read().await;
        if let Some((data, cached_at)) = &cache.future_symbols {
            if cached_at.elapsed() < CACHE_DURATION {
                return Ok(Json(ApiResponse {
                    success: true,
                    data: Some(data.clone()),
                    error: None,
                    processing_time_ms: Some(start_time.elapsed().as_millis() as u64),
                }));
            }
        }
    }

    // Fetch from MCX API
    match app_state.client.fetch_future_symbols().await {
        Ok(data) => {
            // Update cache
            {
                let mut cache = app_state.cache.write().await;
                cache.future_symbols = Some((data.clone(), Instant::now()));
            }

            Ok(Json(ApiResponse {
                success: true,
                data: Some(data),
                error: None,
                processing_time_ms: Some(start_time.elapsed().as_millis() as u64),
            }))
        }
        Err(e) => Ok(Json(ApiResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
            processing_time_ms: Some(start_time.elapsed().as_millis() as u64),
        })),
    }
}

/// GET /api/mcx/historic-data?symbol=COPPER&expiry=23DEC2025&from_date=20251215&to_date=20251219 - Get historic data
async fn get_historic_data(
    Query(query): Query<HistoricDataQuery>,
    State(app_state): State<AppState>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    let start_time = Instant::now();
    let cache_key = format!("historic_{}_{}_{}_{}", query.symbol, query.expiry, query.from_date, query.to_date);

    // Check cache first
    {
        let cache = app_state.cache.read().await;
        if let Some((data, cached_at)) = cache.historic_data.get(&cache_key) {
            if cached_at.elapsed() < CACHE_DURATION {
                return Ok(Json(ApiResponse {
                    success: true,
                    data: Some(data.clone()),
                    error: None,
                    processing_time_ms: Some(start_time.elapsed().as_millis() as u64),
                }));
            }
        }
    }

    // Fetch from MCX API
    match app_state.client.fetch_historic_data(&query.symbol, &query.expiry, &query.from_date, &query.to_date).await {
        Ok(data) => {
            // Update cache
            {
                let mut cache = app_state.cache.write().await;
                cache.historic_data.insert(
                    cache_key,
                    (data.clone(), Instant::now()),
                );
            }

            Ok(Json(ApiResponse {
                success: true,
                data: Some(data),
                error: None,
                processing_time_ms: Some(start_time.elapsed().as_millis() as u64),
            }))
        }
        Err(e) => Ok(Json(ApiResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
            processing_time_ms: Some(start_time.elapsed().as_millis() as u64),
        })),
    }
}

// -----------------------------------------------
// SERVER SETUP
// -----------------------------------------------

pub async fn start_mcx_server(port: u16) -> Result<()> {
    let app_state = AppState::new()?;

    let mcx_routes = Router::new()
        .route("/api/mcx/tickers", get(get_ticker_list))
        .route("/api/mcx/option-chain", get(get_option_chain))
        .route("/api/mcx/future-quote", get(get_future_quote))
        .route("/api/mcx/batch-analysis", post(run_batch_analysis))
        .route("/api/mcx/future-symbols", get(get_future_symbols))
        .route("/api/mcx/historic-data", get(get_historic_data));

    let app = Router::new()
        .route("/health", get(health))
        .merge(mcx_routes)
        .layer(CorsLayer::permissive())
        .with_state(app_state);

    let addr = format!("127.0.0.1:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    
    println!("🚀 MCX API Server running on http://{}", addr);
    println!("📋 Available MCX endpoints:");
    println!("   GET  /health");
    println!("   GET  /api/mcx/tickers");
    println!("   GET  /api/mcx/option-chain?commodity=COPPER&expiry=23DEC2025");
    println!("   GET  /api/mcx/future-quote?commodity=ALUMINI&expiry=31DEC2025");
    println!("   POST /api/mcx/batch-analysis (Latest Expiry Only)");
    println!("   GET  /api/mcx/future-symbols");
    println!("   GET  /api/mcx/historic-data?symbol=COPPER&expiry=23DEC2025&from_date=20251215&to_date=20251219");
    println!();

    axum::serve(listener, app).await?;
    Ok(())
}

/// Get MCX routes to be merged with existing server
pub fn get_mcx_routes() -> Router<AppState> {
    Router::new()
        .route("/api/mcx/tickers", get(get_ticker_list))
        .route("/api/mcx/option-chain", get(get_option_chain))
        .route("/api/mcx/future-quote", get(get_future_quote))
        .route("/api/mcx/batch-analysis", post(run_batch_analysis))
        .route("/api/mcx/future-symbols", get(get_future_symbols))
        .route("/api/mcx/historic-data", get(get_historic_data))
}

/// Get MCX app state for merging with existing server
pub fn get_mcx_app_state() -> Result<AppState> {
    AppState::new()
}