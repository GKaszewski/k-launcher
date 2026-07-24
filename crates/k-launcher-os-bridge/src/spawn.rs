use std::io::Write;
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};

const XDG_OPEN: &str = "xdg-open";
const WL_COPY: &str = "wl-copy";
const XCLIP: &str = "xclip";

pub(crate) fn spawn_detached(bin: &str, args: &[String]) {
    // SAFETY: setsid() is async-signal-safe; called in forked child before exec
    if let Err(e) = unsafe {
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
    } {
        tracing::warn!("failed to spawn detached process '{bin}': {e}");
    }
}

pub(crate) fn open_path(path: &str) {
    if let Err(e) = Command::new(XDG_OPEN).arg(path).spawn() {
        tracing::warn!("failed to open path '{path}': {e}");
    }
}

pub(crate) fn copy_to_clipboard(val: &str) {
    if Command::new(WL_COPY).arg(val).spawn().is_err() {
        copy_to_clipboard_xclip(val);
    }
}

fn copy_to_clipboard_xclip(val: &str) {
    if let Ok(mut child) = Command::new(XCLIP)
        .args(["-selection", "clipboard"])
        .stdin(Stdio::piped())
        .spawn()
        && let Some(stdin) = child.stdin.as_mut()
        && let Err(e) = stdin.write_all(val.as_bytes())
    {
        tracing::warn!("failed to write to xclip stdin: {e}");
    }
}
