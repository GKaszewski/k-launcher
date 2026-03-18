use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};

use k_launcher_kernel::{AppLauncher, LaunchAction};

fn shell_split(cmd: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;

    for ch in cmd.chars() {
        match ch {
            '"' => in_quotes = !in_quotes,
            ' ' | '\t' if !in_quotes => {
                if !current.is_empty() {
                    tokens.push(current.clone());
                    current.clear();
                }
            }
            _ => current.push(ch),
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

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

pub struct UnixAppLauncher;

impl UnixAppLauncher {
    pub fn new() -> Self {
        Self
    }
}

impl Default for UnixAppLauncher {
    fn default() -> Self {
        Self::new()
    }
}

impl AppLauncher for UnixAppLauncher {
    fn execute(&self, action: &LaunchAction) {
        match action {
            LaunchAction::SpawnProcess(cmd) => {
                let parts = shell_split(cmd);
                if let Some((bin, args)) = parts.split_first() {
                    let _ = unsafe {
                        Command::new(bin)
                            .args(args)
                            .stdin(Stdio::null())
                            .stdout(Stdio::null())
                            .stderr(Stdio::null())
                            .pre_exec(|| {
                                libc::setsid();
                                Ok(())
                            })
                            .spawn()
                    };
                }
            }
            LaunchAction::SpawnInTerminal(cmd) => {
                let Some((term_bin, term_args)) = resolve_terminal() else {
                    return;
                };
                let _ = unsafe {
                    Command::new(&term_bin)
                        .args(&term_args)
                        .arg("sh")
                        .arg("-c")
                        .arg(cmd)
                        .stdin(Stdio::null())
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .pre_exec(|| {
                            libc::setsid();
                            Ok(())
                        })
                        .spawn()
                };
            }
            LaunchAction::OpenPath(path) => {
                let _ = Command::new("xdg-open").arg(path).spawn();
            }
            LaunchAction::CopyToClipboard(val) => {
                if Command::new("wl-copy").arg(val).spawn().is_err() {
                    use std::io::Write;
                    if let Ok(mut child) = Command::new("xclip")
                        .args(["-selection", "clipboard"])
                        .stdin(Stdio::piped())
                        .spawn()
                        && let Some(stdin) = child.stdin.as_mut()
                    {
                        let _ = stdin.write_all(val.as_bytes());
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::shell_split;

    #[test]
    fn split_simple() {
        assert_eq!(shell_split("firefox"), vec!["firefox"]);
    }

    #[test]
    fn split_with_args() {
        assert_eq!(
            shell_split("firefox --new-window"),
            vec!["firefox", "--new-window"]
        );
    }

    #[test]
    fn split_quoted_path() {
        assert_eq!(shell_split(r#""My App" --flag"#), vec!["My App", "--flag"]);
    }

    #[test]
    fn split_quoted_with_spaces() {
        assert_eq!(
            shell_split(r#"env "FOO BAR" baz"#),
            vec!["env", "FOO BAR", "baz"]
        );
    }

    #[test]
    fn split_empty() {
        assert!(shell_split("").is_empty());
    }

    #[test]
    fn split_extra_whitespace() {
        assert_eq!(shell_split("  a   b  "), vec!["a", "b"]);
    }
}
