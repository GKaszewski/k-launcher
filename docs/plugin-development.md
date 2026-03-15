# Plugin Development

Plugins are Rust crates that implement the `Plugin` trait from `k-launcher-kernel`. They run concurrently — the kernel fans out every query to all enabled plugins and merges results by score.

> Note: plugins are compiled into the binary at build time. There is no dynamic loading support yet.

## Step-by-Step

### 1. Create a new crate in the workspace

```bash
cargo new --lib crates/plugins/plugin-hello
```

Add it to the workspace root `Cargo.toml`:

```toml
[workspace]
members = [
    # ...existing members...
    "crates/plugins/plugin-hello",
]
```

### 2. Add dependencies

`crates/plugins/plugin-hello/Cargo.toml`:

```toml
[dependencies]
k-launcher-kernel = { path = "../../k-launcher-kernel" }
async-trait = "0.1"
```

### 3. Implement the `Plugin` trait

`crates/plugins/plugin-hello/src/lib.rs`:

```rust
use async_trait::async_trait;
use k_launcher_kernel::{
    LaunchAction, Plugin, PluginName, ResultId, ResultTitle, Score, SearchResult,
};

pub struct HelloPlugin;

impl HelloPlugin {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Plugin for HelloPlugin {
    fn name(&self) -> PluginName {
        "hello"
    }

    async fn search(&self, query: &str) -> Vec<SearchResult> {
        if !query.starts_with("hello") {
            return vec![];
        }

        vec![SearchResult {
            id: ResultId::new("hello:world"),
            title: ResultTitle::new("Hello, World!"),
            description: Some("A greeting from the hello plugin".to_string()),
            icon: None,
            score: Score::new(80),
            action: LaunchAction::CopyToClipboard("Hello, World!".to_string()),
            on_select: None,
        }]
    }
}
```

### 4. Wire up in main.rs

`crates/k-launcher/src/main.rs` — add alongside the existing plugins:

```rust
use plugin_hello::HelloPlugin;

// inside main():
plugins.push(Arc::new(HelloPlugin::new()));
```

Also add the dependency to `crates/k-launcher/Cargo.toml`:

```toml
[dependencies]
plugin-hello = { path = "../plugins/plugin-hello" }
```

---

## Reference

### `SearchResult` Fields

| Field | Type | Description |
|-------|------|-------------|
| `id` | `ResultId` | Unique stable ID (e.g. `"apps:firefox"`) |
| `title` | `ResultTitle` | Primary display text |
| `description` | `Option<String>` | Secondary line shown below title |
| `icon` | `Option<String>` | Icon name or path (currently unused in renderer) |
| `score` | `Score(u32)` | Sort priority — higher wins |
| `action` | `LaunchAction` | What happens on `Enter` |
| `on_select` | `Option<Arc<dyn Fn()>>` | Optional side-effect on selection (e.g. frecency bump) |

### `LaunchAction` Variants

| Variant | Behavior |
|---------|----------|
| `SpawnProcess(String)` | Launch a process directly (e.g. app exec string) |
| `SpawnInTerminal(String)` | Run command inside a terminal emulator |
| `OpenPath(String)` | Open a file or directory with `xdg-open` |
| `CopyToClipboard(String)` | Copy text to clipboard |
| `Custom(Arc<dyn Fn()>)` | Arbitrary closure |

### Scoring Guidance

| Score range | Match type |
|-------------|-----------|
| 100 | Exact match |
| 90–99 | Calc/command result (always relevant) |
| 80 | Prefix match |
| 70 | Abbreviation match |
| 60 | Substring match |
| 50 | Keyword / loose match |

The kernel sorts all results from all plugins by score descending and truncates to `max_results` (default: 8).
