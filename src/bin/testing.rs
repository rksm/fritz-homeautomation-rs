use std::env;

use dotenv::dotenv;
use fritz_homeautomation::schedule;
use fritz_homeautomation::{api, error::Result};

fn main() -> Result<()> {
    dotenv().ok();

    let user = env::var("FRITZ_USER").expect("Need FRITZ_USER env var");
    let password = env::var("FRITZ_PASSWORD").expect("Need FRITZ_PASSWORD env var");

    let sid = api::get_sid(&user, &password)?;

    let devices: Vec<_> = api::device_infos_avm(&sid)?;
    println!("{:#?}", devices);

    let ain = "11657 0272633";
    let file = "/Users/robert/projects/rust/fritz-homeautomation/data/schedule.txt";

    schedule::start_processing_schedule(file, move |action, time| {
        println!(
            "running action {:?} at {}",
            action,
            time.format("%Y-%m-%d %H:%M:%S %Z")
        );

        use schedule::Action::*;
        let result = match action {
            TurnOn => Some(api::turn_on(&sid, ain)),
            TurnOff => Some(api::turn_off(&sid, ain)),
            Unknown => None,
        };
        if let Some(Err(err)) = result {
            eprintln!("action {:?} errored: {}", action, err);
        }
    })?;

    Ok(())
}
