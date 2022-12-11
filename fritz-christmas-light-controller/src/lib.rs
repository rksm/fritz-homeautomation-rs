#[macro_use]
extern crate tracing;

mod config;
pub mod duration;
mod error;
mod fritz_updater;
mod timer;

pub use config::*;
pub use error::{Error, Result};
pub use fritz_updater::{FritzUpdate, RealtFritzUpdater};
pub use timer::Timer;
