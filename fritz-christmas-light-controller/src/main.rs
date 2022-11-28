#[macro_use]
extern crate tracing;

use std::{path::Path, str::FromStr};

use anyhow::Result;
use chrono::{prelude::*, Duration};
use clap::Parser;
use fritz_christmas_light_controller::{Config, Entry, Interval, State, StateChange, When};
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
        Skip,
    }
    use Action::*;

    loop {
        dbg!(&config);

        let action = flume::Selector::new()
            .recv(&config_change_rx, |recv| match recv {
                Err(err) => {
                    tracing::error!("config file watcher closed: {err}");
                    Skip
                }
                Ok(_) => ConfigFileChanged,
            })
            .wait_timeout(config.check_state.to_std().unwrap())
            .unwrap_or(Tick);

        match action {
            Tick => {
                tracing::info!("need to update");
            }
            ConfigFileChanged => {
                tracing::info!("config changed!");
                match Config::from_yaml_file("config.yaml") {
                    Ok(c) => {
                        config = c;
                    }
                    Err(err) => {
                        tracing::error!("Cannot read config file: {err}");
                        tracing::warn!("will continue to use old config!");
                    }
                }
            }
            Skip => {}
        };
    }

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.

    Ok(())
}
