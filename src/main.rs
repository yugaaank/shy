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
