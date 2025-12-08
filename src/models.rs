use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Security {
    pub symbol: String,
    pub security_type: SecurityType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SecurityType {
    Equity,
    Indices,
}

impl Security {
    pub fn equity(symbol: String) -> Self {
        Self { symbol, security_type: SecurityType::Equity }
    }
    
    pub fn index(symbol: String) -> Self {
        Self { symbol, security_type: SecurityType::Indices }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractInfo {
    #[serde(rename = "expiryDates")]
    pub expiry_dates: Vec<String>,
    
    #[serde(rename = "strikePrice")]
    pub strike_prices: Vec<String>,
}

/// Main response structure from NSE option chain API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionChain {
    pub records: Records,
    pub filtered: FilteredData,
}

/// Records section containing timestamp, underlying value, and all strike data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Records {
    pub timestamp: String,
    
    #[serde(rename = "underlyingValue")]
    pub underlying_value: f64,
    
    pub data: Vec<OptionData>,
    
    #[serde(rename = "expiryDates")]
    pub expiry_dates: Vec<String>,
    
    #[serde(rename = "strikePrices")]
    pub strike_prices: Vec<String>,
}

/// Filtered section containing current expiry data and totals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilteredData {
    pub data: Vec<OptionData>,
    
    #[serde(rename = "CE")]
    pub ce_totals: OptionTotals,
    
    #[serde(rename = "PE")]
    pub pe_totals: OptionTotals,
}

/// Totals for CE or PE side
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionTotals {
    #[serde(rename = "totOI")]
    pub total_oi: f64,
}

/// Option data for each strike price
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionData {
    #[serde(rename = "expiryDates")]
    pub expiry_date: Option<String>,  
    
    #[serde(rename = "strikePrice")]
    pub strike_price: Option<f64>,
    
    #[serde(rename = "CE")]
    pub call: Option<OptionDetail>,
    
    #[serde(rename = "PE")]
    pub put: Option<OptionDetail>,
}

/// Detailed option information (CE or PE)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionDetail {
    // #[serde(default)]
    // pub identifier: Option<String>,  
    
    #[serde(rename = "strikePrice")]
    pub strike_price: Option<f64>,
    
    #[serde(rename = "underlyingValue")]
    pub underlying_value: Option<f64>,
    
    #[serde(rename = "openInterest")]
    pub open_interest: Option<f64>,
    
    #[serde(rename = "changeinOpenInterest")]
    pub change_in_oi: Option<f64>,
    
    #[serde(rename = "lastPrice")]
    pub last_price: Option<f64>,
    
     #[serde(rename = "change")]
    pub price_change: Option<f64>,

     #[serde(rename = "pchange")]
    pub per_chg_price: Option<f64>,

     #[serde(rename = "pchangeinOpenInterest")]
    pub per_chg_oi: Option<f64>,
}