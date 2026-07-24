mod app;
mod style;
mod update;
mod view;

use std::sync::Arc;

use k_launcher_config::{AppearanceCfg, SearchCfg, WindowCfg};
use k_launcher_domain::AppLauncher;
use k_launcher_kernel::Kernel;

pub fn run(
    engine_factory: Arc<dyn Fn() -> Arc<Kernel> + Send + Sync>,
    launcher: Arc<dyn AppLauncher>,
    window_cfg: &WindowCfg,
    appearance_cfg: AppearanceCfg,
    search_cfg: &SearchCfg,
) -> Result<(), String> {
    app::run(
        engine_factory,
        launcher,
        window_cfg,
        appearance_cfg,
        search_cfg.debounce_ms,
    )
    .map_err(|e| e.to_string())
}
