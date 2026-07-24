use std::sync::Arc;

use async_trait::async_trait;
use k_launcher_domain::Plugin;
use k_launcher_domain::{LaunchAction, ResultId, ResultTitle, Score, SearchResult};
use k_launcher_kernel::Kernel;

struct MockResult {
    title: &'static str,
    score: u32,
}

struct MockPlugin {
    results: Vec<MockResult>,
}

impl MockPlugin {
    fn returns(results: Vec<(&'static str, u32)>) -> Self {
        Self {
            results: results
                .into_iter()
                .map(|(title, score)| MockResult { title, score })
                .collect(),
        }
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
            .map(|(i, r)| SearchResult {
                id: ResultId::new(format!("id-{i}")),
                title: ResultTitle::new(r.title),
                description: None,
                icon: None,
                score: Score::new(r.score),
                action: LaunchAction::SpawnProcess("mock".to_string()),
            })
            .collect()
    }
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

struct PanicPlugin;

#[async_trait]
impl Plugin for PanicPlugin {
    fn name(&self) -> &str {
        "panic-plugin"
    }

    async fn search(&self, _query: &str) -> Vec<SearchResult> {
        panic!("test panic");
    }
}

#[tokio::test]
async fn kernel_continues_after_plugin_panic() {
    let panic_plugin = Arc::new(PanicPlugin);
    let normal_plugin = Arc::new(MockPlugin::returns(vec![("survivor", 5)]));
    let k = Kernel::new(vec![panic_plugin, normal_plugin], 8);
    let results = k.search("q").await;
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title.as_str(), "survivor");
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
