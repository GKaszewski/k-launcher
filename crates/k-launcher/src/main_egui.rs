use std::sync::Arc;

use k_launcher_os_bridge::UnixAppLauncher;

mod engine;
mod error;
mod logging;

use error::AppError;

fn main() {
    if std::env::args().any(|a| a == "--version" || a == "-V") {
        println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        return;
    }

    let cfg = Arc::new(k_launcher_config::load());
    let _guard = logging::init_logging(&cfg);
    logging::install_panic_hook();

    ctrlc::set_handler(|| {
        tracing::info!("received shutdown signal");
        std::process::exit(0);
    })
    .ok();

    if let Err(e) = run_ui(cfg) {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn run_ui(cfg: Arc<k_launcher_config::Config>) -> Result<(), AppError> {
    let launcher = Arc::new(UnixAppLauncher::new(cfg.terminal.cmd.clone()));
    let kernel = engine::build_engine(cfg.clone());
    k_launcher_ui_egui::run(kernel, launcher, &cfg.window, cfg.appearance.clone())
        .map_err(AppError::Ui)
}
