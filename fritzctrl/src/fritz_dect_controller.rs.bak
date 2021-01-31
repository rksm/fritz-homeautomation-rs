use chrono::Local;
use clap::{App, Arg};
use fritz_homeautomation::{api, schedule};
use schedule::ScheduleWorker;
use std::path::PathBuf;
use std::process::exit;
use std::{env::current_dir, ffi::OsStr};

fn is_on(sid: &str, ain: &str) -> anyhow::Result<bool> {
    let devices: Vec<_> = api::device_infos_avm(&sid)?;

    let is_on = devices.into_iter().find_map(|dev| match dev {
        api::AVMDevice::FritzDect2XX(dev) if dev.identifier == ain => Some(dev.on),
        _ => None,
    });

    Ok(is_on.unwrap_or(false))
}

#[derive(Debug)]
struct ChristmasScheduleWorker {
    pub user: String,
    pub password: String,
    pub ain: String,
    pub schedule: schedule::Schedule,
}

impl ChristmasScheduleWorker {
    fn login(&self) -> anyhow::Result<String> {
        api::get_sid(&self.user, &self.password)
    }
}

impl schedule::ScheduleWorker for ChristmasScheduleWorker {
    fn process_next_action(
        &mut self,
        action: schedule::Action,
        time: chrono::DateTime<chrono::Local>,
    ) -> anyhow::Result<()> {
        println!(
            "running action {:?} at {}",
            action,
            time.format("%Y-%m-%d %H:%M:%S %Z")
        );

        let sid = self.login()?;
        use schedule::Action::*;
        match action {
            TurnOn => api::turn_on(&sid, &self.ain),
            TurnOff => api::turn_off(&sid, &self.ain),
            Unknown => Ok(()),
        }
    }

    fn check_last_action(&mut self) -> anyhow::Result<()> {
        let (time, action) = match self.schedule().last_action(Local::now()) {
            None => {
                println!("No last action");
                return Ok(());
            }
            Some(next) => next,
        };

        println!(
            "checking {:#?} which should have run at {}",
            action,
            time.format("%Y-%m-%d %H:%M:%S %Z")
        );
        let sid = self.login()?;
        let on = is_on(&sid, &self.ain).unwrap_or_default();
        let should_be_on = *action == schedule::Action::TurnOn;
        if on != should_be_on {
            api::toggle(&sid, &self.ain)
        } else {
            Ok(())
        }
    }

    fn schedule(&self) -> &schedule::Schedule {
        &self.schedule
    }

    fn reload_schedule(&mut self) -> anyhow::Result<()> {
        self.schedule = schedule::Schedule::from_file(&self.schedule.schedule_file)?;
        Ok(())
    }
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

    let schedule = schedule::Schedule::from_file(&schedule_file).expect("read schedule");
    let mut worker = Box::new(ChristmasScheduleWorker {
        user: user.to_string(),
        password: password.to_string(),
        ain,
        schedule,
    });

    worker.login().expect("testing login");
    worker.check_last_action().expect("check last action");

    let thread = schedule::start_processing_schedule(worker);

    thread.join().unwrap();
}
