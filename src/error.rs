use serde_xml_rs::Error as XMLError;
use std::error;

#[derive(Debug)]
pub enum MyError {
    LoginError(),
    Other(Box<dyn error::Error>),
}

impl From<reqwest::Error> for MyError {
    fn from(err: reqwest::Error) -> Self {
        MyError::Other(Box::new(err))
    }
}

impl From<XMLError> for MyError {
    fn from(err: XMLError) -> Self {
        MyError::Other(Box::new(err))
    }
}

impl std::fmt::Display for MyError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            MyError::Other(err) => write!(f, "io error: {}", err),
            _ => write!(f, "{:?}", self),
        }
    }
}

pub type Result<T> = std::result::Result<T, MyError>;

#[macro_export]
macro_rules! my_error {
    ( $err:ident ) => {
        $crate::error::MyError::Other(Box::new($err))
    };
}
