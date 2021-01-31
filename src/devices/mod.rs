use crate::api;
use crate::fritz_xml as xml;
use anyhow::anyhow;

mod fritz_dect_2xx;
pub use fritz_dect_2xx::FritzDect2XX;

#[derive(Debug)]
pub enum AVMDevice {
    FritzDect2XX(FritzDect2XX),
    Other(xml::Device),
}

impl AVMDevice {
    pub fn list(sid: &str) -> anyhow::Result<Vec<AVMDevice>> {
        let devices = api::device_infos(sid)?;
        let result: Vec<AVMDevice> = devices
            .into_iter()
            .map(|dev| match &dev {
                xml::Device {
                    identifier,
                    productname,
                    name,
                    switch: Some(xml::Switch { state, .. }),
                    powermeter:
                        Some(xml::PowerMeter {
                            energy,
                            power,
                            voltage,
                            ..
                        }),
                    temperature: Some(xml::Temperature { celsius, .. }),
                    ..
                } if productname.starts_with("FRITZ!DECT 2") => {
                    AVMDevice::FritzDect2XX(FritzDect2XX {
                        identifier: identifier.clone(),
                        productname: productname.clone(),
                        name: name.clone(),
                        on: *state,
                        voltage: *voltage as f32 * 0.001,
                        watts: *power as f32 * 0.001,
                        energy_in_watt_h: *energy,
                        celsius: celsius.parse::<f32>().unwrap_or_default() * 0.1,
                        // raw: dev,
                    })
                }

                _ => AVMDevice::Other(dev),
            })
            .collect();
        Ok(result)
    }

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

    pub fn state(&self) -> &str {
        match self {
            AVMDevice::FritzDect2XX(FritzDect2XX { on: true, .. }) => "on",
            AVMDevice::FritzDect2XX(FritzDect2XX { on: false, .. }) => "off",
            AVMDevice::Other(_) => "",
        }
    }

    pub fn fetch_device_stats(&self, sid: &str) -> anyhow::Result<Vec<xml::DeviceStats>> {
        let ain = self.id();
        let xml = api::request(api::Commands::GetBasicDeviceStats, sid, Some(ain))?;
        match xml::parse_device_stats(xml) {
            Ok(stats) => Ok(stats),
            Err(err) => Err(anyhow!("[parse_device_stats] error: {}", err)),
        }
    }

    pub fn print_info(&self, show_stats: bool, sid: Option<&str>) -> anyhow::Result<()> {
        match self {
            AVMDevice::FritzDect2XX(dev @ FritzDect2XX { .. }) => {
                println!(
                    "Device identifier={:?} productname={:?} name={:?}",
                    dev.identifier, dev.productname, dev.name
                );
            }
            AVMDevice::Other(dev) => {
                println!(
                    "Unsupported device identifier={:?} productname={:?} name={:?}",
                    dev.identifier, dev.productname, dev.name
                );
            }
        }
        if let (true, Some(sid)) = (show_stats, sid) {
            let stats = self.fetch_device_stats(&sid)?;
            for ea in stats {
                println!("{}", ea);
            }
        }
        Ok(())
    }

    pub fn turn_on(&mut self, sid: &str) -> anyhow::Result<()> {
        api::request(api::Commands::SetSwitchOn, sid, Some(self.id()))?;
        Ok(())
    }

    pub fn turn_off(&mut self, sid: &str) -> anyhow::Result<()> {
        api::request(api::Commands::SetSwitchOff, sid, Some(self.id()))?;
        Ok(())
    }

    pub fn toggle(&mut self, sid: &str) -> anyhow::Result<()> {
        api::request(api::Commands::SetSwitchToggle, sid, Some(self.id()))?;
        Ok(())
    }
}
