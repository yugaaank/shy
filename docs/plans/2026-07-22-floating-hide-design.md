# Floating Hide - Design

Hyprland daemon that auto-hides inactive floating windows by moving them off-screen, restoring on focus.

## Architecture

- Raw Unix sockets (socket1 for commands, socket2 for events)
- `std::thread` event loop, no async runtime
- 5 dependencies: serde, serde_json, toml, log, env_logger

## Files

```
src/
  main.rs       # entry, socket discovery, event loop
  config.rs     # TOML config
  ipc.rs        # socket1/socket2 communication
  registry.rs   # window map, geometry save/restore
  handler.rs    # event routing logic
  monitor.rs    # monitor geometry cache
```

## IPC

- Socket2: subscribe to events, read newline-delimited lines
- Socket1: send commands, read JSON response (one connection per command)
- Hyprland Instance Signature from `$HYPRLAND_INSTANCE_SIGNATURE`

## Events

| Event | Action |
|-------|--------|
| activewindowv2 | Hide/show floating windows |
| openwindow | Register if floating |
| closewindow | Remove from registry |
| movewindow | Update saved coords if visible |
| changefloating | Register/deregister |
| monitoradded/removed | Refresh monitor cache |

## Hide/Restore

- Hide: `hyprctl dispatch movewindow pixel <off_screen_x> <saved_y> addr:<addr>`
- Restore: `hyprctl dispatch movewindow pixel <saved_x> <saved_y> addr:<addr>`
- Off-screen x = monitor.x + monitor.width + hide_offset

## Filtering

Case-insensitive substring match on class/title against ignore list.
Layer surfaces, bars, notifications etc. are never registered.

## Config

```toml
hide_offset = 300
debug = false
ignore = ["rofi", "waybar", "walker", "hyprpanel", "noctalia"]
```

Path: `~/.config/floating-hide/config.toml`
