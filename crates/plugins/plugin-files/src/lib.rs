mod platform;

use std::path::Path;

use std::sync::Arc;

use async_trait::async_trait;
use k_launcher_domain::{LaunchAction, Plugin, ResultId, ResultTitle, Score, SearchResult};

const MAX_FILE_RESULTS: usize = 20;
const RESULT_SCORE: u32 = 50;

pub struct FilesPlugin;

impl FilesPlugin {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FilesPlugin {
    fn default() -> Self {
        Self::new()
    }
}

fn expand_query(query: &str) -> Option<String> {
    if query.starts_with("~/") {
        let home = platform::home_dir()?;
        Some(format!("{}{}", home, &query[1..]))
    } else if query.starts_with('/') {
        Some(query.to_string())
    } else {
        None
    }
}

#[async_trait]
impl Plugin for FilesPlugin {
    fn name(&self) -> &str {
        "files"
    }

    async fn search(&self, query: &str) -> Vec<SearchResult> {
        let expanded = match expand_query(query) {
            Some(p) => p,
            None => return vec![],
        };
        let path = Path::new(&expanded);
        let (parent, prefix) = if path.is_dir() {
            (path.to_path_buf(), String::new())
        } else {
            let parent = path.parent().unwrap_or(Path::new("/")).to_path_buf();
            let prefix = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_lowercase();
            (parent, prefix)
        };

        let entries = match std::fs::read_dir(&parent) {
            Ok(e) => e,
            Err(_) => return vec![],
        };

        entries
            .filter_map(|e| e.ok())
            .filter(|e| {
                if prefix.is_empty() {
                    return true;
                }
                e.file_name()
                    .to_str()
                    .map(|n| n.to_lowercase().starts_with(&prefix))
                    .unwrap_or(false)
            })
            .take(MAX_FILE_RESULTS)
            .map(|entry| {
                let full_path = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();
                let is_dir = full_path.is_dir();
                let title = if is_dir { format!("{name}/") } else { name };
                let path_str = full_path.to_string_lossy().to_string();
                SearchResult {
                    id: ResultId::new(&path_str),
                    title: ResultTitle::new(title),
                    description: Some(Arc::from(path_str.as_str())),
                    icon: None,
                    score: Score::new(RESULT_SCORE),
                    action: LaunchAction::OpenPath(path_str),
                }
            })
            .collect()
    }
}
