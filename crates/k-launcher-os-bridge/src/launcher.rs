use k_launcher_domain::{AppLauncher, LaunchAction};

use crate::shell::shell_split;
use crate::spawn::{copy_to_clipboard, open_path, spawn_detached};
use crate::terminal::resolve_terminal;

pub struct UnixAppLauncher {
    terminal_cmd: Option<String>,
}

impl UnixAppLauncher {
    pub fn new(terminal_cmd: Option<String>) -> Self {
        Self { terminal_cmd }
    }
}

impl AppLauncher for UnixAppLauncher {
    fn execute(&self, action: &LaunchAction) {
        match action {
            LaunchAction::SpawnProcess(cmd) => spawn_command(cmd),
            LaunchAction::SpawnInTerminal(cmd) => {
                spawn_in_terminal(cmd, self.terminal_cmd.as_deref())
            }
            LaunchAction::OpenPath(path) => open_path(path),
            LaunchAction::CopyToClipboard(val) => copy_to_clipboard(val),
        }
    }
}

fn spawn_command(cmd: &str) {
    let parts = shell_split(cmd);
    if let Some((bin, args)) = parts.split_first() {
        spawn_detached(bin, args);
    }
}

fn spawn_in_terminal(cmd: &str, configured: Option<&str>) {
    let Some(terminal) = resolve_terminal(configured) else {
        return;
    };
    let mut args = terminal.exec_flag;
    const SHELL: &str = "sh";
    const SHELL_CMD_FLAG: &str = "-c";
    args.extend([
        SHELL.to_string(),
        SHELL_CMD_FLAG.to_string(),
        cmd.to_string(),
    ]);
    spawn_detached(&terminal.bin, &args);
}
