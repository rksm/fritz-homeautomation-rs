#[derive(thiserror::Error, Debug)]
pub enum FritzError {
    #[cfg(not(target_family = "wasm"))]
    #[error("data store disconnected")]
    Ap(#[from] reqwest::Error),

    #[error("Request forbidden. Are you logged in, is the sid correct and recent?")]
    Forbidden,

    #[error("API request failed: `{0}")]
    ApiRequest(String),

    #[error("fritz login error: `{0}`")]
    LoginError(String),

    #[cfg(not(target_family = "wasm"))]
    #[error("cannot parse xml: `{0}`")]
    XMLParseError(#[from] serde_xml_rs::Error),

    #[error("parser error: `{0}")]
    ParserError(String),

    #[error("status code mismatch while triggering high refresh rate. Expected 200, got `{0}`")]
    TriggerHighRefreshRateError(reqwest::StatusCode),

    #[error("unknown fritz api error")]
    Unknown,
}

pub type Result<T> = std::result::Result<T, FritzError>;
