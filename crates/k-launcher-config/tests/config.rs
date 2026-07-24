use k_launcher_config::Config;

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
    assert_eq!(
        cfg.appearance.placeholder,
        "Search apps, type > for commands, = for math"
    );
}

#[test]
fn parse_partial_toml_uses_defaults() {
    let toml_str = "[search]\nmax_results = 5\n";
    let cfg: Config = toml::from_str(toml_str).unwrap();
    assert_eq!(cfg.search.max_results, 5);
    assert_eq!(cfg.window.width, 600.0);
    assert_eq!(cfg.appearance.search_font_size, 18.0);
    assert!(cfg.plugins.apps);
}

#[test]
fn parse_full_toml_roundtrip() {
    let toml_str = r#"
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
    let cfg: Config = toml::from_str(toml_str).unwrap();
    assert_eq!(cfg.window.width, 800.0);
    assert_eq!(cfg.window.height, 500.0);
    assert!(cfg.window.decorations);
    assert!(!cfg.window.transparent);
    assert_eq!(
        cfg.appearance.background_rgba,
        k_launcher_config::Rgba::new(10.0, 10.0, 20.0, 0.8)
    );
    assert_eq!(cfg.appearance.search_font_size, 20.0);
    assert_eq!(cfg.appearance.placeholder, "Type here...");
    assert_eq!(cfg.search.max_results, 12);
    assert!(!cfg.plugins.calc);
    assert!(!cfg.plugins.files);
}
