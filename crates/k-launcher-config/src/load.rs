use k_launcher_domain::constants::{APP_NAME, CONFIG_FILENAME};

use crate::config::Config;
use crate::error::ConfigError;

pub fn load() -> Config {
    match try_load() {
        Ok(cfg) => cfg,
        Err(ConfigError::NoDirFound | ConfigError::ReadFailed { .. }) => Config::default(),
        Err(e @ ConfigError::ParseFailed { .. }) => {
            tracing::warn!("{e}");
            Config::default()
        }
    }
}

pub fn try_load() -> Result<Config, ConfigError> {
    let dir = dirs::config_dir().ok_or(ConfigError::NoDirFound)?;
    let path = dir.join(APP_NAME).join(CONFIG_FILENAME);
    let content = std::fs::read_to_string(&path).map_err(|e| ConfigError::ReadFailed {
        path: path.clone(),
        source: e,
    })?;
    toml::from_str(&content).map_err(|e| ConfigError::ParseFailed { path, source: e })
}
