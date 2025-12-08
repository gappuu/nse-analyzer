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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionChain {
    pub records: Records,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Records {
    pub timestamp: String,
    
    #[serde(rename = "underlyingValue")]
    pub underlying_value: f64,
    
    pub data: Vec<OptionData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionData {
    #[serde(rename = "strikePrice")]
    pub strike_price: f64,
    
    #[serde(rename = "CE")]
    pub call: Option<OptionDetail>,
    
    #[serde(rename = "PE")]
    pub put: Option<OptionDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionDetail {
    #[serde(rename = "openInterest")]
    pub open_interest: f64,
    
    #[serde(rename = "changeinOpenInterest")]
    pub change_in_oi: f64,
    
    #[serde(rename = "lastPrice")]
    pub last_price: f64,
    
    #[serde(rename = "impliedVolatility")]
    pub iv: f64,
}