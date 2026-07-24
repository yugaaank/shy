# shy

A lightweight Hyprland daemon that auto-hides floating windows when you switch to tiled windows, and restores them when you switch back.

Think of it like macOS's window behavior — floating utility windows stay out of the way until you need them.

## Features

- **Auto-hide on focus switch** — floating windows slide below the screen when you Alt+Tab to a tiled window
- **Instant restore** — Alt+Tab back to a floating window and it reappears exactly where you left it
- **Smart hover detection** — moving your mouse over a tiled window won't accidentally hide your floating windows (only keyboard/switcher focus changes trigger hiding)
- **Zero animations** — all window movements are instantaneous, no sliding or fading
- **Cursor warp** — cursor automatically moves to the center of a restored floating window for seamless interaction
- **Position tracking** — manually drag or resize your floating windows and shy remembers the new position
- **Window switcher aware** — detects Alt+Tab switchers (snappy-switcher, rofi, walker, etc.) for reliable focus change detection
- **Per-window ignore list** — exclude specific windows by class or title
- **Multi-monitor support** — works across multiple monitors
- **Event-driven** — listens to Hyprland's IPC socket (no polling, zero CPU when idle)

## Requirements

- **Hyprland ≥ 0.56** (uses Noctalia Lua IPC dispatchers)
- **Rust toolchain** (for building)

## Install

```bash
git clone https://github.com/yugaaank/shy.git
cd shy
cargo install --path .
```

## Usage

```bash
shy
```

For debug logging:

```bash
RUST_LOG=info shy
```

### Hyprland Autostart

Add to your Hyprland config (Noctalia Lua):

```lua
hl.on("hyprland.start", function()
    hl.exec_cmd("~/.cargo/bin/shy")
end)
```

Or in classic Hyprland config:

```conf
exec-once = ~/.cargo/bin/shy
```

## Configuration

Create `~/.config/shy/config.toml`:

```toml
# Pixels below the monitor edge to place hidden windows
hide_offset = 300

# Enable debug logging
debug = false

# Windows matching these class/title substrings are never managed by shy
ignore = ["rofi", "waybar", "walker", "hyprpanel", "noctalia"]

# Ignore mouse hover focus changes (only hide on keyboard/switcher focus)
# Set to false if you want mouse hover to also trigger hiding
ignore_hover = true
```

### Config Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `hide_offset` | `i32` | `300` | Distance below the monitor bottom edge to place hidden windows |
| `debug` | `bool` | `false` | Enable debug logging |
| `ignore` | `string[]` | `[]` | Window class/title substrings to exclude from management |
| `ignore_hover` | `bool` | `true` | When `true`, only keyboard/switcher focus changes trigger hiding (mouse hover is ignored) |

## How It Works

1. **Connects** to Hyprland's UNIX domain socket (`.socket.sock` and `.socket2.sock`)
2. **Scans** all existing floating windows on startup and registers them
3. **Listens** for IPC events (`activewindow`, `openwindow`, `closewindow`, `changefloating`, `openlayer`, `closelayer`)
4. **On tiled focus** — queries each floating window's current position from Hyprland, saves it, then moves the window below the screen
5. **On floating focus** — moves the window back to its saved position and warps the cursor to its center
6. **Switcher detection** — tracks `openlayer`/`closelayer` events for window switchers to distinguish keyboard focus from mouse hover

## Architecture

```
src/
├── main.rs       — entry point, event loop
├── handler.rs    — event dispatcher, switcher state, hover filtering
├── registry.rs   — window tracking, hide/restore logic
├── monitor.rs    — monitor geometry cache
├── ipc.rs        — Hyprland UNIX socket IPC
├── config.rs     — TOML config loading
└── types.rs      — data structures
```

## License

MIT
