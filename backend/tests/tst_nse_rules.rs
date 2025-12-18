use nse_analyzer::nse::{
    ProcessedOptionDetail,
    check_option_rules,
    OptionDetail
};

#[cfg(test)]
mod tests {
    use super::*;
    
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
                oi_rank:Some(1)
            },
            the_money: "OTM".to_string(),
            tambu: None,
            time_val: 4.0,
            days_to_expiry: 15,
        };
        
        let alerts = check_option_rules("NIFTY", 100.0, "30-DEC-2025", "CE", &detail, 2.5, 15,105.0);
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
                oi_rank:Some(10),
            },
            the_money: "OTM".to_string(),
            tambu: None,
            time_val: 4.0,
            days_to_expiry: 10,
        };
        
        let alerts = check_option_rules("NIFTY", 100.0, "30-DEC-2025", "CE", &detail, 3.0, 10,105.0);
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].alert_type, "HUGE_OI_DECREASE");
        assert_eq!(alerts[0].spread, 3.0);
        assert_eq!(alerts[0].values.days_to_expiry, 10);
    }
    
    // following not working. check again
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
                oi_rank:Some(5),
            },
            the_money: "OTM".to_string(),
            tambu: None,
            time_val: 1.5,
            days_to_expiry: 20,
        };
        
        let alerts = check_option_rules("NIFTY", 100.0, "30-DEC-2025", "CE", &detail, 1.8, 20, 105.0);
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
                oi_rank:Some(2),
            },
            the_money: "OTM".to_string(),
            tambu: None,
            time_val: 1.0,
            days_to_expiry: 5,  // Less than 7 days
        };
        
        let alerts = check_option_rules("NIFTY", 100.0, "15-DEC-2025", "CE", &detail, 2.2, 5, 105.0);
        // Should have 3 alerts: HUGE_OI_INCREASE, LOW_PRICE
        assert_eq!(alerts.len(), 2);
        assert!(alerts.iter().all(|a| a.spread == 2.2));
        assert!(alerts.iter().all(|a| a.values.days_to_expiry == 5));
        
        let alert_types: Vec<&String> = alerts.iter().map(|a| &a.alert_type).collect();
        assert!(alert_types.contains(&&"HUGE_OI_INCREASE".to_string()));
        assert!(alert_types.contains(&&"LOW_PRICE".to_string()));
    }
}