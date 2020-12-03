use lazy_static::lazy_static;
use regex::Regex;
use reqwest::blocking::{get as GET, Client, Response};
use serde::Deserialize;
use serde_xml_rs::from_reader;

use crate::error::Result;

// response of login_sid.lua

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
    pub battery: Option<bool>,
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
    pub voltage: u32,
    pub power: u32,
    pub energy: u32,
}

#[derive(Debug, Deserialize)]
pub struct Temperature {
    pub celsius: String,
    pub offset: String,
}

#[derive(Debug)]
pub struct FritzDect2XX {
    pub identifier: String,
    pub on: bool,
    pub voltage: f32,
    pub watts: f32,
    pub energy_in_watt_h: u32,
    pub celsius: f32,
    // raw: Device,
}

#[derive(Debug)]
pub enum AVMDevice {
    FritzDect2XX(FritzDect2XX),
    Other(Device),
}

// stats

#[derive(Debug, Deserialize)]
struct RawDeviceStats {
    temperature: Option<RawManyStats>,
    voltage: Option<RawManyStats>,
    power: Option<RawManyStats>,
    energy: Option<RawManyStats>,
}

#[derive(Debug, Deserialize)]
struct RawManyStats {
    stats: Vec<RawStats>,
}

#[derive(Debug, Deserialize)]
struct RawStats {
    count: usize,
    grid: usize,
    #[serde(rename = "$value")]
    values: String,
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

// -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-

fn parse_session_info(xml: &str) -> Result<SessionInfo> {
    match from_reader(xml.as_bytes()) {
        Ok(info) => Ok(info),
        Err(err) => Err(err.into()),
    }
}

/// Computes the string that we use to authenticate.
/// 1. Replace all non-ascii chars in `password` with "."
/// 2. Concat `challenge` and the modified password
/// 3. Convert that to UTF16le
/// 4. MD5 that byte array
/// 5. concat that as hex with challenge again
fn request_response(password: &str, challenge: &str) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"[^\x00-\x7F]").unwrap();
    }
    let clean_password = RE.replace_all(password, ".");
    let hash_input = format!("{}-{}", challenge, clean_password);
    let bytes: Vec<u8> = hash_input
        .encode_utf16()
        .flat_map(|utf16| utf16.to_le_bytes().to_vec())
        .collect();
    let digest = md5::compute(bytes);
    format!("{}-{:032x}", challenge, digest)
}

const DEFAULT_SID: &str = "0000000000000000";

pub fn get_sid(user: &str, password: &str) -> Result<String> {
    let res: Response = GET("http://fritz.box/login_sid.lua")?.error_for_status()?;
    let xml = res.text()?;
    let info = parse_session_info(&xml)?;
    if DEFAULT_SID != info.sid {
        return Ok(info.sid);
    }
    let response = request_response(password, &info.challenge);
    let url = format!(
        "http://fritz.box/login_sid.lua?username={}&response={}",
        user, response
    );
    let login: Response = GET(&url)?.error_for_status()?;
    let info = parse_session_info(&login.text()?)?;

    if DEFAULT_SID == info.sid {
        return Err(crate::error::MyError::LoginError());
    }

    Ok(info.sid)
}

enum Commands {
    GetDeviceListInfos,
    GetBasicDeviceStats,
    // GetSwitchPower,
    // GetSwitchEnergy,
    // GetSwitchName,
    // GetTemplateListInfos,
    SetSwitchOff,
    SetSwitchOn,
    SetSwitchToggle,
}

fn request(cmd: Commands, sid: &str, ain: Option<&str>) -> Result<String> {
    use Commands::*;
    let cmd = match cmd {
        GetDeviceListInfos => "getdevicelistinfos",
        GetBasicDeviceStats => "getbasicdevicestats",
        // GetSwitchPower => "getswitchpower",
        // GetSwitchEnergy => "getswitchenergy",
        // GetSwitchName => "getswitchname",
        // GetTemplateListInfos => "gettemplatelistinfos",
        SetSwitchOff => "setswitchoff",
        SetSwitchOn => "setswitchon",
        SetSwitchToggle => "setswitchtoggle",
    };
    let url = "http://fritz.box/webservices/homeautoswitch.lua";
    let mut client = Client::new()
        .get(url)
        .query(&[("switchcmd", cmd), ("sid", sid)]);
    if let Some(ain) = ain {
        client = client.query(&[("ain", ain)]);
    }
    let response = client.send()?;
    Ok(response.text()?)
}

// -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-

/// Parses raw [`Device`]s.
pub fn device_infos(sid: &str) -> Result<Vec<Device>> {
    let xml = request(Commands::GetDeviceListInfos, &sid, None)?;
    match from_reader::<&[u8], DeviceList>(xml.as_bytes()) {
        Ok(device_list) => Ok(device_list.devices),
        Err(err) => Err(err.into()),
    }
}

pub fn device_infos_avm(sid: &str) -> Result<Vec<AVMDevice>> {
    let devices = device_infos(sid)?;
    let result: Vec<AVMDevice> = devices
        .into_iter()
        .map(|dev| match &dev {
            Device {
                productname,
                identifier,
                switch: Some(Switch { state, .. }),
                powermeter:
                    Some(PowerMeter {
                        energy,
                        power,
                        voltage,
                        ..
                    }),
                temperature: Some(Temperature { celsius, .. }),
                ..
            } if productname.starts_with("FRITZ!DECT 2") => AVMDevice::FritzDect2XX(FritzDect2XX {
                identifier: identifier.clone(),
                on: *state,
                voltage: *voltage as f32 * 0.001,
                watts: *power as f32 * 0.001,
                energy_in_watt_h: *energy,
                celsius: celsius.parse::<f32>().expect("parsing temperature") * 0.1,
                // raw: dev,
            }),

            _ => AVMDevice::Other(dev),
        })
        .collect();
    Ok(result)
}

// -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-

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

pub fn fetch_device_stats(sid: &str, ain: &str) -> Result<Vec<DeviceStats>> {
    let xml = request(Commands::GetBasicDeviceStats, sid, Some(ain))?;
    let stats: RawDeviceStats = match from_reader(xml.as_bytes()) {
        Err(err) => return Err(err.into()),
        Ok(stats) => stats,
    };

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

// -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-

pub fn turn_on(sid: &str, ain: &str) -> Result<()> {
    request(Commands::SetSwitchOn, sid, Some (ain))?;
    Ok(())
}

pub fn turn_off(sid: &str, ain: &str) -> Result<()> {
    request(Commands::SetSwitchOff, sid, Some (ain))?;
    Ok(())
}

pub fn toggle(sid: &str, ain: &str) -> Result<()> {
    request(Commands::SetSwitchToggle, sid, Some (ain))?;
    Ok(())
}

// -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-

#[cfg(test)]
mod tests {

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
    fn request_response() {
        let response = super::request_response("mühe", "foo");
        assert_eq!(response, "foo-442e12bbceabd35c66964c913a316451");
    }
}
