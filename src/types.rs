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
    pub monitor: i32,
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
    pub id: i32,
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}
