#[derive(Debug)]
pub struct FritzDect2XX {
    pub identifier: String,
    pub name: String,
    pub productname: String,
    pub on: bool,
    pub millivolts: u32,
    pub milliwatts: u32,
    pub energy_in_watt_h: u32,
    pub celsius: f32,
}
