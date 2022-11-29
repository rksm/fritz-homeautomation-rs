#[macro_use]
extern crate tracing;

use std::{path::Path, str::FromStr};

use anyhow::Result;
use chrono::{prelude::*, Duration, DurationRound};
use clap::Parser;
use fritz_christmas_light_controller::{Config, Entry, Interval, State, StateChange, Timer, When};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
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

fn run(args: Args) -> Result<()> {
    let mut config = Config::from_yaml_file("config.yaml")?;
    let mut timer = Timer::with_regular_update(config.check_state);
    timer.set_intervals(&config.intervals());

    let (config_change_tx, config_change_rx) = flume::bounded(0);
    let mut watcher = notify::recommended_watcher(move |res| match res {
        Ok(event) => {
            println!("event: {:?}", event);
            let _ = config_change_tx.send(());
        }
        Err(e) => println!("watch error: {:?}", e),
    })?;

    watcher.watch(Path::new("config.yaml"), RecursiveMode::NonRecursive)?;

    enum Action {
        ConfigFileChanged,
        Tick,
        Error(anyhow::Error),
    }
    use Action::*;

    loop {
        let action = flume::Selector::new()
            .recv(&config_change_rx, |recv| match recv {
                Err(err) => Error(anyhow::anyhow!("config file watcher closed: {err}")),
                Ok(_) => ConfigFileChanged,
            })
            .recv(&timer.timer_rx(), |msg| match msg {
                Err(err) => Error(anyhow::anyhow!("timer channel closed: {err}")),
                Ok(_) => Tick,
            })
            .wait();

        match action {
            Tick => {
                tracing::info!("need to update");
            }
            ConfigFileChanged => {
                tracing::info!("config changed!");
                match Config::from_yaml_file("config.yaml") {
                    Ok(c) => {
                        config = c;
                        timer = Timer::with_regular_update(config.check_state);
                        timer.set_intervals(&config.intervals());
                    }
                    Err(err) => {
                        tracing::error!("Cannot read config file: {err}");
                        tracing::warn!("will continue to use old config!");
                    }
                }
            }
            Error(err) => return Err(err),
        };
    }
}
