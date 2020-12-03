use crate::{error::Result, my_error};
use chrono::prelude::*;
use crossbeam_channel::{bounded, select, unbounded, Receiver};
use lazy_static::lazy_static;
use notify::Watcher;
use regex::Regex;
use std::{fs, path::Path, time::Duration};

#[derive(Debug)]
pub enum Action {
    TurnOn,
    TurnOff,
    Unknown,
}

impl From<&str> for Action {
    fn from(action_str: &str) -> Self {
        match action_str {
            "on" => Action::TurnOn,
            "off" => Action::TurnOff,
            _ => Action::Unknown,
        }
    }
}

fn read_schedule<P: AsRef<Path>>(schedule_file: P) -> Result<Vec<(DateTime<Local>, Action)>> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"(.*) (:?on|off)").unwrap();
    }

    let now = chrono::Local::now();

    let schedule = fs::read_to_string(schedule_file).map_err(|err| my_error!(err))?;

    let mut instructions: Vec<_> = schedule
        .split('\n')
        .filter_map(|ea| {
            let captures = RE.captures(ea);
            let (ts, action) = match captures {
                None => return None,
                Some(captures) => {
                    let ts = captures.get(1).unwrap();
                    let action = captures.get(2).unwrap();
                    (ts, action)
                }
            };

            let action = action.as_str().into();

            let date_time = match NaiveDateTime::parse_from_str(ts.as_str(), "%Y-%m-%d %H:%M:%S") {
                Err(_) => return None,
                Ok(date_time) => Local.from_local_datetime(&date_time).unwrap(),
            };

            if now > date_time {
                println!("Skipping past time {}", date_time);
                return None;
            }

            Some((date_time, action))
        })
        .collect();

    instructions.sort_by(|(a, _), (b, _)| a.cmp(&b));

    Ok(instructions)
}

struct ScheduleWatcher {
    #[allow(dead_code)]
    watcher: notify::RecommendedWatcher,
    rx_file_change: Receiver<()>,
}

impl ScheduleWatcher {
    fn watch<P: AsRef<Path>>(file: P) -> Result<Self> {
        let (tx, rx) = unbounded();
        // notify works with std mpsc so we wrap that
        let (tx2, rx2) = std::sync::mpsc::channel();
        let mut watcher =
            notify::watcher(tx2, Duration::from_secs(1)).map_err(|err| my_error!(err))?;
        watcher
            .watch(file, notify::RecursiveMode::NonRecursive)
            .map_err(|err| my_error!(err))?;
        std::thread::spawn(move || loop {
            match rx2.recv() {
                Err(err) => {
                    eprintln!("file watcher error: {}", err);
                    break;
                }
                Ok(notify::DebouncedEvent::Write(_)) => {
                    println!("file changed!");
                    tx.send(()).unwrap();
                }
                Ok(_) => {}
            }
        });
        Ok(ScheduleWatcher {
            watcher,
            rx_file_change: rx,
        })
    }
}

pub fn start_processing_schedule<P: AsRef<Path>>(
    schedule_file: P,
    on_action: impl Fn(Action, DateTime<Local>),
) -> Result<()> {
    let now = Local::now();

    println!(
        "Starting processing schedule at {}",
        now.format("%Y-%m-%d %H:%M:%S %Z")
    );

    let watcher = ScheduleWatcher::watch(&schedule_file)?;

    loop {
        println!("Reading schedule...");

        let schedule = read_schedule(&schedule_file)?;

        if schedule.is_empty() {
            println!("schedule is empty, waiting for file changes");
            watcher.rx_file_change.recv().unwrap();
            continue;
        }

        println!("Start schedule...");

        for (time, action) in schedule {
            println!("scheduling next action {:#?} to run at {}", action, time);
            let timer = timer::Timer::new();
            let (tx, rx) = bounded(1);
            let _guard = timer.schedule_with_date(time, move || {
                let _ignored = tx.send(()); // Avoid unwrapping here.
            });
            select! {
                recv(rx) -> _ => {
                    on_action(action, time);
                },
                recv(watcher.rx_file_change) -> _ => break,
            }
        }
    }
}
