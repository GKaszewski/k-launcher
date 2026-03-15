# k-launcher Architecture

## Philosophy

- **TDD:** Red-Green-Refactor is mandatory. No functional code without a failing test first.
- **Clean Architecture:** Strict layer separation — Domain, Application, Infrastructure, Main.
- **Newtype Pattern:** All domain primitives wrapped (e.g. `struct Score(f64)`).
- **Small Traits / ISP:** Many focused traits over one "God" trait.
- **No Cyclic Dependencies:** Use IoC (define traits in higher-level modules, implement in lower-level).

---

## Workspace Structure

| Crate | Layer | Responsibility |
|---|---|---|
| `k-launcher-kernel` | Domain + Application | Newtypes (`ResultId`, `ResultTitle`, `Score`), `Plugin` trait, `SearchEngine` trait, `AppLauncher` port, `Kernel` use case |
| `k-launcher-config` | Infrastructure | TOML config loading; `Config`, `WindowCfg`, `AppearanceCfg`, `PluginsCfg` structs |
| `k-launcher-os-bridge` | Infrastructure | `UnixAppLauncher` (process spawning), `WindowConfig` adapter |
| `k-launcher-plugin-host` | Infrastructure | `ExternalPlugin` — JSON-newline IPC protocol for out-of-process plugins |
| `k-launcher-ui` | Infrastructure | iced 0.14 Elm-like UI (`KLauncherApp`, debounced async search, keyboard nav) |
| `k-launcher-ui-egui` | Infrastructure | Alternative egui UI (feature-gated) |
| `plugins/plugin-apps` | Infrastructure | XDG `.desktop` parser, frecency scoring, nucleo fuzzy matching |
| `plugins/plugin-calc` | Infrastructure | `evalexpr`-based calculator |
| `plugins/plugin-cmd` | Infrastructure | Shell command runner |
| `plugins/plugin-files` | Infrastructure | File path search |
| `plugins/plugin-url` | Infrastructure | URL opener |
| `k-launcher` | Main/Entry | DI wiring, CLI arg parsing (`show` command), `run_ui()` composition root |

---

## Dependency Graph

```
k-launcher (main)
  ├── k-launcher-kernel        (Domain/Application)
  ├── k-launcher-config        (Infrastructure — pure data, no kernel dep)
  ├── k-launcher-os-bridge     (Infrastructure)
  ├── k-launcher-plugin-host   (Infrastructure)
  ├── k-launcher-ui            (Infrastructure)
  └── plugins/*                (Infrastructure)
       └── k-launcher-kernel
```

All arrows point inward toward the kernel. The kernel has no external dependencies.

---

## Core Abstractions (kernel)

```rust
// Plugin trait — implemented by every plugin
async fn search(&self, query: &str) -> Vec<SearchResult>;

// SearchEngine trait — implemented by Kernel
async fn search(&self, query: &str) -> Vec<SearchResult>;

// AppLauncher port — implemented by UnixAppLauncher in os-bridge
fn execute(&self, action: &LaunchAction);

// DesktopEntrySource trait (plugin-apps) — swappable .desktop file source
```

---

## Plugin System

Two kinds of plugins:

1. **In-process** — implement `Plugin` in Rust, linked at compile time.
   - `plugin-calc`, `plugin-apps`, `plugin-cmd`, `plugin-files`, `plugin-url`

2. **External / out-of-process** — `ExternalPlugin` in `k-launcher-plugin-host` communicates via JSON newline protocol over stdin/stdout.
   - Query: `{"query": "..."}`
   - Response: `[{"id": "...", "title": "...", "score": 1.0, "description": "...", "icon": "...", "action": "..."}]`

Plugins are enabled/disabled via `~/.config/k-launcher/config.toml`.

---

## Kernel (Application Use Case)

`Kernel::search` fans out to all registered plugins concurrently via `join_all`, merges results, sorts by `Score` descending, truncates to `max_results`.

---

## UI Architecture (iced 0.14 — Elm model)

- **State:** `KLauncherApp` holds engine ref, launcher ref, query string, results, selected index, appearance config.
- **Messages:** `QueryChanged`, `ResultsReady`, `KeyPressed`
- **Update:**
  - `QueryChanged` → spawns debounced async task (50 ms) → `ResultsReady`
  - Epoch guard prevents stale results from out-of-order responses
- **View:** search bar + scrollable result list with icon support (SVG/raster)
- **Subscription:** keyboard events — `Esc` = quit, `Enter` = launch, arrows = navigate
- **Window:** transparent, undecorated, centered (Wayland-compatible)

---

## Frecency (plugin-apps)

`FrecencyStore` records app launches by ID. On empty query, returns top-5 frecent apps instead of search results.

---

## Configuration

`~/.config/k-launcher/config.toml` — sections: `[window]`, `[appearance]`, `[search]`, `[plugins]`.

All fields have sane defaults; a missing file yields defaults without error.
