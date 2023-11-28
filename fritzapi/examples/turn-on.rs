//! Shows how to list and modify devices.

fn main() -> fritzapi::Result<()> {
    let user = "";
    let password = "";
    let mut client = fritzapi::FritzClient::new(user, password);

    // List devices
    let mut devices = client.list_devices()?;

    // If the first device is of, turn it on
    let dev = devices.first_mut().unwrap();
    if !dev.is_on() {
        dev.turn_on(&mut client)?;
    }
    Ok(())
}
