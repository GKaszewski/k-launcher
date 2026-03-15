use std::{path::Path, process::{Command, Stdio}, sync::Arc};
use std::os::unix::process::CommandExt;

use async_trait::async_trait;
use k_launcher_kernel::{Plugin, PluginName, ResultId, ResultTitle, Score, SearchResult};

// --- Domain newtypes ---

#[derive(Debug, Clone)]
pub struct AppName(String);

impl AppName {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct ExecCommand(String);

impl ExecCommand {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct IconPath(String);

impl IconPath {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// --- Desktop entry ---

pub struct DesktopEntry {
    pub name: AppName,
    pub exec: ExecCommand,
    pub icon: Option<IconPath>,
}

// --- Swappable source trait (Application layer principle) ---

pub trait DesktopEntrySource: Send + Sync {
    fn entries(&self) -> Vec<DesktopEntry>;
}

// --- Cached entry (pre-computed at construction) ---

struct CachedEntry {
    name: AppName,
    name_lc: String,
    icon: Option<String>,
    on_execute: Arc<dyn Fn() + Send + Sync>,
}

// --- Plugin ---

pub struct AppsPlugin {
    entries: Vec<CachedEntry>,
}

impl AppsPlugin {
    pub fn new(source: impl DesktopEntrySource) -> Self {
        let entries = source
            .entries()
            .into_iter()
            .map(|e| {
                let name_lc = e.name.as_str().to_lowercase();
                let icon = e.icon.as_ref().and_then(|p| resolve_icon_path(p.as_str()));
                let exec = e.exec.clone();
                CachedEntry {
                    name_lc,
                    icon,
                    on_execute: Arc::new(move || {
                        let parts: Vec<&str> = exec.as_str().split_whitespace().collect();
                        if let Some((cmd, args)) = parts.split_first() {
                            let _ = unsafe {
                                Command::new(cmd)
                                    .args(args)
                                    .stdin(Stdio::null())
                                    .stdout(Stdio::null())
                                    .stderr(Stdio::null())
                                    .pre_exec(|| {
                                        libc::setsid();
                                        Ok(())
                                    })
                                    .spawn()
                            };
                        }
                    }),
                    name: e.name,
                }
            })
            .collect();
        Self { entries }
    }
}

fn resolve_icon_path(name: &str) -> Option<String> {
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

fn score_match(name_lc: &str, query_lc: &str) -> Option<u32> {
    if name_lc == query_lc {
        Some(100)
    } else if name_lc.starts_with(query_lc) {
        Some(80)
    } else if name_lc.contains(query_lc) {
        Some(60)
    } else {
        None
    }
}

#[async_trait]
impl Plugin for AppsPlugin {
    fn name(&self) -> PluginName {
        "apps"
    }

    async fn search(&self, query: &str) -> Vec<SearchResult> {
        if query.is_empty() {
            return vec![];
        }
        let query_lc = query.to_lowercase();
        self.entries
            .iter()
            .filter_map(|e| {
                score_match(&e.name_lc, &query_lc).map(|score| SearchResult {
                    id: ResultId::new(format!("app-{}", e.name.as_str())),
                    title: ResultTitle::new(e.name.as_str()),
                    description: None,
                    icon: e.icon.clone(),
                    score: Score::new(score),
                    on_execute: Arc::clone(&e.on_execute),
                })
            })
            .collect()
    }
}

// --- Filesystem source ---

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
        if let Ok(xdg) = xdg::BaseDirectories::new() {
            dirs.push(xdg.get_data_home().join("applications"));
            for d in xdg.get_data_dirs() {
                dirs.push(d.join("applications"));
            }
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

fn parse_desktop_file(path: &Path) -> Option<DesktopEntry> {
    let content = std::fs::read_to_string(path).ok()?;
    let mut in_section = false;
    let mut name: Option<String> = None;
    let mut exec: Option<String> = None;
    let mut icon: Option<String> = None;
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
    })
}

// --- Tests ---

#[cfg(test)]
mod tests {
    use super::*;

    struct MockSource {
        entries: Vec<(String, String)>, // (name, exec)
    }

    impl MockSource {
        fn with(entries: Vec<(&str, &str)>) -> Self {
            Self {
                entries: entries
                    .into_iter()
                    .map(|(n, e)| (n.to_string(), e.to_string()))
                    .collect(),
            }
        }
    }

    impl DesktopEntrySource for MockSource {
        fn entries(&self) -> Vec<DesktopEntry> {
            self.entries
                .iter()
                .map(|(name, exec)| DesktopEntry {
                    name: AppName::new(name.clone()),
                    exec: ExecCommand::new(exec.clone()),
                    icon: None,
                })
                .collect()
        }
    }

    #[tokio::test]
    async fn apps_prefix_match() {
        let source = MockSource::with(vec![("Firefox", "firefox")]);
        let p = AppsPlugin::new(source);
        let results = p.search("fire").await;
        assert_eq!(results[0].title.as_str(), "Firefox");
    }

    #[tokio::test]
    async fn apps_no_match_returns_empty() {
        let source = MockSource::with(vec![("Firefox", "firefox")]);
        let p = AppsPlugin::new(source);
        assert!(p.search("zz").await.is_empty());
    }

    #[tokio::test]
    async fn apps_empty_query_returns_empty() {
        let source = MockSource::with(vec![("Firefox", "firefox")]);
        let p = AppsPlugin::new(source);
        assert!(p.search("").await.is_empty());
    }
}
