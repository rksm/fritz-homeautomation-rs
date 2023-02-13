use chrono::prelude::*;
use lazy_static::lazy_static;
use regex::Regex;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    TurnOn { time: DateTime<Local>, id: String },
    TurnOff { time: DateTime<Local>, id: String },
}

impl Action {
    pub fn time(&self) -> DateTime<Local> {
        match self {
            Self::TurnOn { time, .. } => *time,
            Self::TurnOff { time, .. } => *time,
        }
    }

    pub fn device_id(&self) -> &str {
        match self {
            Self::TurnOn { id, .. } => id,
            Self::TurnOff { id, .. } => id,
        }
    }
}

impl std::str::FromStr for Action {
    type Err = anyhow::Error;

    fn from_str(line: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref RE: Regex = regex::RegexBuilder::new(
                r"([0-9]{4}-[0-9]{2}-[0-9]{2} [0-9]{2}:[0-9]{2}:[0-9]{2}) (.+) (on|off)"
            )
            .case_insensitive(true)
            .build()
            .unwrap();
        }

        let err = anyhow::anyhow!("does not match schedule action format");
        match RE.captures(line) {
            None => Err(err),
            Some(captures) => {
                let ts = captures.get(1).unwrap().as_str();
                let id = captures
                    .get(2)
                    .unwrap()
                    .as_str()
                    .trim_matches('"')
                    .to_string();
                let action = captures.get(3).unwrap().as_str().to_lowercase();

                match NaiveDateTime::parse_from_str(ts, "%Y-%m-%d %H:%M:%S")
                    .ok()
                    .and_then(|time| Local.from_local_datetime(&time).earliest())
                {
                    Some(time) if action == "on" => Ok(Action::TurnOn { time, id }),
                    Some(time) if action == "off" => Ok(Action::TurnOff { time, id }),
                    _ => Err(err),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn parse_actions() {
        let time =
            NaiveDateTime::parse_from_str("2021-01-31 01:02:03", "%Y-%m-%d %H:%M:%S").unwrap();
        let time = chrono::Local.from_local_datetime(&time).unwrap();
        assert_eq!("hello".parse::<Action>().ok(), None);
        assert_eq!(
            "2021-01-31 01:02:03 aaabbb on".parse::<Action>().unwrap(),
            Action::TurnOn {
                time,
                id: "aaabbb".to_string()
            }
        );
        assert_eq!(
            "2021-01-31 01:02:03 \"123 456\" on"
                .parse::<Action>()
                .unwrap(),
            Action::TurnOn {
                time,
                id: "123 456".to_string()
            }
        );
        assert_eq!(
            "2021-01-31 01:02:03 123 456 off".parse::<Action>().unwrap(),
            Action::TurnOff {
                time,
                id: "123 456".to_string()
            }
        );
    }
}
