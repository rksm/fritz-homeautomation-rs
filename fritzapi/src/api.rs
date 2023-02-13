use std::collections::HashMap;

use lazy_static::lazy_static;
use log::info;
use regex::Regex;
use reqwest::blocking::{get as GET, Client, Response};
use reqwest::redirect::Policy;

use crate::error::{FritzError, Result};
use crate::fritz_xml as xml;

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

/// Requests a temporary token (session id = sid) from the fritz box using user
/// name and password.
pub fn get_sid(user: &str, password: &str) -> Result<String> {
    let res: Response = GET("http://fritz.box/login_sid.lua")?
        .error_for_status()
        .map_err(|err| {
            eprintln!("GET login_sid.lua for user {}", user);
            err
        })?;

    let xml = res.text()?;
    let info = xml::parse_session_info(&xml)?;
    if DEFAULT_SID != info.sid {
        return Ok(info.sid);
    }
    let response = request_response(password, &info.challenge);
    let url = format!(
        "http://fritz.box/login_sid.lua?username={}&response={}",
        user, response
    );
    let login: Response = GET(&url)?.error_for_status()?;
    let info = xml::parse_session_info(&login.text()?)?;

    if DEFAULT_SID == info.sid {
        return Err(FritzError::LoginError(
            "login error - sid is still the default after login attempt".to_string(),
        ));
    }

    Ok(info.sid)
}

pub(crate) enum Commands {
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

/// Sends raw HTTP requests to the fritz box.
pub(crate) fn request(cmd: Commands, sid: &str, ain: Option<&str>) -> Result<String> {
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
    info!(
        "[fritz api] {} status: {:?} {:?}",
        cmd,
        status,
        status.canonical_reason().unwrap_or_default()
    );

    Ok(response.text()?)
}

// -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-

/// Requests & parses raw [`Device`]s.
pub(crate) fn device_infos(sid: &str) -> Result<Vec<xml::Device>> {
    let xml = request(Commands::GetDeviceListInfos, sid, None)?;
    xml::parse_device_infos(xml)
}

/// Requests & parses raw [`DeviceStats`]s.
pub(crate) fn fetch_device_stats(ain: &str, sid: &str) -> Result<Vec<xml::DeviceStats>> {
    let xml = request(Commands::GetBasicDeviceStats, sid, Some(ain))?;
    xml::parse_device_stats(xml)
}

// -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-

/// Triggers a higher refresh rate for smart plugs (Fritz!Dect 2xx).
///
/// *Note: This function uses an unofficial and undocumented API which may stop working at any time.
/// It has been verified to work with a Fritz!Box 7560 running FRITZ!OS 07.29. Other models
/// and software versions are likely to work as well.*
///
/// By default, the consumption data (current watts, voltage, temperature etc.)
/// is updated every 2 minutes. Using this function, the update interval can be
/// reduced to ~10 seconds. The higher refresh rate will last for 1-2 minutes and
/// will fall back to the default (2 minutes) afterwards. Call this function
/// repeatedly (e.g. every 30 seconds) to maintain the higher refresh rate.
///
/// The `fritz_dect_2xx_reader` example shows how to read data from smart plugs
/// using the higher refresh rate.
///
/// ### Background
///
/// During testing of the smart plug API, we discovered that the update interval
/// decreases from 2 minutes to 10 seconds when looking at the consumption data
/// in the browser (e.g. using <http://fritz.box/myfritz/>) or in the app.
///
/// Analysis of the network traffic between the website and the Fritz!Box revealed
/// that the client regularly sends a request that activates the higher refresh rate.
/// The request can be replicated on the terminal using `curl` and a valid session id:
///
/// ```bash
/// curl -d 'sid=123456790&c=smarthome&a=getData' http://fritz.box/myfritz/api/data.lua
/// ```
///
/// This function performs basically the same request as the `curl` command above.
pub fn trigger_high_refresh_rate(sid: &str) -> Result<()> {
    let mut params = HashMap::new();
    params.insert("sid", sid);
    params.insert("c", "smarthome");
    params.insert("a", "getData");
    let client = Client::builder()
        .redirect(Policy::none())
        .build()?
        .post("http://fritz.box/myfritz/api/data.lua")
        .form(&params);
    let response = client.send()?;
    let status = response.status();

    if status != 200 {
        return Err(FritzError::TriggerHighRefreshRateError(status));
    }
    Ok(())
}

// -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-

#[cfg(test)]
mod tests {
    #[test]
    fn request_response() {
        let response = super::request_response("mühe", "foo");
        assert_eq!(response, "foo-442e12bbceabd35c66964c913a316451");
    }
}
