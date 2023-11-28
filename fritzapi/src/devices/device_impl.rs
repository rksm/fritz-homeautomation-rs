use super::{AVMDevice, Device, FritzDect2XX, PowerMeter, Switch, Temperature};
use crate::error::Result;
use crate::FritzClient;

impl AVMDevice {
    pub fn from_xml_device(device: Device) -> Self {
        match device {
            Device {
                identifier,
                productname,
                name,
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
                identifier,
                productname,
                name,
                on: state,
                millivolts: voltage,
                milliwatts: power,
                energy_in_watt_h: energy,
                celsius: celsius.parse::<f32>().unwrap_or_default() * 0.1,
            }),

            _ => AVMDevice::Other(device),
        }
    }

    pub fn fetch_device_stats(
        &self,
        client: &mut FritzClient,
    ) -> Result<Vec<crate::stats::DeviceStats>> {
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
