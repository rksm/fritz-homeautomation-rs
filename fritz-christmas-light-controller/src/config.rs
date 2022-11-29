use std::{io::Read, path::Path};

use chrono::{prelude::*, Duration};
use serde::{Deserialize, Serialize};

use crate::Result;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub device: String,
    pub start: DateTime<Local>,
    pub end: DateTime<Local>,
    #[serde(with = "crate::duration")]
    pub check_state: Duration,
    pub entries: Vec<Entry>,
}

impl Config {
    pub fn from_yaml_file(p: impl AsRef<Path>) -> Result<Self> {
        let f = std::fs::File::open(p)?;
        Self::from_yaml(f)
    }

    pub fn from_yaml(yaml_reader: impl Read) -> Result<Self> {
        Ok(serde_yaml::from_reader(yaml_reader)?)
    }

    pub fn from_string(s: impl ToString) -> Result<Self> {
        Config::from_yaml(s.to_string().as_bytes())
    }

    pub fn intervals(&self) -> Vec<Interval> {
        let state_changes = StateChange::from_entries_between(&self.entries, self.start, self.end);
        state_changes
            .iter()
            .zip(state_changes.iter().skip(1))
            .map(|(a, b)| Interval {
                start: a.when,
                end: b.when,
                state: a.state,
            })
            .collect::<Vec<_>>()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Entry {
    pub when: When,
    pub time: NaiveTime,
    pub state: State,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum When {
    Daily,
    Date(NaiveDate),
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum State {
    #[default]
    Off,
    On,
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            State::Off => write!(f, "off"),
            State::On => write!(f, "on"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct StateChange {
    pub when: DateTime<Local>,
    pub state: State,
}

impl StateChange {
    pub fn from_entries_between(
        entries: &[Entry],
        begin: DateTime<Local>,
        end: DateTime<Local>,
    ) -> Vec<Self> {
        let mut state_changes = Vec::new();
        let start_date = begin.date_naive();
        let end_date = end.date_naive();
        let mut current_date = start_date;
        while current_date < end_date {
            tracing::debug!("computing entries for date {current_date}");
            for entry in entries {
                let (when, state) = match entry.when {
                    When::Daily => (dt(current_date, entry.time), entry.state),
                    When::Date(date) if date == current_date => {
                        (dt(current_date, entry.time), entry.state)
                    }
                    _ => continue,
                };
                state_changes.push(StateChange { when, state });
            }
            current_date += Duration::days(1);
        }

        // stable sort by datetime
        state_changes.sort_by_key(|ea| ea.when);
        // only keep one entry if two entries match in datetime. take the first appearing one.
        state_changes.dedup_by_key(|ea| ea.when);

        // ensure that state changes have start and end
        let needs_begin = state_changes
            .first()
            .map(|s| s.when > begin)
            .unwrap_or(false);
        if needs_begin {
            state_changes.insert(
                0,
                StateChange {
                    when: begin,
                    state: Default::default(),
                },
            );
        }
        let needs_end = state_changes.last().map(|s| s.when < end).unwrap_or(false);
        if needs_end {
            state_changes.push(StateChange {
                when: end,
                state: Default::default(),
            });
        }

        // remove those entries that don't change the state
        let mut state = None;
        state_changes.retain(|ea| {
            if let Some(last_state) = state.take() {
                let result = ea.state != last_state;
                state = Some(ea.state);
                result
            } else {
                state = Some(ea.state);
                true
            }
        });

        tracing::debug!(
            "added {} entries between {:?} and {:?}",
            state_changes.len(),
            state_changes.first().map(|ea| ea.when),
            state_changes.last().map(|ea| ea.when)
        );

        state_changes
    }
}

#[derive(Debug, Clone)]
pub struct Interval {
    pub start: DateTime<Local>,
    pub end: DateTime<Local>,
    pub state: State,
}

impl std::fmt::Display for Interval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}={}", self.start, self.end, self.state)
    }
}

impl Interval {
    pub fn contains_time(&self, t: DateTime<Local>) -> bool {
        self.start <= t && t < self.end
    }

    pub fn is_current(&self) -> bool {
        self.contains_time(Local::now())
    }
}

fn dt(date: NaiveDate, time: NaiveTime) -> DateTime<Local> {
    date.and_time(time).and_local_timezone(Local).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_config() {
        let begin = NaiveDateTime::parse_from_str("2022-11-28 00:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap()
            .and_local_timezone(Local)
            .unwrap();
        let end = begin + Duration::days(4);

        let config = Config {
            device: "...".to_string(),
            start: begin,
            end,
            check_state: Duration::minutes(10),
            entries: vec![
                Entry {
                    when: When::Date(begin.date_naive()),
                    time: NaiveTime::default(),
                    state: Default::default(),
                },
                Entry {
                    when: When::Daily,
                    time: NaiveTime::parse_from_str("12:42", "%H:%M").unwrap(),
                    state: State::On,
                },
            ],
        };
        let result = serde_yaml::to_string(&config).unwrap();
        println!("{result}");
        let expected = "device: '...'
start: 2022-11-28T00:00:00+01:00
end: 2022-12-02T00:00:00+01:00
check_state: 10mins 0secs
entries:
- when: !date 2022-11-28
  time: 00:00:00
  state: off
- when: daily
  time: 12:42:00
  state: on
";
        assert_eq!(expected, result);
    }

    #[test]
    fn create_intervals() {
        let config = "device: '...'
start: 2022-11-28T00:00:00+01:00
end: 2022-12-01T23:59:59+01:00
check_state: 10mins 0secs
entries:
- when: !date 2022-11-30
  time: 16:00:00
  state: on
- when: !date 2022-12-01
  time: 14:00:00
  state: on
- when: daily
  time: 13:00:00
  state: on
- when: daily
  time: 14:00:00
  state: off
- when: daily
  time: 18:00:00
  state: on
- when: daily
  time: 22:00:00
  state: off
";

        let config = Config::from_string(config).expect("read config");
        let result = config
            .intervals()
            .iter()
            .map(|ea| ea.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        let expected = "2022-11-28 00:00:00 +01:00-2022-11-28 13:00:00 +01:00=off
2022-11-28 13:00:00 +01:00-2022-11-28 14:00:00 +01:00=on
2022-11-28 14:00:00 +01:00-2022-11-28 18:00:00 +01:00=off
2022-11-28 18:00:00 +01:00-2022-11-28 22:00:00 +01:00=on
2022-11-28 22:00:00 +01:00-2022-11-29 13:00:00 +01:00=off
2022-11-29 13:00:00 +01:00-2022-11-29 14:00:00 +01:00=on
2022-11-29 14:00:00 +01:00-2022-11-29 18:00:00 +01:00=off
2022-11-29 18:00:00 +01:00-2022-11-29 22:00:00 +01:00=on
2022-11-29 22:00:00 +01:00-2022-11-30 13:00:00 +01:00=off
2022-11-30 13:00:00 +01:00-2022-11-30 14:00:00 +01:00=on
2022-11-30 14:00:00 +01:00-2022-11-30 16:00:00 +01:00=off
2022-11-30 16:00:00 +01:00-2022-11-30 22:00:00 +01:00=on";
        assert_eq!(expected, result);
    }
}
