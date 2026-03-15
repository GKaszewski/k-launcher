use std::{process::{Command, Stdio}, sync::Arc};
use std::os::unix::process::CommandExt;

use async_trait::async_trait;
use k_launcher_kernel::{Plugin, PluginName, ResultId, ResultTitle, Score, SearchResult};

fn parse_term_cmd(s: &str) -> (String, Vec<String>) {
    let mut parts = s.split_whitespace();
    let bin = parts.next().unwrap_or("").to_string();
    let args = parts.map(str::to_string).collect();
    (bin, args)
}

fn which(bin: &str) -> bool {
    Command::new("which")
        .arg(bin)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn resolve_terminal() -> Option<(String, Vec<String>)> {
    if let Ok(val) = std::env::var("TERM_CMD") {
        let val = val.trim().to_string();
        if !val.is_empty() {
            let (bin, args) = parse_term_cmd(&val);
            if !bin.is_empty() {
                return Some((bin, args));
            }
        }
    }
    if let Ok(val) = std::env::var("TERMINAL") {
        let bin = val.trim().to_string();
        if !bin.is_empty() {
            return Some((bin, vec!["-e".to_string()]));
        }
    }
    for (bin, flag) in &[
        ("foot", "-e"),
        ("kitty", "-e"),
        ("alacritty", "-e"),
        ("wezterm", "start"),
        ("konsole", "-e"),
        ("xterm", "-e"),
    ] {
        if which(bin) {
            return Some((bin.to_string(), vec![flag.to_string()]));
        }
    }
    None
}

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
        let cmd_owned = cmd.to_string();
        vec![SearchResult {
            id: ResultId::new(format!("cmd-{cmd}")),
            title: ResultTitle::new(format!("Run: {cmd}")),
            description: None,
            icon: None,
            score: Score::new(95),
            on_execute: Arc::new(move || {
                let Some((term_bin, term_args)) = resolve_terminal() else { return };
                let _ = unsafe {
                    Command::new(&term_bin)
                        .args(&term_args)
                        .arg("sh").arg("-c").arg(&cmd_owned)
                        .stdin(Stdio::null())
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .pre_exec(|| { libc::setsid(); Ok(()) })
                        .spawn()
                };
            }),
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

    #[test]
    fn parse_term_cmd_single_flag() {
        let (bin, args) = parse_term_cmd("foot -e");
        assert_eq!(bin, "foot");
        assert_eq!(args, vec!["-e"]);
    }

    #[test]
    fn parse_term_cmd_multiword() {
        let (bin, args) = parse_term_cmd("wezterm start");
        assert_eq!(bin, "wezterm");
        assert_eq!(args, vec!["start"]);
    }

    #[test]
    fn parse_term_cmd_no_args() {
        let (bin, args) = parse_term_cmd("xterm");
        assert_eq!(bin, "xterm");
        assert!(args.is_empty());
    }
}
