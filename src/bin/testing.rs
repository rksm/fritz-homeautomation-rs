use std::env;

use anyhow::Result;
use dotenv::dotenv;
use fritz_homeautomation::api;
use fritz_homeautomation::schedule;

fn main() -> Result<()> {
    dotenv().ok();

    let user = env::var("FRITZ_USER").expect("Need FRITZ_USER env var");
    let password = env::var("FRITZ_PASSWORD").expect("Need FRITZ_PASSWORD env var");

    let sid = api::get_sid(&user, &password)?;

    let devices: Vec<_> = api::device_infos_avm(&sid)?;
    println!("{:#?}", devices);

    let ain = "11657 0272633";
    let file = "/Users/robert/projects/rust/fritz-homeautomation/data/schedule.txt";

    let dev = devices.into_iter().find_map(|dev| match dev {
        api::AVMDevice::FritzDect2XX(dev) if dev.identifier == ain => Some(dev),
        _ => None,
    });

    println!("{:#?}", dev);

    Ok(())
}
