use std::sync::Arc;

use futures::future::join_all;

use k_launcher_domain::{Plugin, ResultId, SearchResult};

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

    pub fn on_selected(&self, id: &ResultId) {
        for plugin in &self.plugins {
            plugin.on_selected(id);
        }
    }

    pub fn shutdown(&self) {
        for plugin in &self.plugins {
            plugin.shutdown();
        }
    }

    pub async fn search(&self, query: &str) -> Vec<SearchResult> {
        use futures::FutureExt;
        use std::panic::AssertUnwindSafe;

        let futures = self
            .plugins
            .iter()
            .map(|p| AssertUnwindSafe(p.search(query)).catch_unwind());
        let outcomes = join_all(futures).await;
        let mut flat: Vec<SearchResult> = outcomes
            .into_iter()
            .zip(self.plugins.iter())
            .flat_map(|(outcome, plugin)| match outcome {
                Ok(results) => results,
                Err(_) => {
                    tracing::error!(plugin = plugin.name(), "plugin panicked during search");
                    vec![]
                }
            })
            .collect();
        flat.sort_by_key(|r| std::cmp::Reverse(r.score));
        flat.truncate(self.max_results);
        flat
    }
}
