use std::sync::Arc;

use k_launcher_kernel::Kernel;
use k_launcher_plugin_host::ExternalPlugin;
#[cfg(target_os = "linux")]
use plugin_apps::linux::FsDesktopEntrySource;
use plugin_apps::{AppsPlugin, frecency::FrecencyStore};
use plugin_calc::CalcPlugin;
use plugin_cmd::CmdPlugin;
use plugin_files::FilesPlugin;

pub(crate) fn build_engine(cfg: Arc<k_launcher_config::Config>) -> Arc<Kernel> {
    let frecency = FrecencyStore::load(cfg.search.frecency_compact_threshold);
    let mut plugins: Vec<Arc<dyn k_launcher_domain::Plugin>> = vec![];
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
            ext.timeout_secs,
        )));
    }
    Arc::new(Kernel::new(plugins, cfg.search.max_results))
}
