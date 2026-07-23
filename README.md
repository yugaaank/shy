# shy

Hyprland daemon that auto-hides inactive floating windows.

Floating windows are moved off-screen when you switch to a tiled window, and restored when you switch back. Similar to how Windows handles utility windows.

## Install

```bash
cargo install --path .
```

## Usage

```bash
shy
```

Add to your Hyprland config:

```lua
hl.exec_cmd("shy")
```

## Config

Copy `config.toml` to `~/.config/shy/config.toml`:

```toml
hide_offset = 300
debug = false
ignore = ["rofi", "waybar", "walker", "hyprpanel", "noctalia"]
```

## How it works

- Listens to Hyprland socket2 events (no polling)
- Tracks floating windows in a registry
- Moves hidden windows to `monitor_x + monitor_width + hide_offset`
- Restores to exact saved position on focus
