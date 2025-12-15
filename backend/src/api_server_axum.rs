use crate::config;
use crate::models::{Security, SecurityType};
use crate::nse_client::NSEClient;
use crate::{processor, rules};
use anyhow::Result;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;

// -----------------------------------------------
// API REQUEST/RESPONSE MODELS
// -----------------------------------------------

#[derive(Debug, Deserialize)]
pub struct ContractInfoQuery {
    pub symbol: String,
}

#[derive(Debug, Deserialize)]
pub struct SingleAnalysisQuery {
    pub symbol: String,
    pub expiry: String,
}

#[derive(Debug, Deserialize)]
pub struct FuturesDataQuery {
    pub symbol: String,
    pub expiry: String,
}

#[derive(Debug, Deserialize)]
pub struct DerivativesHistoricalQuery {
    pub symbol: String,
    pub instrument_type: String, // "OPTIONS" or "FUTURES"
    pub year: Option<String>,
    pub expiry: String,
    pub strike_price: Option<String>,
    pub option_type: Option<String>, // "CE" or "PE"
    pub from_date: String,
    pub to_date: String,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub processing_time_ms: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct SecurityListResponse {
    pub indices: Vec<SecurityInfo>,
    pub equities: HashMap<String, Vec<SecurityInfo>>,  // Grouped by first letter
}

#[derive(Debug, Serialize, Clone)]
pub struct SecurityInfo {
    pub symbol: String,
    pub security_type: String,
}

#[derive(Debug, Serialize)]
pub struct ContractInfoResponse {
    pub symbol: String,
    pub expiry_dates: Vec<String>,
    pub strike_prices: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct SingleAnalysisResponse {
    pub symbol: String,
    pub timestamp: String,
    pub underlying_value: f64,
    pub spread: f64,
    pub days_to_expiry: i32,
    pub ce_oi: f64,
    pub pe_oi: f64,
    pub processed_data: Vec<processor::ProcessedOptionData>,
    pub alerts: Option<rules::RulesOutput>,
}

#[derive(Debug, Serialize)]
pub struct BatchAnalysisResponse {
    pub summary: BatchSummary,
    pub rules_output: Vec<rules::RulesOutput>,
}

#[derive(Debug, Serialize)]
pub struct BatchSummary {
    pub total_securities: usize,
    pub successful: usize,
    pub failed: usize,
    pub securities_with_alerts: usize,
    pub total_alerts: usize,
    pub processing_time_ms: u64,
}

// -----------------------------------------------
// APPLICATION STATE
// -----------------------------------------------

#[derive(Clone)]
pub struct AppState {
    client: Arc<NSEClient>,
    cache: Arc<RwLock<Cache>>,
}

#[derive(Default)]
struct Cache {
    securities_list: Option<(Vec<Security>, Instant)>,
    contract_info: HashMap<String, (crate::models::ContractInfo, Instant)>,
}

const CACHE_DURATION: Duration = Duration::from_secs(300); // 5 minutes

impl AppState {
    pub fn new() -> Result<Self> {
        Ok(Self {
            client: Arc::new(NSEClient::new()?),
            cache: Arc::new(RwLock::new(Cache::default())),
        })
    }
}

// -----------------------------------------------
// API HANDLERS
// -----------------------------------------------

/// GET /api/securities - Get all FNO securities list
async fn get_securities(State(app_state): State<AppState>) -> Result<Json<ApiResponse<SecurityListResponse>>, StatusCode> {
    let start_time = Instant::now();

    // Check cache first
    {
        let cache = app_state.cache.read().await;
        if let Some((securities, cached_at)) = &cache.securities_list {
            if cached_at.elapsed() < CACHE_DURATION {
                return Ok(Json(format_securities_response(securities.clone(), start_time)));
            }
        }
    }

    // Fetch from API
    match app_state.client.fetch_fno_list().await {
        Ok(securities) => {
            // Update cache
            {
                let mut cache = app_state.cache.write().await;
                cache.securities_list = Some((securities.clone(), Instant::now()));
            }

            Ok(Json(format_securities_response(securities, start_time)))
        }
        Err(e) => Ok(Json(ApiResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
            processing_time_ms: Some(start_time.elapsed().as_millis() as u64),
        })),
    }
}

/// GET /api/contract-info?symbol=NIFTY - Get contract info for a symbol
async fn get_contract_info(
    Query(query): Query<ContractInfoQuery>,
    State(app_state): State<AppState>,
) -> Result<Json<ApiResponse<ContractInfoResponse>>, StatusCode> {
    let start_time = Instant::now();
    let symbol = &query.symbol;

    // Check cache first
    {
        let cache = app_state.cache.read().await;
        if let Some((contract_info, cached_at)) = cache.contract_info.get(symbol) {
            if cached_at.elapsed() < CACHE_DURATION {
                return Ok(Json(ApiResponse {
                    success: true,
                    data: Some(ContractInfoResponse {
                        symbol: symbol.to_string(),
                        expiry_dates: contract_info.expiry_dates.clone(),
                        strike_prices: contract_info.strike_prices.clone(),
                    }),
                    error: None,
                    processing_time_ms: Some(start_time.elapsed().as_millis() as u64),
                }));
            }
        }
    }

    // Fetch from API
    match app_state.client.fetch_contract_info(symbol).await {
        Ok(contract_info) => {
            // Update cache
            {
                let mut cache = app_state.cache.write().await;
                cache.contract_info.insert(
                    symbol.to_string(),
                    (contract_info.clone(), Instant::now()),
                );
            }

            Ok(Json(ApiResponse {
                success: true,
                data: Some(ContractInfoResponse {
                    symbol: symbol.to_string(),
                    expiry_dates: contract_info.expiry_dates,
                    strike_prices: contract_info.strike_prices,
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

/// GET /api/single-analysis?symbol=NIFTY&expiry=30-Dec-2025 - Get single security analysis
async fn get_single_analysis(
    Query(query): Query<SingleAnalysisQuery>,
    State(app_state): State<AppState>,
) -> Result<Json<ApiResponse<SingleAnalysisResponse>>, StatusCode> {
    let start_time = Instant::now();
    let symbol = &query.symbol;
    let expiry = &query.expiry;

    // Determine security type
    let security = if config::NSE_INDICES.contains(&symbol.as_str()) {
        Security::index(symbol.to_string())
    } else {
        Security::equity(symbol.to_string())
    };

    match app_state.client.fetch_option_chain(&security, expiry).await {
        Ok(chain) => {
            // Process the data
            let (processed_data, spread) = processor::process_option_data(
                chain.filtered.data.clone(),
                chain.records.underlying_value
            );

            // Extract days_to_expiry from first processed option
            let days_to_expiry = processed_data.first()
                .map(|opt| opt.days_to_expiry)
                .unwrap_or(0);

            // Run rules on processed data
            let alerts = rules::run_rules(
                &processed_data,
                symbol.to_string(),
                chain.records.timestamp.clone(),
                chain.records.underlying_value,
                spread,
            );

            Ok(Json(ApiResponse {
                success: true,
                data: Some(SingleAnalysisResponse {
                    symbol: symbol.to_string(),
                    timestamp: chain.records.timestamp,
                    underlying_value: chain.records.underlying_value,
                    spread,
                    days_to_expiry,
                    ce_oi: chain.filtered.ce_totals.total_oi,
                    pe_oi: chain.filtered.pe_totals.total_oi,
                    processed_data,
                    alerts,
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

/// GET /api/futures-data?symbol=NIFTY&expiry=30-Dec-2025 - Get futures data
async fn get_futures_data(
    Query(query): Query<FuturesDataQuery>,
    State(app_state): State<AppState>,
) -> Result<Json<ApiResponse<Value>>, StatusCode> {
    let start_time = Instant::now();
    let symbol = &query.symbol;
    let expiry = &query.expiry;

    match app_state.client.fetch_futures_data(symbol, expiry).await {
        Ok(data) => Ok(Json(ApiResponse {
            success: true,
            data: Some(data),
            error: None,
            processing_time_ms: Some(start_time.elapsed().as_millis() as u64),
        })),
        Err(e) => Ok(Json(ApiResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
            processing_time_ms: Some(start_time.elapsed().as_millis() as u64),
        })),
    }
}

/// GET /api/derivatives-historical - Get derivatives historical data
async fn get_derivatives_historical_data(
    Query(query): Query<DerivativesHistoricalQuery>,
    State(app_state): State<AppState>,
) -> Result<Json<ApiResponse<Value>>, StatusCode> {
    let start_time = Instant::now();
    
    // Determine security type
    let security_type = if config::NSE_INDICES.contains(&query.symbol.as_str()) {
        SecurityType::Indices
    } else {
        SecurityType::Equity
    };

    match app_state.client.fetch_derivatives_historical_data(
        &query.symbol,
        &security_type,
        &query.instrument_type,
        query.year.as_deref(),
        &query.expiry,
        query.strike_price.as_deref(),
        query.option_type.as_deref(),
        &query.from_date,
        &query.to_date,
    ).await {
        Ok(data) => Ok(Json(ApiResponse {
            success: true,
            data: Some(data),
            error: None,
            processing_time_ms: Some(start_time.elapsed().as_millis() as u64),
        })),
        Err(e) => Ok(Json(ApiResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
            processing_time_ms: Some(start_time.elapsed().as_millis() as u64),
        })),
    }
}

/// POST /api/batch-analysis - Run batch analysis
async fn run_batch_analysis(
    State(app_state): State<AppState>,
) -> Result<Json<ApiResponse<BatchAnalysisResponse>>, StatusCode> {
    let start_time = Instant::now();

    // Step 1: Fetch all FNO securities
    let securities = match app_state.client.fetch_fno_list().await {
        Ok(securities) => securities,
        Err(e) => {
            return Ok(Json(ApiResponse {
                success: false,
                data: None,
                error: Some(format!("Failed to fetch securities list: {}", e)),
                processing_time_ms: Some(start_time.elapsed().as_millis() as u64),
            }));
        }
    };

    let total_securities = securities.len();
    let max_concurrent = config::DEFAULT_MAX_CONCURRENT;

    // Step 2: Bulk process all securities  
    let results = app_state.client.clone().fetch_all_option_chains(securities.clone(), max_concurrent).await;

    // Step 3: Process results
    let mut successful = Vec::new();
    let mut failed_count = 0;

    for (security, result) in securities.iter().zip(results.iter()) {
        match result {
            Ok((_, chain)) => {
                successful.push((security.clone(), chain.clone()));
            }
            Err(_) => {
                failed_count += 1;
            }
        }
    }

    // Step 4: Process data and run rules
    let mut batch_for_rules = Vec::new();
    
    for (security, chain) in successful.iter() {
        let (processed_data, spread) = processor::process_option_data(
            chain.filtered.data.clone(),
            chain.records.underlying_value
        );
        
        batch_for_rules.push((
            security.symbol.clone(),
            chain.records.timestamp.clone(),
            chain.records.underlying_value,
            processed_data,
            spread,
        ));
    }
    
    // Run rules on all securities
    let rules_outputs = rules::run_batch_rules(batch_for_rules);
    
    let total_alerts: usize = rules_outputs.iter()
        .map(|r| r.alerts.len())
        .sum();

    let summary = BatchSummary {
        total_securities,
        successful: successful.len(),
        failed: failed_count,
        securities_with_alerts: rules_outputs.len(),
        total_alerts,
        processing_time_ms: start_time.elapsed().as_millis() as u64,
    };

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

// -----------------------------------------------
// HELPER FUNCTIONS
// -----------------------------------------------

fn format_securities_response(securities: Vec<Security>, start_time: Instant) -> ApiResponse<SecurityListResponse> {
    let mut indices = Vec::new();
    let mut equities_map: HashMap<String, Vec<SecurityInfo>> = HashMap::new();

    for security in securities {
        let security_info = SecurityInfo {
            symbol: security.symbol.clone(),
            security_type: match security.security_type {
                SecurityType::Equity => "Equity".to_string(),
                SecurityType::Indices => "Indices".to_string(),
            },
        };

        match security.security_type {
            SecurityType::Indices => {
                indices.push(security_info);
            }
            SecurityType::Equity => {
                let first_letter = security.symbol.chars().next()
                    .unwrap_or('A')
                    .to_uppercase()
                    .to_string();
                
                equities_map
                    .entry(first_letter)
                    .or_insert_with(Vec::new)
                    .push(security_info);
            }
        }
    }

    // Sort equities within each group
    for (_, securities) in equities_map.iter_mut() {
        securities.sort_by(|a, b| a.symbol.cmp(&b.symbol));
    }

    // Sort indices
    indices.sort_by(|a, b| a.symbol.cmp(&b.symbol));

    ApiResponse {
        success: true,
        data: Some(SecurityListResponse {
            indices,
            equities: equities_map,
        }),
        error: None,
        processing_time_ms: Some(start_time.elapsed().as_millis() as u64),
    }
}

// -----------------------------------------------
// SERVER SETUP
// -----------------------------------------------

pub async fn start_server(port: u16) -> Result<()> {
    let app_state = AppState::new()?;

    let app = Router::new()
        .route("/api/securities", get(get_securities))
        .route("/api/contract-info", get(get_contract_info))
        .route("/api/single-analysis", get(get_single_analysis))
        .route("/api/batch-analysis", post(run_batch_analysis))
        .route("/api/futures-data", get(get_futures_data))
        .route("/api/derivatives-historical", get(get_derivatives_historical_data))
        .layer(CorsLayer::permissive())
        .with_state(app_state);

    let addr = format!("127.0.0.1:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    
    println!("🚀 NSE API Server running on http://{}", addr);
    println!("📋 Available endpoints:");
    println!("   GET  /api/securities");
    println!("   GET  /api/contract-info?symbol=NIFTY");
    println!("   GET  /api/single-analysis?symbol=NIFTY&expiry=30-Dec-2025");
    println!("   GET  /api/futures-data?symbol=NIFTY&expiry=30-Dec-2025");
    println!("   GET  /api/derivatives-historical?symbol=NIFTY&instrument_type=FUTURES&expiry=30-Dec-2025&from_date=06-11-2025&to_date=06-12-2025");
    println!("   POST /api/batch-analysis");
    println!();

    axum::serve(listener, app).await?;
    Ok(())
}