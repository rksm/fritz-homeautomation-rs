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

pub(crate) mod api;
pub(crate) mod client;
pub(crate) mod devices;
pub mod error;
pub(crate) mod fritz_xml;

pub use client::FritzClient;
pub use devices::{AVMDevice, FritzDect2XX};
pub use error::{FritzError, Result};
pub use fritz_xml::{DeviceStats, DeviceStatsKind, Unit};
