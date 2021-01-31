//! Library for interfacing with the \"AVM Home Automation\" API
//! <https://avm.de/fileadmin/user_upload/Global/Service/Schnittstellen/AHA-HTTP-Interface.pdf>.
//!
//! It is used by the [fritzctrl](https://crates.io/crates/fritzctrl) utility.
//!
//! ## Example
//!
//! ### List devices
//!
//! ```no_run
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

pub mod error;
mod fritz_xml;
mod api;
mod devices;

pub use error::{Result, FritzError};
pub use api::get_sid;
pub use fritz_xml::{DeviceStatsKind,DeviceStats};
pub use devices::{AVMDevice,FritzDect2XX};

pub fn list_devices(sid: &str) -> error::Result<Vec<devices::AVMDevice>> {
    devices::AVMDevice::list(sid)
}
