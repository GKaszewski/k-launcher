use async_trait::async_trait;
use k_launcher_kernel::{LaunchAction, Plugin, ResultId, ResultTitle, Score, SearchResult};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::process::{ChildStdin, ChildStdout, Command};
use tokio::sync::Mutex;

// --- Protocol types ---

#[derive(Serialize)]
struct Query {
    query: String,
}

#[derive(Deserialize)]
struct ExternalResult {
    id: String,
    title: String,
    score: u32,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    icon: Option<String>,
    action: ExternalAction,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
enum ExternalAction {
    SpawnProcess { cmd: String },
    CopyToClipboard { text: String },
    OpenPath { path: String },
}

// --- Process I/O handle ---

struct ProcessIo {
    stdin: BufWriter<ChildStdin>,
    stdout: BufReader<ChildStdout>,
}

async fn do_search(
    io: &mut ProcessIo,
    query: &str,
) -> Result<Vec<ExternalResult>, Box<dyn std::error::Error + Send + Sync>> {
    let line = serde_json::to_string(&Query { query: query.to_string() })?;
    io.stdin.write_all(line.as_bytes()).await?;
    io.stdin.write_all(b"\n").await?;
    io.stdin.flush().await?;
    let mut response = String::new();
    io.stdout.read_line(&mut response).await?;
    Ok(serde_json::from_str(&response)?)
}

// --- ExternalPlugin ---

pub struct ExternalPlugin {
    name: String,
    path: String,
    args: Vec<String>,
    inner: Mutex<Option<ProcessIo>>,
}

impl ExternalPlugin {
    pub fn new(name: impl Into<String>, path: impl Into<String>, args: Vec<String>) -> Self {
        Self {
            name: name.into(),
            path: path.into(),
            args,
            inner: Mutex::new(None),
        }
    }

    async fn spawn(&self) -> std::io::Result<ProcessIo> {
        let mut child = Command::new(&self.path)
            .args(&self.args)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()?;
        let stdin = BufWriter::new(child.stdin.take().unwrap());
        let stdout = BufReader::new(child.stdout.take().unwrap());
        Ok(ProcessIo { stdin, stdout })
    }
}

#[async_trait]
impl Plugin for ExternalPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    async fn search(&self, query: &str) -> Vec<SearchResult> {
        let mut guard = self.inner.lock().await;

        if guard.is_none() {
            match self.spawn().await {
                Ok(io) => *guard = Some(io),
                Err(e) => {
                    tracing::warn!("failed to spawn plugin {}: {e}", self.name);
                    return vec![];
                }
            }
        }

        let result = match guard.as_mut() {
            Some(io) => do_search(io, query).await,
            None => unreachable!(),
        };

        match result {
            Ok(results) => results
                .into_iter()
                .map(|r| SearchResult {
                    id: ResultId::new(r.id),
                    title: ResultTitle::new(r.title),
                    description: r.description,
                    icon: r.icon,
                    score: Score::new(r.score),
                    action: match r.action {
                        ExternalAction::SpawnProcess { cmd } => LaunchAction::SpawnProcess(cmd),
                        ExternalAction::CopyToClipboard { text } => {
                            LaunchAction::CopyToClipboard(text)
                        }
                        ExternalAction::OpenPath { path } => LaunchAction::OpenPath(path),
                    },
                    on_select: None,
                })
                .collect(),
            Err(e) => {
                tracing::warn!("plugin {} error: {e}", self.name);
                *guard = None;
                vec![]
            }
        }
    }
}

// --- Tests ---

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_serializes_correctly() {
        let q = Query { query: "firefox".to_string() };
        assert_eq!(serde_json::to_string(&q).unwrap(), r#"{"query":"firefox"}"#);
    }

    #[test]
    fn result_parses_spawn_action() {
        let json = r#"[{"id":"1","title":"Firefox","score":80,"action":{"type":"SpawnProcess","cmd":"firefox"}}]"#;
        let results: Vec<ExternalResult> = serde_json::from_str(json).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "1");
        assert_eq!(results[0].title, "Firefox");
        assert_eq!(results[0].score, 80);
        assert!(matches!(&results[0].action, ExternalAction::SpawnProcess { cmd } if cmd == "firefox"));
    }

    #[test]
    fn result_parses_copy_action() {
        let json = r#"[{"id":"c","title":"= 4","score":90,"action":{"type":"CopyToClipboard","text":"4"}}]"#;
        let results: Vec<ExternalResult> = serde_json::from_str(json).unwrap();
        assert!(matches!(&results[0].action, ExternalAction::CopyToClipboard { text } if text == "4"));
    }

    #[test]
    fn result_parses_open_path_action() {
        let json = r#"[{"id":"f","title":"/home/user","score":50,"action":{"type":"OpenPath","path":"/home/user"}}]"#;
        let results: Vec<ExternalResult> = serde_json::from_str(json).unwrap();
        assert!(matches!(&results[0].action, ExternalAction::OpenPath { path } if path == "/home/user"));
    }

    #[test]
    fn result_parses_optional_fields() {
        let json = r#"[{"id":"x","title":"X","score":10,"description":"desc","icon":"/icon.png","action":{"type":"SpawnProcess","cmd":"x"}}]"#;
        let results: Vec<ExternalResult> = serde_json::from_str(json).unwrap();
        assert_eq!(results[0].description.as_deref(), Some("desc"));
        assert_eq!(results[0].icon.as_deref(), Some("/icon.png"));
    }

    #[test]
    fn result_parses_missing_optional_fields() {
        let json = r#"[{"id":"x","title":"X","score":10,"action":{"type":"SpawnProcess","cmd":"x"}}]"#;
        let results: Vec<ExternalResult> = serde_json::from_str(json).unwrap();
        assert!(results[0].description.is_none());
        assert!(results[0].icon.is_none());
    }

    #[test]
    fn invalid_json_is_err() {
        assert!(serde_json::from_str::<Vec<ExternalResult>>("not json").is_err());
    }

    // Unused import suppression for Arc (used only in production code path)
    fn _assert_send_sync() {
        fn check<T: Send + Sync>() {}
        check::<ExternalPlugin>();
    }
}
