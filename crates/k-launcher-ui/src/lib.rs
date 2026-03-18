mod app;

use std::sync::Arc;

use k_launcher_config::{AppearanceCfg, WindowCfg};
use k_launcher_kernel::{AppLauncher, SearchEngine};

pub fn run(
    engine_factory: Arc<dyn Fn() -> Arc<dyn SearchEngine> + Send + Sync>,
    launcher: Arc<dyn AppLauncher>,
    window_cfg: &WindowCfg,
    appearance_cfg: AppearanceCfg,
) -> iced::Result {
    app::run(engine_factory, launcher, window_cfg, appearance_cfg)
}
