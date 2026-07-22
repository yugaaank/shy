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
