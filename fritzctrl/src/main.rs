use clap::{App, Arg, ArgMatches};
use std::process::exit;

mod parser;
mod daylight;
mod schedule;
mod list;

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
            let date = chrono::Local::today().naive_local();
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

fn switch(args: &ArgMatches) -> anyhow::Result<()> {
    let user = args.value_of("user").unwrap();
    let password = args.value_of("password").unwrap();
    let ain = args.value_of("device").unwrap();
    let toggle = args.is_present("toggle");
    let on = args.is_present("on");
    let off = args.is_present("off");

    let sid = fritzapi::get_sid(&user, &password)?;
    let devices: Vec<_> = fritzapi::list_devices(&sid)?;

    let mut device = match devices.into_iter().find(|dev| dev.id() == ain) {
        None => {
            return Err(anyhow::anyhow!("Cannot find device with ain {:?}", ain));
        }
        Some(device) => device,
    };

    if toggle {
        device.toggle(&sid)?;
    } else if on {
        device.turn_on(&sid)?;
    } else if off {
        device.turn_off(&sid)?;
    }

    Ok(())
}

// -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-

fn main() {
    let user = Arg::with_name("user")
        .long("user")
        .short("u")
        .takes_value(true)
        .required(true);

    let password = Arg::with_name("password")
        .long("password")
        .short("p")
        .takes_value(true)
        .required(true);

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
                .arg(user)
                .arg(password)
                .arg(device.clone().required(true))
                .arg(Arg::with_name("toggle").long("toggle"))
                .arg(Arg::with_name("on").long("on"))
                .arg(Arg::with_name("off").long("off")),
        )
        .subcommand(
            App::new("daylight")
                .help("Prints the daylight times at a specific location. On MacOS will try to use the corelocation API if no latitude/longitude is specified.")
                .arg(Arg::with_name("latitude")
                     .long("latitude")
                     .takes_value(true)
                     .validator(parser::valid_coord))
                .arg(Arg::with_name("longitude")
                     .long("longitude")
                     .takes_value(true)
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
            list::list(args.subcommand_matches("list").unwrap()).unwrap();
        }
        "switch" => {
            switch(args.subcommand_matches("switch").unwrap()).unwrap();
        }
        _ => {
            app.print_help().unwrap();
            exit(1);
        }
    }
}
