use k_launcher_domain::constants::{APP_NAME, LOG_DIR_NAME, LOG_FILE_PREFIX};

const FALLBACK_LOG_DIR: &str = "/tmp/k-launcher/logs";
const DEFAULT_LOG_LEVEL: &str = "info";

pub(crate) fn init_logging(
    cfg: &k_launcher_config::Config,
) -> tracing_appender::non_blocking::WorkerGuard {
    use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

    let log_dir = dirs::data_local_dir()
        .map(|d| d.join(APP_NAME).join(LOG_DIR_NAME))
        .unwrap_or_else(|| std::path::PathBuf::from(FALLBACK_LOG_DIR));
    std::fs::create_dir_all(&log_dir).ok();

    let file_appender = tracing_appender::rolling::RollingFileAppender::builder()
        .rotation(tracing_appender::rolling::Rotation::DAILY)
        .filename_prefix(LOG_FILE_PREFIX)
        .max_log_files(cfg.logging.max_log_files)
        .build(&log_dir)
        .expect("log appender");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(DEFAULT_LOG_LEVEL));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
        .with(tracing_subscriber::fmt::layer().with_writer(non_blocking))
        .init();

    guard
}

pub(crate) fn install_panic_hook() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let location = info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "unknown".to_string());
        let payload = if let Some(s) = info.payload().downcast_ref::<&str>() {
            (*s).to_string()
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "unknown panic".to_string()
        };
        tracing::error!("PANIC at {location}: {payload}");
        default_hook(info);
    }));
}
