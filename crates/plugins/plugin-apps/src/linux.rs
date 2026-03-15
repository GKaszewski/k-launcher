use std::path::Path;

use crate::{AppName, DesktopEntry, DesktopEntrySource, ExecCommand, IconPath};
use crate::humanize_category;

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

pub fn resolve_icon_path(name: &str) -> Option<String> {
    if name.starts_with('/') && Path::new(name).exists() {
        return Some(name.to_string());
    }
    let candidates = [
        format!("/usr/share/pixmaps/{name}.png"),
        format!("/usr/share/pixmaps/{name}.svg"),
        format!("/usr/share/icons/hicolor/48x48/apps/{name}.png"),
        format!("/usr/share/icons/hicolor/scalable/apps/{name}.svg"),
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
                    category = value.trim()
                        .split(';')
                        .find(|s| !s.is_empty())
                        .map(|s| humanize_category(s.trim()));
                }
                "Keywords" if keywords.is_empty() => {
                    keywords = value.trim()
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

    let exec_clean: String = exec?
        .split_whitespace()
        .filter(|s| !s.starts_with('%'))
        .fold(String::new(), |mut acc, s| {
            if !acc.is_empty() {
                acc.push(' ');
            }
            acc.push_str(s);
            acc
        });

    Some(DesktopEntry {
        name: AppName::new(name?),
        exec: ExecCommand::new(exec_clean),
        icon: icon.map(IconPath::new),
        category,
        keywords,
    })
}
