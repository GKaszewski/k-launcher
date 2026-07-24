use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rgba {
    red: f32,
    green: f32,
    blue: f32,
    alpha: f32,
}

impl<'de> serde::Deserialize<'de> for Rgba {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let [red, green, blue, alpha] = <[f32; 4]>::deserialize(deserializer)?;
        Ok(Self::new(red, green, blue, alpha))
    }
}

impl Rgba {
    pub fn new(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        Self {
            red: red.clamp(0.0, 255.0),
            green: green.clamp(0.0, 255.0),
            blue: blue.clamp(0.0, 255.0),
            alpha: alpha.clamp(0.0, 1.0),
        }
    }

    pub fn red(&self) -> f32 {
        self.red
    }

    pub fn green(&self) -> f32 {
        self.green
    }

    pub fn blue(&self) -> f32 {
        self.blue
    }

    pub fn alpha(&self) -> f32 {
        self.alpha
    }

    pub fn red_u8(&self) -> u8 {
        self.red as u8
    }

    pub fn green_u8(&self) -> u8 {
        self.green as u8
    }

    pub fn blue_u8(&self) -> u8 {
        self.blue as u8
    }

    pub fn alpha_byte(&self) -> u8 {
        (self.alpha * 255.0) as u8
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct WindowCfg {
    pub width: f32,
    pub height: f32,
    pub decorations: bool,
    pub transparent: bool,
    pub resizable: bool,
}

impl Default for WindowCfg {
    fn default() -> Self {
        Self {
            width: 600.0,
            height: 400.0,
            decorations: false,
            transparent: true,
            resizable: false,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct AppearanceCfg {
    pub background_rgba: Rgba,
    pub border_rgba: Rgba,
    pub border_width: f32,
    pub border_radius: f32,
    pub search_font_size: f32,
    pub title_size: f32,
    pub desc_size: f32,
    pub row_radius: f32,
    pub placeholder: String,
    pub selected_row_rgba: Rgba,
    pub selected_text_rgba: Rgba,
    pub selected_description_rgba: Rgba,
    pub unselected_row_rgba: Rgba,
    pub text_rgba: Rgba,
    pub description_rgba: Rgba,
    pub no_results_rgba: Rgba,
    pub error_rgba: Rgba,
    pub icon_size: f32,
}

impl Default for AppearanceCfg {
    fn default() -> Self {
        Self {
            background_rgba: Rgba::new(20.0, 20.0, 30.0, 0.9),
            border_rgba: Rgba::new(229.0, 125.0, 33.0, 1.0),
            border_width: 1.0,
            border_radius: 8.0,
            search_font_size: 18.0,
            title_size: 15.0,
            desc_size: 12.0,
            row_radius: 4.0,
            placeholder: "Search apps, type > for commands, = for math".to_string(),
            selected_row_rgba: Rgba::new(229.0, 125.0, 33.0, 1.0),
            selected_text_rgba: Rgba::new(255.0, 255.0, 255.0, 1.0),
            selected_description_rgba: Rgba::new(240.0, 240.0, 240.0, 0.9),
            unselected_row_rgba: Rgba::new(255.0, 255.0, 255.0, 0.07),
            text_rgba: Rgba::new(255.0, 255.0, 255.0, 1.0),
            description_rgba: Rgba::new(210.0, 215.0, 230.0, 1.0),
            no_results_rgba: Rgba::new(180.0, 180.0, 200.0, 0.5),
            error_rgba: Rgba::new(255.0, 80.0, 80.0, 1.0),
            icon_size: 24.0,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct SearchCfg {
    pub max_results: usize,
    pub debounce_ms: u64,
    pub frecency_compact_threshold: usize,
}

impl Default for SearchCfg {
    fn default() -> Self {
        Self {
            max_results: 8,
            debounce_ms: 50,
            frecency_compact_threshold: 50,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct LoggingCfg {
    pub max_log_files: usize,
}

impl Default for LoggingCfg {
    fn default() -> Self {
        Self { max_log_files: 7 }
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct TerminalCfg {
    pub cmd: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ExternalPluginCfg {
    pub name: String,
    pub path: String,
    pub args: Vec<String>,
    pub timeout_secs: u64,
}

impl Default for ExternalPluginCfg {
    fn default() -> Self {
        Self {
            name: String::new(),
            path: String::new(),
            args: vec![],
            timeout_secs: 5,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct PluginsCfg {
    pub calc: bool,
    pub cmd: bool,
    pub files: bool,
    pub apps: bool,
    pub external: Vec<ExternalPluginCfg>,
}

impl Default for PluginsCfg {
    fn default() -> Self {
        Self {
            calc: true,
            cmd: true,
            files: true,
            apps: true,
            external: vec![],
        }
    }
}
