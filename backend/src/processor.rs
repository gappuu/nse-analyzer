use crate::models::{OptionData, OptionDetail};
use serde::{Deserialize, Serialize};
use chrono::{NaiveDate, Local};
use anyhow::{Result, anyhow};

/// Enhanced option detail with computed fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedOptionDetail {
    #[serde(flatten)]
    pub base: OptionDetail,
    
    pub the_money: String,  // "ATM", "1 ITM", "2 OTM", etc.
    pub tambu: Option<String>,  // "TMJ", "TMG", or None
    pub time_val: f64,
    pub days_to_expiry: i32,  // Days remaining until expiry (0 on expiry day)
}

/// Processed option data with enhanced CE and PE
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedOptionData {
    #[serde(rename = "expiryDates")]
    pub expiry_date: Option<String>,
    
    #[serde(rename = "strikePrice")]
    pub strike_price: Option<f64>,
    
    #[serde(rename = "CE")]
    pub call: Option<ProcessedOptionDetail>,
    
    #[serde(rename = "PE")]
    pub put: Option<ProcessedOptionDetail>,
    
    pub days_to_expiry: i32,  // Days remaining until expiry (0 on expiry day)
}

/// Calculate days to expiry from today's date
fn calculate_days_to_expiry(expiry_date_str: &str) -> Result<i32> {
    // Parse the expiry date (format: "30-Dec-2025")
    let expiry_date = NaiveDate::parse_from_str(expiry_date_str, "%d-%b-%Y")
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

/// Process option chain data
pub fn process_option_data(
    data: Vec<OptionData>,
    underlying_value: f64,
) -> (Vec<ProcessedOptionData>, f64) {
    // Step 1: Identify ATM strike
    let atm_strike = find_atm_strike(&data, underlying_value);
    
    // Step 1.1: Calculate spread from available strikes
    let spread = calculate_spread(&data, atm_strike);
    
    // Step 1.2: Collect all available strikes for indexing
    let mut available_strikes: Vec<f64> = data
        .iter()
        .filter_map(|opt| opt.strike_price)
        .collect();
    available_strikes.sort_by(|a, b| a.partial_cmp(b).unwrap());
    
    // Step 2-4: Process each strike with classifications
    let mut processed: Vec<ProcessedOptionData> = data
        .into_iter()
        .filter_map(|opt| {
            let strike = opt.strike_price.unwrap_or(0.0);
            let expiry_date_str = opt.expiry_date.as_ref()?;
            
            // Calculate days to expiry
            let days_to_expiry = match calculate_days_to_expiry(expiry_date_str) {
                Ok(days) => days,
                Err(e) => {
                    eprintln!("Warning: Failed to calculate days to expiry for {}: {}", expiry_date_str, e);
                    return None; // Skip this option if expiry calculation fails
                }
            };
            
            Some(ProcessedOptionData {
                expiry_date: opt.expiry_date.clone(),
                strike_price: opt.strike_price,
                days_to_expiry,
                call: opt.call.map(|ce| process_option_detail(
                    ce,
                    strike,
                    underlying_value,
                    atm_strike,
                    &available_strikes,
                    true, // is_call
                    days_to_expiry,
                )),
                put: opt.put.map(|pe| process_option_detail(
                    pe,
                    strike,
                    underlying_value,
                    atm_strike,
                    &available_strikes,
                    false, // is_call
                    days_to_expiry,
                )),
            })
        })
        .collect();
    
    // Step 5: Filter to ATM ±6 strikes + high OI outliers
    filter_strikes(&mut processed, atm_strike);
    
    (processed, spread)
}

/// Find ATM strike (closest to underlying, prefer floor)
fn find_atm_strike(data: &[OptionData], underlying_value: f64) -> f64 {
    let mut closest_strike = 0.0;
    let mut min_distance = f64::MAX;
    
    for opt in data {
        if let Some(strike) = opt.strike_price {
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
fn calculate_spread(data: &[OptionData], atm_strike: f64) -> f64 {
    // Get all strike prices and sort them
    let mut strikes: Vec<f64> = data
        .iter()
        .filter_map(|opt| opt.strike_price)
        .collect();
    strikes.sort_by(|a, b| a.partial_cmp(b).unwrap());
    
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

/// Process individual option detail with computed fields
fn process_option_detail(
    detail: OptionDetail,
    strike: f64,
    underlying_value: f64,
    atm_strike: f64,
    available_strikes: &[f64],
    is_call: bool,
    days_to_expiry: i32,
) -> ProcessedOptionDetail {
    // Step 2: Determine "the_money" with distance from ATM using indexing
    let the_money = classify_money_with_distance(strike, atm_strike, available_strikes, is_call);
    
    // Step 3: Calculate "Tambu"
    let tambu = calculate_tambu(&detail);
    
    // Step 4: Calculate "Time_val"
    let time_val = calculate_time_value(
        &detail,
        strike,
        underlying_value,
        is_call,
    );
    
    ProcessedOptionDetail {
        base: detail,
        the_money,
        tambu,
        time_val,
        days_to_expiry,
    }
}

/// Classify option as ATM, N ITM, or N OTM with distance calculation using strike indexing
fn classify_money_with_distance(
    strike: f64, 
    atm_strike: f64, 
    available_strikes: &[f64], // Pass all available strikes for indexing
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

/// Calculate Tambu classification
fn calculate_tambu(detail: &OptionDetail) -> Option<String> {
    let pchange_in_oi = detail.per_chg_oi.unwrap_or(0.0);
    let pchange = detail.per_chg_price.unwrap_or(0.0);
    
    // TMJ: pchangeinOpenInterest > 30% AND pchange < -15%
    if pchange_in_oi > 30.0 && pchange < -15.0 {
        return Some("TMJ".to_string());
    }
    
    // TMG: pchangeinOpenInterest < -10% AND pchange > 16%
    if pchange_in_oi < -10.0 && pchange > 16.0 {
        return Some("TMG".to_string());
    }
    
    None
}

/// Calculate time value
fn calculate_time_value(
    detail: &OptionDetail,
    strike: f64,
    underlying_value: f64,
    is_call: bool,
) -> f64 {
    let last_price = detail.last_price.unwrap_or(0.0);
    
    if is_call {
        // CE: Time_val = lastPrice - (underlyingValue - strikePrice) if underlyingValue > strikePrice
        //     otherwise Time_val = lastPrice
        if underlying_value > strike {
            last_price - (underlying_value - strike)
        } else {
            last_price
        }
    } else {
        // PE: Time_val = lastPrice - (strikePrice - underlyingValue) if strikePrice > underlyingValue
        //     otherwise Time_val = lastPrice
        if strike > underlying_value {
            last_price - (strike - underlying_value)
        } else {
            last_price
        }
    }
}

/// Filter to ATM ±6 strikes plus high OI outliers
fn filter_strikes(processed: &mut Vec<ProcessedOptionData>, atm_strike: f64) {
    // Sort by strike price
    processed.sort_by(|a, b| {
        let a_strike = a.strike_price.unwrap_or(0.0);
        let b_strike = b.strike_price.unwrap_or(0.0);
        a_strike.partial_cmp(&b_strike).unwrap()
    });
    
    // Find ATM index
    let atm_index = processed
        .iter()
        .position(|opt| opt.strike_price.unwrap_or(0.0) == atm_strike)
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
fn get_max_oi(opt: &ProcessedOptionData) -> f64 {
    let ce_oi = opt.call.as_ref()
        .and_then(|c| c.base.open_interest)   // flatten Option<Option<f64>>
        .unwrap_or(0.0);                      // final f64

    let pe_oi = opt.put.as_ref()
        .and_then(|p| p.base.open_interest)   // flatten Option<Option<f64>>
        .unwrap_or(0.0);

    ce_oi.max(pe_oi)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Local, Duration};
    
    #[test]
    fn test_calculate_days_to_expiry() {
        // Test with future date (tomorrow)
        let tomorrow = Local::now().date_naive() + Duration::days(1);
        let tomorrow_str = tomorrow.format("%d-%b-%Y").to_string();
        assert_eq!(calculate_days_to_expiry(&tomorrow_str).unwrap(), 1);
        
        // Test with today (should be 0)
        let today = Local::now().date_naive();
        let today_str = today.format("%d-%b-%Y").to_string();
        assert_eq!(calculate_days_to_expiry(&today_str).unwrap(), 0);
        
        // Test with past date (should error)
        let yesterday = Local::now().date_naive() - Duration::days(1);
        let yesterday_str = yesterday.format("%d-%b-%Y").to_string();
        assert!(calculate_days_to_expiry(&yesterday_str).is_err());
    }
    
    #[test]
    fn test_find_atm_strike() {
        let data = vec![
            OptionData {
                expiry_date: None,
                strike_price: Some(100.0),
                call: None,
                put: None,
            },
            OptionData {
                expiry_date: None,
                strike_price: Some(105.0),
                call: None,
                put: None,
            },
            OptionData {
                expiry_date: None,
                strike_price: Some(110.0),
                call: None,
                put: None,
            },
        ];
        
        // Underlying at 107.5 should choose 105 (floor)
        assert_eq!(find_atm_strike(&data, 107.5), 105.0);
        
        // Underlying at 102.5 should choose 100 (floor)
        assert_eq!(find_atm_strike(&data, 102.5), 100.0);
    }
    
    #[test]
    fn test_calculate_spread() {
        let data = vec![
            OptionData {
                expiry_date: None,
                strike_price: Some(100.0),
                call: None,
                put: None,
            },
            OptionData {
                expiry_date: None,
                strike_price: Some(110.0),
                call: None,
                put: None,
            },
            OptionData {
                expiry_date: None,
                strike_price: Some(120.0),
                call: None,
                put: None,
            },
        ];
        
        // ATM at 100, next higher is 110, spread should be 10
        assert_eq!(calculate_spread(&data, 100.0), 10.0);
    }
    
    #[test]
    fn test_classify_money_with_distance() {
        let strikes = vec![80.0, 90.0, 100.0, 110.0, 120.0];
        let atm = 100.0;
        
        // Call options
        assert_eq!(classify_money_with_distance(100.0, atm, &strikes, true), "ATM");
        assert_eq!(classify_money_with_distance(110.0, atm, &strikes, true), "1 OTM");
        assert_eq!(classify_money_with_distance(120.0, atm, &strikes, true), "2 OTM");
        assert_eq!(classify_money_with_distance(90.0, atm, &strikes, true), "1 ITM");
        assert_eq!(classify_money_with_distance(80.0, atm, &strikes, true), "2 ITM");
        
        // Put options
        assert_eq!(classify_money_with_distance(100.0, atm, &strikes, false), "ATM");
        assert_eq!(classify_money_with_distance(110.0, atm, &strikes, false), "1 ITM");
        assert_eq!(classify_money_with_distance(120.0, atm, &strikes, false), "2 ITM");
        assert_eq!(classify_money_with_distance(90.0, atm, &strikes, false), "1 OTM");
        assert_eq!(classify_money_with_distance(80.0, atm, &strikes, false), "2 OTM");
        
        // Test with uneven strikes
        let uneven_strikes = vec![90.0, 100.0, 120.0, 150.0];
        let atm_uneven = 100.0;
        
        // For uneven strikes: 90(idx=0), 100(idx=1), 120(idx=2), 150(idx=3)
        // Distance from ATM(idx=1): 90=1, 120=1, 150=2
        assert_eq!(classify_money_with_distance(120.0, atm_uneven, &uneven_strikes, true), "1 OTM");
        assert_eq!(classify_money_with_distance(150.0, atm_uneven, &uneven_strikes, true), "2 OTM");
        assert_eq!(classify_money_with_distance(90.0, atm_uneven, &uneven_strikes, true), "1 ITM");
    }
    
    #[test]
    fn test_time_value_calculation() {
        let detail = OptionDetail {
            // identifier: Some(String::new()),
            strike_price: Some(100.0),
            underlying_value: Some(110.0),
            open_interest: Some(1000.0),
            change_in_oi: Some(50.0),
            per_chg_oi: Some(5.0),
            last_price: Some(12.0),
            price_change: Some(1.0),
            per_chg_price: Some(5.0),
        };
        
        // CE: underlying (110) > strike (100), so time_val = 12 - (110-100) = 2
        assert_eq!(calculate_time_value(&detail, 100.0, 110.0, true), 2.0);
        
        // PE: strike (100) < underlying (110), so time_val = 12
        assert_eq!(calculate_time_value(&detail, 100.0, 110.0, false), 12.0);
    }

    #[test]
    fn test_negative_time_value_calculation() {
        let detail = OptionDetail {
            // identifier: Some(String::new()),
            strike_price: Some(100.0),
            underlying_value: Some(110.0),
            open_interest: Some(1000.0),
            change_in_oi: Some(50.0),
            per_chg_oi: Some(5.0),
            last_price: Some(6.0),
            price_change: Some(1.0),
            per_chg_price: Some(5.0),
        };
        
        // CE: underlying (110) > strike (100), so time_val = 6 - (110-100) = -4
        assert_eq!(calculate_time_value(&detail, 100.0, 110.0, true), -4.0);
        
        // PE: strike (100) < underlying (110), so time_val = 6
        assert_eq!(calculate_time_value(&detail, 100.0, 110.0, false), 6.0);
    }
}