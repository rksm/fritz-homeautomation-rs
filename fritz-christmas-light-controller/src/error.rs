use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("unable to open config file: {0}")]
    ConfigFileError(#[from] std::io::Error),
    #[error("unable to parse config file: {0}")]
    ConfigReadError(#[from] serde_yaml::Error),

    #[error("unable to parse duration: {0}")]
    DurationParseError(String),
}

pub type Result<T> = std::result::Result<T, Error>;
