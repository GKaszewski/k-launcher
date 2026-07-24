# Configuration

Config file: `~/.config/k-launcher/config.toml`

The file is optional — all fields have defaults and missing sections fall back to defaults automatically. If the file exists but has a parse error, a warning is logged and defaults are used.

See [config.example.toml](../config.example.toml) for a ready-to-copy template with all options.

## Sections

### [window]

| Field | Type | Default | Description |
|---|---|---|---|
| `width` | float | `600.0` | Window width in pixels |
| `height` | float | `400.0` | Window height in pixels |
| `decorations` | bool | `false` | Show window title bar |
| `transparent` | bool | `true` | Enable background transparency |
| `resizable` | bool | `false` | Allow manual resizing |

### [appearance]

| Field | Type | Default | Description |
|---|---|---|---|
| `background_rgba` | [R,G,B,A] | `[20, 20, 30, 0.9]` | Main background color |
| `border_rgba` | [R,G,B,A] | `[229, 125, 33, 1.0]` | Border/accent color |
| `border_width` | float | `1.0` | Border thickness |
| `border_radius` | float | `8.0` | Window corner radius |
| `search_font_size` | float | `18.0` | Search input font size |
| `title_size` | float | `15.0` | Result title font size |
| `desc_size` | float | `12.0` | Result description font size |
| `row_radius` | float | `4.0` | Result row corner radius |
| `placeholder` | string | `"Search apps, ..."` | Search input placeholder |
| `selected_row_rgba` | [R,G,B,A] | `[229, 125, 33, 1.0]` | Selected result background |
| `unselected_row_rgba` | [R,G,B,A] | `[255, 255, 255, 0.07]` | Unselected result background |
| `description_rgba` | [R,G,B,A] | `[210, 215, 230, 1.0]` | Description text color |
| `no_results_rgba` | [R,G,B,A] | `[180, 180, 200, 0.5]` | "No results" text color |
| `error_rgba` | [R,G,B,A] | `[255, 80, 80, 1.0]` | Error text color |
| `icon_size` | float | `24.0` | App icon size in pixels |

#### RGBA format

Colors use `[R, G, B, A]` arrays where R/G/B are 0–255 (as floats) and A is 0.0–1.0 (opacity). Values are clamped to valid ranges.

### [search]

| Field | Type | Default | Description |
|---|---|---|---|
| `max_results` | integer | `8` | Maximum results shown |
| `debounce_ms` | integer | `50` | Milliseconds to wait after last keystroke before searching |
| `frecency_compact_threshold` | integer | `50` | Frecency log entries before compacting to snapshot |

### [plugins]

| Field | Type | Default | Description |
|---|---|---|---|
| `calc` | bool | `true` | Calculator plugin |
| `cmd` | bool | `true` | Shell command plugin |
| `files` | bool | `true` | File browser plugin |
| `apps` | bool | `true` | Application search plugin |

### [[plugins.external]]

Repeatable block for external plugins.

| Field | Type | Default | Description |
|---|---|---|---|
| `name` | string | required | Plugin display name |
| `path` | string | required | Path to plugin executable |
| `args` | string[] | `[]` | Arguments to pass |
| `timeout_secs` | integer | `5` | Search timeout per query |

### [logging]

| Field | Type | Default | Description |
|---|---|---|---|
| `max_log_files` | integer | `7` | Daily log files to keep |

Logs are stored in `~/.local/share/k-launcher/logs/`.

### [terminal]

| Field | Type | Default | Description |
|---|---|---|---|
| `cmd` | string | auto-detect | Terminal emulator for `>` commands |

If unset, detected from `$TERM_CMD`, `$TERMINAL`, or PATH (foot, kitty, alacritty, wezterm, konsole, xterm).

Example: `cmd = "kitty -e"`
