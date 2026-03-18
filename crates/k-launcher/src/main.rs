use std::sync::Arc;

use k_launcher_kernel::Kernel;
use k_launcher_os_bridge::UnixAppLauncher;
use k_launcher_plugin_host::ExternalPlugin;
#[cfg(target_os = "linux")]
use plugin_apps::linux::FsDesktopEntrySource;
use plugin_apps::{AppsPlugin, frecency::FrecencyStore};
use plugin_calc::CalcPlugin;
use plugin_cmd::CmdPlugin;
use plugin_files::FilesPlugin;

fn init_logging() -> tracing_appender::non_blocking::WorkerGuard {
    use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

    let log_dir = dirs::data_local_dir()
        .map(|d| d.join("k-launcher/logs"))
        .unwrap_or_else(|| std::path::PathBuf::from("/tmp/k-launcher/logs"));
    std::fs::create_dir_all(&log_dir).ok();

    let file_appender = tracing_appender::rolling::daily(&log_dir, "k-launcher.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
        .with(tracing_subscriber::fmt::layer().with_writer(non_blocking))
        .init();

    guard
}

fn main() {
    let _guard = init_logging();

    if let Err(e) = run_ui() {
        eprintln!("error: UI: {e}");
        std::process::exit(1);
    }
}

fn build_engine(cfg: Arc<k_launcher_config::Config>) -> Arc<dyn k_launcher_kernel::SearchEngine> {
    let frecency = FrecencyStore::load();
    let mut plugins: Vec<Arc<dyn k_launcher_kernel::Plugin>> = vec![];
    if cfg.plugins.cmd {
        plugins.push(Arc::new(CmdPlugin::new()));
    }
    if cfg.plugins.calc {
        plugins.push(Arc::new(CalcPlugin::new()));
    }
    if cfg.plugins.files {
        plugins.push(Arc::new(FilesPlugin::new()));
    }
    if cfg.plugins.apps {
        plugins.push(Arc::new(AppsPlugin::new(
            FsDesktopEntrySource::new(),
            frecency,
        )));
    }
    for ext in &cfg.plugins.external {
        plugins.push(Arc::new(ExternalPlugin::new(
            &ext.name,
            &ext.path,
            ext.args.clone(),
        )));
    }
    Arc::new(Kernel::new(plugins, cfg.search.max_results))
}

fn run_ui() -> iced::Result {
    let cfg = Arc::new(k_launcher_config::load());
    let launcher = Arc::new(UnixAppLauncher::new());
    let factory_cfg = cfg.clone();
    let factory: Arc<dyn Fn() -> Arc<dyn k_launcher_kernel::SearchEngine> + Send + Sync> =
        Arc::new(move || build_engine(factory_cfg.clone()));
    k_launcher_ui::run(factory, launcher, &cfg.window, cfg.appearance.clone())
}
