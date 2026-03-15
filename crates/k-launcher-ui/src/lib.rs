mod app;
pub mod theme;

use std::sync::Arc;

use k_launcher_kernel::Kernel;


pub fn run(kernel: Arc<Kernel>) -> iced::Result {
    app::run(kernel)
}
