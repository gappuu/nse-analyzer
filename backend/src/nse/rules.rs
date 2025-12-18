use super::processor::{ProcessedOptionData, ProcessedOptionDetail};
use serde::{Deserialize, Serialize};

/// Alert types for option strikes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub symbol: String,
    pub strike_price: f64,
    pub expiry_date: String,
    pub option_type: String,  // "CE" or "PE"
    pub alert_type: String,   // Type of alert
    pub description: String,
    pub spread: f64,          // Spread value
    pub values: AlertValues,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertValues {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pchange_in_oi: Option<f64>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_price: Option<f64>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_interest: Option<f64>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub the_money: Option<String>,

    pub time_val: f64,
    pub days_to_expiry: i32,  // Days remaining until expiry
}

/// Rules output structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RulesOutput {
    pub symbol: String,
    pub timestamp: String,
    pub underlying_value: f64,
    pub alerts: Vec<Alert>,
}

/// Run rules on processed option data
pub fn run_rules(
    data: &[ProcessedOptionData],
    symbol: String,
    timestamp: String,
    underlying_value: f64,
    spread: f64,  // Added spread parameter
) -> Option<RulesOutput> {
    let mut alerts = Vec::new();
    
    for opt in data {
        let strike = opt.strike_price.unwrap_or(0.0);
        let expiry_str = opt
        .expiry_date
        .as_deref()                  
        .unwrap_or("UNKNOWN");
        let days_to_expiry = opt.days_to_expiry;
        
        // Check CE (Call)
        if let Some(ce) = &opt.call {
            alerts.extend(check_option_rules(&symbol, strike, expiry_str, "CE", ce, spread, days_to_expiry, underlying_value));
        }
        
        // Check PE (Put)
        if let Some(pe) = &opt.put {
            alerts.extend(check_option_rules(&symbol, strike, expiry_str, "PE", pe, spread, days_to_expiry, underlying_value));
        }
    }
    
    // Skip if no alerts
    if alerts.is_empty() {
        return None;
    }

    // valid the_money values
    const VALID_MONEY: [&str; 5] = ["ATM", "1 OTM", "1 ITM", "2 OTM", "2 ITM"];

    // Filter alerts based on the_money values
    alerts.retain(|a| {
        if let Some(ref m) = a.values.the_money {
            VALID_MONEY.contains(&m.as_str())
        } else {
            false
        }
    });

    // Skip after filter too
    if alerts.is_empty() {
        return None;
    }
    
    Some(RulesOutput {
        symbol,
        timestamp,
        underlying_value,
        alerts,
    })
}

/// Check rules for a single option (CE or PE)
pub fn check_option_rules(
    symbol: &str,
    strike: f64,
    expiry: &str,
    option_type: &str,
    detail: &ProcessedOptionDetail,
    spread: f64,  
    days_to_expiry: i32, 
    underlying_value: f64,

) -> Vec<Alert> {
    let mut alerts = Vec::new();
    let pchange_in_oi = detail.base.per_chg_oi.unwrap_or(0.0);
    let last_price = detail.base.last_price;
    let open_interest = detail.base.open_interest;
    
    // Rule 1: Huge OI increase (> 1000%)
    if pchange_in_oi > 1000.0 {
        alerts.push(Alert {
            symbol: symbol.to_string(),
            strike_price: strike,
            expiry_date: expiry.to_string(),
            option_type: option_type.to_string(),
            alert_type: "HUGE_OI_INCREASE".to_string(),
            description: format!(
                "{} {} {} strike has massive OI increase of {:.2}% ({} days to expiry)",
                symbol, option_type, strike, pchange_in_oi, days_to_expiry
            ),
            spread,  
            values: AlertValues {
                pchange_in_oi: Some(pchange_in_oi),
                last_price,
                open_interest,
                the_money: Some(detail.the_money.clone()),
                time_val: detail.time_val.clone(),
                days_to_expiry,
            },
        });
    }
    
    // Rule 2: Huge OI decrease (< -50%)
    if pchange_in_oi < -50.0 {
        alerts.push(Alert {
            symbol: symbol.to_string(),
            strike_price: strike,
            expiry_date: expiry.to_string(),
            option_type: option_type.to_string(),
            alert_type: "HUGE_OI_DECREASE".to_string(),
            description: format!(
                "{} {} {} strike has massive OI decrease of {:.2}% ({} days to expiry)",
                symbol, option_type, strike, pchange_in_oi, days_to_expiry
            ),
            spread,  
            values: AlertValues {
                pchange_in_oi: Some(pchange_in_oi),
                last_price,
                open_interest,
                the_money: Some(detail.the_money.clone()),
                time_val: detail.time_val.clone(),
                days_to_expiry,
            },
        });
    }
    
    // Rule 3: Low price options 
    let tv = detail.time_val.clone();
    let d = days_to_expiry.max(1) as f64; // avoid divide by zero
    let max_factor = if days_to_expiry <= 3 { 0.002 } else { 0.001 };

    let is_cheap =
        tv > 0.0 &&
        tv < max_factor * underlying_value &&     // time value very small vs spot
        (tv / d) < 0.0005 * underlying_value;     // per-day time cost tiny

    if is_cheap && matches!(detail.the_money.as_str(), "ATM" | "1 OTM" | "1 ITM"){
        alerts.push(Alert {
            symbol: symbol.to_string(),
            strike_price: strike,
            expiry_date: expiry.to_string(),
            option_type: option_type.to_string(),
            alert_type: "LOW_PRICE".to_string(),
            description: format!(
                "{} {} {} strike has low price of ₹{:.2} ({} days to expiry)",
                symbol, option_type, strike, last_price.unwrap_or(0.0), days_to_expiry
            ),
            spread,  
            values: AlertValues {
                pchange_in_oi: Some(pchange_in_oi),
                last_price: Some(last_price.unwrap_or(0.0) ),  
                open_interest,
                the_money: Some(detail.the_money.clone()),
                time_val: detail.time_val.clone(),
                days_to_expiry,
            },
        });
        }

    // Rule 4: Negative Time Value
    if tv < -0.0 && last_price.is_some_and(|lp| lp > 0.0) && matches!(detail.the_money.as_str(), "ATM" | "1 OTM" | "1 ITM") {
        alerts.push(Alert {
            symbol: symbol.to_string(),
            strike_price: strike,
            expiry_date: expiry.to_string(),
            option_type: option_type.to_string(),
            alert_type: "NEGATIVE TIME VALUE".to_string(),
            description: format!(
                "{} {} {} strike has Negative Time Value of {} ({} days to expiry)",
                symbol, option_type, strike, tv, days_to_expiry
            ),
            spread,  
            values: AlertValues {
                pchange_in_oi: Some(pchange_in_oi),
                last_price,
                open_interest,
                the_money: Some(detail.the_money.clone()),
                time_val: detail.time_val.clone(),
                days_to_expiry,
            },
        });
    }
    
    alerts
}

/// Run rules on batch data
pub fn run_batch_rules(
    batch_data: Vec<(String, String, f64, Vec<ProcessedOptionData>, f64)>,  // Added spread to tuple
) -> Vec<RulesOutput> {
    batch_data
        .into_iter()
        .filter_map(|(symbol, timestamp, underlying_value, data, spread)| {
            run_rules(&data, symbol, timestamp, underlying_value, spread)
        })
        .collect()
}
