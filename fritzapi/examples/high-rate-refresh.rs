//! Example script printing smart power plug data (from Fritz!Dect2XX devices) to the terminal.
//!
//! It expects login information (username, password) to be passed as arguments. Example:
//!
//! ```bash
//! cargo run --example fritz_dect_2xx_reader -- my_username my_password
//! ```
//!
//! The script queries the Fritz!Box for new data every second and prints the data if something has changed.
//!
//! By default, new data will be available every 2 minutes. Using `trigger_high_refresh_rate()`, this time
//! can be decreased to ~10 seconds. To try that, add "HRR" as the third command line argument:
//!
//! ```bash
//! cargo run --example fritz_dect_2xx_reader -- my_username my_password HRR
//! ```
//!

use fritzapi::{AVMDevice, FritzDect2XX, FritzError};
use std::{
    env::args,
    time::{Duration, Instant},
};

fn main() -> Result<(), FritzError> {
    let start_time = Instant::now();
    let mut args = args().skip(1);
    let user = args
        .next()
        .expect("Expected username to be provided on the command line");
    let password = args
        .next()
        .expect("Expected password to be provided on the command line");
    let hrr = args.next() == Some("HRR".to_string());

    let mut client = fritzapi::FritzClient::new(user, password);

    // start a thread that triggers the high refresh rate every 30 seconds
    if hrr {
        let mut client = client.clone();
        std::thread::spawn(move || loop {
            match client.trigger_high_refresh_rate() {
                Ok(()) => println!("Successfully triggered high refresh rate."),
                Err(e) => println!("Error triggering high refresh rate: {e}"),
            }
            std::thread::sleep(Duration::from_secs(30));
        });
    }

    let mut current_devices = vec![];
    loop {
        // list devices
        let devices = client.list_devices()?;

        // filter for Fritz!Dect 2XX devices
        let dect_2xx_devices = devices
            .into_iter()
            .filter_map(fritz_dect_2xx_filter)
            .collect::<Vec<_>>();

        // print device data if it changed
        if dect_2xx_devices != current_devices {
            current_devices = dect_2xx_devices;
            println!(
                "[{}] {:?}",
                format_elapsed_time(&start_time),
                &current_devices
            );
        }

        // sleep for a bit
        std::thread::sleep(Duration::from_secs(1));
    }
}

fn format_elapsed_time(start_time: &Instant) -> String {
    let secs = start_time.elapsed().as_secs();
    format!("{:3}m{:02}s", secs / 60, secs % 60)
}

fn fritz_dect_2xx_filter(device: AVMDevice) -> Option<FritzDect2XX> {
    if let AVMDevice::FritzDect2XX(x) = device {
        Some(x)
    } else {
        None
    }
}
