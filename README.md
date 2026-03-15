# k-launcher

A lightweight, GPU-accelerated command palette for Linux (Wayland/X11). Zero Electron — every pixel rendered via WGPU. Async search that never blocks the UI.

```
[screenshot placeholder]
```

## Quick Start

```bash
git clone https://github.com/GKaszewski/k-launcher
cd k-launcher
cargo build --release
./target/release/k-launcher
```

## Keybinds

| Key       | Action          |
| --------- | --------------- |
| Type      | Filter results  |
| `↑` / `↓` | Navigate        |
| `Enter`   | Launch selected |
| `Escape`  | Close           |

## Built-in Plugins

| Trigger           | Plugin | Example        |
| ----------------- | ------ | -------------- |
| (any text)        | Apps   | `firefox`      |
| number/expression | Calc   | `2^10 + 5`     |
| `>` prefix        | Shell  | `> echo hello` |
| `/` or `~/`       | Files  | `~/Documents`  |

## External Plugins

Drop in community plugins — any language, no recompilation. Plugins are executables that communicate over stdin/stdout JSON:

```toml
# ~/.config/k-launcher/config.toml
[[plugins.external]]
name = "my-plugin"
path = "/usr/lib/k-launcher/plugins/my-plugin"
```

See [Plugin Development](docs/plugin-development.md) for the full protocol.

## Docs

- [Installation](docs/install.md)
- [Usage & Keybinds](docs/usage.md)
- [Configuration & Theming](docs/configuration.md)
- [Plugin Development](docs/plugin-development.md)
