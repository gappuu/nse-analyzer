
use nse_analyzer::nse::{
    calculate_days_to_expiry, 
    find_atm_strike, 
    calculate_spread, 
    classify_money_with_distance, 
    calculate_time_value,
    calculate_oi_rankings,
    OptionData,
    OptionDetail,
};

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
            oi_rank: None,
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
            oi_rank: None,
        };
        
        // CE: underlying (110) > strike (100), so time_val = 6 - (110-100) = -4
        assert_eq!(calculate_time_value(&detail, 100.0, 110.0, true), -4.0);
        
        // PE: strike (100) < underlying (110), so time_val = 6
        assert_eq!(calculate_time_value(&detail, 100.0, 110.0, false), 6.0);
    }

    #[test]
    fn test_oi_ranking() {
        // Create test data with different OI values
        let mut data = vec![
            OptionData {
                expiry_date: Some("30-Dec-2025".to_string()),
                strike_price: Some(100.0),
                call: Some(OptionDetail {
                    strike_price: Some(100.0),
                    underlying_value: Some(105.0),
                    open_interest: Some(1000.0), // Should get rank 2 for CE
                    change_in_oi: Some(0.0),
                    per_chg_oi: Some(0.0),
                    last_price: Some(5.0),
                    price_change: Some(0.0),
                    per_chg_price: Some(0.0),
                    oi_rank: None,
                }),
                put: Some(OptionDetail {
                    strike_price: Some(100.0),
                    underlying_value: Some(105.0),
                    open_interest: Some(2000.0), // Should get rank 1 for PE
                    change_in_oi: Some(0.0),
                    per_chg_oi: Some(0.0),
                    last_price: Some(3.0),
                    price_change: Some(0.0),
                    per_chg_price: Some(0.0),
                    oi_rank: None,
                }),
            },
            OptionData {
                expiry_date: Some("30-Dec-2025".to_string()),
                strike_price: Some(105.0),
                call: Some(OptionDetail {
                    strike_price: Some(105.0),
                    underlying_value: Some(105.0),
                    open_interest: Some(1500.0), // Should get rank 1 for CE
                    change_in_oi: Some(0.0),
                    per_chg_oi: Some(0.0),
                    last_price: Some(2.5),
                    price_change: Some(0.0),
                    per_chg_price: Some(0.0),
                    oi_rank: None,
                }),
                put: Some(OptionDetail {
                    strike_price: Some(105.0),
                    underlying_value: Some(105.0),
                    open_interest: Some(800.0), // Should get rank 2 for PE
                    change_in_oi: Some(0.0),
                    per_chg_oi: Some(0.0),
                    last_price: Some(2.5),
                    price_change: Some(0.0),
                    per_chg_price: Some(0.0),
                    oi_rank: None,
                }),
            },
        ];

        // Apply OI ranking
        calculate_oi_rankings(&mut data);

        // Check CE rankings: 105 strike (1500 OI) should be rank 1, 100 strike (1000 OI) should be rank 2
        assert_eq!(data[0].call.as_ref().unwrap().oi_rank, Some(2));
        assert_eq!(data[1].call.as_ref().unwrap().oi_rank, Some(1));

        // Check PE rankings: 100 strike (2000 OI) should be rank 1, 105 strike (800 OI) should be rank 2
        assert_eq!(data[0].put.as_ref().unwrap().oi_rank, Some(1));
        assert_eq!(data[1].put.as_ref().unwrap().oi_rank, Some(2));
    }

    #[test]
    fn test_oi_ranking_with_null_values() {
        // Test with some None values for open_interest
        let mut data = vec![
            OptionData {
                expiry_date: Some("30-Dec-2025".to_string()),
                strike_price: Some(100.0),
                call: Some(OptionDetail {
                    strike_price: Some(100.0),
                    underlying_value: Some(105.0),
                    open_interest: None, // Should not get a rank
                    change_in_oi: Some(0.0),
                    per_chg_oi: Some(0.0),
                    last_price: Some(5.0),
                    price_change: Some(0.0),
                    per_chg_price: Some(0.0),
                    oi_rank: None,
                }),
                put: Some(OptionDetail {
                    strike_price: Some(100.0),
                    underlying_value: Some(105.0),
                    open_interest: Some(1000.0), // Should get rank 1
                    change_in_oi: Some(0.0),
                    per_chg_oi: Some(0.0),
                    last_price: Some(3.0),
                    price_change: Some(0.0),
                    per_chg_price: Some(0.0),
                    oi_rank: None,
                }),
            },
            OptionData {
                expiry_date: Some("30-Dec-2025".to_string()),
                strike_price: Some(105.0),
                call: Some(OptionDetail {
                    strike_price: Some(105.0),
                    underlying_value: Some(105.0),
                    open_interest: Some(500.0), // Should get rank 1
                    change_in_oi: Some(0.0),
                    per_chg_oi: Some(0.0),
                    last_price: Some(2.5),
                    price_change: Some(0.0),
                    per_chg_price: Some(0.0),
                    oi_rank: None,
                }),
                put: None, // No PE option
            },
        ];

        // Apply OI ranking
        calculate_oi_rankings(&mut data);

        // Check that null OI doesn't get ranked
        assert_eq!(data[0].call.as_ref().unwrap().oi_rank, None);
        
        // Check that valid OI gets rank 1
        assert_eq!(data[1].call.as_ref().unwrap().oi_rank, Some(1));
        assert_eq!(data[0].put.as_ref().unwrap().oi_rank, Some(1));
    }
}

