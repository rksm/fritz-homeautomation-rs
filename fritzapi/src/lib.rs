//! Library for interfacing with the \"AVM Home Automation\" API
//! <https://avm.de/fileadmin/user_upload/Global/Service/Schnittstellen/AHA-HTTP-Interface.pdf>.
//!
//! It is used by the [fritzctrl](https://crates.io/crates/fritzctrl) utility.
//!
//! ## Example
//!
//! ```no_run
//! # fn main() -> fritzapi::Result<()> {
//! #     let user = "";
//! #     let password = "";
//!     let mut client = fritzapi::FritzClient::new(user, password);
//!     // List devices
//!     let mut devices = client.list_devices()?;
//!     // If the first device is off, turn it on
//!     let dev = devices.first_mut().unwrap();
//!     if !dev.is_on() {
//!         dev.turn_on(&mut client)?;
//!     }
//! #     Ok(())
//! # }
//! ```

#[macro_use]
extern crate tracing;

pub mod devices;
pub mod error;
pub mod stats;

#[cfg(not(target_family = "wasm"))]
pub(crate) mod api;
#[cfg(not(target_family = "wasm"))]
pub(crate) mod client;
#[cfg(not(target_family = "wasm"))]
pub(crate) mod fritz_xml;

pub use devices::{AVMDevice, FritzDect2XX};
pub use error::{FritzError, Result};
pub use stats::{DeviceStats, DeviceStatsKind, Unit};

#[cfg(not(target_family = "wasm"))]
pub use client::FritzClient;
