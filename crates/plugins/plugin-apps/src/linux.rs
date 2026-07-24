use std::path::Path;

use crate::scoring::humanize_category;
use crate::{AppName, DesktopEntry, DesktopEntrySource, ExecCommand, IconPath};

pub struct FsDesktopEntrySource;

impl FsDesktopEntrySource {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FsDesktopEntrySource {
    fn default() -> Self {
        Self::new()
    }
}

impl DesktopEntrySource for FsDesktopEntrySource {
    fn entries(&self) -> Vec<DesktopEntry> {
        let mut dirs = Vec::new();
        let xdg = xdg::BaseDirectories::new();
        if let Some(data_home) = xdg.get_data_home() {
            dirs.push(data_home.join("applications"));
        }
        for d in xdg.get_data_dirs() {
            dirs.push(d.join("applications"));
        }
        let mut entries = Vec::new();
        for dir in &dirs {
            if let Ok(read_dir) = std::fs::read_dir(dir) {
                for entry in read_dir.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|e| e.to_str()) != Some("desktop") {
                        continue;
                    }
                    if let Some(de) = parse_desktop_file(&path) {
                        entries.push(de);
                    }
                }
            }
        }
        entries
    }
}

pub fn clean_exec(exec: &str) -> String {
    // Tokenize respecting double-quoted strings, then filter field codes.
    let mut tokens: Vec<String> = Vec::new();
    let mut chars = exec.chars().peekable();

    while let Some(&ch) = chars.peek() {
        if ch.is_whitespace() {
            chars.next();
            continue;
        }
        if ch == '"' {
            // Consume opening quote
            chars.next();
            let mut token = String::from('"');
            while let Some(&c) = chars.peek() {
                chars.next();
                if c == '"' {
                    token.push('"');
                    break;
                }
                token.push(c);
            }
            // Strip embedded field codes like %f inside the quoted string
            // (between the quotes, before re-assembling)
            let inner = &token[1..token.len().saturating_sub(1)];
            let cleaned_inner: String = inner
                .split_whitespace()
                .filter(|s| !is_field_code(s))
                .collect::<Vec<_>>()
                .join(" ");
            tokens.push(format!("\"{cleaned_inner}\""));
        } else {
            let mut token = String::new();
            while let Some(&c) = chars.peek() {
                if c.is_whitespace() {
                    break;
                }
                chars.next();
                token.push(c);
            }
            if !is_field_code(&token) {
                tokens.push(token);
            }
        }
    }

    tokens.join(" ")
}

fn is_field_code(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 2 && b[0] == b'%' && b[1].is_ascii_alphabetic()
}

const ICON_LOOKUP_SIZE: u16 = 48;
const ICON_THEMES: &[&str] = &["hicolor", "Adwaita", "breeze", "Papirus"];
const PIXMAPS_DIR: &str = "/usr/share/pixmaps";

pub fn resolve_icon_path(name: &str) -> Option<String> {
    if name.starts_with('/') && Path::new(name).exists() {
        return Some(name.to_string());
    }
    for theme in ICON_THEMES {
        if let Some(icon_path) = linicon::lookup_icon(name)
            .from_theme(theme)
            .with_size(ICON_LOOKUP_SIZE)
            .find_map(|r| r.ok())
        {
            return Some(icon_path.path.to_string_lossy().into_owned());
        }
    }
    // Fallback to pixmaps
    let candidates = [
        format!("{PIXMAPS_DIR}/{name}.png"),
        format!("{PIXMAPS_DIR}/{name}.svg"),
    ];
    candidates.into_iter().find(|p| Path::new(p).exists())
}

fn parse_desktop_file(path: &Path) -> Option<DesktopEntry> {
    let content = std::fs::read_to_string(path).ok()?;
    let mut in_section = false;
    let mut name: Option<String> = None;
    let mut exec: Option<String> = None;
    let mut icon: Option<String> = None;
    let mut category: Option<String> = None;
    let mut keywords: Vec<String> = Vec::new();
    let mut is_application = false;
    let mut no_display = false;

    for line in content.lines() {
        let line = line.trim();
        if line == "[Desktop Entry]" {
            in_section = true;
            continue;
        }
        if line.starts_with('[') {
            in_section = false;
            continue;
        }
        if !in_section || line.starts_with('#') || line.is_empty() {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            match key.trim() {
                "Name" if name.is_none() => name = Some(value.trim().to_string()),
                "Exec" if exec.is_none() => exec = Some(value.trim().to_string()),
                "Icon" if icon.is_none() => icon = Some(value.trim().to_string()),
                "Type" if !is_application => is_application = value.trim() == "Application",
                "NoDisplay" => no_display = value.trim().eq_ignore_ascii_case("true"),
                "Categories" if category.is_none() => {
                    category = value
                        .trim()
                        .split(';')
                        .find(|s| !s.is_empty())
                        .map(|s| humanize_category(s.trim()));
                }
                "Keywords" if keywords.is_empty() => {
                    keywords = value
                        .trim()
                        .split(';')
                        .filter(|s| !s.is_empty())
                        .map(|s| s.trim().to_string())
                        .collect();
                }
                _ => {}
            }
        }
    }

    if !is_application || no_display {
        return None;
    }

    let exec_clean: String = clean_exec(&exec?);

    Some(DesktopEntry {
        name: AppName::new(name?),
        exec: ExecCommand::new(exec_clean),
        icon: icon.map(IconPath::new),
        category,
        keywords,
    })
}
