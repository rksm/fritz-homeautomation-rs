use chrono::prelude::*;
use chrono::Datelike;

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

    #[allow(dead_code)]
    pub fn berlin() -> Self {
        Self {
            latitude: 52.520,
            longitude: 13.4050,
            altitude: 61,
            h_accuracy: 0,
            v_accuracy: 0,
        }
    }
}

pub fn default_location() -> anyhow::Result<Location> {
    Ok(Location::berlin())
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

        let sunrise = Local
            .from_utc_datetime(&chrono::NaiveDateTime::from_timestamp_opt(sunrise, 0).unwrap());
        let sunrise = if let Some(shift) = sunrise_shift {
            sunrise + shift
        } else {
            sunrise
        };
        println!("sunrise: {}", sunrise.format("%Y-%m-%d %H:%M:%S"));

        let sunset =
            Local.from_utc_datetime(&chrono::NaiveDateTime::from_timestamp_opt(sunset, 0).unwrap());
        let sunset = if let Some(shift) = sunset_shift {
            sunset + shift
        } else {
            sunset
        };
        println!("sunset: {}", sunset.format("%Y-%m-%d %H:%M:%S"));

        date += chrono::Duration::days(1);
    }
}
