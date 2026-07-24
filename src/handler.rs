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

    log::info!("Received event: {} >> {}", event_name, payload);

    match event_name {
        "activewindowv2" | "activewindow" => handle_focus(payload, registry, ipc, monitors, config),
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

fn normalize_addr(raw: &str) -> String {
    let clean = raw.split(',').next().unwrap_or(raw).trim();
    if clean.starts_with("0x") {
        clean.to_string()
    } else {
        format!("0x{}", clean)
    }
}

fn handle_focus(payload: &str, registry: &mut Registry, ipc: &HyprIpc, monitors: &MonitorCache, config: &Config) {
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

    let focused = match clients.iter().find(|c| c.address == addr) {
        Some(c) => c,
        None => {
            log::debug!("Focused window {} not found in clients list", addr);
            return;
        }
    };

    if focused.floating && focused.mapped && !focused.hidden {
        // Focused window is floating → restore it
        registry.restore(&addr, ipc);
    } else {
        // Focused window is tiled → hide all floating
        // Layer surfaces, bars, notifications etc. don't appear in clients -j,
        // so they're automatically filtered out by the find() call below.
        let addrs: Vec<String> = registry.windows.keys().cloned().collect();
        for a in addrs {
            registry.hide(&a, ipc, monitors, config.hide_offset);
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
    let addr = normalize_addr(fields[0]);
    let class = fields[2];
    let title = fields[3];

    if config.should_ignore(class, title) {
        return;
    }

    // Fetch full client info
    // Layer surfaces, bars, notifications etc. don't appear in clients -j,
    // so they're automatically filtered out by the find() call below.
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
            );
        }
    }
}

fn handle_close(payload: &str, registry: &mut Registry) {
    let addr = normalize_addr(payload);
    registry.remove(&addr);
}

fn handle_move(payload: &str, registry: &mut Registry) {
    // movewindow format: addr|x|y
    let fields: Vec<&str> = payload.split('|').collect();
    if fields.len() < 3 {
        return;
    }
    let addr = normalize_addr(fields[0]);
    if let (Ok(x), Ok(y)) = (fields[1].parse::<i32>(), fields[2].parse::<i32>()) {
        registry.update_position(&addr, x, y);
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
    let addr = normalize_addr(fields[0]);
    let floating = fields[1] == "1";
    registry.update_floating(&addr, floating, ipc, monitors);
}
