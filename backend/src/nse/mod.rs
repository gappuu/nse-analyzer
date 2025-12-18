pub mod config;
pub mod models;
pub mod nse_client;
pub mod processor;
pub mod rules;
pub mod nse_api_server;
pub mod nse_commands;

// Re-exports (public API)
pub use nse_client::NSEClient;
pub use models::{Security, SecurityType, OptionChain, OptionData, OptionDetail};
pub use processor::{
    calculate_days_to_expiry, 
    find_atm_strike, 
    calculate_spread, 
    classify_money_with_distance, 
    calculate_time_value,
    calculate_oi_rankings,
    ProcessedOptionData, 
    ProcessedOptionDetail,
    };
pub use rules::{run_rules, check_option_rules, Alert, AlertValues, RulesOutput};