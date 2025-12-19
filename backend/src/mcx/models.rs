use serde::{Deserialize, Serialize};

// -----------------------------------------------
// NEW MCX DATA STRUCTURES (from working implementation)
// -----------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McxSymbolData {
    #[serde(rename = "ExtensionData")]
    pub extension_data: serde_json::Value,
    
    #[serde(rename = "ExpiryDate")]
    pub expiry_date: String,
    
    #[serde(rename = "InstrumentName")]
    pub instrument_name: String,
    
    #[serde(rename = "Symbol")]
    pub symbol: String,
    
    #[serde(rename = "SymbolValue")]
    pub symbol_value: String,
    
    #[serde(rename = "TodaysTraded")]
    pub todays_traded: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McxContractInfo {
    pub symbol: String,
    pub expiry_dates: Vec<String>,
    pub instrument_name: String,
    pub symbol_value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McxOptionChainResponse {
    pub d: McxOptionChainData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McxOptionChainData {
    #[serde(rename = "__type")]
    pub type_name: String,
    
    #[serde(rename = "ExtensionData")]
    pub extension_data: serde_json::Value,
    
    #[serde(rename = "Data")]
    pub data: Vec<McxOptionData>,
    
    #[serde(rename = "Summary")]
    pub summary: McxOptionSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McxOptionData {
    #[serde(rename = "ExtensionData")]
    pub extension_data: serde_json::Value,
    
    // Call Option (CE) fields
    #[serde(rename = "CE_AbsoluteChange")]
    pub ce_absolute_change: Option<f64>,
    
    #[serde(rename = "CE_AskPrice")]
    pub ce_ask_price: Option<f64>,
    
    #[serde(rename = "CE_AskQty")]
    pub ce_ask_qty: Option<i32>,
    
    #[serde(rename = "CE_BidPrice")]
    pub ce_bid_price: Option<f64>,
    
    #[serde(rename = "CE_BidQty")]
    pub ce_bid_qty: Option<i32>,
    
    #[serde(rename = "CE_ChangeInOI")]
    pub ce_change_in_oi: Option<i32>,
    
    #[serde(rename = "CE_LTP")]
    pub ce_ltp: Option<f64>,
    
    #[serde(rename = "CE_LTT")]
    pub ce_ltt: Option<String>,
    
    #[serde(rename = "CE_NetChange")]
    pub ce_net_change: Option<f64>,
    
    #[serde(rename = "CE_OpenInterest")]
    pub ce_open_interest: Option<i32>,
    
    #[serde(rename = "CE_StrikePrice")]
    pub ce_strike_price: Option<f64>,
    
    #[serde(rename = "CE_Volume")]
    pub ce_volume: Option<i32>,
    
    // Put Option (PE) fields
    #[serde(rename = "PE_AbsoluteChange")]
    pub pe_absolute_change: Option<f64>,
    
    #[serde(rename = "PE_AskPrice")]
    pub pe_ask_price: Option<f64>,
    
    #[serde(rename = "PE_AskQty")]
    pub pe_ask_qty: Option<i32>,
    
    #[serde(rename = "PE_BidPrice")]
    pub pe_bid_price: Option<f64>,
    
    #[serde(rename = "PE_BidQty")]
    pub pe_bid_qty: Option<i32>,
    
    #[serde(rename = "PE_ChangeInOI")]
    pub pe_change_in_oi: Option<i32>,
    
    #[serde(rename = "PE_LTP")]
    pub pe_ltp: Option<f64>,
    
    #[serde(rename = "PE_LTT")]
    pub pe_ltt: Option<String>,
    
    #[serde(rename = "PE_NetChange")]
    pub pe_net_change: Option<f64>,
    
    #[serde(rename = "PE_OpenInterest")]
    pub pe_open_interest: Option<i32>,
    
    #[serde(rename = "PE_Volume")]
    pub pe_volume: Option<i32>,
    
    // Common fields
    #[serde(rename = "ExpiryDate")]
    pub expiry_date: Option<String>,
    
    #[serde(rename = "LTT")]
    pub ltt: Option<String>,
    
    #[serde(rename = "Symbol")]
    pub symbol: Option<String>,
    
    #[serde(rename = "UnderlyingValue")]
    pub underlying_value: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McxOptionSummary {
    #[serde(rename = "ExtensionData")]
    pub extension_data: serde_json::Value,
    
    #[serde(rename = "AsOn")]
    pub as_on: Option<String>,
    
    #[serde(rename = "Count")]
    pub count: Option<i32>,
    
    #[serde(rename = "Status")]
    pub status: Option<String>,
}

// -----------------------------------------------
// LEGACY TICKER DATA FROM INITIAL SCRAPE (for compatibility)
// -----------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ticker {
    #[serde(rename = "ExpiryDate")]
    pub expiry_date: String,
    
    #[serde(rename = "InstrumentName")]
    pub instrument_name: String,
    
    #[serde(rename = "Symbol")]
    pub symbol: String,
    
    #[serde(rename = "SymbolValue")]
    pub symbol_value: String,
    
    #[serde(rename = "TodaysTraded")]
    pub todays_traded: i32,
}

// -----------------------------------------------
// LEGACY OPTION CHAIN RESPONSE STRUCTURES (for compatibility)
// -----------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionChainResponse {
    pub d: OptionChainData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionChainData {
    #[serde(rename = "__type")]
    pub type_name: String,
    
    #[serde(rename = "ExtensionData")]
    pub extension_data: serde_json::Value,
    
    #[serde(rename = "Data")]
    pub data: Vec<OptionData>,
    
    #[serde(rename = "Summary")]
    pub summary: OptionSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionData {
    #[serde(rename = "ExtensionData")]
    pub extension_data: serde_json::Value,
    
    // Call Option (CE) fields
    #[serde(rename = "CE_AbsoluteChange")]
    pub ce_absolute_change: Option<f64>,
    
    #[serde(rename = "CE_AskPrice")]
    pub ce_ask_price: Option<f64>,
    
    #[serde(rename = "CE_AskQty")]
    pub ce_ask_qty: Option<i32>,
    
    #[serde(rename = "CE_BidPrice")]
    pub ce_bid_price: Option<f64>,
    
    #[serde(rename = "CE_BidQty")]
    pub ce_bid_qty: Option<i32>,
    
    #[serde(rename = "CE_ChangeInOI")]
    pub ce_change_in_oi: Option<i32>,
    
    #[serde(rename = "CE_LTP")]
    pub ce_ltp: Option<f64>,
    
    #[serde(rename = "CE_LTT")]
    pub ce_ltt: Option<String>,
    
    #[serde(rename = "CE_NetChange")]
    pub ce_net_change: Option<f64>,
    
    #[serde(rename = "CE_OpenInterest")]
    pub ce_open_interest: Option<i32>,
    
    #[serde(rename = "CE_StrikePrice")]
    pub ce_strike_price: Option<f64>,
    
    #[serde(rename = "CE_Volume")]
    pub ce_volume: Option<i32>,
    
    // Put Option (PE) fields
    #[serde(rename = "PE_AbsoluteChange")]
    pub pe_absolute_change: Option<f64>,
    
    #[serde(rename = "PE_AskPrice")]
    pub pe_ask_price: Option<f64>,
    
    #[serde(rename = "PE_AskQty")]
    pub pe_ask_qty: Option<i32>,
    
    #[serde(rename = "PE_BidPrice")]
    pub pe_bid_price: Option<f64>,
    
    #[serde(rename = "PE_BidQty")]
    pub pe_bid_qty: Option<i32>,
    
    #[serde(rename = "PE_ChangeInOI")]
    pub pe_change_in_oi: Option<i32>,
    
    #[serde(rename = "PE_LTP")]
    pub pe_ltp: Option<f64>,
    
    #[serde(rename = "PE_LTT")]
    pub pe_ltt: Option<String>,
    
    #[serde(rename = "PE_NetChange")]
    pub pe_net_change: Option<f64>,
    
    #[serde(rename = "PE_OpenInterest")]
    pub pe_open_interest: Option<i32>,
    
    #[serde(rename = "PE_Volume")]
    pub pe_volume: Option<i32>,
    
    // Common fields
    #[serde(rename = "ExpiryDate")]
    pub expiry_date: Option<String>,
    
    #[serde(rename = "LTT")]
    pub ltt: Option<String>,
    
    #[serde(rename = "Symbol")]
    pub symbol: Option<String>,
    
    #[serde(rename = "UnderlyingValue")]
    pub underlying_value: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionSummary {
    #[serde(rename = "ExtensionData")]
    pub extension_data: serde_json::Value,
    
    #[serde(rename = "AsOn")]
    pub as_on: Option<String>,
    
    #[serde(rename = "Count")]
    pub count: Option<i32>,
    
    #[serde(rename = "Status")]
    pub status: Option<String>,
}

// -----------------------------------------------
// API PAYLOAD STRUCTURES
// -----------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionChainPayload {
    #[serde(rename = "Commodity")]
    pub commodity: String,
    
    #[serde(rename = "Expiry")]
    pub expiry: String,
}
