# Installation

## Arch Linux (AUR)

```bash
yay -S k-launcher
```

## Build from Source

### Prerequisites

- **Rust** stable toolchain — install via [rustup](https://rustup.rs)
- **git**
- A **Wayland** or **X11** compositor (Linux)

### Build and install

```bash
git clone https://github.com/GKaszewski/k-launcher
cd k-launcher
make install
```

This builds a release binary and copies it to `~/.local/bin/k-launcher`.

Ensure `~/.local/bin` is in your `$PATH`.

### Manual build

```bash
cargo build --release
cp target/release/k-launcher ~/.local/bin/
```

## Compositor Keybind

### Hyprland

Add to `~/.config/hypr/hyprland.conf`:

```
windowrule = float, ^(k-launcher)$
windowrule = center, ^(k-launcher)$
bind = SUPER, Space, exec, k-launcher
```

### Sway

Add to `~/.config/sway/config`:

```
for_window [app_id="k-launcher"] floating enable, move position center
bindsym Mod4+space exec k-launcher
```

## Autostart (optional)

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

## Verify

```bash
k-launcher --version
```
