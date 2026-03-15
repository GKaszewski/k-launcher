use std::sync::Arc;

use k_launcher_kernel::Kernel;
use k_launcher_os_bridge::UnixAppLauncher;
use plugin_apps::{AppsPlugin, frecency::FrecencyStore};
#[cfg(target_os = "linux")]
use plugin_apps::linux::FsDesktopEntrySource;
use plugin_calc::CalcPlugin;
use plugin_cmd::CmdPlugin;
use plugin_files::FilesPlugin;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let launcher = Arc::new(UnixAppLauncher::new());
    let frecency = FrecencyStore::load();
    let kernel: Arc<dyn k_launcher_kernel::SearchEngine> = Arc::new(Kernel::new(vec![
        Arc::new(CmdPlugin::new()),
        Arc::new(CalcPlugin::new()),
        Arc::new(FilesPlugin::new()),
        Arc::new(AppsPlugin::new(FsDesktopEntrySource::new(), frecency)),
    ], 8));
    k_launcher_ui_egui::run(kernel, launcher)?;
    Ok(())
}
