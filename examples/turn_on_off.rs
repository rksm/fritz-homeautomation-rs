use dotenv::dotenv;
use fritz_homeautomation::api::*;
use fritz_homeautomation::error::Result;
use std::env;
use std::thread;
use std::time::Duration;

fn main() -> Result<()> {
    dotenv().ok();

    let user = env::var("FRITZ_USER").expect("Need FRITZ_USER env var");
    let password = env::var("FRITZ_PASSWORD").expect("Need FRITZ_PASSWORD env var");
    let sid = get_sid(&user, &password)?;

    turn_on(&sid, "11657 0272633").expect("turn on");
    thread::sleep(Duration::from_secs(5));
    turn_off(&sid, "11657 0272633").expect("turn off");

    Ok(())
}
