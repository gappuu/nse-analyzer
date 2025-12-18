use serde::{Deserialize, Serialize};

// -----------------------------------------------
// MCX MODELS - RAW API RESPONSE ONLY
// -----------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McxSecurity {
    pub symbol: String,
    pub symbol_value: String,
    pub instrument_name: String,
}

impl McxSecurity {
    pub fn new(symbol: String, symbol_value: String, instrument_name: String) -> Self {
        Self {
            symbol,
            symbol_value,
            instrument_name,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McxContractInfo {
    pub symbol: String,
    pub expiry_dates: Vec<String>,
    pub instrument_name: String,
    pub symbol_value: String,
}

// MCX option chain raw response - matches actual API structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McxOptionChainResponse {
    #[serde(rename = "d")]
    pub d: Option<McxResponseData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McxResponseData {
    #[serde(rename = "__type")]
    pub type_name: Option<String>,
    
    #[serde(rename = "ExtensionData")]
    pub extension_data: Option<serde_json::Value>,
    
    #[serde(rename = "Data")]
    pub data: Option<Vec<McxOptionData>>,
    
    #[serde(rename = "Summary")]
    pub summary: Option<McxSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McxOptionData {
    #[serde(rename = "ExtensionData")]
    pub extension_data: Option<serde_json::Value>,
    
    // Call option fields
    #[serde(rename = "CE_AbsoluteChange")]
    pub ce_absolute_change: Option<f64>,
    
    #[serde(rename = "CE_AskPrice")]
    pub ce_ask_price: Option<f64>,
    
    #[serde(rename = "CE_AskQty")]
    pub ce_ask_qty: Option<f64>,
    
    #[serde(rename = "CE_BidPrice")]
    pub ce_bid_price: Option<f64>,
    
    #[serde(rename = "CE_BidQty")]
    pub ce_bid_qty: Option<f64>,
    
    #[serde(rename = "CE_ChangeInOI")]
    pub ce_change_in_oi: Option<f64>,
    
    #[serde(rename = "CE_LTP")]
    pub ce_ltp: Option<f64>,
    
    #[serde(rename = "CE_LTT")]
    pub ce_ltt: Option<String>,
    
    #[serde(rename = "CE_NetChange")]
    pub ce_net_change: Option<f64>,
    
    #[serde(rename = "CE_OpenInterest")]
    pub ce_open_interest: Option<f64>,
    
    #[serde(rename = "CE_StrikePrice")]
    pub ce_strike_price: Option<f64>,
    
    #[serde(rename = "CE_Volume")]
    pub ce_volume: Option<f64>,
    
    // Put option fields
    #[serde(rename = "PE_AbsoluteChange")]
    pub pe_absolute_change: Option<f64>,
    
    #[serde(rename = "PE_AskPrice")]
    pub pe_ask_price: Option<f64>,
    
    #[serde(rename = "PE_AskQty")]
    pub pe_ask_qty: Option<f64>,
    
    #[serde(rename = "PE_BidPrice")]
    pub pe_bid_price: Option<f64>,
    
    #[serde(rename = "PE_BidQty")]
    pub pe_bid_qty: Option<f64>,
    
    #[serde(rename = "PE_ChangeInOI")]
    pub pe_change_in_oi: Option<f64>,
    
    #[serde(rename = "PE_LTP")]
    pub pe_ltp: Option<f64>,
    
    #[serde(rename = "PE_LTT")]
    pub pe_ltt: Option<String>,
    
    #[serde(rename = "PE_NetChange")]
    pub pe_net_change: Option<f64>,
    
    #[serde(rename = "PE_OpenInterest")]
    pub pe_open_interest: Option<f64>,
    
    #[serde(rename = "PE_Volume")]
    pub pe_volume: Option<f64>,
    
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
pub struct McxSummary {
    #[serde(rename = "ExtensionData")]
    pub extension_data: Option<serde_json::Value>,
    
    #[serde(rename = "AsOn")]
    pub as_on: Option<String>,
    
    #[serde(rename = "Count")]
    pub count: Option<i32>,
    
    #[serde(rename = "Status")]
    pub status: Option<String>,
}

// Raw symbol data structure from the script tag
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McxSymbolData {
    #[serde(rename = "ExpiryDate")]
    pub expiry_date: String,
    
    #[serde(rename = "InstrumentName")]
    pub instrument_name: String,
    
    #[serde(rename = "Symbol")]
    pub symbol: String,
    
    #[serde(rename = "SymbolValue")]
    pub symbol_value: String,
    
    #[serde(rename = "TodaysTraded")]
    pub todays_traded: Option<i32>,
}