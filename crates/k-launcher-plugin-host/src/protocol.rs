use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct Query {
    pub query: String,
}

#[derive(Deserialize)]
pub struct ExternalResult {
    pub id: String,
    pub title: String,
    pub score: u32,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub icon: Option<String>,
    pub action: ExternalAction,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum ExternalAction {
    SpawnProcess { cmd: String },
    SpawnInTerminal { cmd: String },
    CopyToClipboard { text: String },
    OpenPath { path: String },
}
