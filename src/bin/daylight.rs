use chrono::Datelike;

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

    let start = chrono::NaiveDate::from_ymd(2021, 1, 1);
    let end = chrono::NaiveDate::from_ymd(2021, 1, 31);

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

        println!("{} 05:45:00 on", date.format("%Y-%m-%d"));

        let t = chrono::NaiveDateTime::from_timestamp(sunrise, 0) - chrono::Duration::minutes(10);
        println!("{} off", t.format("%Y-%m-%d %H:%M:%S"));

        let t = chrono::NaiveDateTime::from_timestamp(sunset, 0) + chrono::Duration::minutes(10);
        println!("{} on", t.format("%Y-%m-%d %H:%M:%S"));
        println!("{} 23:05:00 off", date.format("%Y-%m-%d"));

        println!();
    }
}
