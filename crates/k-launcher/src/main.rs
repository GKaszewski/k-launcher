use std::sync::Arc;

use k_launcher_kernel::Kernel;
use plugin_apps::{AppsPlugin, FsDesktopEntrySource, frecency::FrecencyStore};
use plugin_calc::CalcPlugin;
use plugin_cmd::CmdPlugin;
use plugin_files::FilesPlugin;

fn main() -> iced::Result {
    let frecency = FrecencyStore::load();
    let kernel = Arc::new(Kernel::new(vec![
        Arc::new(CmdPlugin::new()),
        Arc::new(CalcPlugin::new()),
        Arc::new(FilesPlugin::new()),
        Arc::new(AppsPlugin::new(FsDesktopEntrySource::new(), frecency)),
    ]));
    k_launcher_ui::run(kernel)
}
