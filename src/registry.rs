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
        let response = ipc.send_command("j/clients");
        match serde_json::from_str::<Vec<crate::types::HyprClient>>(&response) {
            Ok(clients) => {
                for c in clients {
                    if c.floating && c.mapped && !c.hidden {
                        let mon_name = monitors.monitor_name(c.monitor);
                        let off_y = monitors
                            .monitor_id(&mon_name)
                            .map(|id| monitors.get_offscreen_y(id, hide_offset))
                            .unwrap_or(1200);

                        let is_offscreen = c.at[1] >= (off_y - 50);

                        let valid_y = if is_offscreen {
                            200
                        } else {
                            c.at[1]
                        };

                        let entry = WindowEntry {
                            addr: c.address.clone(),
                            workspace: c.workspace.id,
                            monitor: mon_name.clone(),
                            saved_x: c.at[0],
                            saved_y: valid_y,
                            width: c.size[0],
                            height: c.size[1],
                            hidden: is_offscreen,
                        };
                        let prop_cmd = format!("dispatch hl.dsp.window.set_prop({{ prop = \"animation\", value = \"none\", window = \"address:{}\" }})", c.address);
                        ipc.send_command(&prop_cmd);
                        log::info!("Registered window {} on {} with saved_y={}", c.address, mon_name, valid_y);
                        self.windows.insert(c.address, entry);
                    }
                }
            }
            Err(e) => log::error!("Failed to parse clients: {}", e),
        }
    }

    pub fn register(
        &mut self,
        ipc: &HyprIpc,
        addr: &str,
        workspace: i32,
        monitor: &str,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        monitors: &MonitorCache,
        hide_offset: i32,
    ) {
        if self.windows.contains_key(addr) {
            return;
        }

        let off_y = monitors
            .monitor_id(monitor)
            .map(|id| monitors.get_offscreen_y(id, hide_offset))
            .unwrap_or(1200);

        let is_offscreen = y >= (off_y - 50);

        let valid_y = if is_offscreen {
            200
        } else {
            y
        };

        let entry = WindowEntry {
            addr: addr.to_string(),
            workspace,
            monitor: monitor.to_string(),
            saved_x: x,
            saved_y: valid_y,
            width,
            height,
            hidden: is_offscreen,
        };
        let prop_cmd = format!("dispatch hl.dsp.window.set_prop({{ prop = \"animation\", value = \"none\", window = \"address:{}\" }})", addr);
        ipc.send_command(&prop_cmd);
        log::info!("Registered new window {} with saved_y={}", addr, valid_y);
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
                return;
            }

            // Query current position from Hyprland before hiding so manual drags are captured
            let response = ipc.send_command("j/clients");
            if let Ok(clients) = serde_json::from_str::<Vec<crate::types::HyprClient>>(&response) {
                if let Some(c) = clients.iter().find(|c| c.address == addr) {
                    let off_y = monitors.monitor_id(&entry.monitor)
                        .map(|id| monitors.get_offscreen_y(id, hide_offset))
                        .unwrap_or(1200);
                    if c.at[1] < (off_y - 50) {
                        entry.saved_x = c.at[0];
                        entry.saved_y = c.at[1];
                    }
                    // Also update size in case of resize
                    entry.width = c.size[0];
                    entry.height = c.size[1];
                }
            }

            entry.hidden = true;
            let off_y = monitors.monitor_id(&entry.monitor)
                .map(|id| monitors.get_offscreen_y(id, hide_offset))
                .unwrap_or(10000);
            let cmd = format!("dispatch hl.dsp.window.move({{ x = {}, y = {}, window = \"address:{}\" }})", entry.saved_x, off_y, addr);
            ipc.send_command(&cmd);
            log::info!("Hidden window {} to y={} (saved pos: {}, {})", addr, off_y, entry.saved_x, entry.saved_y);
        }
    }

    pub fn restore(&mut self, addr: &str, ipc: &HyprIpc, monitors: &MonitorCache, hide_offset: i32) {
        if let Some(entry) = self.windows.get_mut(addr) {
            if !entry.hidden {
                return;
            }

            let off_y = monitors.monitor_id(&entry.monitor)
                .map(|id| monitors.get_offscreen_y(id, hide_offset))
                .unwrap_or(1200);

            if entry.saved_y >= (off_y - 50) {
                entry.saved_y = 200;
                log::warn!("Reset corrupt saved_y for {} to {}", addr, entry.saved_y);
            }

            entry.hidden = false;
            let cmd = format!("dispatch hl.dsp.window.move({{ x = {}, y = {}, window = \"address:{}\" }})", entry.saved_x, entry.saved_y, addr);
            ipc.send_command(&cmd);

            let center_x = entry.saved_x + (entry.width / 2);
            let center_y = entry.saved_y + (entry.height / 2);
            let cursor_cmd = format!("dispatch hl.dsp.cursor.move({{ x = {}, y = {} }})", center_x, center_y);
            ipc.send_command(&cursor_cmd);

            log::info!("Restored window {} to {},{}", addr, entry.saved_x, entry.saved_y);
        }
    }

    pub fn restore_all(&mut self, ipc: &HyprIpc, monitors: &MonitorCache, hide_offset: i32) {
        let addrs: Vec<String> = self.windows.keys().cloned().collect();
        for addr in addrs {
            self.restore(&addr, ipc, monitors, hide_offset);
        }
    }

    pub fn hide_all(&mut self, ipc: &HyprIpc, monitors: &MonitorCache, hide_offset: i32) {
        let addrs: Vec<String> = self.windows.keys().cloned().collect();
        for addr in addrs {
            self.hide(&addr, ipc, monitors, hide_offset);
        }
    }

    pub fn update_position(&mut self, addr: &str, x: i32, y: i32, monitors: &MonitorCache, hide_offset: i32) {
        if let Some(entry) = self.windows.get_mut(addr) {
            let off_y = monitors.monitor_id(&entry.monitor)
                .map(|id| monitors.get_offscreen_y(id, hide_offset))
                .unwrap_or(1200);
            if !entry.hidden && y < (off_y - 50) {
                entry.saved_x = x;
                entry.saved_y = y;
            }
        }
    }

    pub fn update_floating(&mut self, addr: &str, floating: bool, ipc: &HyprIpc, monitors: &MonitorCache, hide_offset: i32) {
        if floating {
            let response = ipc.send_command("j/clients");
            if let Ok(clients) = serde_json::from_str::<Vec<crate::types::HyprClient>>(&response) {
                if let Some(c) = clients.iter().find(|c| c.address == addr) {
                    let mon_name = monitors.monitor_name(c.monitor);
                    self.register(
                        ipc,
                        addr,
                        c.workspace.id,
                        &mon_name,
                        c.at[0],
                        c.at[1],
                        c.size[0],
                        c.size[1],
                        monitors,
                        hide_offset,
                    );
                }
            }
        } else {
            self.remove(addr);
        }
    }
}
