use crate::api;
use crate::error::{FritzError, Result};
use crate::fritz_xml;
use crate::AVMDevice;

/// The main interface to get data from the fritz box API.
#[derive(Clone)]
pub struct FritzClient {
    user: String,
    password: String,
    sid: Option<String>,
}

impl FritzClient {
    pub fn new(user: impl ToString, password: impl ToString) -> Self {
        FritzClient {
            user: user.to_string(),
            password: password.to_string(),
            sid: None,
        }
    }

    /// Returns list of all smart home devices. See [devices::AVMDevice].
    pub fn list_devices(&mut self) -> Result<Vec<AVMDevice>> {
        let xml = self.request(api::Commands::GetDeviceListInfos)?;
        let devices = fritz_xml::parse_device_infos(xml)?;
        Ok(devices
            .into_iter()
            .map(AVMDevice::from_xml_device)
            .collect())
    }

    pub fn device_stats(&mut self, ain: impl ToString) -> Result<Vec<crate::stats::DeviceStats>> {
        let ain = ain.to_string();
        let xml = self.request(api::Commands::GetBasicDeviceStats { ain })?;
        fritz_xml::parse_device_stats(xml)
    }

    pub fn turn_on(&mut self, ain: impl ToString) -> Result<()> {
        let ain = ain.to_string();
        self.request(api::Commands::SetSwitchOn { ain })?;
        Ok(())
    }

    pub fn turn_off(&mut self, ain: impl ToString) -> Result<()> {
        let ain = ain.to_string();
        self.request(api::Commands::SetSwitchOff { ain })?;
        Ok(())
    }

    pub fn toggle(&mut self, ain: impl ToString) -> Result<()> {
        let ain = ain.to_string();
        self.request(api::Commands::SetSwitchToggle { ain })?;
        Ok(())
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
    pub fn trigger_high_refresh_rate(&mut self) -> Result<()> {
        let sid = match self.sid.clone().or_else(|| self.update_sid().ok()) {
            None => return Err(FritzError::Forbidden),
            Some(sid) => sid,
        };
        let mut params = std::collections::HashMap::new();
        params.insert("sid", sid.as_ref());
        params.insert("c", "smarthome");
        params.insert("a", "getData");
        let client = reqwest::blocking::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
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

    // -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-

    fn update_sid(&mut self) -> Result<String> {
        let sid = api::get_sid(&self.user, &self.password)?;
        self.sid = Some(sid.clone());
        Ok(sid)
    }

    #[instrument(level = "trace", skip(self))]
    fn request(&mut self, cmd: api::Commands) -> Result<String> {
        self.request_attempt(cmd, 0)
    }

    #[instrument(level = "trace", skip(self))]
    fn request_attempt(&mut self, cmd: api::Commands, request_count: usize) -> Result<String> {
        let sid = match self.sid.clone().or_else(|| self.update_sid().ok()) {
            None => return Err(FritzError::Forbidden),
            Some(sid) => sid,
        };
        match api::request(cmd.clone(), sid) {
            Err(FritzError::Forbidden) if request_count == 0 => {
                let _ = self.update_sid();
                self.request_attempt(cmd, request_count + 1)
            }
            result => result,
        }
    }
}
