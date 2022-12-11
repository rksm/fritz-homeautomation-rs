use crate::{error::Result, State};

pub trait FritzUpdate {
    fn set_state(&self, desired_state: State, id: impl AsRef<str>) -> Result<bool>;
}

pub struct RealtFritzUpdater {
    user: String,
    password: String,
}

impl RealtFritzUpdater {
    pub fn new(user: impl ToString, password: impl ToString) -> Self {
        Self {
            user: user.to_string(),
            password: password.to_string(),
        }
    }
}

impl FritzUpdate for RealtFritzUpdater {
    fn set_state(&self, desired_state: State, id: impl AsRef<str>) -> Result<bool> {
        let id = id.as_ref();
        let Self { user, password } = self;
        let mut client = fritzapi::FritzClient::new(user, password);

        let device = client.list_devices()?.into_iter().find(|d| d.id() == id);

        let device = if let Some(device) = device {
            device
        } else {
            warn!("did not find device {id}, skipping update");
            return Ok(false);
        };

        let changed = match (device.is_on(), desired_state) {
            (true, State::On) => {
                debug!("device {id} is already on");
                false
            }
            (false, State::Off) => {
                debug!("device {id} is already off");
                false
            }
            (true, State::Off) => {
                debug!("turning device off");
                true
            }
            (false, State::On) => {
                debug!("turning device on");
                true
            }
        };

        if changed {
            client.toggle(id)?;
        }

        Ok(changed)
    }
}
