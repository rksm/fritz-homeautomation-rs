//! Small Rust project to inspect and control [FRITZ!DECT](https://avm.de/produkte/fritzdect/) devices.
//!
//! ## Usage
//!
//! The command line tool has several subcommands:
//! - list: List all devices or list sensor data of individual device.
//! - switch: Turn device on / off.
//! - schedule: Reads and parses lines from stdin that contain date, device id, and state. Runs until all commands are processed.
//! - daylight: Helper command that prints sunrise / sunset times for a given location and time range.
//!
//! Pretty much all commands need the fritz.box user name and password. You can set it in an env vars `FRTIZ_USER` and `FRITZ_PASSWORD` or pass it as arguments to the subcommands (the user / password combo is the same you use for <http://fritz.box>).
//!
//! ## Examples
//!
//! ### List all devices
//!
//! `$ fritzctrl list --user xxx --password yyy`
//!
//! ```text
//!       id       |    product     |            name             | state
//! ---------------+----------------+-----------------------------+-------
//!  11630 0069103 | FRITZ!DECT 200 | FRITZ!DECT 200 Laufband     | on
//!  11657 0272633 | FRITZ!DECT 210 | FRITZ!DECT 210 #2           | off
//!  11630 0128064 | FRITZ!DECT 200 | FRITZ!DECT 200 Schreibtisch | on
//!  09995 0335100 | FRITZ!DECT 301 | FRITZ!DECT 301 #4           |
//!  11630 0123723 | FRITZ!DECT 200 | FRITZ!DECT 200 #5           | off
//! ```
//!
//! ### List last 5 temperature readings of one device
//!
//! `$ fritzctrl list --device "11630 0123723" --kinds temp --limit 3`
//!
//! ```text
//!         time         | Temperature (°C)
//! ---------------------+------------------
//!  2021-01-31 23:42:31 |             22.0
//!  2021-01-31 23:27:31 |             23.0
//!  2021-01-31 23:12:31 |             23.0
//!  2021-01-31 22:57:31 |             23.0
//! ```
//!
//! ### Turn device on
//!
//! `$ fritzctrl switch --device "11630 0123723" --on`
//!
//!
//! ### Schedule switching a device based on daylight hours
//!
//! 1. First figure out what the times you want to turn the device on / off are. E.g.
//! `$ fritzctrl daylight --from-date 2021-02-01 --to-date 2021-02-03 --shift-from="-30min" --shift-to="30hour"`
//! generates sunrise / sunset times shifted by -30 minutes (sunrise) and +30 minutes sunset:
//!
//! ```text
//! using device location (_, _)
//! sunrise: 2021-02-01 07:17:57
//! sunset: 2021-02-01 17:20:41
//! sunrise: 2021-02-02 07:16:20
//! sunset: 2021-02-02 17:22:36
//! sunrise: 2021-02-03 07:14:40
//! sunset: 2021-02-03 17:24:30
//! ```
//!
//! Then store some commands into a file:
//!
//! `fritz-commands.txt`:
//!
//! ```text
//! 2021-02-01 06:00:00 11630 0123723 on
//! 2021-02-01 07:17:57 11630 0123723 off
//! 2021-02-01 17:20:41 11630 0123723 on
//! 2021-02-01 22:30:00 11630 0123723 off
//! ```
//!
//! You can run start processing those commands with
//! `$ cat fritz-commands.txt | fritzctrl schedule`
//!
//! The program will wait until the next command should run and then toggle the device state. Once all commands are done the app will exit.
//!
//! ## Why???
//!
//! Useful for scheduling your Christmas lights!
//!
//! ## Fritz API
//!
//! Uses the [fritz HTTP API](https://avm.de/fileadmin/user_upload/Global/Service/Schnittstellen/AHA-HTTP-Interface.pdf).
//!
//! ### Rust API
//!
//! If you want to integrate directly with the API have a look at the [fritzapi crate](https://crates.io/crates/fritzapi).
//!

use clap::{App, Arg, ArgMatches};
use std::io::Read;
use std::process::exit;

mod daylight;
mod list;
mod parser;
mod schedule;
mod switch;

fn daylight(args: &ArgMatches) {
    // get date arguments
    let date = args
        .value_of("date")
        .and_then(|val| chrono::NaiveDate::parse_from_str(val, "%Y-%m-%d").ok());
    let from_date = args
        .value_of("from-date")
        .and_then(|val| chrono::NaiveDate::parse_from_str(val, "%Y-%m-%d").ok());
    let to_date = args
        .value_of("to-date")
        .and_then(|val| chrono::NaiveDate::parse_from_str(val, "%Y-%m-%d").ok());
    let (from_date, to_date) = match (from_date, to_date, date) {
        (Some(from_date), Some(to_date), _) => (from_date, to_date),
        (_, _, Some(date)) => (date, date),
        _ => {
            let date = chrono::Local::now().date_naive();
            (date, date)
        }
    };

    // get shift
    let shift_from = args.value_of("shift-from").and_then(parser::parse_duration);
    let shift_to = args.value_of("shift-to").and_then(parser::parse_duration);

    // get location
    let latitude: Option<f64> = args.value_of("latitude").and_then(|val| val.parse().ok());
    let longitude: Option<f64> = args.value_of("longitude").and_then(|val| val.parse().ok());
    let location = match (latitude, longitude) {
        (Some(latitude), Some(longitude)) => daylight::Location::new(latitude, longitude),
        _ => {
            if let Ok(loc) = daylight::default_location() {
                loc
            } else {
                println!("Could not determine location for daylight time. Maybe use --latitude / --longitude?");
                exit(1);
            }
        }
    };

    daylight::print_daylight_times(location, from_date, to_date, shift_from, shift_to);
}

// -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-

fn main() {
    env_logger::init();

    dotenv::dotenv().ok();

    let user = Arg::with_name("user")
        .long("user")
        .short("u")
        .takes_value(true)
        .required(true)
        .env("FRITZ_USER");

    let password = Arg::with_name("password")
        .long("password")
        .short("p")
        .takes_value(true)
        .required(true)
        .env("FRITZ_PASSWORD");

    let device = Arg::with_name("device")
        .long("device")
        .takes_value(true)
        .required(true)
        .help("The device identifier (ain) of the device to query / control.");

    let mut app = App::new(env!("CARGO_PKG_NAME"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .version(env!("CARGO_PKG_VERSION"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .subcommand(
            App::new("list")
                .about("List all connected devices or list sensor data of individual device (when used with --device ID)")
                .arg(user.clone())
                .arg(password.clone())
                .arg(device.clone().required(false))
                .arg(Arg::with_name("limit")
                     .long("limit")
                     .short("l")
                     .takes_value(true)
                     .validator(parser::valid_usize))
                .arg(Arg::with_name("kinds")
                     .long("kinds")
                     .takes_value(true)
                     .validator(parser::valid_kinds)
                     .requires("device")
                     .help("Comma separated list of the detail categories to show. Possible values: temperature, voltage, power, energy")),
        )
        .subcommand(
            App::new("switch")
                .about("Toggle device on / off")
                .arg(user.clone())
                .arg(password.clone())
                .arg(device.clone().required(true))
                .arg(Arg::with_name("toggle").long("toggle"))
                .arg(Arg::with_name("on").long("on"))
                .arg(Arg::with_name("off").long("off")),
        )
        .subcommand(
            App::new("daylight")
                .about("Prints the daylight times at a specific location.")
                .arg(Arg::with_name("latitude")
                     .long("latitude")
                     .takes_value(true)
                     .required(true)
                     .env("LATITUDE")
                     .validator(parser::valid_coord))
                .arg(Arg::with_name("longitude")
                     .long("longitude")
                     .takes_value(true)
                     .required(true)
                     .env("LONGITUDE")
                     .validator(parser::valid_coord))
                .arg(Arg::with_name("date")
                     .long("date")
                     .takes_value(true)
                     .validator(parser::valid_date))
                .arg(Arg::with_name("from-date")
                     .long("from-date")
                     .takes_value(true)
                     .validator(parser::valid_date))
                .arg(Arg::with_name("to-date")
                     .long("to-date")
                     .takes_value(true)
                     .validator(parser::valid_date))
                .arg(Arg::with_name("shift-from")
                     .long("shift-from")
                     .takes_value(true)
                     .validator(parser::valid_shift))
                .arg(Arg::with_name("shift-to")
                     .long("shift-to")
                     .takes_value(true)
                     .validator(parser::valid_shift))
        )
        .subcommand(
            App::new("schedule")
                .about("Reads newline separated commands from stdin and then runs until the last command is done.")
                .arg(user.clone())
                .arg(password.clone())
        );

    let args = app.clone().get_matches();

    let cmd = match args.subcommand {
        None => {
            app.print_help().unwrap();
            exit(1);
        }
        Some(ref cmd) => cmd.name.as_str(),
    };

    match cmd {
        "daylight" => {
            let args = args.subcommand_matches("daylight").unwrap();
            daylight(args);
        }
        "list" => {
            if let Err(err) = list::list(args.subcommand_matches("list").unwrap()) {
                println!("{}", err);
                exit(2);
            }
        }
        "switch" => {
            if let Err(err) = switch::switch(args.subcommand_matches("switch").unwrap()) {
                println!("{}", err);
                exit(2);
            }
        }
        "schedule" => {
            let args = args.subcommand_matches("schedule").unwrap();
            let user = args.value_of("user").unwrap();
            let password = args.value_of("password").unwrap();
            let stdin = std::io::stdin();
            let mut input = String::new();
            stdin.lock().read_to_string(&mut input).unwrap();
            if let Err(err) = schedule::Schedule::from_string(input)
                .and_then(|mut schedule| schedule.start(user, password))
            {
                eprintln!("Error running schedule: {}", err);
                exit(3);
            };
        }
        _ => {
            app.print_help().unwrap();
            exit(1);
        }
    }
}
