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

    pub fn scan(&mut self, ipc: &HyprIpc, monitors: &MonitorCache, _hide_offset: i32) {
        let response = ipc.send_command("clients -j");
        match serde_json::from_str::<Vec<crate::types::HyprClient>>(&response) {
            Ok(clients) => {
                for c in clients {
                    if c.floating && c.mapped && !c.hidden {
                        let mon_name = monitors.monitor_name(c.monitor);
                        let entry = WindowEntry {
                            addr: c.address.clone(),
                            workspace: c.workspace.id,
                            monitor: mon_name.clone(),
                            saved_x: c.at[0],
                            saved_y: c.at[1],
                            width: c.size[0],
                            height: c.size[1],
                            hidden: false,
                        };
                        log::info!("Registered window {} on {}", c.address, mon_name);
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
                return;
            }
            entry.hidden = true;
            let off_x = monitors.monitor_id(&entry.monitor)
                .map(|id| monitors.get_offscreen_x(id, hide_offset))
                .unwrap_or(10000);
            let cmd = format!("dispatch hl.dsp.movewindow pixel {} {} addr:{}", off_x, entry.saved_y, addr);
            ipc.send_command(&cmd);
            log::info!("Hidden window {} to x={}", addr, off_x);
        }
    }

    pub fn restore(&mut self, addr: &str, ipc: &HyprIpc) {
        if let Some(entry) = self.windows.get_mut(addr) {
            if !entry.hidden {
                return;
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
            let response = ipc.send_command("clients -j");
            if let Ok(clients) = serde_json::from_str::<Vec<crate::types::HyprClient>>(&response) {
                if let Some(c) = clients.iter().find(|c| c.address == addr) {
                    let mon_name = monitors.monitor_name(c.monitor);
                    self.register(
                        addr,
                        c.workspace.id,
                        &mon_name,
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
