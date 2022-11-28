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
    let user = args.get_one::<String>("user").unwrap();
    let password = args.get_one::<String>("password").unwrap();
    let ain = args.get_one::<String>("device").unwrap();
    let toggle = args.get_flag("toggle");
    let on = args.get_flag("on");
    let off = args.get_flag("off");

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

#[tracing::instrument(level = "trace", skip(password))]
pub fn run(user: &str, password: &str, ain: &str, action: SwitchAction) -> anyhow::Result<()> {
    let mut client = fritzapi::FritzClient::new(user, password);
    let devices: Vec<_> = client.list_devices()?;

    let device = match devices.into_iter().find(|dev| dev.id() == ain) {
        None => {
            return Err(anyhow::anyhow!("Cannot find device with ain {:?}", ain));
        }
        Some(device) => device,
    };

    match action {
        SwitchAction::On => client.turn_on(device.id())?,
        SwitchAction::Off => client.turn_off(device.id())?,
        SwitchAction::Toggle => client.toggle(device.id())?,
    };

    Ok(())
}
