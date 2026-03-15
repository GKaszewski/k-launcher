# K-Launcher

K-Launcher is a lightweight, GPU-accelerated command palette for Linux (Wayland/X11), macOS, and Windows. It reimagines the "Spotlight" experience through the lens of Frutiger Aero—focusing on gloss, glass, and skeuomorphism—powered by a non-blocking, multi-threaded Rust kernel.

## Core Philosophy

- Zero Webview: No Chromium, no Electron. Every pixel is rendered via WGPU (Iced) for sub-5ms input-to-render latency.

- Async-First: Search queries never block the UI. If the file-searcher is indexing, the calculator still feels instant.

- The "Aero" Standard: Deep support for Gaussian blur (via Layer Shell), linear gradients, and high-gloss textures.

## High-Level Architecture

We are utilizing a "Hub-and-Spoke" model within a Cargo Workspace. The k-launcher-kernel acts as the central hub, dispatching user input to various "Spokes" (Plugins).

### The Crate Hierarchy

| Crate                      | Responsibility                                            | Key Dependencies                       |
| -------------------------- | --------------------------------------------------------- | -------------------------------------- |
| **`k-launcher`**           | The entry-point binary. Glues everything together.        | `k-launcher-ui`, `k-launcher-kernel`   |
| **`k-launcher-ui`**        | The Iced-based view layer. Handles animations/theming.    | `iced`, `lyon` (for vector paths)      |
| **`k-launcher-kernel`**    | The "Brain." Manages state, history, and plugin dispatch. | `tokio`, `tracing`                     |
| **`k-launcher-os-bridge`** | OS-specific windowing (Layer Shell for Wayland, Win32).   | `iced_layershell`, `raw-window-handle` |
| **`plugins/*`**            | Individual features (Calc, Files, Apps, Web).             | `plugin-api` (Shared traits)           |

## Data & Communication Flow

K-Launcher operates on an Event loop.

```
sequenceDiagram
    participant User
    participant UI as k-launcher-ui
    participant Kernel as k-launcher-kernel
    participant Plugins as plugin-file-search

    User->>UI: Types "p"
    UI->>Kernel: QueryUpdate("p")
    par Parallel Search
        Kernel->>Plugins: async search("p")
        Plugins-->>Kernel: List<SearchResult>
    end
    Kernel->>UI: NewResults(Vec)
    UI-->>User: Render Glass Result List
```

## Technical Specifications

To ensure "Plug and Play" capability, all features must implement the `Plugin` trait. This allows the user to swap the default `file-searcher` for something like `fzf` or `plocate` without recompiling the UI.

To achieve the 2000s aesthetic without a browser:

- Background Blur: On Wayland, we request blur through the org_kde_kwin_blur or fractional-scale protocols.
- Shaders: We will use Iced’s canvas to draw glossy "shine" overlays that respond to mouse hovering.
- Icons: We will prefer .svg and .png with high-depth shadows over flat icon fonts.
