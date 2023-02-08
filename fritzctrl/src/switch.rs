use crate::schedule::Action;
use clap::ArgMatches;

#[derive(Debug, Clone, Copy)]
pub enum SwitchAction {
    On,
    Off,
    Toggle,
}

impl From<Action> for SwitchAction {
    fn from(action: Action) -> Self {
        match action {
            Action::TurnOn { .. } => SwitchAction::On,
            Action::TurnOff { .. } => SwitchAction::Off,
        }
    }
}

pub fn switch(args: &ArgMatches) -> anyhow::Result<()> {
    let user = args.value_of("user").unwrap();
    let password = args.value_of("password").unwrap();
    let ain = args.value_of("device").unwrap();
    let toggle = args.is_present("toggle");
    let on = args.is_present("on");
    let off = args.is_present("off");

    let action = if on {
        SwitchAction::On
    } else if off {
        SwitchAction::Off
    } else if toggle {
        SwitchAction::Toggle
    } else {
        return Err(anyhow::anyhow!("invalid switch options"));
    };

    run(user, password, ain, action)
}

pub fn run(user: &str, password: &str, ain: &str, action: SwitchAction) -> anyhow::Result<()> {
    let sid = fritzapi::get_sid(user, password)?;
    let devices: Vec<_> = fritzapi::list_devices(&sid)?;

    let mut device = match devices.into_iter().find(|dev| dev.id() == ain) {
        None => {
            return Err(anyhow::anyhow!("Cannot find device with ain {:?}", ain));
        }
        Some(device) => device,
    };

    match action {
        SwitchAction::On => device.turn_on(&sid)?,
        SwitchAction::Off => device.turn_off(&sid)?,
        SwitchAction::Toggle => device.toggle(&sid)?,
    };

    Ok(())
}
