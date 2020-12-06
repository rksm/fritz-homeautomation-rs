use crate::{error::Result, my_error};
use chrono::prelude::*;
use crossbeam_channel::{bounded, select, unbounded, Receiver};
use lazy_static::lazy_static;
use notify::Watcher;
use regex::Regex;
use std::{fs, path::Path, path::PathBuf, time::Duration};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug)]
pub struct Schedule {
    pub actions: Vec<(DateTime<Local>, Action)>,
}

impl Schedule {
    pub fn from_file<P: AsRef<Path>>(schedule_file: P) -> Result<Self> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"(.*) (:?on|off)").unwrap();
        }

        let schedule_lines = fs::read_to_string(schedule_file).map_err(|err| my_error!(err))?;
        let mut schedule = Schedule {
            actions: Vec::new(),
        };

        for line in schedule_lines.split('\n') {
            let captures = RE.captures(line);
            let (ts, action) = match captures {
                None => continue,
                Some(captures) => {
                    let ts = captures.get(1).unwrap();
                    let action = captures.get(2).unwrap();
                    (ts, action)
                }
            };

            let action = action.as_str().into();

            let date_time = match NaiveDateTime::parse_from_str(ts.as_str(), "%Y-%m-%d %H:%M:%S") {
                Err(_) => {
                    eprintln!("Cannot read date/time at line {:?}", line);
                    continue;
                }
                Ok(date_time) => Local.from_local_datetime(&date_time).unwrap(),
            };

            schedule.actions.push((date_time, action));
        }

        schedule.actions.sort_by(|(a, _), (b, _)| a.cmp(&b));

        Ok(schedule)
    }

    pub fn next_action(&self, at: DateTime<Local>) -> Option<&(DateTime<Local>, Action)> {
        self.actions.iter().find(|(time, _)| time > &at)
    }

    pub fn last_action(&self, at: DateTime<Local>) -> Option<&(DateTime<Local>, Action)> {
        self.actions
            .iter()
            .take_while(|(time, _)| time <= &at)
            .last()
    }
}

struct ScheduleWatcher {
    #[allow(dead_code)]
    file_watcher: notify::RecommendedWatcher,
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
            file_watcher: watcher,
            rx_file_change: rx,
        })
    }
}

pub struct ScheduleWorker {
    thread: std::thread::JoinHandle<()>,
}

impl ScheduleWorker {
    pub fn start_processing_schedule<
        F: 'static + Fn(Action, DateTime<Local>) + Send,
        F2: 'static + Fn(Action, DateTime<Local>) + Send,
    >(
        schedule_file: PathBuf,
        on_action: F,
        check_action: F2,
    ) -> Self {
        let thread = std::thread::spawn(move || {
            let now = Local::now();

            println!(
                "Starting processing schedule at {}",
                now.format("%Y-%m-%d %H:%M:%S %Z")
            );

            let watcher = ScheduleWatcher::watch(&schedule_file).expect("file watcher");
            let mut schedule = Schedule::from_file(&schedule_file).expect("read schedule");
            let mut schedule_changed = false;

            loop {
                if schedule_changed {
                    println!("Reading schedule...");
                    schedule = Schedule::from_file(&schedule_file).expect("read schedule");
                }

                let now = Local::now();
                let next = schedule.next_action(now);

                match next {
                    None => {
                        println!("schedule is empty, waiting for file changes");
                        watcher.rx_file_change.recv().unwrap();
                        continue;
                    }
                    Some((time, action)) => {
                        println!("scheduling next action {:#?} to run at {}", action, time);
                        let timer = timer::Timer::new();
                        let (tx, rx) = bounded(1);
                        let _guard = timer.schedule_with_date(*time, move || {
                            let _ignored = tx.send(()); // Avoid unwrapping here.
                        });
                        select! {
                            recv(rx) -> _ => {
                                on_action(*action, *time);
                            },
                            recv(watcher.rx_file_change) -> _ => {
                                schedule_changed = true;
                            },
                            default(Duration::from_secs(60 * 5)) => {
                                if let Some((time, action)) = schedule.last_action(Local::now()) {
                                    check_action(*action, *time);
                                }
                            },
                        }
                    }
                }
            }
        });

        ScheduleWorker { thread }
    }

    pub fn join(self) -> std::thread::Result<()> {
        self.thread.join()
    }
}
