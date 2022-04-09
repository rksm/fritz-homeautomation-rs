use crate::error::Result;
use crate::fritz_xml as xml;
use crate::FritzClient;

mod fritz_dect_2xx;
pub use fritz_dect_2xx::FritzDect2XX;
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum AVMDevice {
    FritzDect2XX(FritzDect2XX),
    Other(xml::Device),
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
    pub fn from_xml_device(device: xml::Device) -> Self {
        match device {
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
            } if productname.starts_with("FRITZ!DECT 2") => AVMDevice::FritzDect2XX(FritzDect2XX {
                identifier,
                productname,
                name,
                on: state,
                voltage: voltage as f32 * 0.001,
                watts: power as f32 * 0.001,
                energy_in_watt_h: energy,
                celsius: celsius.parse::<f32>().unwrap_or_default() * 0.1,
            }),

            _ => AVMDevice::Other(device),
        }
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

    pub fn fetch_device_stats(&self, client: &mut FritzClient) -> Result<Vec<xml::DeviceStats>> {
        client.device_stats(self.id())
    }

    pub fn turn_on(&mut self, client: &mut FritzClient) -> Result<()> {
        client.turn_on(self.id())
    }

    pub fn turn_off(&mut self, client: &mut FritzClient) -> Result<()> {
        client.turn_off(self.id())
    }

    pub fn toggle(&mut self, client: &mut FritzClient) -> Result<()> {
        client.toggle(self.id())
    }
}
