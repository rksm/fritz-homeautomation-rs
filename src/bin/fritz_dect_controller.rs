use clap::{App, Arg};
use fritz_homeautomation::{api, error, schedule};
use std::path::PathBuf;
use std::process::exit;
use std::{env::current_dir, ffi::OsStr};

fn is_on(sid: &str, ain: &str) -> error::Result<bool> {
    let devices: Vec<_> = api::device_infos_avm(&sid)?;

    let is_on = devices.into_iter().find_map(|dev| match dev {
        api::AVMDevice::FritzDect2XX(dev) if dev.identifier == ain => Some(dev.on),
        _ => None,
    });

    Ok(is_on.unwrap_or(false))
}

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
    let ain = matches.value_of("ain").unwrap().to_string();
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

    let ain2 = ain.clone();
    let sid2 = sid.clone();

    let worker = schedule::ScheduleWorker::start_processing_schedule(
        schedule_file,
        move |action, time| {
            println!(
                "running action {:?} at {}",
                action,
                time.format("%Y-%m-%d %H:%M:%S %Z")
            );

            use schedule::Action::*;
            let result = match action {
                TurnOn => Some(api::turn_on(&sid, &ain)),
                TurnOff => Some(api::turn_off(&sid, &ain)),
                Unknown => None,
            };
            if let Some(Err(err)) = result {
                eprintln!("action {:?} errored: {}", action, err);
            }
        },
        move |action, time| {
            println!(
                "checking {:#?} which should have run at {}",
                action,
                time.format("%Y-%m-%d %H:%M:%S %Z")
            );
            let on = is_on(&sid2, &ain2).unwrap_or_default();
            let should_be_on = action == schedule::Action::TurnOn;
            if on != should_be_on {
                api::toggle(&sid2, &ain2).expect("cannot toggle");
            }
        },
    );

    worker.join().unwrap();
}
