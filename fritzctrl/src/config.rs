use directories::UserDirs;
use serde::Deserialize;

/// A configuration that is constructed from ~/.fritzctrl[.toml|.yaml|.json] and
/// environment vars FRITZ_USER and FRITZ_PASSWORD.
#[derive(Debug, Deserialize)]
pub struct EnvConfig {
    pub user: Option<String>,
    pub password: Option<String>,
}

impl EnvConfig {
    pub fn new() -> Result<Self, config::ConfigError> {
        let mut s = config::Config::new();

        UserDirs::new().and_then(|dirs| {
            dirs.home_dir()
                .join(".fritzctrl")
                .to_str()
                .map(|path| s.merge(config::File::with_name(path).required(false)).ok())
        });

        s.merge(config::Environment::with_prefix("fritz"))?;

        s.try_into()
    }
}
