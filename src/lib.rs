mod fritz_xml;
mod api;
pub mod devices;
pub mod daylight;
pub mod schedule;

pub use api::get_sid;

pub fn list_devices(sid: &str) -> anyhow::Result<Vec<devices::AVMDevice>> {
    devices::AVMDevice::list(sid)
}
