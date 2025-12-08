use crate::models::{OptionData, OptionDetail};
use serde::{Deserialize, Serialize};

/// Enhanced option detail with computed fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedOptionDetail {
    #[serde(flatten)]
    pub base: OptionDetail,
    
    pub the_money: String,  // "ATM", "ITM", "OTM"
    pub tambu: Option<String>,  // "TMJ", "TMG", or None
    pub time_val: f64,
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
}

/// Process option chain data
pub fn process_option_data(
    data: Vec<OptionData>,
    underlying_value: f64,
) -> Vec<ProcessedOptionData> {
    // Step 1: Identify ATM strike
    let atm_strike = find_atm_strike(&data, underlying_value);
    
    // Step 2-4: Process each strike with classifications
    let mut processed: Vec<ProcessedOptionData> = data
        .into_iter()
        .map(|opt| {
            let strike = opt.strike_price.unwrap_or(0.0);
            
            ProcessedOptionData {
                expiry_date: opt.expiry_date.clone(),
                strike_price: opt.strike_price,
                call: opt.call.map(|ce| process_option_detail(
                    ce,
                    strike,
                    underlying_value,
                    atm_strike,
                    true, // is_call
                )),
                put: opt.put.map(|pe| process_option_detail(
                    pe,
                    strike,
                    underlying_value,
                    atm_strike,
                    false, // is_call
                )),
            }
        })
        .collect();
    
    // Step 5: Filter to ATM ±6 strikes + high OI outliers
    filter_strikes(&mut processed, atm_strike);
    
    processed
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

/// Process individual option detail with computed fields
fn process_option_detail(
    detail: OptionDetail,
    strike: f64,
    underlying_value: f64,
    atm_strike: f64,
    is_call: bool,
) -> ProcessedOptionDetail {
    // Step 2: Determine "the_money"
    let the_money = classify_money(strike, atm_strike, is_call);
    
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
    }
}

/// Classify option as ATM, ITM, or OTM
fn classify_money(strike: f64, atm_strike: f64, is_call: bool) -> String {
    if strike == atm_strike {
        "ATM".to_string()
    } else if is_call {
        // Call: Above ATM = OTM, Below ATM = ITM
        if strike > atm_strike {
            "OTM".to_string()
        } else {
            "ITM".to_string()
        }
    } else {
        // Put: Above ATM = ITM, Below ATM = OTM
        if strike > atm_strike {
            "ITM".to_string()
        } else {
            "OTM".to_string()
        }
    }
}

/// Calculate Tambu classification
fn calculate_tambu(detail: &OptionDetail) -> Option<String> {
    let pchange_in_oi = detail.per_chg_oi.unwrap_or(0.0);
    let pchange = detail.per_chg_oi.unwrap_or(0.0);
    
    // TMJ: pchangeinOpenInterest > 10% AND pchange > -15%
    if pchange_in_oi > 10.0 && pchange > -15.0 {
        return Some("TMJ".to_string());
    }
    
    // TMG: pchangeinOpenInterest < -11% AND pchange > 16%
    if pchange_in_oi < -11.0 && pchange > 16.0 {
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
        a.strike_price
            .unwrap_or(0.0)
            .partial_cmp(&b.strike_price.unwrap_or(0.0))
            .unwrap()
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
        .map(|c| c.base.open_interest)
        .unwrap_or(Some(0.0));
    
    let pe_oi = opt.put.as_ref()
        .map(|p| p.base.open_interest)
        .unwrap_or(Some(0.0));
    
    ce_oi.max(pe_oi)
}

#[cfg(test)]
mod tests {
    use super::*;
    
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
    fn test_classify_money() {
        let atm = 100.0;
        
        // Call options
        assert_eq!(classify_money(100.0, atm, true), "ATM");
        assert_eq!(classify_money(105.0, atm, true), "OTM");
        assert_eq!(classify_money(95.0, atm, true), "ITM");
        
        // Put options
        assert_eq!(classify_money(100.0, atm, false), "ATM");
        assert_eq!(classify_money(105.0, atm, false), "ITM");
        assert_eq!(classify_money(95.0, atm, false), "OTM");
    }
    
    #[test]
    fn test_time_value_calculation() {
        let detail = OptionDetail {
            identifier: String::new(),
            strike_price: 100.0,
            underlying_value: 110.0,
            open_interest: 1000.0,
            change_in_oi: 50.0,
            per_chg_oi: 5.0,
            last_price: 12.0,
            price_change: 1.0,
            per_chg_price: 5.0,
        };
        
        // CE: underlying (110) > strike (100), so time_val = 12 - (110-100) = 2
        assert_eq!(calculate_time_value(&detail, 100.0, 110.0, true), 2.0);
        
        // PE: strike (100) < underlying (110), so time_val = 12
        assert_eq!(calculate_time_value(&detail, 100.0, 110.0, false), 12.0);
    }
}