#[derive(Clone)]
pub enum LaunchAction {
    SpawnProcess(String),
    SpawnInTerminal(String),
    OpenPath(String),
    CopyToClipboard(String),
}
