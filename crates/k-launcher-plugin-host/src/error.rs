use thiserror::Error;

#[derive(Debug, Error)]
pub enum PluginError {
    #[error("plugin spawn failed: {0}")]
    SpawnFailed(#[from] std::io::Error),
    #[error("search timed out after {timeout_secs}s")]
    Timeout { timeout_secs: u64 },
    #[error("protocol error: {0}")]
    Protocol(String),
    #[error("plugin process error: {0}")]
    ProcessError(String),
}
