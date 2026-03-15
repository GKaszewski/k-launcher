use serde::Deserialize;

// RGBA: [r, g, b, a] where r/g/b are 0–255 as f32, a is 0.0–1.0
pub type Rgba = [f32; 4];

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct Config {
    pub window: WindowCfg,
    pub appearance: AppearanceCfg,
    pub search: SearchCfg,
    pub plugins: PluginsCfg,
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
}

impl Default for AppearanceCfg {
    fn default() -> Self {
        Self {
            background_rgba: [20.0, 20.0, 30.0, 0.9],
            border_rgba: [229.0, 125.0, 33.0, 1.0],
            border_width: 1.0,
            border_radius: 8.0,
            search_font_size: 18.0,
            title_size: 15.0,
            desc_size: 12.0,
            row_radius: 4.0,
            placeholder: "Search...".to_string(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct SearchCfg {
    pub max_results: usize,
}

impl Default for SearchCfg {
    fn default() -> Self {
        Self { max_results: 8 }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct PluginsCfg {
    pub calc: bool,
    pub cmd: bool,
    pub files: bool,
    pub apps: bool,
}

impl Default for PluginsCfg {
    fn default() -> Self {
        Self {
            calc: true,
            cmd: true,
            files: true,
            apps: true,
        }
    }
}

pub fn load() -> Config {
    let path = dirs::config_dir()
        .map(|d| d.join("k-launcher").join("config.toml"));
    let Some(path) = path else {
        return Config::default();
    };
    let Ok(content) = std::fs::read_to_string(&path) else {
        return Config::default();
    };
    toml::from_str(&content).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_sane_values() {
        let cfg = Config::default();
        assert_eq!(cfg.search.max_results, 8);
        assert_eq!(cfg.window.width, 600.0);
        assert_eq!(cfg.window.height, 400.0);
        assert!(!cfg.window.decorations);
        assert!(cfg.window.transparent);
        assert!(!cfg.window.resizable);
        assert!(cfg.plugins.calc);
        assert!(cfg.plugins.apps);
        assert_eq!(cfg.appearance.search_font_size, 18.0);
        assert_eq!(cfg.appearance.placeholder, "Search...");
    }

    #[test]
    fn parse_partial_toml_uses_defaults() {
        let toml = "[search]\nmax_results = 5\n";
        let cfg: Config = toml::from_str(toml).unwrap();
        assert_eq!(cfg.search.max_results, 5);
        assert_eq!(cfg.window.width, 600.0);
        assert_eq!(cfg.appearance.search_font_size, 18.0);
        assert!(cfg.plugins.apps);
    }

    #[test]
    fn parse_full_toml_roundtrip() {
        let toml = r#"
[window]
width = 800.0
height = 500.0
decorations = true
transparent = false
resizable = true

[appearance]
background_rgba = [10.0, 10.0, 20.0, 0.8]
border_rgba = [100.0, 200.0, 255.0, 1.0]
border_width = 2.0
border_radius = 12.0
search_font_size = 20.0
title_size = 16.0
desc_size = 13.0
row_radius = 6.0
placeholder = "Type here..."

[search]
max_results = 12

[plugins]
calc = false
cmd = true
files = false
apps = true
"#;
        let cfg: Config = toml::from_str(toml).unwrap();
        assert_eq!(cfg.window.width, 800.0);
        assert_eq!(cfg.window.height, 500.0);
        assert!(cfg.window.decorations);
        assert!(!cfg.window.transparent);
        assert_eq!(cfg.appearance.background_rgba, [10.0, 10.0, 20.0, 0.8]);
        assert_eq!(cfg.appearance.search_font_size, 20.0);
        assert_eq!(cfg.appearance.placeholder, "Type here...");
        assert_eq!(cfg.search.max_results, 12);
        assert!(!cfg.plugins.calc);
        assert!(!cfg.plugins.files);
    }
}
