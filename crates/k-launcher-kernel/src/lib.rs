use std::sync::Arc;

use async_trait::async_trait;
use futures::future::join_all;

// --- Newtypes ---

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct ResultId(String);

impl ResultId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ResultTitle(String);

impl ResultTitle {
    pub fn new(title: impl Into<String>) -> Self {
        Self(title.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub struct Score(u32);

impl Score {
    pub fn new(value: u32) -> Self {
        Self(value)
    }
    pub fn value(self) -> u32 {
        self.0
    }
}

// --- LaunchAction (port) ---

pub enum LaunchAction {
    SpawnProcess(String),
    SpawnInTerminal(String),
    OpenPath(String),
    CopyToClipboard(String),
    Custom(Arc<dyn Fn() + Send + Sync>),
}

// --- AppLauncher port trait ---

pub trait AppLauncher: Send + Sync {
    fn execute(&self, action: &LaunchAction);
}

// --- SearchResult ---

pub struct SearchResult {
    pub id: ResultId,
    pub title: ResultTitle,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub score: Score,
    pub action: LaunchAction,
    pub on_select: Option<Arc<dyn Fn() + Send + Sync>>,
}

impl std::fmt::Debug for SearchResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SearchResult")
            .field("id", &self.id)
            .field("title", &self.title)
            .field("icon", &self.icon)
            .field("score", &self.score)
            .finish_non_exhaustive()
    }
}

// --- Plugin trait ---

#[async_trait]
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    async fn search(&self, query: &str) -> Vec<SearchResult>;
}

// --- SearchEngine port trait ---

#[async_trait]
pub trait SearchEngine: Send + Sync {
    async fn search(&self, query: &str) -> Vec<SearchResult>;
}

// --- Kernel (Application use case) ---

pub struct Kernel {
    plugins: Vec<Arc<dyn Plugin>>,
    max_results: usize,
}

impl Kernel {
    pub fn new(plugins: Vec<Arc<dyn Plugin>>, max_results: usize) -> Self {
        Self {
            plugins,
            max_results,
        }
    }

    pub async fn search(&self, query: &str) -> Vec<SearchResult> {
        let futures = self.plugins.iter().map(|p| p.search(query));
        let nested: Vec<Vec<SearchResult>> = join_all(futures).await;
        let mut flat: Vec<SearchResult> = nested.into_iter().flatten().collect();
        flat.sort_by(|a, b| b.score.cmp(&a.score));
        flat.truncate(self.max_results);
        flat
    }
}

#[async_trait]
impl SearchEngine for Kernel {
    async fn search(&self, query: &str) -> Vec<SearchResult> {
        self.search(query).await
    }
}

// --- Tests ---

#[cfg(test)]
mod tests {
    use super::*;

    struct MockPlugin {
        results: Vec<(&'static str, u32)>,
    }

    impl MockPlugin {
        fn returns(results: Vec<(&'static str, u32)>) -> Self {
            Self { results }
        }
    }

    #[async_trait]
    impl Plugin for MockPlugin {
        fn name(&self) -> &str {
            "mock"
        }

        async fn search(&self, _query: &str) -> Vec<SearchResult> {
            self.results
                .iter()
                .enumerate()
                .map(|(i, (title, score))| SearchResult {
                    id: ResultId::new(format!("id-{i}")),
                    title: ResultTitle::new(*title),
                    description: None,
                    icon: None,
                    score: Score::new(*score),
                    action: LaunchAction::Custom(Arc::new(|| {})),
                    on_select: None,
                })
                .collect()
        }
    }

    #[test]
    fn newtype_result_id() {
        assert_eq!(ResultId::new("x").as_str(), "x");
    }

    #[test]
    fn newtype_score() {
        assert_eq!(Score::new(42).value(), 42);
    }

    #[test]
    fn newtype_title() {
        assert_eq!(ResultTitle::new("hello").as_str(), "hello");
    }

    #[tokio::test]
    async fn empty_kernel_returns_empty() {
        let k = Kernel::new(vec![], 8);
        assert!(k.search("x").await.is_empty());
    }

    #[tokio::test]
    async fn kernel_sorts_by_score_desc() {
        let plugin = Arc::new(MockPlugin::returns(vec![
            ("lower", 5),
            ("higher", 10),
            ("middle", 7),
        ]));
        let k = Kernel::new(vec![plugin], 8);
        let results = k.search("q").await;
        assert_eq!(results[0].score.value(), 10);
        assert_eq!(results[1].score.value(), 7);
        assert_eq!(results[2].score.value(), 5);
    }

    #[tokio::test]
    async fn kernel_truncates_at_max_results() {
        let plugin = Arc::new(MockPlugin::returns(vec![
            ("a", 10),
            ("b", 9),
            ("c", 8),
            ("d", 7),
            ("e", 6),
        ]));
        let k = Kernel::new(vec![plugin], 3);
        let results = k.search("q").await;
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].score.value(), 10);
        assert_eq!(results[2].score.value(), 8);
    }
}
