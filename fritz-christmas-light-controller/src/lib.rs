#[macro_use]
extern crate tracing;

mod config;
pub mod duration;
mod error;
mod timer;

pub use config::*;
pub use error::{Error, Result};
pub use timer::Timer;
