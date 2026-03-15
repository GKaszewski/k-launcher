# Configuration

Config file: `~/.config/k-launcher/config.toml`

The file is optional — all fields have defaults and missing sections fall back to defaults automatically. Create it manually if you want to customize behavior.

## Full Annotated Example

```toml
[window]
width       = 600.0   # window width in logical pixels
height      = 400.0   # window height in logical pixels
decorations = false   # show window title bar / frame
transparent = true    # allow background transparency
resizable   = false   # allow manual resizing

[appearance]
# RGBA: r/g/b are 0–255 as floats, a is 0.0–1.0
background_rgba  = [20.0, 20.0, 30.0, 0.9]    # main background
border_rgba      = [229.0, 125.0, 33.0, 1.0]  # accent/border color
border_width     = 1.0    # border thickness in pixels
border_radius    = 8.0    # corner radius of the window
search_font_size = 18.0   # font size of the search input
title_size       = 15.0   # font size of result titles
desc_size        = 12.0   # font size of result descriptions
row_radius       = 4.0    # corner radius of result rows
placeholder      = "Search..."  # search input placeholder text

[search]
max_results = 8   # maximum results shown at once

[plugins]
calc  = true   # math expression evaluator
cmd   = true   # shell command runner (> prefix)
files = true   # filesystem browser (/ or ~/ prefix)
apps  = true   # XDG application launcher

# External (dynamic) plugins — repeat block for each plugin
[[plugins.external]]
name = "my-plugin"              # display name / identifier
path = "/path/to/my-plugin"    # path to executable
args = []                       # optional extra arguments
```

## RGBA Format

Colors use `[r, g, b, a]` arrays where:
- `r`, `g`, `b` — red, green, blue channels as floats **0.0–255.0**
- `a` — alpha (opacity) as a float **0.0–1.0**

Example — semi-transparent white: `[255.0, 255.0, 255.0, 0.5]`
