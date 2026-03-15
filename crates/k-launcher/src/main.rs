use std::sync::Arc;

use k_launcher_kernel::Kernel;
use k_launcher_os_bridge::UnixAppLauncher;
use plugin_apps::{AppsPlugin, frecency::FrecencyStore};
#[cfg(target_os = "linux")]
use plugin_apps::linux::FsDesktopEntrySource;
use plugin_calc::CalcPlugin;
use plugin_cmd::CmdPlugin;
use plugin_files::FilesPlugin;

fn main() -> iced::Result {
    let cfg = k_launcher_config::load();
    let launcher = Arc::new(UnixAppLauncher::new());
    let frecency = FrecencyStore::load();

    let mut plugins: Vec<Arc<dyn k_launcher_kernel::Plugin>> = vec![];
    if cfg.plugins.cmd   { plugins.push(Arc::new(CmdPlugin::new())); }
    if cfg.plugins.calc  { plugins.push(Arc::new(CalcPlugin::new())); }
    if cfg.plugins.files { plugins.push(Arc::new(FilesPlugin::new())); }
    if cfg.plugins.apps  {
        plugins.push(Arc::new(AppsPlugin::new(FsDesktopEntrySource::new(), frecency)));
    }

    let kernel: Arc<dyn k_launcher_kernel::SearchEngine> =
        Arc::new(Kernel::new(plugins, cfg.search.max_results));

    k_launcher_ui::run(kernel, launcher, &cfg.window, cfg.appearance)
}
