# Plugin Development

Plugins are queried concurrently — the kernel fans out every search to all enabled plugins and merges results by score.

There are two kinds of plugins:

- **External plugins** — executables that speak a JSON protocol over stdin/stdout. Any language, no compilation required. Recommended for community plugins.
- **Built-in plugins** — Rust crates compiled into the binary. For performance-critical or tightly integrated plugins.

---

## External Plugins

An external plugin is any executable that:

1. Reads a JSON object from stdin (one line per query)
2. Writes a JSON array of results to stdout (one line per response)

### Protocol

**Input** (one line, newline-terminated):
```json
{"query": "firefox"}
```

**Output** (one line, newline-terminated):
```json
[{"id":"app-firefox","title":"Firefox","score":80,"description":"Web Browser","action":{"type":"SpawnProcess","cmd":"firefox"}}]
```

The process is kept alive between queries — do **not** exit after each response.

### Action types

| `"type"` | Extra fields | Behavior |
|----------|-------------|---------|
| `SpawnProcess` | `"cmd"` | Launch process directly |
| `CopyToClipboard` | `"text"` | Copy text to clipboard |
| `OpenPath` | `"path"` | Open file/dir with xdg-open |

### Optional result fields

| Field | Type | Description |
|-------|------|-------------|
| `description` | `string` | Secondary line shown below title |
| `icon` | `string` | Icon path (future use) |

### Enabling an external plugin

In `~/.config/k-launcher/config.toml`:

```toml
[[plugins.external]]
name = "my-plugin"
path = "/usr/lib/k-launcher/plugins/my-plugin"
args = []        # optional
```

Multiple `[[plugins.external]]` blocks are supported.

### Example: shell plugin

```bash
#!/usr/bin/env bash
# A plugin that greets the user.
while IFS= read -r line; do
    query=$(echo "$line" | python3 -c "import sys,json; print(json.load(sys.stdin)['query'])")
    if [[ "$query" == hello* ]]; then
        echo '[{"id":"greet","title":"Hello, World!","score":80,"action":{"type":"CopyToClipboard","text":"Hello, World!"}}]'
    else
        echo '[]'
    fi
done
```

### Example: Python plugin

```python
#!/usr/bin/env python3
import sys, json

for line in sys.stdin:
    query = json.loads(line)["query"]
    results = []
    if query.startswith("hello"):
        results.append({
            "id": "greet",
            "title": "Hello, World!",
            "score": 80,
            "action": {"type": "CopyToClipboard", "text": "Hello, World!"},
        })
    print(json.dumps(results), flush=True)
```

---

## Built-in Plugins (compiled-in)

Built-in plugins implement the `Plugin` trait from `k-launcher-kernel` as Rust crates compiled into the binary.

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
use k_launcher_kernel::{LaunchAction, Plugin, ResultId, ResultTitle, Score, SearchResult};

pub struct HelloPlugin;

impl HelloPlugin {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Plugin for HelloPlugin {
    fn name(&self) -> &str {
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
