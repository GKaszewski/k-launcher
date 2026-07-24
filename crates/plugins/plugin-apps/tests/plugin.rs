use std::sync::Arc;

use k_launcher_domain::Plugin;
use plugin_apps::frecency::FrecencyStore;
use plugin_apps::{
    AppName, AppsPlugin, DesktopEntry, DesktopEntrySource, ExecCommand, build_entries,
    humanize_category, load_from_path, new_matcher, parse_pattern, save_to_path, score_match,
};

fn ephemeral_frecency() -> Arc<FrecencyStore> {
    FrecencyStore::new_for_test()
}

struct MockEntry {
    name: String,
    exec: String,
    category: Option<String>,
    keywords: Vec<String>,
}

struct MockSource {
    entries: Vec<MockEntry>,
}

impl MockSource {
    fn with(entries: Vec<(&str, &str)>) -> Self {
        Self {
            entries: entries
                .into_iter()
                .map(|(n, e)| MockEntry {
                    name: n.to_string(),
                    exec: e.to_string(),
                    category: None,
                    keywords: vec![],
                })
                .collect(),
        }
    }

    fn with_categories(entries: Vec<(&str, &str, &str)>) -> Self {
        Self {
            entries: entries
                .into_iter()
                .map(|(n, e, c)| MockEntry {
                    name: n.to_string(),
                    exec: e.to_string(),
                    category: Some(c.to_string()),
                    keywords: vec![],
                })
                .collect(),
        }
    }

    fn with_keywords(entries: Vec<(&str, &str, Vec<&str>)>) -> Self {
        Self {
            entries: entries
                .into_iter()
                .map(|(n, e, kw)| MockEntry {
                    name: n.to_string(),
                    exec: e.to_string(),
                    category: None,
                    keywords: kw.into_iter().map(|s| s.to_string()).collect(),
                })
                .collect(),
        }
    }
}

impl DesktopEntrySource for MockSource {
    fn entries(&self) -> Vec<DesktopEntry> {
        self.entries
            .iter()
            .map(|e| DesktopEntry {
                name: AppName::new(e.name.clone()),
                exec: ExecCommand::new(e.exec.clone()),
                icon: None,
                category: e.category.clone(),
                keywords: e.keywords.clone(),
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
    let mut matcher = new_matcher();
    let pattern = parse_pattern("vsc");
    let mut buf = Vec::new();
    assert!(
        score_match(
            &mut matcher,
            &pattern,
            "visual studio code",
            &mut buf,
            "visual studio code",
            "vsc"
        )
        .is_some()
    );
}

#[test]
fn score_match_exact_beats_prefix() {
    let mut matcher = new_matcher();
    let mut buf = Vec::new();
    let exact_pattern = parse_pattern("firefox");
    let fire_pattern = parse_pattern("fire");
    let gf_pattern = parse_pattern("gf");

    let exact = score_match(
        &mut matcher,
        &exact_pattern,
        "firefox",
        &mut buf,
        "firefox",
        "firefox",
    );
    let prefix = score_match(
        &mut matcher,
        &fire_pattern,
        "firefox",
        &mut buf,
        "firefox",
        "fire",
    );
    let abbrev = score_match(
        &mut matcher,
        &gf_pattern,
        "gnu firefox",
        &mut buf,
        "gnu firefox",
        "gf",
    );
    let substr = score_match(
        &mut matcher,
        &fire_pattern,
        "ice firefox",
        &mut buf,
        "ice firefox",
        "fire",
    );
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
    let cache_file =
        std::env::temp_dir().join(format!("k-launcher-test-{}.bin", std::process::id()));

    let source = MockSource::with(vec![("Firefox", "firefox")]);
    let entries = build_entries(&source, &frecency);
    save_to_path(&cache_file, &entries);

    let loaded = load_from_path(&cache_file).unwrap();
    assert!(loaded.contains_key("app-Firefox:firefox"));

    std::fs::remove_file(&cache_file).ok();
}
