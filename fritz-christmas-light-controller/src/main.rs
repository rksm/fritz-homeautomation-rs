#[macro_use]
extern crate tracing;

use std::str::FromStr;

use anyhow::Result;
use chrono::{prelude::*, Duration};
use clap::Parser;
use fritz_christmas_light_controller::{Entry, Interval, State, StateChange, When};
use serde::{Deserialize, Serialize};
use tracing_subscriber::prelude::*;

#[derive(Parser)]
struct Args {
    #[clap(short, long, action)]
    verbose: bool,
}

fn main() {
    dotenv::dotenv().ok();

    let args = Args::parse();

    let log_level = if args.verbose {
        "info,fritz=trace,reqwest=debug"
    } else {
        "info"
    };

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::builder().parse_lossy(log_level))
        .with(tracing_forest::ForestLayer::default())
        .init();

    run(args).unwrap();
}

#[derive(Debug, Serialize)]
struct Config {
    start: DateTime<Local>,
    end: DateTime<Local>,
    #[serde(with = "fritz_christmas_light_controller::duration")]
    check_state: Duration,
    entries: Vec<Entry>,
}

fn run(args: Args) -> Result<()> {
    let begin = NaiveDateTime::from_str("2022-11-28T00:00:00")
        .unwrap()
        .and_local_timezone(Local)
        .unwrap();
    let end = begin + Duration::days(4);
    tracing::info!(%begin, %end);

    if false {
        // let begin = DateTime::parse_from_rfc3339("2022-11-28T00:00:00+01:00")?;
        // let end = DateTime::parse_from_rfc3339("2022-12-06T00:00:00+01:00")?;

        let intervals = chrono_intervals::IntervalGenerator::new()
            .with_grouping(chrono_intervals::Grouping::PerDay)
            .without_extended_begin()
            .get_intervals(begin, end);
        dbg!(intervals);
    }

    if true {
        let config = Config {
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
        tracing::info!("{}", serde_yaml::to_string(&config)?);
    }

    if true {
        let entries: Vec<Entry> = serde_yaml::from_str(
            r#"
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
"#,
        )?;

        let state_changes = StateChange::from_entries_between(&entries, begin, end);

        let intervals = state_changes
            .iter()
            .zip(state_changes.iter().skip(1))
            .map(|(a, b)| Interval {
                start: a.when,
                end: b.when,
                state: a.state,
            })
            .collect::<Vec<_>>();

        dbg!(&intervals);

        let now = intervals.into_iter().find(|ea| ea.is_current());

        dbg!(now);

        // let mut intervals = Vec::new();

        // let mut now = begin;
        // let mut day = now.date_naive();
        // let mut state = State::default();

        // while now <= end {
        //     let x = daily.iter().find(|ea| {
        //         let t = day.and_time(ea.time).and_local_timezone(Local).unwrap();
        //         t == now
        //     });

        //     if let Some(x) = x {
        //         // tracing::info!("found {x:?}");
        //         if x.state != state {
        //             let start = intervals.last().map(|(_, end, _)| *end).unwrap_or(begin);
        //             intervals.push((start, now, state));
        //             state = x.state;
        //         }
        //     }
        //     // let on = now.date().and_time(daily.on);
        //     // let off = now.date().and_time(daily.off);

        //     now += Duration::minutes(1);

        //     if now.date_naive() > day {
        //         day = now.date_naive();
        //     }
        // }

        // dbg!(intervals);
    }

    Ok(())
}
