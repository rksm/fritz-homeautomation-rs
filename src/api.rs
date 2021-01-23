use anyhow::{anyhow, Context};
use lazy_static::lazy_static;
use regex::Regex;
use reqwest::blocking::{get as GET, Client, Response};

use crate::fritz_xml;
use crate::fritz_xml::*;

// -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-

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

pub fn get_sid(user: &str, password: &str) -> anyhow::Result<String> {
    let res: Response = GET("http://fritz.box/login_sid.lua")?
        .error_for_status()
        .with_context(|| format!("GET login_sid.lua for user {}", user))?;
    let xml = res.text()?;
    let info = fritz_xml::parse_session_info(&xml)?;
    if DEFAULT_SID != info.sid {
        return Ok(info.sid);
    }
    let response = request_response(password, &info.challenge);
    let url = format!(
        "http://fritz.box/login_sid.lua?username={}&response={}",
        user, response
    );
    let login: Response = GET(&url)?.error_for_status()?;
    let info = fritz_xml::parse_session_info(&login.text()?)?;

    if DEFAULT_SID == info.sid {
        return Err(anyhow!(
            "login error - sid is still the default after login attempt"
        ));
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

fn request(cmd: Commands, sid: &str, ain: Option<&str>) -> anyhow::Result<String> {
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
    let status = response.status();
    println!(
        "[fritz api] {} status: {:?} {:?}",
        cmd,
        status,
        status.canonical_reason().unwrap_or_default()
    );

    Ok(response.text()?)
}

// -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-

/// Parses raw [`Device`]s.
pub fn device_infos(sid: &str) -> anyhow::Result<Vec<Device>> {
    let xml = request(Commands::GetDeviceListInfos, &sid, None)?;
    match parse_device_infos(xml) {
        Ok(infos) => Ok(infos),
        Err(err) => Err(anyhow!("[parse_device_infos] error: {}", err)),
    }
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

pub fn device_infos_avm(sid: &str) -> anyhow::Result<Vec<AVMDevice>> {
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
                celsius: celsius.parse::<f32>().unwrap_or_default() * 0.1,
                // raw: dev,
            }),

            _ => AVMDevice::Other(dev),
        })
        .collect();
    Ok(result)
}

pub fn fetch_device_stats(sid: &str, ain: &str) -> anyhow::Result<Vec<DeviceStats>> {
    let xml = request(Commands::GetBasicDeviceStats, sid, Some(ain))?;
    match parse_device_stats(xml) {
        Ok(stats) => Ok(stats),
        Err(err) => Err(anyhow!("[parse_device_stats] error: {}", err)),
    }
}

// -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-

pub fn turn_on(sid: &str, ain: &str) -> anyhow::Result<()> {
    request(Commands::SetSwitchOn, sid, Some(ain))?;
    Ok(())
}

pub fn turn_off(sid: &str, ain: &str) -> anyhow::Result<()> {
    request(Commands::SetSwitchOff, sid, Some(ain))?;
    Ok(())
}

pub fn toggle(sid: &str, ain: &str) -> anyhow::Result<()> {
    request(Commands::SetSwitchToggle, sid, Some(ain))?;
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
