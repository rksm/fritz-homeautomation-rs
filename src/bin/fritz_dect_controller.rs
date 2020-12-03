use clap::{App, Arg};
use fritz_homeautomation::{api, schedule};
use std::path::PathBuf;
use std::process::exit;
use std::{env::current_dir, ffi::OsStr};

fn main() {
    let matches = App::new("fritz_dect_controller")
        .version("1.0.0")
        .arg(
            Arg::with_name("user")
                .long("user")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("password")
                .long("password")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("ain")
                .long("ain")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("schedule")
                .long("schedule")
                .takes_value(true)
                .required(true),
        )
        .get_matches();

    let schedule_file_arg: &OsStr = matches.value_of_os("schedule").unwrap();
    let schedule_file = match PathBuf::from(schedule_file_arg).canonicalize() {
        Err(err) => {
            eprintln!(
                "schedule file {:?} does not exist in {:?}: {}",
                schedule_file_arg,
                current_dir().unwrap(),
                err
            );
            exit(1);
        }
        Ok(file) => file,
    };

    if !schedule_file.exists() {
        eprintln!("schedule file {:?} does not exist", schedule_file);
        exit(1);
    }

    let user = matches.value_of("user").unwrap();
    let password = matches.value_of("password").unwrap();
    let ain = matches.value_of("ain").unwrap();

    let sid = match api::get_sid(user, password) {
        Err(fritz_homeautomation::error::MyError::LoginError()) => {
            eprintln!("cannot login to fritz");
            exit(2);
        }
        Err(err) => {
            eprintln!("{:?}", err);
            exit(3);
        }
        Ok(sid) => sid,
    };

    schedule::start_processing_schedule(&schedule_file, move |action, time| {
        println!(
            "running action {:?} at {}",
            action,
            time.format("%Y-%m-%d %H:%M:%S %Z")
        );

        use schedule::Action::*;
        let result = match action {
            TurnOn => Some(api::turn_on(&sid, ain)),
            TurnOff => Some(api::turn_off(&sid, ain)),
            Unknown => None,
        };
        if let Some(Err(err)) = result {
            eprintln!("action {:?} errored: {}", action, err);
        }
    }).unwrap();
}
