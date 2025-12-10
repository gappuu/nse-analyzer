use crate::processor::{ProcessedOptionData, ProcessedOptionDetail};
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
            alerts.extend(check_option_rules(&symbol, strike, expiry_str, "CE", ce, spread, days_to_expiry));
        }
        
        // Check PE (Put)
        if let Some(pe) = &opt.put {
            alerts.extend(check_option_rules(&symbol, strike, expiry_str, "PE", pe, spread, days_to_expiry));
        }
    }
    
    // Skip if no alerts
    if alerts.is_empty() {
        return None;
    }

    // valid the_money values
    const VALID_MONEY: [&str; 7] = ["ATM", "1 OTM", "1 ITM", "2 OTM", "2 ITM", "3 OTM", "3 ITM"];

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
fn check_option_rules(
    symbol: &str,
    strike: f64,
    expiry: &str,
    option_type: &str,
    detail: &ProcessedOptionDetail,
    spread: f64,  // Added spread parameter
    days_to_expiry: i32,  // Added days_to_expiry parameter
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
            spread,  // Include spread value
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
            spread,  // Include spread value
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
    
    // Rule 3: Low price options (< 2)
    if let Some(lp) = last_price {
        if lp > 0.0 && lp < 2.0 {
        alerts.push(Alert {
            symbol: symbol.to_string(),
            strike_price: strike,
            expiry_date: expiry.to_string(),
            option_type: option_type.to_string(),
            alert_type: "LOW_PRICE".to_string(),
            description: format!(
                "{} {} {} strike has low price of ₹{:.2} ({} days to expiry)",
                symbol, option_type, strike, lp, days_to_expiry
            ),
            spread,  // Include spread value
            values: AlertValues {
                pchange_in_oi: Some(pchange_in_oi),
                last_price: Some(lp),  
                open_interest,
                the_money: Some(detail.the_money.clone()),
                time_val: detail.time_val.clone(),
                days_to_expiry,
            },
        });
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::OptionDetail;
    use crate::processor::ProcessedOptionDetail;
    
    #[test]
    fn test_huge_oi_increase_detection() {
        let detail = ProcessedOptionDetail {
            base: OptionDetail {
                // identifier: "TEST".to_string(),
                strike_price: Some(100.0),
                underlying_value: Some(105.0),
                open_interest: Some(10000.0),
                change_in_oi: Some(9000.0),
                per_chg_oi: Some(1500.0),  // 1500% increase
                last_price: Some(5.0),
                price_change: Some(1.0),
                per_chg_price: Some(20.0),
            },
            the_money: "OTM".to_string(),
            tambu: None,
            time_val: 4.0,
            days_to_expiry: 15,
        };
        
        let alerts = check_option_rules("NIFTY", 100.0, "30-DEC-2025", "CE", &detail, 2.5, 15);
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].alert_type, "HUGE_OI_INCREASE");
        assert_eq!(alerts[0].spread, 2.5);
        assert_eq!(alerts[0].values.days_to_expiry, 15);
    }
    
    #[test]
    fn test_huge_oi_decrease_detection() {
        let detail = ProcessedOptionDetail {
            base: OptionDetail {
                // identifier: "TEST".to_string(),
                strike_price: Some(100.0),
                underlying_value: Some(105.0),
                open_interest: Some(5000.0),
                change_in_oi: Some(-5000.0),
                per_chg_oi: Some(-60.0),  // -60% decrease
                last_price: Some(5.0),
                price_change: Some(-1.0),
                per_chg_price: Some(-20.0),
            },
            the_money: "OTM".to_string(),
            tambu: None,
            time_val: 4.0,
            days_to_expiry: 10,
        };
        
        let alerts = check_option_rules("NIFTY", 100.0, "30-DEC-2025", "CE", &detail, 3.0, 10);
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].alert_type, "HUGE_OI_DECREASE");
        assert_eq!(alerts[0].spread, 3.0);
        assert_eq!(alerts[0].values.days_to_expiry, 10);
    }
    
    #[test]
    fn test_low_price_detection() {
        let detail = ProcessedOptionDetail {
            base: OptionDetail {
                // identifier: "TEST".to_string(),
                strike_price: Some(100.0),
                underlying_value: Some(105.0),
                open_interest: Some(10000.0),
                change_in_oi: Some(100.0),
                per_chg_oi: Some(5.0),
                last_price: Some(1.5),  // Low price
                price_change: Some(0.5),
                per_chg_price: Some(50.0),
            },
            the_money: "OTM".to_string(),
            tambu: None,
            time_val: 1.5,
            days_to_expiry: 20,
        };
        
        let alerts = check_option_rules("NIFTY", 100.0, "30-DEC-2025", "CE", &detail, 1.8, 20);
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].alert_type, "LOW_PRICE");
        assert_eq!(alerts[0].spread, 1.8);
        assert_eq!(alerts[0].values.days_to_expiry, 20);
    }
    
    #[test]
    fn test_multiple_alerts() {
        let detail = ProcessedOptionDetail {
            base: OptionDetail {
                // identifier: "TEST".to_string(),
                strike_price: Some(100.0),
                underlying_value: Some(105.0),
                open_interest: Some(10000.0),
                change_in_oi: Some(9000.0),
                per_chg_oi: Some(1500.0),  // Huge increase
                last_price: Some(1.0),  // Low price
                price_change: Some(0.5),
                per_chg_price: Some(50.0),
            },
            the_money: "OTM".to_string(),
            tambu: None,
            time_val: 1.0,
            days_to_expiry: 5,  // Less than 7 days
        };
        
        let alerts = check_option_rules("NIFTY", 100.0, "15-DEC-2025", "CE", &detail, 2.2, 5);
        // Should have 3 alerts: HUGE_OI_INCREASE, LOW_PRICE
        assert_eq!(alerts.len(), 2);
        assert!(alerts.iter().all(|a| a.spread == 2.2));
        assert!(alerts.iter().all(|a| a.values.days_to_expiry == 5));
        
        let alert_types: Vec<&String> = alerts.iter().map(|a| &a.alert_type).collect();
        assert!(alert_types.contains(&&"HUGE_OI_INCREASE".to_string()));
        assert!(alert_types.contains(&&"LOW_PRICE".to_string()));
    }
}