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

## Docs

- [Installation](docs/install.md)
- [Usage & Keybinds](docs/usage.md)
- [Configuration & Theming](docs/configuration.md)
- [Plugin Development](docs/plugin-development.md)
