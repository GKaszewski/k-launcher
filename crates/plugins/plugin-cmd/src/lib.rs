use async_trait::async_trait;
use k_launcher_domain::{LaunchAction, Plugin, ResultId, ResultTitle, Score, SearchResult};

const CMD_PREFIX: char = '>';
const RESULT_SCORE: u32 = 95;

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
    fn name(&self) -> &str {
        "cmd"
    }

    async fn search(&self, query: &str) -> Vec<SearchResult> {
        let Some(rest) = query.strip_prefix(CMD_PREFIX) else {
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
            score: Score::new(RESULT_SCORE),
            action: LaunchAction::SpawnInTerminal(cmd.to_string()),
        }]
    }
}
