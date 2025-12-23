use super::config;
use super::models::{Ticker, OptionChainResponse};
use super::mcx_client::MCXClient;
use super::processor;
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
use chrono::NaiveDate;

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
pub struct SpecificOptionQuoteQuery {
    pub commodity: String,
    pub expiry: String,
    pub option_type: String,  // CE or PE
    pub strike_price: String, // e.g., "1120.00"
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
    pub rules_output: Vec<super::rules::McxRulesOutput>,
}

#[derive(Debug, Serialize)]
pub struct BatchSummary {
    pub total_tickers: usize,
    pub total_unique_symbols: usize,
    pub filtered_latest_expiry: usize,
    pub successful: usize,
    pub failed: usize,
    pub securities_with_alerts: usize,
    pub total_alerts: usize,
    pub processing_time_ms: u64,
}

#[derive(Debug, Serialize)]
pub struct BatchResult {
    pub ticker: Ticker,
    pub rules_output: Option<super::rules::McxRulesOutput>,
    pub error: Option<String>,
}

// New structs for future symbols integration
#[derive(Debug, Deserialize)]
pub struct FutureSymbolsResponse {
    #[serde(rename = "InstrumentName")]
    pub instrument_name: String,
    #[serde(rename = "Products")]
    pub products: Vec<ProductData>,
}

#[derive(Debug, Deserialize)]
pub struct ProductData {
    #[serde(rename = "Product")]
    pub product: String,
    #[serde(rename = "ExpiryDates")]
    pub expiry_dates: Vec<String>,
}

// Enhanced response structure to include latest expiry
#[derive(Debug, Serialize)]
pub struct EnhancedSingleAnalysisResponse {
    #[serde(flatten)]
    pub analysis: processor::McxSingleAnalysisResponse,
    pub latest_future_expiry: Option<String>,
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
    option_quotes: HashMap<String, (serde_json::Value, Instant)>,
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
// HELPER FUNCTIONS
// -----------------------------------------------

// Helper function to parse date from "DD-MMM-YYYY" format
fn parse_expiry_date(date_str: &str) -> Option<NaiveDate> {
    // Parse format like "24-Feb-2026"
    NaiveDate::parse_from_str(date_str, "%d-%b-%Y").ok()
}

// Helper function to find the earliest expiry date from a list
fn find_earliest_expiry(expiry_dates: &[String]) -> Option<String> {
    expiry_dates
        .iter()
        .filter_map(|date_str| {
            parse_expiry_date(date_str).map(|date| (date, date_str.clone()))
        })
        .min_by_key(|(date, _)| *date)
        .map(|(_, date_str)| date_str)
}

// -----------------------------------------------
// API HANDLERS
// -----------------------------------------------

/// GET /mcx_health - Health check endpoint
async fn health() -> &'static str {
    "MCX server -> OK"
}

/// GET /api/mcx/tickers - Get all available MCX tickers
async fn get_ticker_list(State(app_state): State<AppState>) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    let start_time = Instant::now();

    // Check cache first
    {
        let cache = app_state.cache.read().await;
        if let Some((tickers, cached_at)) = &cache.ticker_list {
            if cached_at.elapsed() < CACHE_DURATION {
                let processed_data = processor::process_mcx_tickers(tickers.clone());
                return Ok(Json(ApiResponse {
                    success: true,
                    data: Some(processed_data),
                    error: None,
                    processing_time_ms: Some(start_time.elapsed().as_millis() as u64),
                }));
            }
        }
    }

    // Fetch from web scraping
    match app_state.client.fetch_ticker_list().await {
        Ok(tickers) => {
            // Update cache
            {
                let mut cache = app_state.cache.write().await;
                cache.ticker_list = Some((tickers.clone(), Instant::now()));
            }

            let processed_data = processor::process_mcx_tickers(tickers);
            Ok(Json(ApiResponse {
                success: true,
                data: Some(processed_data),
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

/// GET /api/mcx/option-chain?commodity=COPPER&expiry=23DEC2025 - Get processed option chain for specific commodity and expiry
async fn get_option_chain(
    Query(query): Query<OptionChainQuery>,
    State(app_state): State<AppState>,
) -> Result<Json<ApiResponse<EnhancedSingleAnalysisResponse>>, StatusCode> {
    let start_time = Instant::now();
    let cache_key = format!("{}_{}", query.commodity, query.expiry);

    // Step 1: Fetch future symbols to get latest expiry date
    let latest_future_expiry = match app_state.client.fetch_future_symbols().await {
        Ok(symbols_data) => {
            // Process the symbols data to find matching commodity
            match processor::process_mcx_future_symbols(symbols_data) {
                Ok(processed_symbols) => {
                    // Parse the processed symbols to find matching product
                    if let Ok(future_symbols) = serde_json::from_value::<FutureSymbolsResponse>(processed_symbols) {
                        // Find the product that matches our commodity
                        let matching_product = future_symbols.products
                            .iter()
                            .find(|p| p.product.to_uppercase() == query.commodity.to_uppercase());
                        
                        if let Some(product) = matching_product {
                            find_earliest_expiry(&product.expiry_dates)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                Err(_) => None,
            }
        }
        Err(_) => None,
    };

    // Step 2: Check cache for option chain (existing logic)
    {
        let cache = app_state.cache.read().await;
        if let Some((option_chain, cached_at)) = cache.option_chains.get(&cache_key) {
            if cached_at.elapsed() < CACHE_DURATION {
                // Process the cached data
                // Get underlying value from first available data point
                let underlying_value = option_chain.d.data.iter()
                    .find_map(|d| d.underlying_value)
                    .unwrap_or(0.0);
                    
                match processor::process_mcx_option_data(
                    option_chain.d.data.clone(),
                    underlying_value,
                    &query.expiry,
                ) {
                    Ok((processed_data, spread, days_to_expiry, ce_oi, pe_oi)) => {
                        let analysis_response = processor::create_single_analysis_response(
                            query.commodity.clone(),
                            option_chain.d.summary.as_on.clone().unwrap_or_else(|| "".to_string()),
                            underlying_value,
                            processed_data,
                            spread,
                            days_to_expiry,
                            ce_oi,
                            pe_oi,
                        );
                        
                        let enhanced_response = EnhancedSingleAnalysisResponse {
                            analysis: analysis_response,
                            latest_future_expiry: latest_future_expiry.clone(),
                        };
                        
                        return Ok(Json(ApiResponse {
                            success: true,
                            data: Some(enhanced_response),
                            error: None,
                            processing_time_ms: Some(start_time.elapsed().as_millis() as u64),
                        }));
                    }
                    Err(e) => {
                        return Ok(Json(ApiResponse {
                            success: false,
                            data: None,
                            error: Some(format!("Failed to process option chain data: {}", e)),
                            processing_time_ms: Some(start_time.elapsed().as_millis() as u64),
                        }));
                    }
                }
            }
        }
    }

    // Step 3: Fetch from API (existing logic)
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

            // Process the data
            // Get underlying value from first available data point
            let underlying_value = option_chain.d.data.iter()
                .find_map(|d| d.underlying_value)
                .unwrap_or(0.0);
                
            match processor::process_mcx_option_data(
                option_chain.d.data.clone(),
                underlying_value,
                &query.expiry,
            ) {
                Ok((processed_data, spread, days_to_expiry, ce_oi, pe_oi)) => {
                    let analysis_response = processor::create_single_analysis_response(
                        query.commodity.clone(),
                        option_chain.d.summary.as_on.clone().unwrap_or_else(|| "".to_string()),
                        underlying_value,
                        processed_data,
                        spread,
                        days_to_expiry,
                        ce_oi,
                        pe_oi,
                    );
                    
                    let enhanced_response = EnhancedSingleAnalysisResponse {
                        analysis: analysis_response,
                        latest_future_expiry,
                    };
                    
                    Ok(Json(ApiResponse {
                        success: true,
                        data: Some(enhanced_response),
                        error: None,
                        processing_time_ms: Some(start_time.elapsed().as_millis() as u64),
                    }))
                }
                Err(e) => Ok(Json(ApiResponse {
                    success: false,
                    data: None,
                    error: Some(format!("Failed to process option chain data: {}", e)),
                    processing_time_ms: Some(start_time.elapsed().as_millis() as u64),
                }))
            }
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

    // Step 4: Process results and apply processor + rules
    let mut batch_results = Vec::new();
    let mut successful_count = 0;
    let mut failed_count = 0;
    let mut batch_for_rules = Vec::new();

    for (ticker, result) in filtered_tickers.iter().zip(results.iter()) {
        match result {
            Ok((_, chain)) => {
                // Process the MCX option chain data
                // Get underlying value from first available data point  
                let underlying_value = chain.d.data.iter()
                    .find_map(|d| d.underlying_value)
                    .unwrap_or(0.0);
                    
                match processor::process_mcx_option_data(
                    chain.d.data.clone(),
                    underlying_value,
                    &ticker.expiry_date,
                ) {
                    Ok((processed_data, spread, _days_to_expiry, _ce_oi, _pe_oi)) => {
                        // Store for rules processing
                        batch_for_rules.push((
                            ticker.symbol.clone(),
                            chain.d.summary.as_on.clone().unwrap_or_else(|| "".to_string()),
                            underlying_value,
                            processed_data,
                            spread,
                        ));
                        
                        successful_count += 1;
                    }
                    Err(e) => {
                        failed_count += 1;
                        batch_results.push(BatchResult {
                            ticker: ticker.clone(),
                            rules_output: None,
                            error: Some(format!("Processing error: {}", e)),
                        });
                    }
                }
            }
            Err(e) => {
                failed_count += 1;
                batch_results.push(BatchResult {
                    ticker: ticker.clone(),
                    rules_output: None,
                    error: Some(e.to_string()),
                });
            }
        }
    }
    
    // Step 5: Run rules on all successfully processed securities
    let rules_outputs = super::rules::run_mcx_batch_rules(batch_for_rules);
    
    // Step 6: Add rules outputs to batch results (only securities with alerts)
    for rules_output in rules_outputs {
        // Find the corresponding ticker for this symbol
        if let Some(ticker) = filtered_tickers.iter().find(|t| t.symbol == rules_output.symbol) {
            batch_results.push(BatchResult {
                ticker: ticker.clone(),
                rules_output: Some(rules_output),
                error: None,
            });
        }
    }

    // Calculate alerts count
    let securities_with_alerts = batch_results.iter()
        .filter(|r| r.rules_output.is_some())
        .count();
        
    let total_alerts: usize = batch_results.iter()
        .filter_map(|r| r.rules_output.as_ref())
        .map(|r| r.alerts.len())
        .sum();

    let summary = BatchSummary {
        total_tickers,
        total_unique_symbols,
        filtered_latest_expiry: filtered_count,
        successful: successful_count,
        failed: failed_count,
        securities_with_alerts,
        total_alerts,
        processing_time_ms: start_time.elapsed().as_millis() as u64,
    };

    // Extract only the rules outputs (alerts) for the response
    let rules_outputs: Vec<super::rules::McxRulesOutput> = batch_results
        .into_iter()
        .filter_map(|r| r.rules_output)
        .collect();

    println!("✅ Batch analysis completed:");
    println!("   Successful: {}/{}", successful_count, filtered_count);
    println!("   Failed: {}/{}", failed_count, filtered_count);
    println!("   Securities with alerts: {}", securities_with_alerts);
    println!("   Total alerts: {}", total_alerts);
    println!("   Processing time: {}ms", summary.processing_time_ms);

    Ok(Json(ApiResponse {
        success: true,
        data: Some(BatchAnalysisResponse {
            summary,
            rules_output: rules_outputs,
        }),
        error: None,
        processing_time_ms: Some(start_time.elapsed().as_millis() as u64),
    }))
}

/// GET /api/mcx/future-quote?commodity=ALUMINI&expiry=31DEC2025 - Get future quote for specific commodity and expiry
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
        Ok(mut quote_data) => {

             processor::enrich_mcx_future_quote(&mut quote_data);
             
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

/// GET /api/mcx/option-quote?commodity=COPPER&expiry=23DEC2025&option_type=CE&strike_price=1120.00 - Get specific option quote
async fn get_option_quote(
    Query(query): Query<SpecificOptionQuoteQuery>,
    State(app_state): State<AppState>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    let start_time = Instant::now();
    let cache_key = format!("option_{}_{}_{}_{}",
        query.commodity, query.expiry, query.option_type, query.strike_price);

    // Check cache first
    {
        let cache = app_state.cache.read().await;
        if let Some((quote_data, cached_at)) = cache.option_quotes.get(&cache_key) {
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
    match app_state.client.fetch_option_quote(
        &query.commodity,
        &query.expiry,
        &query.option_type,
        &query.strike_price,
    ).await {
        Ok(quote_data) => {
            // Update cache
            {
                let mut cache = app_state.cache.write().await;
                cache.option_quotes.insert(
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
                // Process the cached data
                match processor::process_mcx_future_symbols(data.clone()) {
                    Ok(processed_data) => {
                        return Ok(Json(ApiResponse {
                            success: true,
                            data: Some(processed_data),
                            error: None,
                            processing_time_ms: Some(start_time.elapsed().as_millis() as u64),
                        }));
                    }
                    Err(e) => {
                        return Ok(Json(ApiResponse {
                            success: false,
                            data: None,
                            error: Some(format!("Failed to process future symbols data: {}", e)),
                            processing_time_ms: Some(start_time.elapsed().as_millis() as u64),
                        }));
                    }
                }
            }
        }
    }

    // Fetch from MCX API
    match app_state.client.fetch_future_symbols().await {
        Ok(data) => {
            // Update cache with raw data
            {
                let mut cache = app_state.cache.write().await;
                cache.future_symbols = Some((data.clone(), Instant::now()));
            }

            // Process the data (parse JSON string and convert timestamps)
            match processor::process_mcx_future_symbols(data) {
                Ok(processed_data) => {
                    Ok(Json(ApiResponse {
                        success: true,
                        data: Some(processed_data),
                        error: None,
                        processing_time_ms: Some(start_time.elapsed().as_millis() as u64),
                    }))
                }
                Err(e) => {
                    Ok(Json(ApiResponse {
                        success: false,
                        data: None,
                        error: Some(format!("Failed to process future symbols data: {}", e)),
                        processing_time_ms: Some(start_time.elapsed().as_millis() as u64),
                    }))
                }
            }
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
        .route("/api/mcx/option-quote", get(get_option_quote))
        .route("/api/mcx/batch-analysis", post(run_batch_analysis))
        .route("/api/mcx/future-symbols", get(get_future_symbols))
        .route("/api/mcx/historic-data", get(get_historic_data));

    let app = Router::new()
        .route("/mcx_health", get(health))
        .merge(mcx_routes)
        .layer(CorsLayer::permissive())
        .with_state(app_state);

    let addr = format!("127.0.0.1:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    
    println!("🚀 MCX API Server running on http://{}", addr);
    println!("📋 Available MCX endpoints:");
    println!("   GET  /mcx_health");
    println!("   GET  /api/mcx/tickers");
    println!("   GET  /api/mcx/option-chain?commodity=COPPER&expiry=23DEC2025 (Processed Data + Latest Expiry)");
    println!("   GET  /api/mcx/future-quote?commodity=ALUMINI&expiry=31DEC2025");
    println!("   GET  /api/mcx/option-quote?commodity=COPPER&expiry=23DEC2025&option_type=CE&strike_price=1120.00");
    println!("   POST /api/mcx/batch-analysis (Latest Expiry Only - Processed Data)");
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
        .route("/api/mcx/option-quote", get(get_option_quote))
        .route("/api/mcx/batch-analysis", post(run_batch_analysis))
        .route("/api/mcx/future-symbols", get(get_future_symbols))
        .route("/api/mcx/historic-data", get(get_historic_data))
}

/// Get MCX app state for merging with existing server
pub fn get_mcx_app_state() -> Result<AppState> {
    AppState::new()
}