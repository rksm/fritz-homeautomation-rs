use dotenv::dotenv;
use fritz_homeautomation::api::*;
use fritz_homeautomation::error::Result;
use std::env;

fn main() -> Result<()> {
    dotenv().ok();

    let user = env::var("FRITZ_USER").expect("Need FRITZ_USER env var");
    let password = env::var("FRITZ_PASSWORD").expect("Need FRITZ_PASSWORD env var");

    let sid = get_sid(&user, &password)?;
    let devices: Vec<_> = device_infos_avm(&sid)?;

    println!("found {} devices", devices.len());

    if let [AVMDevice::FritzDect2XX(dev @ FritzDect2XX { .. }), ..] = &devices[..] {
        let stats = fetch_device_stats(&sid, &dev.identifier)?;
        println!("{:#?}", stats);
    }

    Ok(())
}
