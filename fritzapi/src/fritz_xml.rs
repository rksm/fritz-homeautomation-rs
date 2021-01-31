#![allow(dead_code)]

use crate::error::{FritzError, Result};
use serde::{Deserialize, Deserializer};
use serde_xml_rs::from_reader;

// response of login_sid.lua

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

#[derive(Debug, Deserialize)]
pub struct SessionInfo {
    #[serde(alias = "SID")]
    pub sid: String,
    #[serde(alias = "Challenge")]
    pub challenge: String,
    #[serde(alias = "BlockTime")]
    pub block_time: i32,
}

// response of getdevicelistinfos

#[derive(Debug, Deserialize)]
pub struct DeviceList {
    #[serde(alias = "device")]
    pub devices: Vec<Device>,
}

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
pub struct Switch {
    pub state: bool,
    pub lock: bool,
    pub devicelock: bool,
    pub mode: String,
}

#[derive(Debug, Deserialize)]
pub struct SimpleOnOff {
    pub state: bool,
}

#[derive(Debug, Deserialize)]
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
#[derive(Debug, Deserialize)]
pub struct Temperature {
    pub celsius: String,
    pub offset: String,
}

// -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-

pub fn parse_session_info(xml: &str) -> Result<SessionInfo> {
    from_reader(xml.as_bytes()).map_err(|err| {
        eprintln!("cannot parse session info");
        err.into()
    })
}

/// Parses raw [`Device`]s.
pub fn parse_device_infos(xml: String) -> Result<Vec<Device>> {
    from_reader::<&[u8], DeviceList>(xml.as_bytes())
        .map(|list| list.devices)
        .map_err(|err| {
            eprintln!("cannot parse device infos");
            err.into()
        })
}

// -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-
// features

#[derive(Default, Debug)]
pub struct DeviceFeatures {
    hanfun_unit: bool,
    microfon: bool,
    dect_repeater: bool,
    outlet: bool,
    temperature_sensor: bool,
    energy_sensor: bool,
    heater: bool,
    alarm: bool,
    hanfun_device: bool,
}

const HANFUN_UNIT: u32 = 0b1000000000000;
const MICROFON: u32 = 0b0100000000000;
const DECT_REPEATER: u32 = 0b0010000000000;
const OUTLET: u32 = 0b0001000000000;
const TEMPERATURE_SENSOR: u32 = 0b0000100000000;
const ENERGY_SENSOR: u32 = 0b0000010000000;
const HEATER: u32 = 0b0000001000000;
const ALARM: u32 = 0b0000000010000;
const HANFUN_DEVICE: u32 = 0b0000000000001;

/// Given a raw device, will determine its feature set according to
/// [`DeviceFeatures`].
pub fn features(device: &Device) -> DeviceFeatures {
    match device.functionbitmask.parse::<u32>() {
        Err(_) => Default::default(),
        Ok(num) => DeviceFeatures {
            hanfun_unit: num & HANFUN_UNIT > 0,
            microfon: num & MICROFON > 0,
            dect_repeater: num & DECT_REPEATER > 0,
            outlet: num & OUTLET > 0,
            temperature_sensor: num & TEMPERATURE_SENSOR > 0,
            energy_sensor: num & ENERGY_SENSOR > 0,
            heater: num & HEATER > 0,
            alarm: num & ALARM > 0,
            hanfun_device: num & HANFUN_DEVICE > 0,
        },
    }
}

// -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-
// stats

#[derive(Debug, Deserialize)]
pub struct RawDeviceStats {
    pub temperature: Option<RawManyStats>,
    pub voltage: Option<RawManyStats>,
    pub power: Option<RawManyStats>,
    pub energy: Option<RawManyStats>,
}

#[derive(Debug, Deserialize)]
pub struct RawManyStats {
    pub stats: Vec<RawStats>,
}

#[derive(Debug, Deserialize)]
pub struct RawStats {
    pub count: usize,
    pub grid: usize,
    #[serde(rename = "$value")]
    pub values: String,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Unit {
    Celsius,
    Watt,
    WattHour,
    Volt,
}

impl std::fmt::Display for Unit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Unit::Celsius => write!(f, "°C"),
            Unit::Watt => write!(f, "W"),
            Unit::WattHour => write!(f, "Wh"),
            Unit::Volt => write!(f, "V"),
        }
    }
}

/// Category of measurements that the fritz devices may provide.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum DeviceStatsKind {
    Temperature,
    Voltage,
    Power,
    Energy,
}

impl DeviceStatsKind {
    pub fn unit(&self) -> Unit {
        match self {
            DeviceStatsKind::Temperature => Unit::Celsius,
            DeviceStatsKind::Voltage => Unit::Volt,
            DeviceStatsKind::Power => Unit::Watt,
            DeviceStatsKind::Energy => Unit::WattHour,
        }
    }
}

impl std::str::FromStr for DeviceStatsKind {
    type Err = FritzError;

    fn from_str(input: &str) -> Result<Self> {
        match input.to_lowercase().as_str() {
            "temp" | "temperature" | "celsius" | "c" => Ok(DeviceStatsKind::Temperature),
            "power" | "watt" | "w" => Ok(DeviceStatsKind::Power),
            "energy" | "wh" => Ok(DeviceStatsKind::Energy),
            "volt" | "v" | "voltage" => Ok(DeviceStatsKind::Voltage),
            _ => Err(FritzError::ParserError(format!(
                "Cannot convert {:?} to DeviceStatsKind",
                input
            ))),
        }
    }
}

#[derive(Debug)]
pub struct DeviceStats {
    pub kind: DeviceStatsKind,
    pub values: Vec<DeviceStatValues>,
}

#[derive(Debug)]
pub struct DeviceStatValues {
    pub values: Vec<f32>,
    pub grid: usize,
}

pub fn parse_device_stats(xml: String) -> Result<Vec<DeviceStats>, > {
    let stats: RawDeviceStats = from_reader(xml.as_bytes())?;

    let mut result: Vec<DeviceStats> = Vec::new();

    fn process_raw(
        raw: Option<RawManyStats>,
        kind: DeviceStatsKind,
        multiplier: f32,
        result: &mut Vec<DeviceStats>,
    ) {
        if let Some(raw) = raw {
            result.push(DeviceStats {
                kind,
                values: raw
                    .stats
                    .into_iter()
                    .map(|ea| DeviceStatValues {
                        grid: ea.grid,
                        values: ea
                            .values
                            .split(',')
                            .filter_map(|val| val.parse::<f32>().ok().map(|val| (val * multiplier).round()))
                            .collect(),
                    })
                    .collect(),
            })
        }
    }

    process_raw(
        stats.temperature,
        DeviceStatsKind::Temperature,
        0.1,
        &mut result,
    );
    process_raw(stats.energy, DeviceStatsKind::Energy, 1.0, &mut result);
    process_raw(stats.power, DeviceStatsKind::Power, 0.001, &mut result);
    process_raw(stats.voltage, DeviceStatsKind::Voltage, 0.001, &mut result);

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_session_info() {
        let xml = r##"
<?xml version="1.0" encoding="utf-8"?>
<SessionInfo>
  <SID>0000000000000000</SID>
  <Challenge>63233c3d</Challenge>
  <BlockTime>0</BlockTime>
  <Rights></Rights>
</SessionInfo>
"##;

        let info = super::parse_session_info(xml).unwrap();
        assert_eq!(info.block_time, 0);
        assert_eq!(info.challenge, "63233c3d");
        assert_eq!(info.sid, "0000000000000000");
    }

    #[test]
    fn parse_device_stat_kind() {
        assert_eq!(
            "temperature".parse::<DeviceStatsKind>().unwrap(),
            DeviceStatsKind::Temperature
        );
        assert_eq!(
            "celsius".parse::<DeviceStatsKind>().unwrap(),
            DeviceStatsKind::Temperature
        );
        assert_eq!(
            "c".parse::<DeviceStatsKind>().unwrap(),
            DeviceStatsKind::Temperature
        );
        assert_eq!(
            "Temperature".parse::<DeviceStatsKind>().unwrap(),
            DeviceStatsKind::Temperature
        );
        assert_eq!(
            "temp".parse::<DeviceStatsKind>().unwrap(),
            DeviceStatsKind::Temperature
        );
        assert_eq!(
            "power".parse::<DeviceStatsKind>().unwrap(),
            DeviceStatsKind::Power
        );
        assert_eq!(
            "energy".parse::<DeviceStatsKind>().unwrap(),
            DeviceStatsKind::Energy
        );
        assert_eq!(
            "v".parse::<DeviceStatsKind>().unwrap(),
            DeviceStatsKind::Voltage
        );
    }
}
