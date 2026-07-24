pub(crate) struct TerminalCommand {
    pub bin: String,
    pub exec_flag: Vec<String>,
}

struct KnownTerminal {
    bin: &'static str,
    exec_flag: &'static str,
}

const KNOWN_TERMINALS: &[KnownTerminal] = &[
    KnownTerminal {
        bin: "ghostty",
        exec_flag: "-e",
    },
    KnownTerminal {
        bin: "foot",
        exec_flag: "-e",
    },
    KnownTerminal {
        bin: "kitty",
        exec_flag: "-e",
    },
    KnownTerminal {
        bin: "alacritty",
        exec_flag: "-e",
    },
    KnownTerminal {
        bin: "wezterm",
        exec_flag: "start",
    },
    KnownTerminal {
        bin: "konsole",
        exec_flag: "-e",
    },
    KnownTerminal {
        bin: "xterm",
        exec_flag: "-e",
    },
];

fn find_in_path(bin: &str) -> bool {
    std::env::var_os("PATH")
        .iter()
        .flat_map(|p| std::env::split_paths(p))
        .any(|dir| dir.join(bin).is_file())
}

fn parse_term_cmd(s: &str) -> TerminalCommand {
    let mut parts = s.split_whitespace();
    let bin = parts.next().unwrap_or("").to_string();
    let exec_flag = parts.map(str::to_string).collect();
    TerminalCommand { bin, exec_flag }
}

pub(crate) fn resolve_terminal(configured: Option<&str>) -> Option<TerminalCommand> {
    if let Some(cmd) = configured.filter(|s| !s.is_empty()) {
        let term = parse_term_cmd(cmd);
        if !term.bin.is_empty() {
            return Some(term);
        }
    }
    if let Ok(val) = std::env::var("TERM_CMD") {
        let val = val.trim().to_string();
        if !val.is_empty() {
            let term = parse_term_cmd(&val);
            if !term.bin.is_empty() {
                return Some(term);
            }
        }
    }
    if let Ok(val) = std::env::var("TERMINAL") {
        let bin = val.trim().to_string();
        if !bin.is_empty() {
            return Some(TerminalCommand {
                bin,
                exec_flag: vec!["-e".to_string()],
            });
        }
    }
    for terminal in KNOWN_TERMINALS {
        if find_in_path(terminal.bin) {
            return Some(TerminalCommand {
                bin: terminal.bin.to_string(),
                exec_flag: vec![terminal.exec_flag.to_string()],
            });
        }
    }
    None
}
