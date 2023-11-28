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

#[macro_use]
extern crate tracing;

use chrono::{prelude::*, Duration};
use clap::{value_parser, Arg, ArgAction, ArgMatches, Command};
use std::io::Read;
use std::process::exit;
use tracing_subscriber::prelude::*;

mod daylight;
mod list;
mod parser;
mod schedule;
mod switch;

fn daylight(args: &ArgMatches) {
    // get date arguments
    let date = args.get_one::<NaiveDate>("date");
    let from_date = args.get_one::<NaiveDate>("from-date");
    let to_date = args.get_one::<NaiveDate>("to-date");

    let (from_date, to_date) = match (from_date, to_date, date) {
        (Some(from_date), Some(to_date), _) => (*from_date, *to_date),
        (_, _, Some(date)) => (*date, *date),
        _ => {
            let date = Local::now().date_naive();
            (date, date)
        }
    };

    // get shift
    let shift_from = args.get_one::<Duration>("shift-from").copied();
    let shift_to = args.get_one::<Duration>("shift-to").copied();

    // get location
    let latitude = args.get_one::<f64>("latitude");
    let longitude = args.get_one::<f64>("longitude");
    let location = match (latitude, longitude) {
        (Some(latitude), Some(longitude)) => daylight::Location::new(*latitude, *longitude),
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

#[derive(Debug, Clone, Copy)]
enum Commands {
    List,
    Switch,
    Daylight,
    Schedule,
}

fn main() {
    dotenv::dotenv().ok();

    let user = Arg::new("user")
        .long("user")
        .short('u')
        .value_name("USER")
        .required(true)
        .env("FRITZ_USER");

    let password = Arg::new("password")
        .long("password")
        .short('p')
        .required(true)
        .env("FRITZ_PASSWORD");

    let device = Arg::new("device")
        .required(true)
        .help("The device identifier (ain) of the device to query / control.");

    let mut app = Command::new(env!("CARGO_PKG_NAME"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .version(env!("CARGO_PKG_VERSION"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(Arg::new("verbose").long("verbose").short('v').action(ArgAction::SetTrue))
        .subcommand(
            Command::new("list")
                .about("List all connected devices or list sensor data of individual device (when used with --device ID)")
                .arg(user.clone())
                .arg(password.clone())
                .arg(device.clone().required(false))
                .arg(Arg::new("limit")
                     .long("limit")
                     .short('l')
                     .value_parser(value_parser!(usize)))
                .arg(Arg::new("kinds")
                     .long("kinds")
                    .value_parser(parser::parse_kinds)
                     .requires("device")
                     .help("Comma separated list of the detail categories to show. Possible values: temperature, voltage, power, energy")),
        )
        .subcommand(
            Command::new("switch")
                .about("Toggle device on / off")
                .arg(user.clone())
                .arg(password.clone())
                .arg(device.required(true))
                .arg(Arg::new("toggle").long("toggle").action(ArgAction::SetTrue))
                .arg(Arg::new("on").long("on").action(ArgAction::SetTrue))
                .arg(Arg::new("off").long("off").action(ArgAction::SetTrue)),
        )
        .subcommand(
            Command::new("daylight")
                .about("Prints the daylight times at a specific location.")
                .arg(Arg::new("latitude")
                     .long("latitude")
                     .required(true)
                     .env("LATITUDE")
                     .value_parser(value_parser!(f64)))
                .arg(Arg::new("longitude")
                     .long("longitude")
                     .required(true)
                     .env("LONGITUDE")
                     .value_parser(value_parser!(f64)))
                .arg(Arg::new("date")
                     .long("date")
                     .value_parser(parser::valid_date))
                .arg(Arg::new("from-date")
                     .long("from-date")
                     .value_parser(parser::valid_date))
                .arg(Arg::new("to-date")
                     .long("to-date")
                     .value_parser(parser::valid_date))
                .arg(Arg::new("shift-from")
                     .long("shift-from")
                     .value_parser(parser::parse_duration))
                .arg(Arg::new("shift-to")
                     .long("shift-to")
                     .value_parser(parser::parse_duration))
        )
        .subcommand(
            Command::new("schedule")
                .about("Reads newline separated commands from stdin and then runs until the last command is done.")
                .arg(user)
                .arg(password)
        );

    let args = app.clone().get_matches();

    let log_level = if args.get_flag("verbose") {
        "info,fritz=trace,reqwest=debug"
    } else {
        "info"
    };

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::builder().parse_lossy(log_level))
        .init();

    let cmd = match args.subcommand() {
        None => {
            app.print_help().unwrap();
            exit(1);
        }
        Some((cmd, _args)) => match cmd {
            "daylight" => Commands::Daylight,
            "list" => Commands::List,
            "switch" => Commands::Switch,
            "schedule" => Commands::Schedule,
            _ => {
                app.print_help().unwrap();
                exit(1);
            }
        },
    };

    match cmd {
        Commands::Daylight => {
            let args = args.subcommand_matches("daylight").unwrap();
            daylight(args);
        }

        Commands::List => {
            if let Err(err) = list::list(args.subcommand_matches("list").unwrap()) {
                println!("{}", err);
                exit(2);
            }
        }

        Commands::Switch => {
            if let Err(err) = switch::switch(args.subcommand_matches("switch").unwrap()) {
                println!("Error: {}", err);
                exit(2);
            }
        }

        Commands::Schedule => {
            let args = args.subcommand_matches("schedule").unwrap();
            let user = args.get_one::<String>("user").unwrap();
            let password = args.get_one::<String>("password").unwrap();
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
    }
}
