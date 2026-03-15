# Usage

## Running

```bash
k-launcher
```

## Keybinds

| Key | Action |
|-----|--------|
| Type | Filter results |
| `↑` / `↓` | Navigate list |
| `Enter` | Launch selected result |
| `Escape` | Close launcher |

## Built-in Plugins

### Apps

Type any app name to search installed applications. An empty query shows your most frequently launched apps (frecency-ranked top results).

### Calc

Type a math expression — the result appears instantly and is copied to clipboard on `Enter`.

```
2^10 + 5       → 1029
sqrt(144)      → 12
sin(pi / 2)   → 1
```

### Shell Command

Prefix your input with `>` to run a shell command in a terminal:

```
> echo hello
> htop
```

### Files

Start your query with `/` or `~/` to browse the filesystem:

```
/etc/hosts
~/Documents/report.pdf
```
