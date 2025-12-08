use std::fmt;

#[derive(Debug)]
pub enum NSEError {
    Request(String),
    NonJsonResponse(String),
    Parse(String),
}

impl fmt::Display for NSEError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NSEError::Request(msg) => write!(f, "Request error: {}", msg),
            NSEError::NonJsonResponse(preview) => write!(f, "Non-JSON response: {}", preview),
            NSEError::Parse(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl std::error::Error for NSEError {}

impl From<reqwest::Error> for NSEError {
    fn from(err: reqwest::Error) -> Self {
        NSEError::Request(err.to_string())
    }
}

impl From<serde_json::Error> for NSEError {
    fn from(err: serde_json::Error) -> Self {
        NSEError::Parse(err.to_string())
    }
}