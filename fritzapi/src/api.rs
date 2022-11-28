use lazy_static::lazy_static;
use log::info;
use regex::Regex;
use reqwest::blocking::{get as GET, Client, Response};

use crate::error::{FritzError, Result};
use crate::fritz_xml as xml;

// -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-

/// Computes the string that we use to authenticate.
/// 1. Replace all non-ascii chars in `password` with "."
/// 2. Concat `challenge` and the modified password
/// 3. Convert that to UTF16le
/// 4. MD5 that byte array
/// 5. concat that as hex with challenge again
fn request_response(password: impl AsRef<str>, challenge: impl AsRef<str>) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"[^\x00-\x7F]").unwrap();
    }
    let clean_password = RE.replace_all(password.as_ref(), ".");
    let hash_input = format!("{}-{}", challenge.as_ref(), clean_password);
    let bytes: Vec<u8> = hash_input
        .encode_utf16()
        .flat_map(|utf16| utf16.to_le_bytes().to_vec())
        .collect();
    let digest = md5::compute(bytes);
    format!("{}-{:032x}", challenge.as_ref(), digest)
}

const DEFAULT_SID: &str = "0000000000000000";

/// Requests a temporary token (session id = sid) from the fritz box using user
/// name and password.
pub fn get_sid(user: impl AsRef<str>, password: impl AsRef<str>) -> Result<String> {
    let res: Response = GET("http://fritz.box/login_sid.lua")?
        .error_for_status()
        .map_err(|err| {
            eprintln!("GET login_sid.lua for user {}", user.as_ref());
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
        user.as_ref(),
        response
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

/// Commands for [FritzClient::request].
#[derive(Clone)]
pub(crate) enum Commands {
    GetDeviceListInfos,
    GetBasicDeviceStats { ain: String },
    // GetSwitchPower,
    // GetSwitchEnergy,
    // GetSwitchName,
    // GetTemplateListInfos,
    SetSwitchOff { ain: String },
    SetSwitchOn { ain: String },
    SetSwitchToggle { ain: String },
}

/// Sends raw HTTP requests to the fritz box.
pub(crate) fn request(cmd: Commands, sid: impl AsRef<str>) -> Result<String> {
    use Commands::*;
    let (cmd, ain) = match cmd {
        GetDeviceListInfos => ("getdevicelistinfos", None),
        GetBasicDeviceStats { ain } => ("getbasicdevicestats", Some(ain)),
        // GetSwitchPower => "getswitchpower",
        // GetSwitchEnergy => "getswitchenergy",
        // GetSwitchName => "getswitchname",
        // GetTemplateListInfos => "gettemplatelistinfos",
        SetSwitchOff { ain } => ("setswitchoff", Some(ain)),
        SetSwitchOn { ain } => ("setswitchon", Some(ain)),
        SetSwitchToggle { ain } => ("setswitchtoggle", Some(ain)),
    };
    let url = "http://fritz.box/webservices/homeautoswitch.lua";
    let mut client = Client::new()
        .get(url)
        .query(&[("switchcmd", cmd), ("sid", sid.as_ref())]);
    if let Some(ain) = ain {
        client = client.query(&[("ain", ain)]);
    }
    let response = client.send()?;
    let status = response.status();
    let status_message = format!(
        "[fritz api] {} status: {:?} {:?}",
        cmd,
        status,
        status.canonical_reason().unwrap_or_default()
    );
    info!("{}", status_message);

    if !status.is_success() {
        if status == 403 {
            Err(FritzError::Forbidden)
        } else {
            Err(FritzError::ApiRequest(status_message))
        }
    } else {
        Ok(response.text()?)
    }
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
