use async_trait::async_trait;
use k_launcher_kernel::{LaunchAction, Plugin, PluginName, ResultId, ResultTitle, Score, SearchResult};

pub struct CmdPlugin;

impl CmdPlugin {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CmdPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for CmdPlugin {
    fn name(&self) -> PluginName {
        "cmd"
    }

    async fn search(&self, query: &str) -> Vec<SearchResult> {
        let Some(rest) = query.strip_prefix('>') else {
            return vec![];
        };
        let cmd = rest.trim();
        if cmd.is_empty() {
            return vec![];
        }
        vec![SearchResult {
            id: ResultId::new(format!("cmd-{cmd}")),
            title: ResultTitle::new(format!("Run: {cmd}")),
            description: None,
            icon: None,
            score: Score::new(95),
            action: LaunchAction::SpawnInTerminal(cmd.to_string()),
            on_select: None,
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn cmd_prefix_triggers() {
        let p = CmdPlugin::new();
        let results = p.search("> echo hello").await;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title.as_str(), "Run: echo hello");
        assert_eq!(results[0].score.value(), 95);
    }

    #[tokio::test]
    async fn cmd_empty_remainder_returns_empty() {
        let p = CmdPlugin::new();
        assert!(p.search(">").await.is_empty());
        assert!(p.search(">   ").await.is_empty());
    }

    #[tokio::test]
    async fn cmd_no_prefix_returns_empty() {
        let p = CmdPlugin::new();
        assert!(p.search("echo hello").await.is_empty());
        assert!(p.search("firefox").await.is_empty());
    }
}
