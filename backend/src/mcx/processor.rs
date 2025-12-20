use super::models::{ OptionData as McxOptionData};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Local, NaiveDate};
use anyhow::{Result, anyhow};

/// Convert MCX timestamp format "/Date(1766159098000)/" to readable format
pub fn convert_mcx_timestamp(mcx_timestamp: &str) -> String {
    // Extract epoch from "/Date(1766159098000)/" format
    if let Some(start) = mcx_timestamp.find('(') {
        if let Some(end) = mcx_timestamp.find(')') {
            if let Ok(epoch_millis) = mcx_timestamp[start + 1..end].parse::<i64>() {
                // Convert milliseconds to seconds for DateTime
                let epoch_secs = epoch_millis / 1000;
                let naive = DateTime::from_timestamp(epoch_secs, 0);
                
                if let Some(dt) = naive {
                    // Convert to local timezone and format
                    let local_dt = dt.with_timezone(&chrono::Local);
                    return local_dt.format("%d-%b-%Y %H:%M:%S").to_string();
                }
            }
        }
    }
    
    // Fallback to current time if parsing fails
    Local::now().format("%d-%b-%Y %H:%M:%S").to_string()
}

/// Convert MCX expiry format "23DEC2025" to "30-Dec-2025" format
pub fn convert_mcx_expiry_format(mcx_expiry: &str) -> String {
    if let Ok(date) = NaiveDate::parse_from_str(mcx_expiry, "%d%b%Y") {
        date.format("%d-%b-%Y").to_string()
    } else {
        mcx_expiry.to_string() // Fallback to original if parsing fails
    }
}

/// Enhanced MCX option detail with computed fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedMcxOptionDetail {
    #[serde(rename = "strikePrice")]
    pub strike_price: f64,
    
    #[serde(rename = "underlyingValue")]
    pub underlying_value: f64,
    
    #[serde(rename = "openInterest")]
    pub open_interest: Option<f64>,
    
    #[serde(rename = "lastPrice")]
    pub last_price: Option<f64>,
    
    pub change: Option<f64>,
    
    pub pchange: Option<f64>,
    
    #[serde(rename = "changeinOpenInterest")]
    pub change_in_oi: Option<f64>,
    
    #[serde(rename = "pchangeinOpenInterest")]
    pub pchange_in_oi: Option<f64>,
    
    // Computed fields
    pub the_money: String,  // "ATM", "1 ITM", "2 OTM", etc.
    pub tambu: Option<String>,  // "TMJ", "TMG", or None
    pub time_val: f64,
    pub days_to_expiry: i32,
    
    #[serde(rename = "oiRank")]
    pub oi_rank: Option<u32>,
}

/// Processed MCX option data with enhanced CE and PE
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedMcxOptionData {
    #[serde(rename = "strikePrice")]
    pub strike_price: f64,
    
    #[serde(rename = "expiryDates")]
    pub expiry_date: Option<String>,
    
    #[serde(rename = "CE")]
    pub call: Option<ProcessedMcxOptionDetail>,
    
    #[serde(rename = "PE")]
    pub put: Option<ProcessedMcxOptionDetail>,
    
    pub days_to_expiry: i32,
}

/// Single Analysis Response for MCX (matching NSE structure)
#[derive(Debug, Serialize)]
pub struct McxSingleAnalysisResponse {
    pub symbol: String,
    pub timestamp: String,
    
    #[serde(rename = "underlyingValue")]
    pub underlying_value: f64,
    
    pub spread: f64,
    pub days_to_expiry: i32,
    pub ce_oi: f64,
    pub pe_oi: f64,
    pub processed_data: Vec<ProcessedMcxOptionData>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alerts: Option<super::rules::McxRulesOutput>,
}

/// Calculate days to expiry from today's date for MCX format
pub fn calculate_days_to_expiry(expiry_date_str: &str) -> Result<i32> {
    // Parse MCX expiry date format (e.g., "23DEC2025")
    let expiry_date = NaiveDate::parse_from_str(expiry_date_str, "%d%b%Y")
        .map_err(|e| anyhow!("Failed to parse expiry date '{}': {}", expiry_date_str, e))?;
    
    // Get today's date
    let today = Local::now().date_naive();
    
    // Calculate difference in days
    let days_diff = (expiry_date - today).num_days() as i32;
    
    // Check if today is after expiry (should not happen)
    if days_diff < 0 {
        return Err(anyhow!(
            "Current date ({}) is after expiry date ({}). Days difference: {}",
            today, expiry_date, days_diff
        ));
    }
    
    Ok(days_diff)
}

/// Process MCX option chain data
pub fn process_mcx_option_data(
    data: Vec<McxOptionData>,
    underlying_value: f64,
    expiry_date: &str,
) -> Result<(Vec<ProcessedMcxOptionData>, f64, i32, f64, f64)> {
    // Calculate days to expiry
    let days_to_expiry = calculate_days_to_expiry(expiry_date)?;
    
    // Step 1: Identify ATM strike
    let atm_strike = find_atm_strike(&data, underlying_value);
    
    // Step 2: Calculate spread from available strikes
    let spread = calculate_spread(&data, atm_strike);
    
    // Step 3: Collect all available strikes for indexing
    let mut available_strikes: Vec<f64> = data
        .iter()
        .filter_map(|opt| opt.ce_strike_price)
        .collect();
    available_strikes.sort_by(|a, b| a.partial_cmp(b).unwrap());
    available_strikes.dedup();
    
    // Step 4: Process each strike with classifications
    let mut processed: Vec<ProcessedMcxOptionData> = Vec::new();
    let mut processed_strikes = std::collections::HashSet::new();
    
    for opt in data.iter() {
        let strike = opt.ce_strike_price.unwrap_or(0.0);
        
        // Skip duplicate strikes
        if processed_strikes.contains(&(strike as i64)) {
            continue;
        }
        processed_strikes.insert(strike as i64);
        
        let call_detail = if opt.ce_open_interest.is_some() {
            let change_in_oi = opt.ce_change_in_oi.map(|x| x as f64);
            let open_interest = opt.ce_open_interest.map(|x| x as f64);
            
            // Calculate pchange_in_oi
            let pchange_in_oi = calculate_pchange_in_oi(change_in_oi, open_interest);
            
            Some(ProcessedMcxOptionDetail {
                strike_price: strike,
                underlying_value,
                open_interest,
                last_price: opt.ce_ltp,
                change: opt.ce_absolute_change,
                pchange: opt.ce_net_change,
                change_in_oi,
                pchange_in_oi,
                the_money: classify_money_with_distance(strike, atm_strike, &available_strikes, true),
                tambu: calculate_tambu(pchange_in_oi, opt.ce_net_change),
                time_val: calculate_time_value(opt.ce_ltp, strike, underlying_value, true),
                days_to_expiry,
                oi_rank: None, // Will be calculated separately if needed
            })
        } else {
            None
        };
        
        let put_detail = if opt.pe_open_interest.is_some() {
            let change_in_oi = opt.pe_change_in_oi.map(|x| x as f64);
            let open_interest = opt.pe_open_interest.map(|x| x as f64);
            
            // Calculate pchange_in_oi
            let pchange_in_oi = calculate_pchange_in_oi(change_in_oi, open_interest);
            
            Some(ProcessedMcxOptionDetail {
                strike_price: strike,
                underlying_value,
                open_interest,
                last_price: opt.pe_ltp,
                change: opt.pe_absolute_change,
                pchange: opt.pe_net_change,
                change_in_oi,
                pchange_in_oi,
                the_money: classify_money_with_distance(strike, atm_strike, &available_strikes, false),
                tambu: calculate_tambu(pchange_in_oi, opt.pe_net_change),
                time_val: calculate_time_value(opt.pe_ltp, strike, underlying_value, false),
                days_to_expiry,
                oi_rank: None, // Will be calculated separately if needed
            })
        } else {
            None
        };
        
        if call_detail.is_some() || put_detail.is_some() {
            processed.push(ProcessedMcxOptionData {
                strike_price: strike,
                expiry_date: Some(convert_mcx_expiry_format(expiry_date)),
                call: call_detail,
                put: put_detail,
                days_to_expiry,
            });
        }
    }
    
    // Step 5: Calculate OI rankings for processed data
    calculate_processed_oi_rankings(&mut processed);
    
    // Step 6: Filter to ATM ±6 strikes + high OI outliers
    filter_strikes(&mut processed, atm_strike);
    
    // Calculate total CE and PE OI by summing all values (MCX doesn't provide summary totals like NSE)
    let ce_oi: f64 = processed.iter()
        .filter_map(|opt| opt.call.as_ref()?.open_interest)
        .sum();
    
    let pe_oi: f64 = processed.iter()
        .filter_map(|opt| opt.put.as_ref()?.open_interest)
        .sum();
    
    Ok((processed, spread, days_to_expiry, ce_oi, pe_oi))
}

/// Find ATM strike (closest to underlying, prefer floor)
pub fn find_atm_strike(data: &[McxOptionData], underlying_value: f64) -> f64 {
    let mut closest_strike = 0.0;
    let mut min_distance = f64::MAX;
    
    for opt in data {
        if let Some(strike) = opt.ce_strike_price {
            let distance = (strike - underlying_value).abs();
            
            // If same distance, prefer lower strike (floor)
            if distance < min_distance || (distance == min_distance && strike < closest_strike) {
                min_distance = distance;
                closest_strike = strike;
            }
        }
    }
    
    closest_strike
}

/// Calculate spread using ATM strike and next higher strike
pub fn calculate_spread(data: &[McxOptionData], atm_strike: f64) -> f64 {
    // Get all strike prices and sort them
    let mut strikes: Vec<f64> = data
        .iter()
        .filter_map(|opt| opt.ce_strike_price)
        .collect();
    strikes.sort_by(|a, b| a.partial_cmp(b).unwrap());
    strikes.dedup();
    
    // Find the position of ATM strike
    if let Some(atm_pos) = strikes.iter().position(|&s| s == atm_strike) {
        // If there's a next higher strike, calculate spread
        if atm_pos + 1 < strikes.len() {
            return strikes[atm_pos + 1] - atm_strike;
        }
    }
    
    // Fallback: calculate average spread if ATM is not found or is the highest
    if strikes.len() >= 2 {
        let total_diff: f64 = strikes.windows(2).map(|w| w[1] - w[0]).sum();
        total_diff / (strikes.len() - 1) as f64
    } else {
        0.0
    }
}

/// Classify option as ATM, N ITM, or N OTM with distance calculation using strike indexing
pub fn classify_money_with_distance(
    strike: f64, 
    atm_strike: f64, 
    available_strikes: &[f64], 
    is_call: bool
) -> String {
    if strike == atm_strike {
        "ATM".to_string()
    } else {
        // Find the index positions of current strike and ATM strike
        let atm_index = available_strikes.iter().position(|&s| s == atm_strike);
        let strike_index = available_strikes.iter().position(|&s| s == strike);
        
        if let (Some(atm_idx), Some(strike_idx)) = (atm_index, strike_index) {
            // Calculate distance based on index difference
            let distance = (strike_idx as i32 - atm_idx as i32).abs();
            
            if is_call {
                // Call: Above ATM = OTM, Below ATM = ITM
                if strike > atm_strike {
                    format!("{} OTM", distance)
                } else {
                    format!("{} ITM", distance)
                }
            } else {
                // Put: Above ATM = ITM, Below ATM = OTM
                if strike > atm_strike {
                    format!("{} ITM", distance)
                } else {
                    format!("{} OTM", distance)
                }
            }
        } else {
            // Fallback if strike not found in list
            if is_call {
                if strike > atm_strike { "OTM".to_string() } else { "ITM".to_string() }
            } else {
                if strike > atm_strike { "ITM".to_string() } else { "OTM".to_string() }
            }
        }
    }
}

/// Calculate OI rankings for processed CE and PE options separately
pub fn calculate_processed_oi_rankings(processed: &mut [ProcessedMcxOptionData]) {
    // Collect all CE options with their indices and OI values for ranking
    let mut ce_options: Vec<(usize, f64)> = Vec::new();
    let mut pe_options: Vec<(usize, f64)> = Vec::new();
    
    // Gather CE and PE options with valid OI
    for (i, opt) in processed.iter().enumerate() {
        if let Some(ref call) = opt.call {
            if let Some(oi) = call.open_interest {
                ce_options.push((i, oi));
            }
        }
        
        if let Some(ref put) = opt.put {
            if let Some(oi) = put.open_interest {
                pe_options.push((i, oi));
            }
        }
    }
    
    // Sort CE options by OI in descending order (highest first)
    ce_options.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    
    // Sort PE options by OI in descending order (highest first)
    pe_options.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    
    // Assign ranks to CE options
    for (rank, &(data_index, _)) in ce_options.iter().enumerate() {
        if let Some(ref mut call) = processed[data_index].call {
            call.oi_rank = Some((rank + 1) as u32);
        }
    }
    
    // Assign ranks to PE options
    for (rank, &(data_index, _)) in pe_options.iter().enumerate() {
        if let Some(ref mut put) = processed[data_index].put {
            put.oi_rank = Some((rank + 1) as u32);
        }
    }
}

/// Calculate percentage change in Open Interest
pub fn calculate_pchange_in_oi(change_in_oi: Option<f64>, open_interest: Option<f64>) -> Option<f64> {
    let change_in_oi_val = change_in_oi.unwrap_or(0.0);
    let open_interest_val = open_interest.unwrap_or(0.0);
    
    if open_interest_val + change_in_oi_val != 0.0 {
        Some((change_in_oi_val / (open_interest_val + change_in_oi_val)) * 100.0)
    } else {
        Some(0.0)
    }
}

/// Calculate Tambu classification for MCX
pub fn calculate_tambu(pchange_in_oi: Option<f64>, pchange: Option<f64>) -> Option<String> {
    let pchange_in_oi_val = pchange_in_oi.unwrap_or(0.0);
    let pchange_val = pchange.unwrap_or(0.0);
    
    // TMJ: pchange_in_oi > 30% AND pchange < -15%
    if pchange_in_oi_val > 30.0 && pchange_val < -15.0 {
        return Some("TMJ".to_string());
    }
    
    // TMG: pchange_in_oi < -10% AND pchange > 15%
    if pchange_in_oi_val < -10.0 && pchange_val > 15.0 {
        return Some("TMG".to_string());
    }
    
    None
}

/// Calculate time value for MCX options
pub fn calculate_time_value(
    last_price: Option<f64>,
    strike: f64,
    underlying_value: f64,
    is_call: bool,
) -> f64 {
    let ltp = last_price.unwrap_or(0.0);
    
    if is_call {
        // CE: Time_val = lastPrice - (underlyingValue - strikePrice) if underlyingValue > strikePrice
        //     otherwise Time_val = lastPrice
        if underlying_value > strike {
            ltp - (underlying_value - strike)
        } else {
            ltp
        }
    } else {
        // PE: Time_val = lastPrice - (strikePrice - underlyingValue) if strikePrice > underlyingValue
        //     otherwise Time_val = lastPrice
        if strike > underlying_value {
            ltp - (strike - underlying_value)
        } else {
            ltp
        }
    }
}

/// Filter to ATM ±6 strikes plus high OI outliers
pub fn filter_strikes(processed: &mut Vec<ProcessedMcxOptionData>, atm_strike: f64) {
    // Sort by strike price
    processed.sort_by(|a, b| {
        a.strike_price.partial_cmp(&b.strike_price).unwrap()
    });
    
    // Find ATM index
    let atm_index = processed
        .iter()
        .position(|opt| opt.strike_price == atm_strike)
        .unwrap_or(0);
    
    // Select ATM ±6 strikes (13 total)
    let start = atm_index.saturating_sub(6);
    let end = (atm_index + 7).min(processed.len());
    
    // Find max OI in selected range
    let max_oi_in_range = processed[start..end]
        .iter()
        .map(|opt| get_max_oi(opt))
        .fold(0.0, f64::max);
    
    // Collect indices to keep
    let indices_to_keep: Vec<usize> = (0..processed.len())
        .filter(|&idx| {
            // Keep if in selected range
            if idx >= start && idx < end {
                return true;
            }
            
            // Keep if OI exceeds max in selected range
            get_max_oi(&processed[idx]) > max_oi_in_range
        })
        .collect();
    
    // Keep only selected strikes
    let mut new_processed = Vec::new();
    for idx in indices_to_keep {
        new_processed.push(processed[idx].clone());
    }
    *processed = new_processed;
}

/// Get maximum OI from CE or PE for a strike
pub fn get_max_oi(opt: &ProcessedMcxOptionData) -> f64 {
    let ce_oi = opt.call.as_ref()
        .and_then(|c| c.open_interest)
        .unwrap_or(0.0);

    let pe_oi = opt.put.as_ref()
        .and_then(|p| p.open_interest)
        .unwrap_or(0.0);

    ce_oi.max(pe_oi)
}

/// Process MCX tickers data - group by SymbolValue and collect expiry dates
pub fn process_mcx_tickers(tickers: Vec<super::models::Ticker>) -> serde_json::Value {
    // Group by SymbolValue and collect expiry dates
    let mut symbols: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    
    for ticker in tickers {
        let formatted_date = convert_mcx_expiry_format(&ticker.expiry_date);
        symbols.entry(ticker.symbol_value.clone())
            .or_insert_with(Vec::new)
            .push(formatted_date);
    }
    
    // Sort expiry dates for each symbol and remove duplicates
    for dates in symbols.values_mut() {
        dates.sort();
        dates.dedup();
    }
    
    // Create the response structure
    serde_json::json!({
        "InstrumentName": "OPTFUT",
        "Symbols": symbols.into_iter()
            .map(|(symbol, dates)| {
                serde_json::json!({
                    "SymbolValue": symbol,
                    "ExpiryDates": dates
                })
            })
            .collect::<Vec<_>>()
    })
}

/// Convert MCX timestamp to date only format (without time)
pub fn convert_mcx_timestamp_to_date(mcx_timestamp: &str) -> String {
    // Extract epoch from "/Date(1766159098000)/" format
    if let Some(start) = mcx_timestamp.find('(') {
        if let Some(end) = mcx_timestamp.find(')') {
            if let Ok(epoch_millis) = mcx_timestamp[start + 1..end].parse::<i64>() {
                // Convert milliseconds to seconds for DateTime
                let epoch_secs = epoch_millis / 1000;
                let naive = DateTime::from_timestamp(epoch_secs, 0);
                
                if let Some(dt) = naive {
                    // Convert to local timezone and format as date only
                    let local_dt = dt.with_timezone(&chrono::Local);
                    return local_dt.format("%d-%b-%Y").to_string();
                }
            }
        }
    }
    
    // Fallback to current date if parsing fails
    Local::now().format("%d-%b-%Y").to_string()
}

/// Process MCX future symbols data - parse JSON string, group by Product, and format dates
pub fn process_mcx_future_symbols(json_string_data: serde_json::Value) -> Result<serde_json::Value> {
    // The data comes as a JSON string inside the JSON response, need to parse it
    let json_str = json_string_data.as_str()
        .ok_or_else(|| anyhow!("Future symbols data is not a string"))?;
    
    // Parse the inner JSON string
    let symbols: Vec<serde_json::Value> = serde_json::from_str(json_str)
        .map_err(|e| anyhow!("Failed to parse future symbols JSON: {}", e))?;
    
    // Group by Product and collect expiry dates
    let mut products: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    
    for symbol in symbols {
        if let Some(obj) = symbol.as_object() {
            if let (Some(product), Some(end_date)) = (
                obj.get("Product").and_then(|v| v.as_str()),
                obj.get("EndDate").and_then(|v| v.as_str())
            ) {
                let formatted_date = convert_mcx_timestamp_to_date(end_date);
                products.entry(product.to_string())
                    .or_insert_with(Vec::new)
                    .push(formatted_date);
            }
        }
    }
    
    // Sort expiry dates for each product and remove duplicates
    for dates in products.values_mut() {
        dates.sort();
        dates.dedup();
    }
    
    // Create the response structure
    let response = serde_json::json!({
        "InstrumentName": "FUTCOM",
        "Products": products.into_iter()
            .map(|(product, dates)| {
                serde_json::json!({
                    "Product": product,
                    "ExpiryDates": dates
                })
            })
            .collect::<Vec<_>>()
    });
    
    Ok(response)
}

/// Create MCX Single Analysis Response from processed data (matching NSE API structure)
pub fn create_single_analysis_response(
    symbol: String,
    timestamp: String,
    underlying_value: f64,
    processed_data: Vec<ProcessedMcxOptionData>,
    spread: f64,
    days_to_expiry: i32,
    ce_oi: f64,
    pe_oi: f64,
) -> McxSingleAnalysisResponse {
    // Run rules on processed data
    let alerts = super::rules::run_mcx_rules(
        &processed_data,
        symbol.clone(),
        convert_mcx_timestamp(&timestamp),
        underlying_value,
        spread,
    );
    
    McxSingleAnalysisResponse {
        symbol,
        timestamp: convert_mcx_timestamp(&timestamp),
        underlying_value,
        spread,
        days_to_expiry,
        ce_oi,
        pe_oi,
        processed_data,
        alerts,
    }
}