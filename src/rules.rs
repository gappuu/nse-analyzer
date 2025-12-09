use crate::processor::{ProcessedOptionData, ProcessedOptionDetail};
use serde::{Deserialize, Serialize};

/// Alert types for option strikes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub symbol: String,
    pub strike_price: f64,
    pub option_type: String,  // "CE" or "PE"
    pub alert_type: String,   // Type of alert
    pub description: String,
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
) -> Option<RulesOutput> {
    let mut alerts = Vec::new();
    
    for opt in data {
        let strike = opt.strike_price.unwrap_or(0.0);
        // Check CE (Call)
        if let Some(ce) = &opt.call {
            alerts.extend(check_option_rules(&symbol,strike, "CE", ce));
        }
        
        // Check PE (Put)
        if let Some(pe) = &opt.put {
            alerts.extend(check_option_rules(&symbol,strike, "PE", pe));
        }
    }
    
    // Skip if no alerts
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
    option_type: &str,
    detail: &ProcessedOptionDetail,
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
            option_type: option_type.to_string(),
            alert_type: "HUGE_OI_INCREASE".to_string(),
            description: format!(
                "{} {} {} strike has massive OI increase of {:.2}%",
                symbol, option_type, strike, pchange_in_oi
            ),
            values: AlertValues {
                pchange_in_oi: Some(pchange_in_oi),
                last_price,
                open_interest,
                the_money: Some(detail.the_money.clone()),
                time_val: detail.time_val.clone(),
            },
        });
    }
    
    // Rule 2: Huge OI decrease (< -50%)
    if pchange_in_oi < -50.0 {
        alerts.push(Alert {
            symbol: symbol.to_string(),
            strike_price: strike,
            option_type: option_type.to_string(),
            alert_type: "HUGE_OI_DECREASE".to_string(),
            description: format!(
                "{} {} {} strike has massive OI decrease of {:.2}%",
                symbol, option_type, strike, pchange_in_oi
            ),
            values: AlertValues {
                pchange_in_oi: Some(pchange_in_oi),
                last_price,
                open_interest,
                the_money: Some(detail.the_money.clone()),
                time_val: detail.time_val.clone(),
            },
        });
    }
    
    // Rule 3: Low price options (< 2)
    if let Some(lp) = last_price {
        if lp > 0.0 && lp < 2.0 {
        alerts.push(Alert {
            symbol: symbol.to_string(),
            strike_price: strike,
            option_type: option_type.to_string(),
            alert_type: "LOW_PRICE".to_string(),
            description: format!(
                "{} {} {} strike has low price of ₹{:.2}",
                symbol, option_type, strike, lp
            ),
            values: AlertValues {
                pchange_in_oi: Some(pchange_in_oi),
                last_price: Some(lp),  
                open_interest,
                the_money: Some(detail.the_money.clone()),
                time_val: detail.time_val.clone(),
            },
        });
        }
    }
    
    alerts
}

/// Run rules on batch data
pub fn run_batch_rules(
    batch_data: Vec<(String, String, f64, Vec<ProcessedOptionData>)>,
) -> Vec<RulesOutput> {
    batch_data
        .into_iter()
        .filter_map(|(symbol, timestamp, underlying_value, data)| {
            run_rules(&data, symbol, timestamp, underlying_value)
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
        };
        
        let alerts = check_option_rules("NIFTY",100.0, "CE", &detail);
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].alert_type, "HUGE_OI_INCREASE");
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
        };
        
        let alerts = check_option_rules("NIFTY",100.0, "CE", &detail);
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].alert_type, "HUGE_OI_DECREASE");
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
        };
        
        let alerts = check_option_rules("NIFTY",100.0, "CE", &detail);
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].alert_type, "LOW_PRICE");
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
        };
        
        let alerts = check_option_rules("NIFTY",100.0, "CE", &detail);
        assert_eq!(alerts.len(), 2);  // Both HUGE_OI_INCREASE and LOW_PRICE
    }
}