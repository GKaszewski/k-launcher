use std::sync::Arc;

use async_trait::async_trait;
use k_launcher_domain::{LaunchAction, Plugin, ResultId, ResultTitle, Score, SearchResult};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::process::{ChildStdin, ChildStdout, Command};
use tokio::sync::Mutex;

use crate::error::PluginError;
use crate::protocol::{ExternalAction, ExternalResult, Query};

struct ProcessIo {
    stdin: BufWriter<ChildStdin>,
    stdout: BufReader<ChildStdout>,
}

async fn do_search(io: &mut ProcessIo, query: &str) -> Result<Vec<ExternalResult>, PluginError> {
    let line = serde_json::to_string(&Query {
        query: query.to_string(),
    })
    .map_err(|e| PluginError::Protocol(e.to_string()))?;
    io.stdin
        .write_all(line.as_bytes())
        .await
        .map_err(|e| PluginError::ProcessError(e.to_string()))?;
    io.stdin
        .write_all(b"\n")
        .await
        .map_err(|e| PluginError::ProcessError(e.to_string()))?;
    io.stdin
        .flush()
        .await
        .map_err(|e| PluginError::ProcessError(e.to_string()))?;
    let mut response = String::new();
    io.stdout
        .read_line(&mut response)
        .await
        .map_err(|e| PluginError::ProcessError(e.to_string()))?;
    serde_json::from_str(&response).map_err(|e| PluginError::Protocol(e.to_string()))
}

pub struct ExternalPlugin {
    name: String,
    path: String,
    args: Vec<String>,
    timeout_secs: u64,
    inner: Mutex<Option<ProcessIo>>,
}

impl ExternalPlugin {
    pub fn new(
        name: impl Into<String>,
        path: impl Into<String>,
        args: Vec<String>,
        timeout_secs: u64,
    ) -> Self {
        Self {
            name: name.into(),
            path: path.into(),
            args,
            timeout_secs,
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
            Some(io) => tokio::time::timeout(
                std::time::Duration::from_secs(self.timeout_secs),
                do_search(io, query),
            )
            .await
            .unwrap_or(Err(PluginError::Timeout {
                timeout_secs: self.timeout_secs,
            })),
            None => unreachable!(),
        };

        match result {
            Ok(results) => results
                .into_iter()
                .map(|r| SearchResult {
                    id: ResultId::new(r.id),
                    title: ResultTitle::new(r.title),
                    description: r.description.map(Arc::from),
                    icon: r.icon.map(Arc::from),
                    score: Score::new(r.score),
                    action: match r.action {
                        ExternalAction::SpawnProcess { cmd } => LaunchAction::SpawnProcess(cmd),
                        ExternalAction::SpawnInTerminal { cmd } => {
                            LaunchAction::SpawnInTerminal(cmd)
                        }
                        ExternalAction::CopyToClipboard { text } => {
                            LaunchAction::CopyToClipboard(text)
                        }
                        ExternalAction::OpenPath { path } => LaunchAction::OpenPath(path),
                    },
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
