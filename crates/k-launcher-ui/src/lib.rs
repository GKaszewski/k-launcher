mod app;
pub mod theme;

use std::sync::Arc;

use k_launcher_kernel::{AppLauncher, SearchEngine};

pub fn run(engine: Arc<dyn SearchEngine>, launcher: Arc<dyn AppLauncher>) -> iced::Result {
    app::run(engine, launcher)
}
