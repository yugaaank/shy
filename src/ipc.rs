use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::net::UnixStream;

pub struct HyprIpc {
    socket: String,
    socket2: String,
}

impl HyprIpc {
    pub fn connect() -> Self {
        let his = std::env::var("HYPRLAND_INSTANCE_SIGNATURE")
            .expect("HYPRLAND_INSTANCE_SIGNATURE not set - is Hyprland running?");
        let runtime = std::env::var("XDG_RUNTIME_DIR")
            .unwrap_or_else(|_| format!("/run/user/{}", std::process::id()));

        let socket = format!("{}/hypr/{}/.socket.sock", runtime, his);
        let socket2 = format!("{}/hypr/{}/.socket2.sock", runtime, his);

        log::info!("Socket: {}, Socket2: {}", socket, socket2);

        HyprIpc { socket, socket2 }
    }

    pub fn send_command(&self, command: &str) -> String {
        match UnixStream::connect(&self.socket) {
            Ok(mut stream) => {
                if let Err(e) = stream.write_all(command.as_bytes()) {
                    log::error!("Failed to write to socket {}: {}", self.socket, e);
                    return String::new();
                }
                let mut response = String::new();
                if let Err(e) = stream.read_to_string(&mut response) {
                    log::error!("Failed to read from socket {}: {}", self.socket, e);
                    return String::new();
                }
                response.trim().to_string()
            }
            Err(e) => {
                log::error!("Failed to connect to socket {}: {}", self.socket, e);
                String::new()
            }
        }
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