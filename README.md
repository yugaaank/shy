<div align="center">

<img src="assets/logo.png" width="140" />

# shy

**Auto-hide floating windows for Hyprland**

[![Hyprland](https://img.shields.io/badge/Hyprland-%E2%89%A5_0.56-blue?style=flat-square&logo=wayland&logoColor=white)](https://hyprland.org)
[![Rust](https://img.shields.io/badge/Rust-stable-orange?style=flat-square&logo=rust&logoColor=white)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/License-MIT-green?style=flat-square)](LICENSE)
[![GitHub Stars](https://img.shields.io/github/stars/yugaaank/shy?style=flat-square&color=yellow)](https://github.com/yugaaank/shy)

*Floating windows that know when to get out of the way.*

[Installation](#install) · [Configuration](#configuration) · [How It Works](#how-it-works)

---

</div>

## Why shy?

Tiling window managers are great — until you have floating windows cluttering your view. **shy** makes floating windows disappear when you don't need them and reappear instantly when you do, with zero config required.

Switch to a tiled window → floating windows vanish below the screen.
Switch back → they're right where you left them. Instantly.

## Features

- **🪟 Auto-hide on focus switch** — floating windows slide below the screen when you Alt+Tab to a tiled window
- **⚡ Instant restore** — switch back to a floating window and it reappears exactly where you left it
- **🖱️ Smart hover detection** — mouse hover won't accidentally hide your floating windows, only keyboard/switcher focus changes trigger hiding
- **🚫 Zero animations** — all window movements are instantaneous, no sliding or fading
- **🎯 Cursor warp** — cursor automatically moves to the center of a restored window
- **📐 Position tracking** — drag or resize your floating windows freely, shy remembers the new position
- **🖥️ Multi-monitor support** — works across all your monitors
- **⏱️ Event-driven** — listens to Hyprland's IPC socket, zero CPU when idle

## Install

```bash
git clone https://github.com/yugaaank/shy.git
cd shy
cargo install --path .
```

## Quick Start

```bash
shy
```

Add to your Hyprland config for autostart:

<details>
<summary><b>Noctalia (Lua config)</b></summary>

```lua
hl.on("hyprland.start", function()
    hl.exec_cmd("~/.cargo/bin/shy")
end)
```

</details>

<details>
<summary><b>Classic (hyprlang config)</b></summary>

```conf
exec-once = ~/.cargo/bin/shy
```

</details>

## Configuration

Create `~/.config/shy/config.toml`:

```toml
hide_offset = 300
debug = false
ignore = ["rofi", "waybar", "walker", "hyprpanel", "noctalia"]
ignore_hover = true
```

<details>
<summary><b>Config Reference</b></summary>

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `hide_offset` | `int` | `300` | Pixels below the monitor edge to place hidden windows |
| `debug` | `bool` | `false` | Enable debug logging |
| `ignore` | `string[]` | `[]` | Window class/title substrings to exclude from management |
| `ignore_hover` | `bool` | `true` | Only keyboard/switcher focus triggers hiding (mouse hover ignored) |

</details>

## How It Works

```
                  ┌─────────────┐
                  │  Hyprland   │
                  │  Socket IPC │
                  └──────┬──────┘
                         │ events
                         ▼
              ┌──────────────────────┐
              │      shy daemon      │
              ├──────────┬───────────┤
              │ Handler  │ Registry  │
              │ (events) │ (windows) │
              └──────────┴───────────┘
                         │
            ┌────────────┼────────────┐
            ▼            ▼            ▼
    ┌──────────┐  ┌────────────┐  ┌──────────┐
    │  Focus   │  │  Position  │  │  Monitor │
    │ Tracking │  │  Save/Load │  │  Cache   │
    └──────────┘  └────────────┘  └──────────┘
```

1. **Connects** to Hyprland's UNIX domain sockets
2. **Scans** all existing floating windows on startup
3. **Listens** for `activewindow`, `openwindow`, `closewindow`, `changefloating`, `openlayer`, `closelayer` events
4. **On tiled focus** — queries each floating window's real position, saves it, moves the window below the screen
5. **On floating focus** — restores the window to its saved position, warps cursor to center

## Architecture

```
src/
├── main.rs       Entry point, event loop
├── handler.rs    Event dispatch, switcher state, hover filtering
├── registry.rs   Window tracking, hide/restore logic
├── monitor.rs    Monitor geometry cache
├── ipc.rs        Hyprland UNIX socket IPC
├── config.rs     TOML config loading
└── types.rs      Data structures
```

<div align="center">

---

**shy** is built for [Hyprland](https://hyprland.org) · MIT License

</div>
