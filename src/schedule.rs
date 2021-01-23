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
    pub schedule_file: PathBuf,
}

impl Schedule {
    pub fn from_file<P: AsRef<Path>>(schedule_file: P) -> anyhow::Result<Self> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"(.*) (:?on|off)").unwrap();
        }

        let schedule_lines = fs::read_to_string(&schedule_file)?;
        let mut schedule = Schedule {
            actions: Vec::new(),
            schedule_file: schedule_file.as_ref().to_path_buf(),
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

    pub fn next_action(&self, at: DateTime<Local>) -> Option<(DateTime<Local>, Action)> {
        self.actions.iter().find(|(time, _)| time > &at).cloned()
    }

    pub fn last_action(&self, at: DateTime<Local>) -> Option<&(DateTime<Local>, Action)> {
        self.actions
            .iter()
            .take_while(|(time, _)| time <= &at)
            .last()
    }

    fn watch(&self) -> anyhow::Result<ScheduleWatcher> {
        ScheduleWatcher::watch(&self.schedule_file)
    }
}

struct ScheduleWatcher {
    #[allow(dead_code)]
    file_watcher: notify::RecommendedWatcher,
    rx_file_change: Receiver<()>,
}

impl ScheduleWatcher {
    fn watch<P: AsRef<Path>>(file: P) -> anyhow::Result<Self> {
        let (tx, rx) = unbounded();
        // notify works with std mpsc so we wrap that
        let (tx2, rx2) = std::sync::mpsc::channel();
        let mut watcher = notify::watcher(tx2, Duration::from_secs(1))?;
        watcher.watch(file, notify::RecursiveMode::NonRecursive)?;
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

pub trait ScheduleWorker {
    fn process_next_action(&mut self, action: Action, time: DateTime<Local>) -> anyhow::Result<()>;
    fn check_last_action(&mut self) -> anyhow::Result<()>;
    fn schedule(&self) -> &Schedule;
    fn reload_schedule(&mut self) -> anyhow::Result<()>;
}

pub fn start_processing_schedule(
    mut worker: Box<dyn ScheduleWorker + Send>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let now = Local::now();
        println!(
            "Starting processing schedule at {}",
            now.format("%Y-%m-%d %H:%M:%S %Z")
        );

        let watcher = worker.schedule().watch().expect("watcher");
        let mut schedule_changed = false;

        loop {
            if schedule_changed {
                println!("Reading schedule...");
                worker.reload_schedule().expect("reload schedule");
            }

            let now = Local::now();
            let next = worker.schedule().next_action(now);

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
                    let _guard = timer.schedule_with_date(time, move || {
                        let _ignored = tx.send(()); // Avoid unwrapping here.
                    });
                    select! {
                        recv(rx) -> _ => {
                            if let Err(err) = worker.process_next_action(action, time) {
                                eprintln!("action {:?} errored: {}", action, err);
                            }
                        },
                        recv(watcher.rx_file_change) -> _ => {
                            schedule_changed = true;
                        },
                        default(Duration::from_secs(60 * 5)) => {
                            if let Err(err) = worker.check_last_action() {
                                eprintln!("check last action errored: {}", err);
                            }
                        },
                    }
                }
            }
        }
    })
}
