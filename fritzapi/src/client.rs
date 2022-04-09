use crate::api;
use crate::error::{FritzError, Result};
use crate::fritz_xml;
use crate::AVMDevice;

/// The main interface to get data from the fritz box API.
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

    pub fn device_stats(&mut self, ain: impl ToString) -> Result<Vec<fritz_xml::DeviceStats>> {
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

    // -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-

    fn update_sid(&mut self) -> Result<String> {
        let sid = api::get_sid(&self.user, &self.password)?;
        self.sid = Some(sid.clone());
        Ok(sid)
    }

    fn request(&mut self, cmd: api::Commands) -> Result<String> {
        self.request_attempt(cmd, 0)
    }

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
