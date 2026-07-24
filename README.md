# k-launcher

A lightweight command palette for Linux (Wayland/X11). Fuzzy search, frecency ranking, plugin system. Written in Rust.

## Quick Start

```bash
git clone https://github.com/GKaszewski/k-launcher
cd k-launcher
make install
```

Or with cargo directly:

```bash
cargo build --release
cp target/release/k-launcher ~/.local/bin/
```

### Arch Linux (AUR)

```bash
yay -S k-launcher
```

## Usage

| Input | What it does | Example |
|---|---|---|
| any text | Fuzzy-search installed apps | `firefox` |
| empty | Show most-used apps (frecency) | |
| `>` prefix | Run shell command in terminal | `> htop` |
| `=` or math | Evaluate expression, copy result | `2^10 + 5` |
| `/` or `~/` | Browse filesystem | `~/Documents` |

## Keybinds

| Key | Action |
|---|---|
| `↑` / `↓` | Navigate results |
| `Enter` | Launch / copy |
| `Escape` | Close |

## Configuration

`~/.config/k-launcher/config.toml` — all fields optional, sensible defaults.

See [config.example.toml](config.example.toml) for all available options.

## Compositor Setup

**Hyprland** (`~/.config/hypr/hyprland.conf`):

```
windowrule = float, ^(k-launcher)$
windowrule = center, ^(k-launcher)$
bind = SUPER, Space, exec, k-launcher
```

**Sway** (`~/.config/sway/config`):

```
for_window [app_id="k-launcher"] floating enable, move position center
bindsym Mod4+space exec k-launcher
```

## Plugins

Built-in plugins (calc, apps, shell, files) are enabled by default. External plugins communicate via JSON over stdin/stdout — any language, no recompilation:

```toml
[[plugins.external]]
name = "my-plugin"
path = "/path/to/plugin"
timeout_secs = 5
```

See [Plugin Development](docs/plugin-development.md) for the full protocol.

## Docs

- [Installation](docs/install.md)
- [Usage](docs/usage.md)
- [Configuration](docs/configuration.md)
- [Plugin Development](docs/plugin-development.md)
- `man k-launcher`

## License

[MIT](LICENSE)
