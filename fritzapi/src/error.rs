use thiserror::Error;

#[derive(Error, Debug)]
pub enum FritzError {
    #[error("data store disconnected")]
    ApiRequest(#[from] reqwest::Error),

    #[error("fritz login error: `{0}`")]
    LoginError(String),

    #[error("cannot parse xml: `{0}`")]
    XMLParseError(#[from] serde_xml_rs::Error),

    #[error("parser error: `{0}")]
    ParserError(String),

    #[error("unknown fritz api error")]
    Unknown,
}

pub type Result<T> = std::result::Result<T, FritzError>;

// #[derive(Error, Debug)]
// pub enum FritzError {
//     #[error("data store disconnected")]
//     Disconnect(#[from] io::Error),
//     #[error("the data for key `{0}` is not available")]
//     Redaction(String),
//     #[error("invalid header (expected {expected:?}, found {found:?})")]
//     InvalidHeader {
//         expected: String,
//         found: String,
//     },
//     #[error("unknown data store error")]
//     Unknown,
// }
