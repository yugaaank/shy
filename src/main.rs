mod config;
mod ipc;
mod types;

fn main() {
    env_logger::init();
    let cfg = config::Config::load();
    log::info!("floating-hide starting (debug={})", cfg.debug);
}
