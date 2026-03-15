mod app;
pub mod theme;

use std::sync::Arc;

use k_launcher_config::{AppearanceCfg, WindowCfg};
use k_launcher_kernel::{AppLauncher, SearchEngine};

pub fn run(
    engine: Arc<dyn SearchEngine>,
    launcher: Arc<dyn AppLauncher>,
    window_cfg: &WindowCfg,
    appearance_cfg: AppearanceCfg,
) -> iced::Result {
    app::run(engine, launcher, window_cfg, appearance_cfg)
}
