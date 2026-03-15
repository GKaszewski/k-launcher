mod app;

use std::sync::Arc;

use k_launcher_kernel::{AppLauncher, SearchEngine};

pub fn run(
    engine: Arc<dyn SearchEngine>,
    launcher: Arc<dyn AppLauncher>,
) -> Result<(), eframe::Error> {
    app::run(engine, launcher)
}
