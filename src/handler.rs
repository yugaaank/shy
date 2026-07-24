use crate::config::Config;
use crate::ipc::HyprIpc;
use crate::monitor::MonitorCache;
use crate::registry::Registry;
use crate::types::HyprClient;
use std::time::{Duration, Instant};

pub struct State {
    pub switcher_active: bool,
    pub last_switcher_time: Instant,
}

impl State {
    pub fn new() -> Self {
        Self {
            switcher_active: false,
            last_switcher_time: Instant::now() - Duration::from_secs(10),
        }
    }
}

pub fn handle(
    event: &str,
    registry: &mut Registry,
    ipc: &HyprIpc,
    monitors: &mut MonitorCache,
    config: &Config,
    state: &mut State,
) {
    let parts: Vec<&str> = event.splitn(2, ">>").collect();
    if parts.len() < 2 {
        return;
    }
    let event_name = parts[0];
    let payload = parts[1];

    log::info!("Received event: {} >> {}", event_name, payload);

    match event_name {
        "activewindowv2" | "activewindow" => handle_focus(payload, registry, ipc, monitors, config, state),
        "openlayer" => handle_open_layer(payload, state),
        "closelayer" => handle_close_layer(payload, state),
        "openwindow" => handle_open(payload, registry, ipc, monitors, config),
        "closewindow" => handle_close(payload, registry),
        "movewindow" => handle_move(payload, registry, monitors, config),
        "changefloating" => handle_float_change(payload, registry, ipc, monitors, config),
        "monitoradded" | "monitorremoved" => {
            monitors.refresh(ipc);
        }
        _ => {} // workspace, focusedmon, etc. → no-op
    }
}

fn is_switcher_layer(layer_name: &str) -> bool {
    let l = layer_name.to_lowercase();
    l.contains("switcher") || l.contains("rofi") || l.contains("walker") || l.contains("overview") || l.contains("tab") || l.contains("launcher")
}

fn handle_open_layer(payload: &str, state: &mut State) {
    if is_switcher_layer(payload) {
        log::info!("Switcher layer opened: {}", payload);
        state.switcher_active = true;
        state.last_switcher_time = Instant::now();
    }
}

fn handle_close_layer(payload: &str, state: &mut State) {
    if is_switcher_layer(payload) {
        log::info!("Switcher layer closed: {}", payload);
        state.switcher_active = false;
        state.last_switcher_time = Instant::now();
    }
}

fn normalize_addr(raw: &str) -> String {
    let clean = raw.split(',').next().unwrap_or(raw).trim();
    if clean.starts_with("0x") {
        clean.to_string()
    } else {
        format!("0x{}", clean)
    }
}

fn split_fields(payload: &str) -> Vec<&str> {
    payload.split(|c| c == ',' || c == '|').collect()
}

fn handle_focus(
    payload: &str,
    registry: &mut Registry,
    ipc: &HyprIpc,
    monitors: &MonitorCache,
    config: &Config,
    state: &mut State,
) {
    let addr = normalize_addr(payload);
    if addr == "0x" {
        return;
    }

    let response = ipc.send_command("j/clients");
    let clients: Vec<HyprClient> = match serde_json::from_str(&response) {
        Ok(c) => c,
        Err(e) => {
            log::error!("Failed to parse clients JSON in handle_focus: {}", e);
            return;
        }
    };

    // Dynamically register any floating window not yet in the registry
    for c in &clients {
        if c.floating && c.mapped && !c.hidden {
            if !registry.windows.contains_key(&c.address) {
                let mon_name = monitors.monitor_name(c.monitor);
                registry.register(
                    ipc,
                    &c.address,
                    c.workspace.id,
                    &mon_name,
                    c.at[0],
                    c.at[1],
                    c.size[0],
                    c.size[1],
                    monitors,
                    config.hide_offset,
                );
            }
        }
    }

    let focused = match clients.iter().find(|c| c.address == addr) {
        Some(c) => c,
        None => {
            log::debug!("Focused window {} not found in clients list", addr);
            return;
        }
    };

    if focused.floating && focused.mapped && !focused.hidden {
        // Focused window is floating → restore it
        registry.restore(&addr, ipc, monitors, config.hide_offset);
        // Clear switcher flag so subsequent sudden mouse movements onto tiled windows are treated as hover!
        state.switcher_active = false;
        state.last_switcher_time = Instant::now() - Duration::from_secs(10);
    } else {
        // Focused window is tiled → hide floating windows only if switcher active or ignore_hover is false
        let is_switcher = state.switcher_active || state.last_switcher_time.elapsed() < Duration::from_millis(600);
        if !config.ignore_hover || is_switcher {
            let addrs: Vec<String> = registry.windows.keys().cloned().collect();
            for a in addrs {
                registry.hide(&a, ipc, monitors, config.hide_offset);
            }
            state.switcher_active = false;
            state.last_switcher_time = Instant::now() - Duration::from_secs(10);
        } else {
            log::info!("Ignoring mouse hover focus change on tiled window {}", addr);
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
    let fields = split_fields(payload);
    if fields.len() < 4 {
        return;
    }
    let addr = normalize_addr(fields[0]);
    let class = fields[2];
    let title = fields[3];

    if config.should_ignore(class, title) {
        return;
    }

    let response = ipc.send_command("j/clients");
    let clients: Vec<HyprClient> = match serde_json::from_str(&response) {
        Ok(c) => c,
        Err(_) => return,
    };

    if let Some(c) = clients.iter().find(|c| c.address == addr) {
        if c.floating && c.mapped {
            let mon_name = monitors.monitor_name(c.monitor);
            registry.register(
                ipc,
                &addr,
                c.workspace.id,
                &mon_name,
                c.at[0],
                c.at[1],
                c.size[0],
                c.size[1],
                monitors,
                config.hide_offset,
            );
        }
    }
}

fn handle_close(payload: &str, registry: &mut Registry) {
    let addr = normalize_addr(payload);
    registry.remove(&addr);
}

fn handle_move(payload: &str, registry: &mut Registry, monitors: &MonitorCache, config: &Config) {
    let fields = split_fields(payload);
    if fields.len() < 3 {
        return;
    }
    let addr = normalize_addr(fields[0]);
    if let (Ok(x), Ok(y)) = (fields[1].parse::<i32>(), fields[2].parse::<i32>()) {
        registry.update_position(&addr, x, y, monitors, config.hide_offset);
    }
}

fn handle_float_change(
    payload: &str,
    registry: &mut Registry,
    ipc: &HyprIpc,
    monitors: &MonitorCache,
    config: &Config,
) {
    let fields = split_fields(payload);
    if fields.len() < 2 {
        return;
    }
    let addr = normalize_addr(fields[0]);
    let floating = fields[1] == "1";
    registry.update_floating(&addr, floating, ipc, monitors, config.hide_offset);
}
