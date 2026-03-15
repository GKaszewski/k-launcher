mod unix_launcher;

pub use unix_launcher::UnixAppLauncher;

pub struct WindowConfig {
    pub width: f32,
    pub height: f32,
    pub decorations: bool,
    pub transparent: bool,
    pub resizable: bool,
}

impl WindowConfig {
    pub fn from_cfg(w: &k_launcher_config::WindowCfg) -> Self {
        Self {
            width: w.width,
            height: w.height,
            decorations: w.decorations,
            transparent: w.transparent,
            resizable: w.resizable,
        }
    }
}
