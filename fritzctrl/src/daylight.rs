use chrono::prelude::*;
use chrono::Datelike;

/// <https://developer.apple.com/documentation/corelocation/cllocation>
/// <https://developer.apple.com/documentation/corelocation/cllocationcoordinate2d>
pub struct Location {
    /// The latitude in degrees.
    pub latitude: f64,
    /// The longitude in degrees.
    pub longitude: f64,
    /// The altitude, measured in meters.
    pub altitude: i64,
    /// The radius of uncertainty for the location, measured in meters.
    pub h_accuracy: i64,
    /// The accuracy of the altitude value, measured in meters.
    pub v_accuracy: i64,
}

impl Location {
    pub fn new(latitude: f64, longitude: f64) -> Self {
        Self {
            latitude,
            longitude,
            altitude: 0,
            h_accuracy: 0,
            v_accuracy: 0,
        }
    }

    pub fn bernau() -> Self {
        Self {
            latitude: 52.671,
            longitude: 13.555,
            altitude: 61,
            h_accuracy: 0,
            v_accuracy: 0,
        }
    }
}

#[cfg(target_os = "macos")]
impl From<corelocation_rs::Location> for Location {
    fn from(loc: corelocation_rs::Location) -> Self {
        let corelocation_rs::Location {
            latitude,
            longitude,
            altitude,
            h_accuracy,
            v_accuracy,
        } = loc;
        Location {
            latitude,
            longitude,
            altitude,
            h_accuracy,
            v_accuracy,
        }
    }
}

#[cfg(target_os = "macos")]
pub fn default_location() -> anyhow::Result<Location> {
    use corelocation_rs::Locator;
    let loc = corelocation_rs::Location::get()?;
    println!(
        "using device location ({:.3}, {:.3})",
        loc.latitude, loc.longitude
    );
    Ok(loc.into())
}

#[cfg(not(target_os = "macos"))]
pub fn default_location() -> anyhow::Result<Location> {
    Ok(Location::bernau())
}

pub fn print_daylight_times(
    location: Location,
    from_date: chrono::NaiveDate,
    to_date: chrono::NaiveDate,
    sunrise_shift: Option<chrono::Duration>,
    sunset_shift: Option<chrono::Duration>,
) {
    if to_date < from_date {
        panic!("print_daylight_times: to_date is before from_date");
    }

    let mut date = from_date;

    while date <= to_date {
        let (sunrise, sunset) = sunrise::sunrise_sunset(
            location.latitude,
            location.longitude,
            date.year(),
            date.month(),
            date.day(),
        );

        let sunrise = Local.from_utc_datetime(&chrono::NaiveDateTime::from_timestamp(sunrise, 0));
        let sunrise = if let Some(shift) = sunrise_shift {
            sunrise + shift
        } else {
            sunrise
        };
        println!("sunrise: {}", sunrise.format("%Y-%m-%d %H:%M:%S"));

        let sunset = Local.from_utc_datetime(&chrono::NaiveDateTime::from_timestamp(sunset, 0));
        let sunset = if let Some(shift) = sunset_shift {
            sunset + shift
        } else {
            sunset
        };
        println!("sunset: {}", sunset.format("%Y-%m-%d %H:%M:%S"));

        date += chrono::Duration::days(1);
    }
}
