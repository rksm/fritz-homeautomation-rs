use reqwest::StatusCode;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FritzError {
    #[error("data store disconnected")]
    Ap(#[from] reqwest::Error),

    #[error("API request failed: `{0}")]
    ApiRequest(String),

    #[error("fritz login error: `{0}`")]
    LoginError(String),

    #[error("cannot parse xml: `{0}`")]
    XMLParseError(#[from] serde_xml_rs::Error),

    #[error("parser error: `{0}")]
    ParserError(String),

    #[error("status code mismatch while triggering high refresh rate. Expected 200, got `{0}`")]
    TriggerHighRefreshRateError(StatusCode),

    #[error("unknown fritz api error")]
    Unknown,
}

pub type Result<T> = std::result::Result<T, FritzError>;
