use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;
use k_launcher_kernel::{Plugin, PluginName, ResultId, ResultTitle, Score, SearchResult};

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
        let home = std::env::var("HOME").ok()?;
        Some(format!("{}{}", home, &query[1..]))
    } else if query.starts_with('/') {
        Some(query.to_string())
    } else {
        None
    }
}

#[async_trait]
impl Plugin for FilesPlugin {
    fn name(&self) -> PluginName {
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
            .take(20)
            .enumerate()
            .map(|(i, entry)| {
                let full_path = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();
                let is_dir = full_path.is_dir();
                let title = if is_dir {
                    format!("{name}/")
                } else {
                    name
                };
                let path_str = full_path.to_string_lossy().to_string();
                SearchResult {
                    id: ResultId::new(format!("file-{i}")),
                    title: ResultTitle::new(title),
                    description: Some(path_str.clone()),
                    icon: None,
                    score: Score::new(50),
                    on_execute: Arc::new(move || {
                        let _ = std::process::Command::new("xdg-open").arg(&path_str).spawn();
                    }),
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn files_ignores_non_path_query() {
        let p = FilesPlugin::new();
        assert!(p.search("firefox").await.is_empty());
    }

    #[tokio::test]
    async fn files_handles_root() {
        let p = FilesPlugin::new();
        let results = p.search("/").await;
        assert!(!results.is_empty());
    }
}
