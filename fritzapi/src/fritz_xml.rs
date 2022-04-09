#![allow(dead_code)]

use crate::{
    devices::{Device, DeviceList, DeviceOrGroup},
    error::{FritzError, Result},
};
use serde::Deserialize;
use serde_xml_rs::from_reader;

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

// -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-

pub fn parse_session_info(xml: &str) -> Result<SessionInfo> {
    from_reader(xml.as_bytes()).map_err(|err| {
        eprintln!("cannot parse session info");
        err.into()
    })
}

/// Parses raw [`Device`]s.
pub fn parse_device_infos(xml: String) -> Result<Vec<Device>> {
    from_reader::<&[u8], DeviceList>(xml.as_bytes())
        .map(|list| {
            list.list
                .into_iter()
                .filter_map(|item| -> Option<_> {
                    match item {
                        DeviceOrGroup::Device(device) => Some(device),
                        // 2022-03-12 ignore groups for now
                        DeviceOrGroup::Group(_) => None,
                    }
                })
                .collect()
        })
        .map_err(|err| {
            eprintln!("cannot parse device infos: {err}");
            err.into()
        })
}

// -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-
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

#[derive(Debug, Deserialize)]
pub struct RawDeviceStats {
    pub temperature: Option<RawManyStats>,
    pub voltage: Option<RawManyStats>,
    pub power: Option<RawManyStats>,
    pub energy: Option<RawManyStats>,
}

#[derive(Debug, Deserialize)]
pub struct RawManyStats {
    pub stats: Vec<RawStats>,
}

#[derive(Debug, Deserialize)]
pub struct RawStats {
    pub count: usize,
    pub grid: usize,
    #[serde(rename = "$value")]
    pub values: String,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Unit {
    Celsius,
    Watt,
    WattHour,
    Volt,
}

impl std::fmt::Display for Unit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Unit::Celsius => write!(f, "°C"),
            Unit::Watt => write!(f, "W"),
            Unit::WattHour => write!(f, "Wh"),
            Unit::Volt => write!(f, "V"),
        }
    }
}

/// Category of measurements that the fritz devices may provide.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum DeviceStatsKind {
    Temperature,
    Voltage,
    Power,
    Energy,
}

impl std::fmt::Display for DeviceStatsKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.name(), self.unit())
    }
}

impl DeviceStatsKind {
    pub fn name(&self) -> &'static str {
        match self {
            DeviceStatsKind::Temperature => "temperature",
            DeviceStatsKind::Voltage => "voltage",
            DeviceStatsKind::Power => "power",
            DeviceStatsKind::Energy => "energy",
        }
    }

    pub fn unit(&self) -> Unit {
        match self {
            DeviceStatsKind::Temperature => Unit::Celsius,
            DeviceStatsKind::Voltage => Unit::Volt,
            DeviceStatsKind::Power => Unit::Watt,
            DeviceStatsKind::Energy => Unit::WattHour,
        }
    }
}

impl std::str::FromStr for DeviceStatsKind {
    type Err = FritzError;

    fn from_str(input: &str) -> Result<Self> {
        match input.to_lowercase().as_str() {
            "temp" | "temperature" | "celsius" | "c" => Ok(DeviceStatsKind::Temperature),
            "power" | "watt" | "w" => Ok(DeviceStatsKind::Power),
            "energy" | "wh" => Ok(DeviceStatsKind::Energy),
            "volt" | "v" | "voltage" => Ok(DeviceStatsKind::Voltage),
            _ => Err(FritzError::ParserError(format!(
                "Cannot convert {:?} to DeviceStatsKind",
                input
            ))),
        }
    }
}

#[derive(Debug)]
pub struct DeviceStats {
    pub kind: DeviceStatsKind,
    pub values: Vec<DeviceStatValues>,
}

#[derive(Debug)]
pub struct DeviceStatValues {
    pub values: Vec<f32>,
    pub grid: usize,
}

pub fn parse_device_stats(xml: String) -> Result<Vec<DeviceStats>> {
    let stats: RawDeviceStats = from_reader(xml.as_bytes())?;

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
                values: raw
                    .stats
                    .into_iter()
                    .map(|ea| DeviceStatValues {
                        grid: ea.grid,
                        values: ea
                            .values
                            .split(',')
                            .filter_map(|val| {
                                val.parse::<f32>()
                                    .ok()
                                    .map(|val| (val * multiplier).round())
                            })
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
    process_raw(stats.power, DeviceStatsKind::Power, 0.01, &mut result);
    process_raw(stats.voltage, DeviceStatsKind::Voltage, 0.001, &mut result);

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn parse_device_stat_kind() {
        assert_eq!(
            "temperature".parse::<DeviceStatsKind>().unwrap(),
            DeviceStatsKind::Temperature
        );
        assert_eq!(
            "celsius".parse::<DeviceStatsKind>().unwrap(),
            DeviceStatsKind::Temperature
        );
        assert_eq!(
            "c".parse::<DeviceStatsKind>().unwrap(),
            DeviceStatsKind::Temperature
        );
        assert_eq!(
            "Temperature".parse::<DeviceStatsKind>().unwrap(),
            DeviceStatsKind::Temperature
        );
        assert_eq!(
            "temp".parse::<DeviceStatsKind>().unwrap(),
            DeviceStatsKind::Temperature
        );
        assert_eq!(
            "power".parse::<DeviceStatsKind>().unwrap(),
            DeviceStatsKind::Power
        );
        assert_eq!(
            "energy".parse::<DeviceStatsKind>().unwrap(),
            DeviceStatsKind::Energy
        );
        assert_eq!(
            "v".parse::<DeviceStatsKind>().unwrap(),
            DeviceStatsKind::Voltage
        );
    }

    #[test]
    fn parse_devices() -> Result<()> {
        let xml = r##"
<devicelist version="1" fwversion="7.21">
  <device identifier="11630 0069103" id="16" functionbitmask="35712" fwversion="04.16" manufacturer="AVM" productname="FRITZ!DECT 200">
    <present>1
    </present>
    <txbusy>0
    </txbusy>
    <name>FRITZ!DECT 200 Laufband Seite
    </name>
    <switch>
      <state>1
      </state>
      <mode>manuell
      </mode>
      <lock>0
      </lock>
      <devicelock>0
      </devicelock>
    </switch>
    <simpleonoff>
      <state>1
      </state>
    </simpleonoff>
    <powermeter>
      <voltage>235330
      </voltage>
      <power>18450
      </power>
      <energy>1060474
      </energy>
    </powermeter>
    <temperature>
      <celsius>210
      </celsius>
      <offset>0
      </offset>
    </temperature>
  </device>
  <group synchronized="0" identifier="grp424E2B-3D5C11C33" id="900" functionbitmask="37504" fwversion="1.0" manufacturer="AVM" productname="">
    <present>1
    </present>
    <txbusy>0
    </txbusy>
    <name>Alles in Bernau
    </name>
    <switch>
      <state>1
      </state>
      <mode>manuell
      </mode>
      <lock>0
      </lock>
      <devicelock>0
      </devicelock>
    </switch>
    <simpleonoff>
      <state>1
      </state>
    </simpleonoff>
    <powermeter>
      <voltage>235107
      </voltage>
      <power>67780
      </power>
      <energy>2431027
      </energy>
    </powermeter>
    <groupinfo>
      <masterdeviceid>0
      </masterdeviceid>
      <members>16,17,18,20,22
      </members>
    </groupinfo>
  </group>
  <device identifier="11657 0272633" id="17" functionbitmask="35712" fwversion="04.17" manufacturer="AVM" productname="FRITZ!DECT 210">
    <present>1
    </present>
    <txbusy>0
    </txbusy>
    <name>FRITZ!DECT 210 draußen
    </name>
    <switch>
      <state>1
      </state>
      <mode>manuell
      </mode>
      <lock>0
      </lock>
      <devicelock>0
      </devicelock>
    </switch>
    <simpleonoff>
      <state>1
      </state>
    </simpleonoff>
    <powermeter>
      <voltage>235313
      </voltage>
      <power>0
      </power>
      <energy>323710
      </energy>
    </powermeter>
    <temperature>
      <celsius>80
      </celsius>
      <offset>0
      </offset>
    </temperature>
  </device>
  <device identifier="11630 0128064" id="18" functionbitmask="35712" fwversion="04.16" manufacturer="AVM" productname="FRITZ!DECT 200">
    <present>1
    </present>
    <txbusy>0
    </txbusy>
    <name>FRITZ!DECT 200 Schreibtisch
    </name>
    <switch>
      <state>1
      </state>
      <mode>manuell
      </mode>
      <lock>0
      </lock>
      <devicelock>0
      </devicelock>
    </switch>
    <simpleonoff>
      <state>1
      </state>
    </simpleonoff>
    <powermeter>
      <voltage>235200
      </voltage>
      <power>4070
      </power>
      <energy>812673
      </energy>
    </powermeter>
    <temperature>
      <celsius>180
      </celsius>
      <offset>0
      </offset>
    </temperature>
  </device>
  <device identifier="09995 0335100" id="19" functionbitmask="320" fwversion="04.94" manufacturer="AVM" productname="FRITZ!DECT 301">
    <present>1
    </present>
    <txbusy>0
    </txbusy>
    <name>FRITZ!DECT 301 #4
    </name>
    <battery>1
    </battery>
    <batterylow>1
    </batterylow>
    <temperature>
      <celsius>195
      </celsius>
      <offset>0
      </offset>
    </temperature>
    <hkr>
      <tist>48
      </tist>
      <tsoll>40
      </tsoll>
      <absenk>34
      </absenk>
      <komfort>40
      </komfort>
      <lock>0
      </lock>
      <devicelock>0
      </devicelock>
      <errorcode>0
      </errorcode>
      <windowopenactiv>0
      </windowopenactiv>
      <windowopenactiveendtime>0
      </windowopenactiveendtime>
      <boostactive>0
      </boostactive>
      <boostactiveendtime>0
      </boostactiveendtime>
      <batterylow>1
      </batterylow>
      <battery>1
      </battery>
      <nextchange>
        <endperiod>1647134100
        </endperiod>
        <tchange>34
        </tchange>
      </nextchange>
      <summeractive>0
      </summeractive>
      <holidayactive>0
      </holidayactive>
    </hkr>
  </device>
  <device identifier="11630 0123723" id="20" functionbitmask="35712" fwversion="04.16" manufacturer="AVM" productname="FRITZ!DECT 200">
    <present>1
    </present>
    <txbusy>0
    </txbusy>
    <name>FRITZ!DECT 200 Laufband hinten
    </name>
    <switch>
      <state>1
      </state>
      <mode>manuell
      </mode>
      <lock>0
      </lock>
      <devicelock>0
      </devicelock>
    </switch>
    <simpleonoff>
      <state>1
      </state>
    </simpleonoff>
    <powermeter>
      <voltage>234877
      </voltage>
      <power>4570
      </power>
      <energy>43714
      </energy>
    </powermeter>
    <temperature>
      <celsius>195
      </celsius>
      <offset>0
      </offset>
    </temperature>
  </device>
  <device identifier="11630 0266726" id="22" functionbitmask="35712" fwversion="04.16" manufacturer="AVM" productname="FRITZ!DECT 200">
    <present>1
    </present>
    <txbusy>0
    </txbusy>
    <name>FRITZ!DECT 200 Router
    </name>
    <switch>
      <state>1
      </state>
      <mode>manuell
      </mode>
      <lock>0
      </lock>
      <devicelock>0
      </devicelock>
    </switch>
    <simpleonoff>
      <state>1
      </state>
    </simpleonoff>
    <powermeter>
      <voltage>235297
      </voltage>
      <power>40620
      </power>
      <energy>190458
      </energy>
    </powermeter>
    <temperature>
      <celsius>210
      </celsius>
      <offset>0
      </offset>
    </temperature>
  </device>
  <device identifier="11657 0492712" id="23" functionbitmask="1024" fwversion="03.64" manufacturer="AVM" productname="FRITZ!DECT Repeater 100">
    <present>0
    </present>
    <txbusy>0
    </txbusy>
    <name>FRITZ!DECT Repeater 100 #8
    </name>
  </device>
</devicelist>
"##;

        let _ = parse_device_infos(xml.to_string())?;

        Ok(())
    }
}
