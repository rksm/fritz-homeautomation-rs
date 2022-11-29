#[macro_use]
extern crate tracing;

use anyhow::Result;
use clap::Parser;
use fritz_christmas_light_controller::{Config, Timer};
use notify::{RecursiveMode, Watcher};
use std::path::PathBuf;
use tracing_subscriber::prelude::*;

#[derive(Parser)]
struct Args {
    #[clap(short, long, action)]
    verbose: bool,
    #[clap(short, long, action, help = "yaml config file")]
    config: PathBuf,
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
    let mut config = Config::from_yaml_file(&args.config)?;
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

    watcher.watch(&args.config, RecursiveMode::NonRecursive)?;

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
                info!("need to update");
            }
            ConfigFileChanged => {
                info!("config changed!");
                match Config::from_yaml_file(&args.config) {
                    Ok(c) => {
                        config = c;
                        timer = Timer::with_regular_update(config.check_state);
                        timer.set_intervals(&config.intervals());
                    }
                    Err(err) => {
                        error!("Cannot read config file: {err}");
                        warn!("will continue to use old config!");
                    }
                }
            }
            Error(err) => return Err(err),
        };
    }
}
