# Floating Hide Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a Hyprland daemon that auto-hides inactive floating windows by moving them off-screen and restores them on focus.

**Architecture:** Raw Unix sockets (socket1 for commands, socket2 for events), std::thread event loop, JSON IPC. Single binary, no async runtime.

**Tech Stack:** Rust, serde, serde_json, toml, log, env_logger

---

### Task 1: Project Scaffold

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`
- Create: `config.toml`

**Step 1: Create Cargo.toml**

```toml
[package]
name = "floating-hide"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"
log = "0.4"
env_logger = "0.11"
```

**Step 2: Create src/main.rs skeleton**

```rust
fn main() {
    env_logger::init();
    log::info!("floating-hide starting");
}
```

**Step 3: Create default config.toml**

```toml
hide_offset = 300
debug = false
ignore = ["rofi", "waybar", "walker", "hyprpanel", "noctalia"]
```

**Step 4: Commit**

```bash
git init
git add -A
git commit -m "chore: project scaffold"
```

---

### Task 2: Config Module

**Files:**
- Create: `src/config.rs`

**Step 1: Write config.rs**

```rust
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub hide_offset: i32,
    pub debug: bool,
    pub ignore: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            hide_offset: 300,
            debug: false,
            ignore: vec![],
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let paths = [
            dirs_config_path(),
            Some(PathBuf::from("config.toml")),
        ];
        for path in paths.into_iter().flatten() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(config) = toml::from_str::<Config>(&content) {
                    log::info!("Loaded config from {}", path.display());
                    return config;
                }
            }
        }
        log::info!("Using default config");
        Config::default()
    }

    pub fn should_ignore(&self, class: &str, title: &str) -> bool {
        let class_lower = class.to_lowercase();
        let title_lower = title.to_lowercase();
        self.ignore.iter().any(|s| {
            let s_lower = s.to_lowercase();
            class_lower.contains(&s_lower) || title_lower.contains(&s_lower)
        })
    }
}

fn dirs_config_path() -> Option<PathBuf> {
    let home = std::env::var("HOME").ok()?;
    Some(PathBuf::from(home).join(".config/floating-hide/config.toml"))
}
```

**Step 2: Wire into main.rs**

```rust
mod config;

fn main() {
    env_logger::init();
    let cfg = config::Config::load();
    log::info!("floating-hide starting (debug={})", cfg.debug);
}
```

**Step 3: Commit**

```bash
git add src/config.rs src/main.rs
git commit -m "feat: TOML config loading"
```

---

### Task 3: IPC Module - Socket Discovery

**Files:**
- Create: `src/ipc.rs`

**Step 1: Write socket discovery + command function**

```rust
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;

pub struct HyprIpc {
    socket1: String, // command socket path
}

impl HyprIpc {
    pub fn connect() -> Self {
        let his = std::env::var("HYPRLAND_INSTANCE_SIGNATURE")
            .expect("HYPRLAND_INSTANCE_SIGNATURE not set - is Hyprland running?");
        let runtime = std::env::var("XDG_RUNTIME_DIR")
            .unwrap_or_else(|_| format!("/run/user/{}", std::process::id()));

        let socket1 = format!("{}/hypr/{}/.socket.sock", runtime, his);
        let socket2 = format!("{}/hypr/{}/.socket2.sock", runtime, his);

        log::info!("Socket1: {}", socket1);
        log::info!("Socket2: {}", socket2);

        HyprIpc { socket1 }
    }

    pub fn socket2_path(&self) -> String {
        let his = std::env::var("HYPRLAND_INSTANCE_SIGNATURE").unwrap();
        let runtime = std::env::var("XDG_RUNTIME_DIR")
            .unwrap_or_else(|_| format!("/run/user/{}", std::process::id()));
        format!("{}/hypr/{}/.socket2.sock", runtime, his)
    }

    pub fn send_command(&self, command: &str) -> String {
        let mut stream = UnixStream::connect(&self.socket1)
            .expect("Failed to connect to Hyprland socket1");
        stream.write_all(command.as_bytes()).expect("Failed to write");
        stream.write_all(b"\n").expect("Failed to write newline");
        stream.shutdown(std::net::Shutdown::Write).expect("Failed to shutdown write");

        let mut reader = BufReader::new(&stream);
        let mut response = String::new();
        reader.read_line(&mut response).expect("Failed to read response");
        response.trim().to_string()
    }
}
```

**Step 2: Commit**

```bash
git add src/ipc.rs
git commit -m "feat: Hyprland IPC socket discovery and command sending"
```

---

### Task 4: IPC Module - Event Listener

**Files:**
- Modify: `src/ipc.rs`

**Step 1: Add event listener to ipc.rs**

Append to the HyprIpc impl:

```rust
pub fn listen_events(&self, tx: std::sync::mpsc::Sender<String>) {
    let path = self.socket2_path();
    let stream = UnixStream::connect(&path)
        .expect("Failed to connect to Hyprland socket2");
    let reader = BufReader::new(stream);

    log::info!("Listening for events on socket2");

    for line in reader.lines() {
        match line {
            Ok(line) => {
                if !line.is_empty() {
                    if tx.send(line).is_err() {
                        break; // receiver dropped
                    }
                }
            }
            Err(e) => {
                log::error!("Socket2 read error: {}", e);
                break;
            }
        }
    }
}
```

**Step 2: Commit**

```bash
git add src/ipc.rs
git commit -m "feat: socket2 event listener"
```

---

### Task 5: Data Types

**Files:**
- Create: `src/types.rs`

**Step 1: Write types.rs**

```rust
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct WindowEntry {
    pub addr: String,
    pub workspace: i32,
    pub monitor: String,
    pub saved_x: i32,
    pub saved_y: i32,
    pub width: i32,
    pub height: i32,
    pub hidden: bool,
}

#[derive(Debug, Clone)]
pub struct MonitorGeometry {
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Deserialize)]
pub struct HyprClient {
    pub address: String,
    pub workspace: HyprWorkspace,
    pub monitor: String,
    pub at: [i32; 2],
    pub size: [i32; 2],
    pub floating: bool,
    pub mapped: bool,
    pub hidden: bool,
    #[serde(rename = "focusHistoryID")]
    pub focus_history_id: i32,
}

#[derive(Debug, Deserialize)]
pub struct HyprWorkspace {
    pub id: i32,
}

#[derive(Debug, Deserialize)]
pub struct HyprMonitor {
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}
```

**Step 2: Add mod to main.rs**

```rust
mod config;
mod ipc;
mod types;
```

**Step 3: Commit**

```bash
git add src/types.rs src/main.rs
git commit -m "feat: data types for windows, monitors, IPC responses"
```

---

### Task 6: Monitor Cache

**Files:**
- Create: `src/monitor.rs`

**Step 1: Write monitor.rs**

```rust
use crate::ipc::HyprIpc;
use crate::types::MonitorGeometry;
use std::collections::HashMap;

pub struct MonitorCache {
    pub monitors: HashMap<String, MonitorGeometry>,
}

impl MonitorCache {
    pub fn new() -> Self {
        MonitorCache {
            monitors: HashMap::new(),
        }
    }

    pub fn refresh(&mut self, ipc: &HyprIpc) {
        let response = ipc.send_command("monitors -j");
        match serde_json::from_str::<Vec<crate::types::HyprMonitor>>(&response) {
            Ok(monitors) => {
                self.monitors.clear();
                for m in monitors {
                    log::info!("Monitor: {} {}x{}+{},{}", m.name, m.width, m.height, m.x, m.y);
                    self.monitors.insert(
                        m.name.clone(),
                        MonitorGeometry {
                            name: m.name,
                            x: m.x,
                            y: m.y,
                            width: m.width,
                            height: m.height,
                        },
                    );
                }
            }
            Err(e) => log::error!("Failed to parse monitors: {}", e),
        }
    }

    pub fn get_offscreen_x(&self, monitor_name: &str, offset: i32) -> i32 {
        self.monitors
            .get(monitor_name)
            .map(|m| m.x + m.width + offset)
            .unwrap_or(10000) // fallback
    }
}
```

**Step 2: Add mod to main.rs**

```rust
mod config;
mod ipc;
mod monitor;
mod types;
```

**Step 3: Commit**

```bash
git add src/monitor.rs src/main.rs
git commit -m "feat: monitor geometry cache"
```

---

### Task 7: Window Registry

**Files:**
- Create: `src/registry.rs`

**Step 1: Write registry.rs**

```rust
use crate::ipc::HyprIpc;
use crate::monitor::MonitorCache;
use crate::types::WindowEntry;
use std::collections::HashMap;

pub struct Registry {
    pub windows: HashMap<String, WindowEntry>,
}

impl Registry {
    pub fn new() -> Self {
        Registry {
            windows: HashMap::new(),
        }
    }

    pub fn scan(&mut self, ipc: &HyprIpc, monitors: &MonitorCache, hide_offset: i32) {
        let response = ipc.send_command("clients -j");
        match serde_json::from_str::<Vec<crate::types::HyprClient>>(&response) {
            Ok(clients) => {
                for c in clients {
                    if c.floating && c.mapped && !c.hidden {
                        let entry = WindowEntry {
                            addr: c.address.clone(),
                            workspace: c.workspace.id,
                            monitor: c.monitor.clone(),
                            saved_x: c.at[0],
                            saved_y: c.at[1],
                            width: c.size[0],
                            height: c.size[1],
                            hidden: false,
                        };
                        log::info!("Registered window {} on {}", c.address, c.monitor);
                        self.windows.insert(c.address, entry);
                    }
                }
            }
            Err(e) => log::error!("Failed to parse clients: {}", e),
        }
    }

    pub fn register(
        &mut self,
        addr: &str,
        workspace: i32,
        monitor: &str,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) {
        let entry = WindowEntry {
            addr: addr.to_string(),
            workspace,
            monitor: monitor.to_string(),
            saved_x: x,
            saved_y: y,
            width,
            height,
            hidden: false,
        };
        log::info!("Registered new window {}", addr);
        self.windows.insert(addr.to_string(), entry);
    }

    pub fn remove(&mut self, addr: &str) {
        if self.windows.remove(addr).is_some() {
            log::info!("Removed window {}", addr);
        }
    }

    pub fn hide(&mut self, addr: &str, ipc: &HyprIpc, monitors: &MonitorCache, hide_offset: i32) {
        if let Some(entry) = self.windows.get_mut(addr) {
            if entry.hidden {
                return; // already hidden
            }
            // Save current position only once
            entry.hidden = true;
            let off_x = monitors.get_offscreen_x(&entry.monitor, hide_offset);
            let cmd = format!("dispatch movewindow pixel {} {} addr:{}", off_x, entry.saved_y, addr);
            ipc.send_command(&cmd);
            log::info!("Hidden window {} to x={}", addr, off_x);
        }
    }

    pub fn restore(&mut self, addr: &str, ipc: &HyprIpc) {
        if let Some(entry) = self.windows.get_mut(addr) {
            if !entry.hidden {
                return; // already visible
            }
            entry.hidden = false;
            let cmd = format!("dispatch movewindow pixel {} {} addr:{}", entry.saved_x, entry.saved_y, addr);
            ipc.send_command(&cmd);
            log::info!("Restored window {} to {},{}", addr, entry.saved_x, entry.saved_y);
        }
    }

    pub fn restore_all(&mut self, ipc: &HyprIpc) {
        let addrs: Vec<String> = self.windows.keys().cloned().collect();
        for addr in addrs {
            self.restore(&addr, ipc);
        }
    }

    pub fn hide_all(&mut self, ipc: &HyprIpc, monitors: &MonitorCache, hide_offset: i32) {
        let addrs: Vec<String> = self.windows.keys().cloned().collect();
        for addr in addrs {
            self.hide(&addr, ipc, monitors, hide_offset);
        }
    }

    pub fn update_position(&mut self, addr: &str, x: i32, y: i32) {
        if let Some(entry) = self.windows.get_mut(addr) {
            if !entry.hidden {
                entry.saved_x = x;
                entry.saved_y = y;
            }
        }
    }

    pub fn update_floating(&mut self, addr: &str, floating: bool, ipc: &HyprIpc, monitors: &MonitorCache) {
        if floating {
            // Will be registered by openwindow or we need to fetch info
            let response = ipc.send_command(&format!("clients -j"));
            if let Ok(clients) = serde_json::from_str::<Vec<crate::types::HyprClient>>(&response) {
                if let Some(c) = clients.iter().find(|c| c.address == addr) {
                    self.register(
                        addr,
                        c.workspace.id,
                        &c.monitor,
                        c.at[0],
                        c.at[1],
                        c.size[0],
                        c.size[1],
                    );
                }
            }
        } else {
            self.remove(addr);
        }
    }
}
```

**Step 2: Add mod to main.rs**

```rust
mod config;
mod ipc;
mod monitor;
mod registry;
mod types;
```

**Step 3: Commit**

```bash
git add src/registry.rs src/main.rs
git commit -m "feat: window registry with hide/restore"
```

---

### Task 8: Event Handler

**Files:**
- Create: `src/handler.rs`

**Step 1: Write handler.rs**

```rust
use crate::config::Config;
use crate::ipc::HyprIpc;
use crate::monitor::MonitorCache;
use crate::registry::Registry;
use crate::types::HyprClient;

pub fn handle(
    event: &str,
    registry: &mut Registry,
    ipc: &HyprIpc,
    monitors: &mut MonitorCache,
    config: &Config,
) {
    let parts: Vec<&str> = event.splitn(2, ">>").collect();
    if parts.len() < 2 {
        return;
    }
    let event_name = parts[0];
    let payload = parts[1];

    match event_name {
        "activewindowv2" => handle_focus(payload, registry, ipc, config),
        "openwindow" => handle_open(payload, registry, ipc, monitors, config),
        "closewindow" => handle_close(payload, registry),
        "movewindow" => handle_move(payload, registry),
        "changefloating" => handle_float_change(payload, registry, ipc, monitors),
        "monitoradded" | "monitorremoved" => {
            monitors.refresh(ipc);
        }
        _ => {} // workspace, focusedmon, etc. → no-op
    }
}

fn handle_focus(addr: &str, registry: &mut Registry, ipc: &HyprIpc, config: &Config) {
    let response = ipc.send_command(&format!("clients -j"));
    let clients: Vec<HyprClient> = match serde_json::from_str(&response) {
        Ok(c) => c,
        Err(_) => return,
    };

    let focused = match clients.iter().find(|c| c.address == addr) {
        Some(c) => c,
        None => return,
    };

    if focused.floating && focused.mapped && !focused.hidden {
        // Focused window is floating → restore it
        registry.restore(addr, ipc);
    } else {
        // Focused window is tiled → hide all floating
        let addrs: Vec<String> = registry.windows.keys().cloned().collect();
        for a in addrs {
            registry.hide(&a, ipc, &MonitorCache::new(), config.hide_offset);
        }
    }
}

fn handle_open(
    payload: &str,
    registry: &mut Registry,
    ipc: &HyprIpc,
    monitors: &MonitorCache,
    config: &Config,
) {
    // openwindow format: addr|workspace|class|title|...
    let fields: Vec<&str> = payload.split('|').collect();
    if fields.len() < 4 {
        return;
    }
    let addr = fields[0];
    let class = fields[2];
    let title = fields[3];

    if config.should_ignore(class, title) {
        return;
    }

    // Fetch full client info
    let response = ipc.send_command("clients -j");
    let clients: Vec<HyprClient> = match serde_json::from_str(&response) {
        Ok(c) => c,
        Err(_) => return,
    };

    if let Some(c) = clients.iter().find(|c| c.address == addr) {
        if c.floating && c.mapped {
            registry.register(
                addr,
                c.workspace.id,
                &c.monitor,
                c.at[0],
                c.at[1],
                c.size[0],
                c.size[1],
            );
        }
    }
}

fn handle_close(addr: &str, registry: &mut Registry) {
    registry.remove(addr);
}

fn handle_move(payload: &str, registry: &mut Registry) {
    // movewindow format: addr|x|y
    let fields: Vec<&str> = payload.split('|').collect();
    if fields.len() < 3 {
        return;
    }
    let addr = fields[0];
    if let (Ok(x), Ok(y)) = (fields[1].parse::<i32>(), fields[2].parse::<i32>()) {
        registry.update_position(addr, x, y);
    }
}

fn handle_float_change(
    payload: &str,
    registry: &mut Registry,
    ipc: &HyprIpc,
    monitors: &MonitorCache,
) {
    // changefloating format: addr|floating
    let fields: Vec<&str> = payload.split('|').collect();
    if fields.len() < 2 {
        return;
    }
    let addr = fields[0];
    let floating = fields[1] == "1";
    registry.update_floating(addr, floating, ipc, monitors);
}
```

**Step 2: Add mod + wire event loop into main.rs**

```rust
mod config;
mod handler;
mod ipc;
mod monitor;
mod registry;
mod types;

use std::sync::mpsc;

fn main() {
    env_logger::init();
    let config = config::Config::load();
    log::info!("floating-hide starting (debug={})", config.debug);

    let ipc = ipc::HyprIpc::connect();
    let mut monitors = monitor::MonitorCache::new();
    monitors.refresh(&ipc);

    let mut registry = registry::Registry::new();
    registry.scan(&ipc, &monitors, config.hide_offset);

    let (tx, rx) = mpsc::channel::<String>();

    // Event listener thread
    let ipc_clone = ipc::HyprIpc::connect();
    std::thread::spawn(move || {
        ipc_clone.listen_events(tx);
    });

    // Main loop - process events
    for event in rx {
        handler::handle(&event, &mut registry, &ipc, &mut monitors, &config);
    }

    log::error!("Event stream ended, exiting");
}
```

**Step 3: Commit**

```bash
git add src/handler.rs src/main.rs
git commit -m "feat: event handler with focus/open/close/move/float logic"
```

---

### Task 9: Fix hide_all to use correct monitor

**Files:**
- Modify: `src/handler.rs`
- Modify: `src/registry.rs`

The `handle_focus` function calls `hide` with `MonitorCache::new()` instead of the real monitor cache. Fix this by passing the monitor cache through.

**Step 1: Update handle_focus signature and call**

In handler.rs, update `handle` to pass `monitors` to `handle_focus`, and update `handle_focus` to use it:

```rust
fn handle_focus(addr: &str, registry: &mut Registry, ipc: &HyprIpc, monitors: &MonitorCache, config: &Config) {
    // ...
    let addrs: Vec<String> = registry.windows.keys().cloned().collect();
    for a in addrs {
        registry.hide(&a, ipc, monitors, config.hide_offset);
    }
}
```

Update the `handle` match to pass monitors:

```rust
"activewindowv2" => handle_focus(payload, registry, ipc, monitors, config),
```

**Step 2: Commit**

```bash
git add src/handler.rs
git commit -m "fix: pass monitor cache to hide_all in focus handler"
```

---

### Task 10: Handle layer surfaces / non-window events

**Files:**
- Modify: `src/handler.rs`

Socket2 sends events for layer surfaces too. Filter them out in handle_open by checking if the client exists in the clients list. If `clients -j` doesn't return it, skip.

Already handled: the `handle_open` function fetches clients and only registers if found. Layer surfaces won't appear in `clients -j`.

No change needed. Commit a comment documenting this.

**Step 1: Add comment in handler.rs**

```rust
// Layer surfaces, bars, notifications etc. don't appear in clients -j,
// so they're automatically filtered out by the find() call below.
```

**Step 2: Commit**

```bash
git add src/handler.rs
git commit -m "docs: note layer surface filtering is automatic"
```

---

### Task 11: Test on live Hyprland

**Step 1: Build**

```bash
cd ~/Projects/floating-hide
cargo build
```

**Step 2: Run with debug**

```bash
RUST_LOG=info cargo run
```

Expected output:
```
INFO  floating-hide starting (debug=false)
INFO  Socket1: /run/user/1000/hypr/.../.socket.sock
INFO  Loaded config from /home/user/.config/floating-hide/config.toml
INFO  Monitor: HDMI-A-1 1920x1200+0+0
INFO  Registered window 0x... on HDMI-A-1
INFO  Listening for events on socket2
```

**Step 3: Test workflow**

1. Open a tiled window (terminal)
2. Open a floating window (calculator)
3. Focus terminal → calculator should disappear
4. Alt+Tab to calculator → it should reappear
5. Check logs for Hidden/Restored messages

**Step 4: Install**

```bash
cargo install --path .
```

Binary goes to `~/.cargo/bin/floating-hide`.

**Step 5: Add to Hyprland exec**

In `execs.lua`, add:
```lua
hl.exec_cmd("floating-hide")
```

**Step 6: Final commit**

```bash
git add -A
git commit -m "feat: working daemon"
```

---

### Task 12: README

**Files:**
- Create: `README.md`

**Step 1: Write README.md**

```markdown
# floating-hide

Hyprland daemon that auto-hides inactive floating windows.

Floating windows are moved off-screen when you switch to a tiled window, and restored when you switch back. Similar to how Windows handles utility windows.

## Install

```bash
cargo install --path .
```

## Usage

```bash
floating-hide
```

Add to your Hyprland config:

```lua
hl.exec_cmd("floating-hide")
```

## Config

Copy `config.toml` to `~/.config/floating-hide/config.toml`:

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
```

**Step 2: Commit**

```bash
git add README.md
git commit -m "docs: README"
```
