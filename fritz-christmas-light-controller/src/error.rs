use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("unable to open config file: {0}")]
    ConfigFileError(#[from] std::io::Error),
    #[error("unable to parse config file: {0}")]
    ConfigReadError(#[from] serde_yaml::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
