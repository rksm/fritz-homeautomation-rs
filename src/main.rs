use dotenv::dotenv;
use fritz::api::*;
use fritz::error::Result;
use std::env;
use std::thread;
use std::time::Duration;

fn main() -> Result<()> {
    dotenv().ok();

    let user = env::var("FRITZ_USER").expect("Need FRITZ_USER env var");
    let password = env::var("FRITZ_PASSWORD").expect("Need FRITZ_PASSWORD env var");

    let sid = get_sid(&user, &password)?;

    let devices: Vec<_> = device_infos_avm(&sid)?;
    println!("{:#?}", devices);

    turn_on(&sid, "11657 0272633").expect("turn on");

    thread::sleep(Duration::from_secs(5));

    turn_off(&sid, "11657 0272633").expect("turn off");

    // if let [AVMDevice::FritzDect2XX(dev @ FritzDect2XX { .. }), ..] = &devices[..] {
    //     println!("{:#?}", dev);
    //     fetch_device_stats(&sid, &dev.identifier).unwrap();
    // }

    Ok(())
}
