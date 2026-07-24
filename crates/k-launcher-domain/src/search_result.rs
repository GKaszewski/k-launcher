use std::sync::Arc;

use crate::action::LaunchAction;
use crate::newtypes::{ResultId, ResultTitle, Score};

#[derive(Clone)]
pub struct SearchResult {
    pub id: ResultId,
    pub title: ResultTitle,
    pub description: Option<Arc<str>>,
    pub icon: Option<Arc<str>>,
    pub score: Score,
    pub action: LaunchAction,
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
