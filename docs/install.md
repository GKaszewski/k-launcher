# Installation

## Prerequisites

- **Rust** stable toolchain — install via [rustup](https://rustup.rs)
- **git**
- A **Wayland** or **X11** compositor (Linux)

## Build from Source

```bash
git clone https://github.com/GKaszewski/k-launcher
cd k-launcher
cargo build --release
```

Binary location: `target/release/k-launcher`

### Optional: install to PATH

```bash
cp target/release/k-launcher ~/.local/bin/
```

Ensure `~/.local/bin` is in your `$PATH`.

## Autostart

### Hyprland

Add to `~/.config/hypr/hyprland.conf`:

```
exec-once = k-launcher
```

### systemd user service

Create `~/.config/systemd/user/k-launcher.service`:

```ini
[Unit]
Description=k-launcher command palette

[Service]
ExecStart=%h/.local/bin/k-launcher
Restart=on-failure

[Install]
WantedBy=graphical-session.target
```

Then enable it:

```bash
systemctl --user enable --now k-launcher
```
