mod app;
mod input;
mod render;
mod style;

use std::sync::Arc;

use k_launcher_config::AppearanceCfg;
use k_launcher_domain::AppLauncher;
use k_launcher_kernel::Kernel;

pub fn run(
    engine: Arc<Kernel>,
    launcher: Arc<dyn AppLauncher>,
    window_cfg: &k_launcher_config::WindowCfg,
    appearance_cfg: AppearanceCfg,
) -> Result<(), String> {
    app::run(engine, launcher, window_cfg, appearance_cfg).map_err(|e| e.to_string())
}
