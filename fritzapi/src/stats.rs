use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct RawDeviceStats {
    pub(crate) temperature: Option<RawManyStats>,
    pub(crate) voltage: Option<RawManyStats>,
    pub(crate) power: Option<RawManyStats>,
    pub(crate) energy: Option<RawManyStats>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RawManyStats {
    pub stats: Vec<RawStats>,
}

#[derive(Debug, Deserialize, Serialize)]
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
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input.to_lowercase().as_str() {
            "temp" | "temperature" | "celsius" | "c" => Ok(DeviceStatsKind::Temperature),
            "power" | "watt" | "w" => Ok(DeviceStatsKind::Power),
            "energy" | "wh" => Ok(DeviceStatsKind::Energy),
            "volt" | "v" | "voltage" => Ok(DeviceStatsKind::Voltage),
            _ => Err(format!("Cannot convert {:?} to DeviceStatsKind", input)),
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
