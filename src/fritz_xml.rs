#![allow(dead_code)]

use serde::{Deserialize, Deserializer};
use serde_xml_rs::from_reader;

// response of login_sid.lua

fn deserialize_maybe_u32<'de, D>(d: D) -> Result<u32, D::Error>
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
    pub batterylow: bool,
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
    #[serde(deserialize_with = "deserialize_maybe_u32")]
    pub voltage: u32,
    #[serde(deserialize_with = "deserialize_maybe_u32")]
    pub power: u32,
    #[serde(deserialize_with = "deserialize_maybe_u32")]
    pub energy: u32,
}

#[derive(Debug, Deserialize)]
pub struct Temperature {
    pub celsius: String,
    pub offset: String,
}

// -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-

pub fn parse_session_info(xml: &str) -> Result<SessionInfo, serde_xml_rs::Error> {
    from_reader(xml.as_bytes())
}

/// Parses raw [`Device`]s.
pub fn parse_device_infos(xml: String) -> Result<Vec<Device>, serde_xml_rs::Error> {
    from_reader::<&[u8], DeviceList>(xml.as_bytes()).map(|list| list.devices)
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

#[derive(Debug)]
pub enum DeviceStatsKind {
    Temperature,
    Voltage,
    Power,
    Energy,
}

#[derive(Debug)]
pub struct DeviceStats {
    pub kind: DeviceStatsKind,
    pub stats: Vec<DeviceStatValues>,
}

#[derive(Debug)]
pub struct DeviceStatValues {
    pub values: Vec<f32>,
    pub grid: usize,
}

pub fn parse_device_stats(xml: String) -> Result<Vec<DeviceStats>, serde_xml_rs::Error> {
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
                stats: raw
                    .stats
                    .into_iter()
                    .map(|ea| DeviceStatValues {
                        grid: ea.grid,
                        values: ea
                            .values
                            .split(',')
                            .map(|val| val.parse::<f32>().unwrap() * multiplier)
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
    process_raw(stats.power, DeviceStatsKind::Power, 1.0, &mut result);
    process_raw(stats.voltage, DeviceStatsKind::Voltage, 0.001, &mut result);

    Ok(result)
}
