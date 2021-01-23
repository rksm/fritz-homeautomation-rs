use chrono::Datelike;
use chrono::prelude::*;

// use corelocation_rs::{Location, Locator};

// const LOOKUP_LOCATION: bool = false;

struct Location {
    latitude: f64,
    longitude: f64,
    h_accuracy: i32,
    altitude: i32,
    v_accuracy: i32,
}

fn bernau() -> Location {
    Location {
        latitude: 52.671,
        longitude: 13.555,
        h_accuracy: 165,
        altitude: 61,
        v_accuracy: 10,
    }
}

fn main() {
    println!("{}", chrono::Local::now().format("%Y-%m-%d"));

    let location = bernau();

    let start = chrono::NaiveDate::from_ymd(2021, 1, 20);
    let end = chrono::NaiveDate::from_ymd(2021, 2, 28);

    let duration = end - start;
    for days in 0..duration.num_days() {
        let date = start + chrono::Duration::days(days);

        let (sunrise, sunset) = sunrise::sunrise_sunset(
            location.latitude,
            location.longitude,
            date.year(),
            date.month(),
            date.day(),
        );

        // println!("{} {}", date.format("%Y-%m-%d"));
        let sunrise = Local.from_utc_datetime(&chrono::NaiveDateTime::from_timestamp(sunrise, 0));
        let sunset = Local.from_utc_datetime(&chrono::NaiveDateTime::from_timestamp(sunset, 0));
        // println!(
        //     "{} {} - {}",
        //     date.format("%Y-%m-%d"),
        //     sunrise.format("%Y-%m-%d %H:%M:%S"),
        //     sunset.format("%Y-%m-%d %H:%M:%S")
        // );

        let t = sunrise - chrono::Duration::minutes(25);
        println!("{} 05:45:00 on", date.format("%Y-%m-%d"));
        println!("{} off", t.format("%Y-%m-%d %H:%M:%S"));

        let t = sunset + chrono::Duration::minutes(25);
        println!("{} on", t.format("%Y-%m-%d %H:%M:%S"));
        println!("{} 23:05:00 off", date.format("%Y-%m-%d"));

        println!();
    }
}
