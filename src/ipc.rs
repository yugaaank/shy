use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::process::Command;

pub struct HyprIpc {
    socket2: String,
}

impl HyprIpc {
    pub fn connect() -> Self {
        let his = std::env::var("HYPRLAND_INSTANCE_SIGNATURE")
            .expect("HYPRLAND_INSTANCE_SIGNATURE not set - is Hyprland running?");
        let runtime = std::env::var("XDG_RUNTIME_DIR")
            .unwrap_or_else(|_| format!("/run/user/{}", std::process::id()));

        let socket2 = format!("{}/hypr/{}/.socket2.sock", runtime, his);

        log::info!("Socket2: {}", socket2);

        HyprIpc { socket2 }
    }

    pub fn send_command(&self, command: &str) -> String {
        let parts: Vec<&str> = command.split_whitespace().collect();
        let output = Command::new("hyprctl")
            .args(&parts)
            .output()
            .expect("Failed to execute hyprctl");

        if !output.status.success() {
            log::error!("hyprctl {} failed: {}", command, String::from_utf8_lossy(&output.stderr));
            return String::new();
        }

        String::from_utf8_lossy(&output.stdout).trim().to_string()
    }

    pub fn listen_events(&self, tx: std::sync::mpsc::Sender<String>) {
        let stream = UnixStream::connect(&self.socket2)
            .expect("Failed to connect to Hyprland socket2");
        let reader = BufReader::new(stream);

        log::info!("Listening for events on socket2");

        for line in reader.lines() {
            match line {
                Ok(line) => {
                    if !line.is_empty() {
                        if tx.send(line).is_err() {
                            break;
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
}