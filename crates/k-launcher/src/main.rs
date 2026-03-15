use std::sync::Arc;

use k_launcher_kernel::Kernel;
use plugin_apps::{AppsPlugin, FsDesktopEntrySource};
use plugin_calc::CalcPlugin;

fn main() -> iced::Result {
    let kernel = Arc::new(Kernel::new(vec![
        Arc::new(CalcPlugin::new()),
        Arc::new(AppsPlugin::new(FsDesktopEntrySource::new())),
    ]));
    k_launcher_ui::run(kernel)
}
