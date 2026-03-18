pub mod frecency;
#[cfg(target_os = "linux")]
pub mod linux;

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

use async_trait::async_trait;
use k_launcher_kernel::{LaunchAction, Plugin, ResultId, ResultTitle, Score, SearchResult};

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
    keywords_lc: Vec<String>,
    category: Option<String>,
    icon: Option<String>,
    exec: String,
}

// --- Serializable cache data (no closures) ---

#[derive(serde::Serialize, serde::Deserialize)]
struct CachedEntryData {
    id: String,
    name: String,
    keywords_lc: Vec<String>,
    category: Option<String>,
    icon: Option<String>,
    exec: String,
}

fn cache_path() -> Option<PathBuf> {
    dirs::cache_dir().map(|d| d.join("k-launcher/apps.bin"))
}

fn load_from_path(path: &Path) -> Option<HashMap<String, CachedEntry>> {
    let data = std::fs::read(path).ok()?;
    let (entries_data, _): (Vec<CachedEntryData>, _) =
        bincode::serde::decode_from_slice(&data, bincode::config::standard()).ok()?;
    let map = entries_data
        .into_iter()
        .map(|e| {
            let cached = CachedEntry {
                id: e.id.clone(),
                name: AppName::new(e.name),
                keywords_lc: e.keywords_lc,
                category: e.category,
                icon: e.icon,
                exec: e.exec,
            };
            (e.id, cached)
        })
        .collect();
    Some(map)
}

fn save_to_path(path: &Path, entries: &HashMap<String, CachedEntry>) {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).ok();
    }
    let data: Vec<CachedEntryData> = entries
        .values()
        .map(|e| CachedEntryData {
            id: e.id.clone(),
            name: e.name.as_str().to_string(),
            keywords_lc: e.keywords_lc.clone(),
            category: e.category.clone(),
            icon: e.icon.clone(),
            exec: e.exec.clone(),
        })
        .collect();
    if let Ok(encoded) = bincode::serde::encode_to_vec(&data, bincode::config::standard()) {
        std::fs::write(path, encoded).ok();
    }
}

fn build_entries(source: &impl DesktopEntrySource, frecency: &Arc<FrecencyStore>) -> HashMap<String, CachedEntry> {
    source
        .entries()
        .into_iter()
        .map(|e| {
            let id = format!("app-{}:{}", e.name.as_str(), e.exec.as_str());
            let keywords_lc = e.keywords.iter().map(|k| k.to_lowercase()).collect();
            #[cfg(target_os = "linux")]
            let icon = e
                .icon
                .as_ref()
                .and_then(|p| linux::resolve_icon_path(p.as_str()));
            #[cfg(not(target_os = "linux"))]
            let icon: Option<String> = None;
            let exec = e.exec.as_str().to_string();
            let cached = CachedEntry {
                id: id.clone(),
                keywords_lc,
                category: e.category,
                icon,
                exec,
                name: e.name,
            };
            (id, cached)
        })
        .collect()
}

// --- Plugin ---

pub struct AppsPlugin {
    entries: Arc<RwLock<HashMap<String, CachedEntry>>>,
    frecency: Arc<FrecencyStore>,
}

impl AppsPlugin {
    pub fn new(source: impl DesktopEntrySource + 'static, frecency: Arc<FrecencyStore>) -> Self {
        Self::new_impl(source, frecency, cache_path())
    }

    fn new_impl(
        source: impl DesktopEntrySource + 'static,
        frecency: Arc<FrecencyStore>,
        cp: Option<PathBuf>,
    ) -> Self {
        let cached = cp.as_deref().and_then(load_from_path);

        let entries = if let Some(from_cache) = cached {
            // Serve cache immediately; refresh in background.
            let map = Arc::new(RwLock::new(from_cache));
            let entries_bg = Arc::clone(&map);
            let frecency_bg = Arc::clone(&frecency);
            let cp_bg = cp.clone();
            std::thread::spawn(move || {
                let fresh = build_entries(&source, &frecency_bg);
                if let Some(path) = cp_bg {
                    save_to_path(&path, &fresh);
                }
                *entries_bg.write().unwrap() = fresh;
            });
            map
        } else {
            // No cache: build synchronously, then persist.
            let initial = build_entries(&source, &frecency);
            if let Some(path) = &cp {
                save_to_path(path, &initial);
            }
            Arc::new(RwLock::new(initial))
        };

        Self { entries, frecency }
    }

    #[cfg(test)]
    fn new_for_test(source: impl DesktopEntrySource + 'static, frecency: Arc<FrecencyStore>) -> Self {
        Self::new_impl(source, frecency, None)
    }
}

fn initials(name_lc: &str) -> String {
    name_lc
        .split_whitespace()
        .filter_map(|w| w.chars().next())
        .collect()
}

fn score_match(name: &str, query: &str) -> Option<u32> {
    use nucleo_matcher::{
        Config, Matcher, Utf32Str,
        pattern::{CaseMatching, Normalization, Pattern},
    };

    let mut matcher = Matcher::new(Config::DEFAULT);
    let pattern = Pattern::parse(query, CaseMatching::Ignore, Normalization::Smart);

    let mut name_chars: Vec<char> = name.chars().collect();
    let haystack = Utf32Str::new(name, &mut name_chars);
    let score = pattern.score(haystack, &mut matcher);

    if let Some(s) = score {
        let name_lc = name.to_lowercase();
        let query_lc = query.to_lowercase();
        let bonus: u32 = if initials(&name_lc).starts_with(&query_lc) {
            20
        } else {
            0
        };
        Some(s.saturating_add(bonus))
    } else {
        None
    }
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
    fn name(&self) -> &str {
        "apps"
    }

    fn on_selected(&self, id: &ResultId) {
        self.frecency.record(id.as_str());
    }

    async fn search(&self, query: &str) -> Vec<SearchResult> {
        let entries = self.entries.read().unwrap();
        if query.is_empty() {
            return self
                .frecency
                .top_ids(5)
                .iter()
                .filter_map(|id| {
                    let e = entries.get(id)?;
                    let score = self.frecency.frecency_score(id).max(1);
                    Some(SearchResult {
                        id: ResultId::new(id),
                        title: ResultTitle::new(e.name.as_str()),
                        description: e.category.clone(),
                        icon: e.icon.clone(),
                        score: Score::new(score),
                        action: LaunchAction::SpawnProcess(e.exec.clone()),
                    })
                })
                .collect();
        }

        let query_lc = query.to_lowercase();
        entries
            .values()
            .filter_map(|e| {
                let score = score_match(e.name.as_str(), query).or_else(|| {
                    e.keywords_lc
                        .iter()
                        .any(|k| k.contains(&query_lc))
                        .then_some(50)
                })?;
                Some(SearchResult {
                    id: ResultId::new(&e.id),
                    title: ResultTitle::new(e.name.as_str()),
                    description: e.category.clone(),
                    icon: e.icon.clone(),
                    score: Score::new(score),
                    action: LaunchAction::SpawnProcess(e.exec.clone()),
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
                    .map(|(n, e, kw)| {
                        (
                            n.to_string(),
                            e.to_string(),
                            None,
                            kw.into_iter().map(|s| s.to_string()).collect(),
                        )
                    })
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
        let p = AppsPlugin::new_for_test(
            MockSource::with(vec![("Firefox", "firefox")]),
            ephemeral_frecency(),
        );
        let results = p.search("fire").await;
        assert_eq!(results[0].title.as_str(), "Firefox");
    }

    #[tokio::test]
    async fn apps_no_match_returns_empty() {
        let p = AppsPlugin::new_for_test(
            MockSource::with(vec![("Firefox", "firefox")]),
            ephemeral_frecency(),
        );
        assert!(p.search("zz").await.is_empty());
    }

    #[tokio::test]
    async fn apps_empty_query_no_frecency_returns_empty() {
        let p = AppsPlugin::new_for_test(
            MockSource::with(vec![("Firefox", "firefox")]),
            ephemeral_frecency(),
        );
        assert!(p.search("").await.is_empty());
    }

    #[test]
    fn score_match_abbreviation() {
        assert_eq!(initials("visual studio code"), "vsc");
        assert!(score_match("visual studio code", "vsc").is_some());
    }

    #[test]
    fn score_match_exact_beats_prefix() {
        let exact = score_match("firefox", "firefox");
        let prefix = score_match("firefox", "fire");
        let abbrev = score_match("gnu firefox", "gf");
        let substr = score_match("ice firefox", "fire");
        assert!(exact.is_some());
        assert!(prefix.is_some());
        assert!(abbrev.is_some());
        assert!(substr.is_some());
        assert!(exact.unwrap() > prefix.unwrap());
    }

    #[tokio::test]
    async fn apps_abbreviation_match() {
        let p = AppsPlugin::new_for_test(
            MockSource::with(vec![("Visual Studio Code", "code")]),
            ephemeral_frecency(),
        );
        let results = p.search("vsc").await;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title.as_str(), "Visual Studio Code");
        assert!(results[0].score.value() > 0);
    }

    #[tokio::test]
    async fn apps_keyword_match() {
        let p = AppsPlugin::new_for_test(
            MockSource::with_keywords(vec![("Code", "code", vec!["editor", "ide"])]),
            ephemeral_frecency(),
        );
        let results = p.search("editor").await;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].score.value(), 50);
    }

    #[tokio::test]
    async fn apps_fuzzy_typo_match() {
        let p = AppsPlugin::new_for_test(
            MockSource::with(vec![("Firefox", "firefox")]),
            ephemeral_frecency(),
        );
        let results = p.search("frefox").await;
        assert!(
            !results.is_empty(),
            "nucleo should fuzzy-match 'frefox' to 'Firefox'"
        );
        assert!(results[0].score.value() > 0);
    }

    #[test]
    fn humanize_category_splits_camel_case() {
        assert_eq!(humanize_category("TextEditor"), "Text Editor");
        assert_eq!(humanize_category("WebBrowser"), "Web Browser");
        assert_eq!(humanize_category("Development"), "Development");
    }

    #[tokio::test]
    async fn apps_category_appears_in_description() {
        let p = AppsPlugin::new_for_test(
            MockSource::with_categories(vec![("Code", "code", "Text Editor")]),
            ephemeral_frecency(),
        );
        let results = p.search("code").await;
        assert_eq!(results[0].description.as_deref(), Some("Text Editor"));
    }

    #[tokio::test]
    async fn apps_empty_query_returns_top_frecent() {
        let frecency = ephemeral_frecency();
        frecency.record("app-Code:code");
        frecency.record("app-Code:code");
        frecency.record("app-Firefox:firefox");
        let p = AppsPlugin::new_for_test(
            MockSource::with(vec![("Firefox", "firefox"), ("Code", "code")]),
            frecency,
        );
        let results = p.search("").await;
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].title.as_str(), "Code");
    }

    #[test]
    fn apps_loads_from_cache_when_source_is_empty() {
        let frecency = ephemeral_frecency();
        let cache_file = std::env::temp_dir()
            .join(format!("k-launcher-test-{}.bin", std::process::id()));

        // Build entries from a real source and save to temp path
        let source = MockSource::with(vec![("Firefox", "firefox")]);
        let entries = build_entries(&source, &frecency);
        save_to_path(&cache_file, &entries);

        // Load from temp path — should contain Firefox
        let loaded = load_from_path(&cache_file).unwrap();
        assert!(loaded.contains_key("app-Firefox:firefox"));

        std::fs::remove_file(&cache_file).ok();
    }
}
