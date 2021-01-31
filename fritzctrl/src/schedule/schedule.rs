use super::action::Action;
use crate::switch;
use chrono::prelude::*;
use std::{fs, path::Path};

#[derive(Debug)]
pub struct Schedule {
    pub actions: Vec<Action>,
}

impl Schedule {
    #[allow(dead_code)]
    pub fn from_file<P: AsRef<Path>>(schedule_file: P) -> anyhow::Result<Self> {
        let content = fs::read_to_string(&schedule_file)?;
        Self::from_string(content)
    }

    pub fn from_string(string: String) -> anyhow::Result<Self> {
        let lines = string.lines();
        let actions: Vec<Action> = lines
            .into_iter()
            .filter_map(|line| {
                line.parse()
                    .map_err(|_| {
                        eprintln!("[schedule] cannot parse line {:?}", line);
                    })
                    .ok()
            })
            .collect();

        let mut schedule = Schedule { actions };
        schedule.actions.sort_by_key(|ea| ea.time());
        Ok(schedule)
    }

    pub fn next_action(&self, at: DateTime<Local>) -> Option<Action> {
        self.actions
            .iter()
            .find(|action| action.time() > at)
            .cloned()
    }

    #[allow(dead_code)]
    pub fn last_action(&self, at: DateTime<Local>) -> Option<&Action> {
        self.actions
            .iter()
            .take_while(|action| action.time() <= at)
            .last()
    }

    pub fn start(&mut self, user: &str, password: &str) -> anyhow::Result<()> {
        let now = Local::now();
        println!(
            "[schedule] starting processing at time {}",
            now.format("%Y-%m-%d %H:%M:%S %Z")
        );

        loop {
            let now = Local::now();
            let next = self.next_action(now);

            match next {
                None => {
                    println!(
                        "[schedule] no actions left stopping at time {}",
                        now.format("%Y-%m-%d %H:%M:%S %Z")
                    );
                    return Ok(());
                }
                Some(action) => {
                    let time = action.time();
                    let duration = time - now;

                    println!(
                        "[schedule] scheduling next action {:?}, sleeping for {}",
                        action, duration
                    );
                    std::thread::sleep(duration.to_std()?);
                    if let Err(err) = self.run(action, user, password) {
                        eprintln!("[schedule] error running action: {:?}", err);
                    }
                }
            }
        }
    }

    fn run(&mut self, action: Action, user: &str, password: &str) -> anyhow::Result<()> {
        switch::run(user, password, action.device_id(), action.clone().into())
    }
}
