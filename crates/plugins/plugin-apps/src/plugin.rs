use std::{collections::HashMap, path::PathBuf, sync::Arc};

use parking_lot::RwLock;

use async_trait::async_trait;
use k_launcher_domain::{LaunchAction, Plugin, ResultId, ResultTitle, Score, SearchResult};

use crate::cache::{CachedEntry, build_entries, cache_path, load_from_path, save_to_path};
use crate::frecency::FrecencyStore;
use crate::scoring::{new_matcher, parse_pattern, score_match};
use crate::types::DesktopEntrySource;

const FRECENT_RESULTS_COUNT: usize = 5;
const KEYWORD_MATCH_SCORE: u32 = 50;

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
                *entries_bg.write() = fresh;
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

    pub fn new_for_test(
        source: impl DesktopEntrySource + 'static,
        frecency: Arc<FrecencyStore>,
    ) -> Self {
        Self::new_impl(source, frecency, None)
    }
}

#[async_trait]
impl Plugin for AppsPlugin {
    fn name(&self) -> &str {
        "apps"
    }

    fn on_selected(&self, id: &ResultId) {
        self.frecency.record(id.as_str());
    }

    fn shutdown(&self) {
        self.frecency.shutdown();
    }

    async fn search(&self, query: &str) -> Vec<SearchResult> {
        let entries = self.entries.read();
        if query.is_empty() {
            return self
                .frecency
                .top_ids(FRECENT_RESULTS_COUNT)
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

        let query_lowercase = query.to_lowercase();
        let first_char = query_lowercase.chars().next().unwrap_or_default();
        let mut matcher = new_matcher();
        let pattern = parse_pattern(query);
        let mut char_buf: Vec<char> = Vec::with_capacity(64);

        entries
            .values()
            .filter(|e| {
                e.name_lowercase.contains(first_char)
                    || e.keywords_lowercase.iter().any(|k| k.contains(first_char))
            })
            .filter_map(|e| {
                let match_score = score_match(
                    &mut matcher,
                    &pattern,
                    e.name.as_str(),
                    &mut char_buf,
                    &e.name_lowercase,
                    &query_lowercase,
                )
                .or_else(|| {
                    e.keywords_lowercase
                        .iter()
                        .any(|k| k.contains(query_lowercase.as_str()))
                        .then_some(KEYWORD_MATCH_SCORE)
                })?;
                let frecency_boost = self.frecency.frecency_score(&e.id);
                Some(SearchResult {
                    id: ResultId::new(&e.id),
                    title: ResultTitle::new(e.name.as_str()),
                    description: e.category.clone(),
                    icon: e.icon.clone(),
                    score: Score::new(match_score.saturating_add(frecency_boost)),
                    action: LaunchAction::SpawnProcess(e.exec.clone()),
                })
            })
            .collect()
    }
}
