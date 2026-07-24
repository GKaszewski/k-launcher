use serde::Deserialize;

use crate::types::*;

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct Config {
    pub window: WindowCfg,
    pub appearance: AppearanceCfg,
    pub search: SearchCfg,
    pub plugins: PluginsCfg,
    pub logging: LoggingCfg,
    pub terminal: TerminalCfg,
}
