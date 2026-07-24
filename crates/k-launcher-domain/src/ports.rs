use async_trait::async_trait;

use crate::{LaunchAction, ResultId, SearchResult};

pub trait AppLauncher: Send + Sync {
    fn execute(&self, action: &LaunchAction);
}

#[async_trait]
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    async fn search(&self, query: &str) -> Vec<SearchResult>;
    fn on_selected(&self, _id: &ResultId) {}
    fn shutdown(&self) {}
}
