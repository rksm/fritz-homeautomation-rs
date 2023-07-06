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
//! # fn main() -> fritzapi::Result<()> {
//! # let user = "";
//! # let password = "";
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
//! # Ok(())
//! # }
//! ```

mod api;
mod devices;
pub mod error;
mod fritz_xml;

pub use api::{get_sid, trigger_high_refresh_rate};
pub use devices::{AVMDevice, FritzDect2XX};
pub use error::{FritzError, Result};
pub use fritz_xml::{DeviceStats, DeviceStatsKind};

pub fn list_devices(sid: &str) -> error::Result<Vec<devices::AVMDevice>> {
    devices::AVMDevice::list(sid)
}
