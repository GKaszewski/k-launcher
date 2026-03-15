/// Configuration for the launcher window.
pub struct WindowConfig {
    pub width: f32,
    pub height: f32,
    pub decorations: bool,
    pub transparent: bool,
    pub resizable: bool,
}

impl WindowConfig {
    pub fn launcher() -> Self {
        Self {
            width: 600.0,
            height: 400.0,
            decorations: false,
            transparent: true,
            resizable: false,
        }
    }
}
