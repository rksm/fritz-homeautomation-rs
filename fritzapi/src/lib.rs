//! Library for interfacing with the \"AVM Home Automation\" API
//! <https://avm.de/fileadmin/user_upload/Global/Service/Schnittstellen/AHA-HTTP-Interface.pdf>.

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
