pub mod frecency;
#[cfg(target_os = "linux")]
pub mod linux;

use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use k_launcher_kernel::{LaunchAction, Plugin, PluginName, ResultId, ResultTitle, Score, SearchResult};

use crate::frecency::FrecencyStore;

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
    pub category: Option<String>,
    pub keywords: Vec<String>,
}

// --- Swappable source trait (Application layer principle) ---

pub trait DesktopEntrySource: Send + Sync {
    fn entries(&self) -> Vec<DesktopEntry>;
}

// --- Cached entry (pre-computed at construction) ---

struct CachedEntry {
    id: String,
    name: AppName,
    name_lc: String,
    keywords_lc: Vec<String>,
    category: Option<String>,
    icon: Option<String>,
    exec: String,
    on_select: Arc<dyn Fn() + Send + Sync>,
}

// --- Plugin ---

pub struct AppsPlugin {
    entries: HashMap<String, CachedEntry>,
    frecency: Arc<FrecencyStore>,
}

impl AppsPlugin {
    pub fn new(source: impl DesktopEntrySource, frecency: Arc<FrecencyStore>) -> Self {
        let entries = source
            .entries()
            .into_iter()
            .map(|e| {
                let id = format!("app-{}", e.name.as_str());
                let name_lc = e.name.as_str().to_lowercase();
                let keywords_lc = e.keywords.iter().map(|k| k.to_lowercase()).collect();
                #[cfg(target_os = "linux")]
                let icon = e.icon.as_ref().and_then(|p| linux::resolve_icon_path(p.as_str()));
                #[cfg(not(target_os = "linux"))]
                let icon: Option<String> = None;
                let exec = e.exec.as_str().to_string();
                let store = Arc::clone(&frecency);
                let record_id = id.clone();
                let on_select: Arc<dyn Fn() + Send + Sync> = Arc::new(move || {
                    store.record(&record_id);
                });
                let cached = CachedEntry {
                    id: id.clone(),
                    name_lc,
                    keywords_lc,
                    category: e.category,
                    icon,
                    exec,
                    on_select,
                    name: e.name,
                };
                (id, cached)
            })
            .collect();
        Self { entries, frecency }
    }
}

fn initials(name_lc: &str) -> String {
    name_lc.split_whitespace().filter_map(|w| w.chars().next()).collect()
}

fn score_match(name_lc: &str, query_lc: &str) -> Option<u32> {
    if name_lc == query_lc           { return Some(100); }
    if name_lc.starts_with(query_lc) { return Some(80);  }
    if name_lc.contains(query_lc)    { return Some(60);  }
    if initials(name_lc).starts_with(query_lc) { return Some(70); }
    None
}

pub(crate) fn humanize_category(s: &str) -> String {
    let mut result = String::new();
    for ch in s.chars() {
        if ch.is_uppercase() && !result.is_empty() {
            result.push(' ');
        }
        result.push(ch);
    }
    result
}

#[async_trait]
impl Plugin for AppsPlugin {
    fn name(&self) -> PluginName {
        "apps"
    }

    async fn search(&self, query: &str) -> Vec<SearchResult> {
        if query.is_empty() {
            return self.frecency.top_ids(5)
                .iter()
                .filter_map(|id| {
                    let e = self.entries.get(id)?;
                    let score = self.frecency.frecency_score(id).max(1);
                    Some(SearchResult {
                        id: ResultId::new(id),
                        title: ResultTitle::new(e.name.as_str()),
                        description: e.category.clone(),
                        icon: e.icon.clone(),
                        score: Score::new(score),
                        action: LaunchAction::SpawnProcess(e.exec.clone()),
                        on_select: Some(Arc::clone(&e.on_select)),
                    })
                })
                .collect();
        }

        let query_lc = query.to_lowercase();
        self.entries
            .values()
            .filter_map(|e| {
                let score = score_match(&e.name_lc, &query_lc).or_else(|| {
                    e.keywords_lc.iter().any(|k| k.contains(&query_lc)).then_some(50)
                })?;
                Some(SearchResult {
                    id: ResultId::new(&e.id),
                    title: ResultTitle::new(e.name.as_str()),
                    description: e.category.clone(),
                    icon: e.icon.clone(),
                    score: Score::new(score),
                    action: LaunchAction::SpawnProcess(e.exec.clone()),
                    on_select: Some(Arc::clone(&e.on_select)),
                })
            })
            .collect()
    }
}

// --- Tests ---

#[cfg(test)]
mod tests {
    use super::*;

    fn ephemeral_frecency() -> Arc<FrecencyStore> {
        FrecencyStore::new_for_test()
    }

    struct MockSource {
        entries: Vec<(String, String, Option<String>, Vec<String>)>, // (name, exec, category, keywords)
    }

    impl MockSource {
        fn with(entries: Vec<(&str, &str)>) -> Self {
            Self {
                entries: entries
                    .into_iter()
                    .map(|(n, e)| (n.to_string(), e.to_string(), None, vec![]))
                    .collect(),
            }
        }

        fn with_categories(entries: Vec<(&str, &str, &str)>) -> Self {
            Self {
                entries: entries
                    .into_iter()
                    .map(|(n, e, c)| (n.to_string(), e.to_string(), Some(c.to_string()), vec![]))
                    .collect(),
            }
        }

        fn with_keywords(entries: Vec<(&str, &str, Vec<&str>)>) -> Self {
            Self {
                entries: entries
                    .into_iter()
                    .map(|(n, e, kw)| (n.to_string(), e.to_string(), None, kw.into_iter().map(|s| s.to_string()).collect()))
                    .collect(),
            }
        }
    }

    impl DesktopEntrySource for MockSource {
        fn entries(&self) -> Vec<DesktopEntry> {
            self.entries
                .iter()
                .map(|(name, exec, category, keywords)| DesktopEntry {
                    name: AppName::new(name.clone()),
                    exec: ExecCommand::new(exec.clone()),
                    icon: None,
                    category: category.clone(),
                    keywords: keywords.clone(),
                })
                .collect()
        }
    }

    #[tokio::test]
    async fn apps_prefix_match() {
        let p = AppsPlugin::new(MockSource::with(vec![("Firefox", "firefox")]), ephemeral_frecency());
        let results = p.search("fire").await;
        assert_eq!(results[0].title.as_str(), "Firefox");
    }

    #[tokio::test]
    async fn apps_no_match_returns_empty() {
        let p = AppsPlugin::new(MockSource::with(vec![("Firefox", "firefox")]), ephemeral_frecency());
        assert!(p.search("zz").await.is_empty());
    }

    #[tokio::test]
    async fn apps_empty_query_no_frecency_returns_empty() {
        let p = AppsPlugin::new(MockSource::with(vec![("Firefox", "firefox")]), ephemeral_frecency());
        assert!(p.search("").await.is_empty());
    }

    #[test]
    fn score_match_abbreviation() {
        assert_eq!(initials("visual studio code"), "vsc");
        assert_eq!(score_match("visual studio code", "vsc"), Some(70));
    }

    #[test]
    fn score_match_exact_beats_prefix_beats_abbrev_beats_substr() {
        assert_eq!(score_match("firefox", "firefox"), Some(100));
        assert_eq!(score_match("firefox", "fire"), Some(80));
        assert_eq!(score_match("gnu firefox", "gf"), Some(70));
        assert_eq!(score_match("ice firefox", "fire"), Some(60));
    }

    #[tokio::test]
    async fn apps_abbreviation_match() {
        let p = AppsPlugin::new(
            MockSource::with(vec![("Visual Studio Code", "code")]),
            ephemeral_frecency(),
        );
        let results = p.search("vsc").await;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title.as_str(), "Visual Studio Code");
        assert_eq!(results[0].score.value(), 70);
    }

    #[tokio::test]
    async fn apps_keyword_match() {
        let p = AppsPlugin::new(
            MockSource::with_keywords(vec![("Code", "code", vec!["editor", "ide"])]),
            ephemeral_frecency(),
        );
        let results = p.search("editor").await;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].score.value(), 50);
    }

    #[test]
    fn humanize_category_splits_camel_case() {
        assert_eq!(humanize_category("TextEditor"), "Text Editor");
        assert_eq!(humanize_category("WebBrowser"), "Web Browser");
        assert_eq!(humanize_category("Development"), "Development");
    }

    #[tokio::test]
    async fn apps_category_appears_in_description() {
        let p = AppsPlugin::new(
            MockSource::with_categories(vec![("Code", "code", "Text Editor")]),
            ephemeral_frecency(),
        );
        let results = p.search("code").await;
        assert_eq!(results[0].description.as_deref(), Some("Text Editor"));
    }

    #[tokio::test]
    async fn apps_empty_query_returns_top_frecent() {
        let frecency = ephemeral_frecency();
        frecency.record("app-Code");
        frecency.record("app-Code");
        frecency.record("app-Firefox");
        let p = AppsPlugin::new(
            MockSource::with(vec![("Firefox", "firefox"), ("Code", "code")]),
            frecency,
        );
        let results = p.search("").await;
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].title.as_str(), "Code");
    }
}
