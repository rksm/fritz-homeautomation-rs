use std::collections::HashSet;

use crate::api;
use crate::fritz_xml as xml;

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
        api::fetch_device_stats(self.id(), sid)
    }

    pub fn print_info(&self, sid: &str, kinds: Option<Vec<xml::DeviceStatsKind>>, limit: Option<usize>) -> anyhow::Result<()> {
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

        let stats = self.fetch_device_stats(&sid)?;
        let kinds = kinds.map(|val| val.into_iter().collect());
        for stat in stats {
            print_stat(&stat, &kinds, limit);
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

fn print_stat(
    stat: &xml::DeviceStats,
    kinds: &Option<HashSet<xml::DeviceStatsKind>>,
    limit: Option<usize>,
) {
    let now = chrono::Local::now();
    println!("{:?}", stat.kind);

    match kinds {
        Some(kinds) if !kinds.contains(&stat.kind) => return,
        _ => {},
    }

    for values in &stat.values {
        let mut n = 0;
        let mut time = now;
        println!("grid: {}", values.grid);
        for val in &values.values {
            println!(
                "{time} {val}{unit}",
                time = time.format("%y-%m-%d %H:%M:%S"),
                val = val,
                unit = stat.kind.unit()
            );
            time = time - chrono::Duration::seconds(values.grid as i64);
            n += 1;
            match limit {
                Some(limit) if n > limit => break,
                _ => continue,
            }
        }
    }
}
