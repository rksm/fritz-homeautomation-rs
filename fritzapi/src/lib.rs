//! Library for interfacing with the \"AVM Home Automation\" API
//! <https://avm.de/fileadmin/user_upload/Global/Service/Schnittstellen/AHA-HTTP-Interface.pdf>.
//!
//! It is used by the [fritzctrl](https://crates.io/crates/fritzctrl) utility.
//!
//! ## Example
//!
//! ### List devices
//!
//! ```ignore
//! // Get a session id
//! let sid = fritzapi::get_sid(&user, &password)?;
//!
//! // List devices
//! let mut devices = fritzapi::list_devices(&sid)?;
//!
//! // If the first device is of, turn it on
//! let dev = devices.first_mut().unwrap();
//! if !dev.is_on() {
//!     dev.turn_on(&sid)?;
//! }
//! ```

#[macro_use]
extern crate tracing;

pub mod devices;
pub mod stats;
pub use devices::{AVMDevice, FritzDect2XX};
pub use stats::{DeviceStats, DeviceStatsKind, Unit};

#[cfg(not(target_family = "wasm"))]
pub(crate) mod api;
#[cfg(not(target_family = "wasm"))]
pub(crate) mod client;
#[cfg(not(target_family = "wasm"))]
pub mod error;
#[cfg(not(target_family = "wasm"))]
pub(crate) mod fritz_xml;

#[cfg(not(target_family = "wasm"))]
pub use client::FritzClient;
#[cfg(not(target_family = "wasm"))]
pub use error::{FritzError, Result};
