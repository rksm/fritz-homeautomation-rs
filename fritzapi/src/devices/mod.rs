#[cfg(not(target_family = "wasm"))]
mod device_impl;
pub mod fritz_dect_2xx;

pub use fritz_dect_2xx::FritzDect2XX;
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AVMDevice {
    FritzDect2XX(FritzDect2XX),
    Other(Device),
}

impl std::fmt::Display for AVMDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AVMDevice::FritzDect2XX(dev @ FritzDect2XX { .. }) => {
                writeln!(
                    f,
                    "identifier={:?} productname={:?} name={:?}",
                    dev.identifier, dev.productname, dev.name
                )?;
            }
            AVMDevice::Other(dev) => {
                writeln!(
                    f,
                    "Unsupported device identifier={:?} productname={:?} name={:?}",
                    dev.identifier, dev.productname, dev.name
                )?;
            }
        };
        Ok(())
    }
}

impl AVMDevice {
    pub fn id(&self) -> &str {
        match self {
            AVMDevice::FritzDect2XX(dev @ FritzDect2XX { .. }) => &dev.identifier,
            AVMDevice::Other(dev) => &dev.identifier,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            AVMDevice::FritzDect2XX(dev @ FritzDect2XX { .. }) => &dev.name,
            AVMDevice::Other(dev) => &dev.name,
        }
    }

    pub fn productname(&self) -> &str {
        match self {
            AVMDevice::FritzDect2XX(dev @ FritzDect2XX { .. }) => &dev.productname,
            AVMDevice::Other(dev) => &dev.productname,
        }
    }

    pub fn is_on(&self) -> bool {
        match self {
            AVMDevice::FritzDect2XX(FritzDect2XX { on, .. }) => *on,
            // TODO
            AVMDevice::Other(_) => false,
        }
    }

    pub fn state(&self) -> &str {
        match self {
            AVMDevice::FritzDect2XX(FritzDect2XX { on: true, .. }) => "on",
            AVMDevice::FritzDect2XX(FritzDect2XX { on: false, .. }) => "off",
            AVMDevice::Other(_) => "",
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DeviceOrGroup {
    Device(Device),
    Group(DeviceGroup),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Device {
    pub identifier: String,
    pub id: String,
    pub functionbitmask: String,
    pub fwversion: String,
    pub manufacturer: String,
    pub productname: String,
    pub present: bool,
    pub txbusy: bool,
    pub name: String,
    pub battery: Option<i32>,
    pub batterylow: Option<bool>,
    pub switch: Option<Switch>,
    pub simpleonoff: Option<SimpleOnOff>,
    pub powermeter: Option<PowerMeter>,
    pub temperature: Option<Temperature>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceGroup {
    pub synchronized: bool,
    pub identifier: String,
    pub id: String,
    pub functionbitmask: String,
    pub fwversion: String,
    pub manufacturer: String,
    pub present: bool,
    pub txbusy: bool,
    pub name: String,
    pub switch: Option<Switch>,
    pub simpleonoff: Option<SimpleOnOff>,
    pub powermeter: Option<PowerMeter>,
    // groupinfo: ... // TODO
}

#[derive(Debug, Deserialize)]
pub struct DeviceList {
    #[serde(rename = "$value")]
    pub list: Vec<DeviceOrGroup>,
    // pub devices: Vec<Device>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Switch {
    pub state: bool,
    pub lock: bool,
    pub devicelock: bool,
    pub mode: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SimpleOnOff {
    pub state: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PowerMeter {
    /// Wert in 0,001 V (aktuelle Spannung, wird etwa alle 2 Minuten aktualisiert)
    #[serde(deserialize_with = "deserialize_maybe_u32")]
    pub voltage: u32,
    /// Wert in 0,001 W (aktuelle Leistung, wird etwa alle 2 Minuten aktualisiert)
    #[serde(deserialize_with = "deserialize_maybe_u32")]
    pub power: u32,
    /// Wert in 1.0 Wh (absoluter Verbrauch seit Inbetriebnahme)
    #[serde(deserialize_with = "deserialize_maybe_u32")]
    pub energy: u32,
}

/// celsius: Wert in 0,1 °C, negative und positive Werte möglich
/// offset: Wert in 0,1 °C, negative und positive Werte möglich
#[derive(Debug, Serialize, Deserialize)]
pub struct Temperature {
    pub celsius: String,
    pub offset: String,
}

fn deserialize_maybe_u32<'de, D>(d: D) -> std::result::Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(d)?;
    match &s[..] {
        "" => Ok(0),
        _ => Ok(s.parse::<u32>().unwrap()),
    }
}
