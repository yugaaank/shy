use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;

pub struct HyprIpc {
    socket1: String,
    socket2: String,
}

impl HyprIpc {
    pub fn connect() -> Self {
        let his = std::env::var("HYPRLAND_INSTANCE_SIGNATURE")
            .expect("HYPRLAND_INSTANCE_SIGNATURE not set - is Hyprland running?");
        let runtime = std::env::var("XDG_RUNTIME_DIR")
            .unwrap_or_else(|_| format!("/run/user/{}", std::process::id()));

        let socket1 = format!("{}/hypr/{}/.socket.sock", runtime, his);
        let socket2 = format!("{}/hypr/{}/.socket2.sock", runtime, his);

        log::info!("Socket1: {}", socket1);
        log::info!("Socket2: {}", socket2);

        HyprIpc { socket1, socket2 }
    }

    pub fn send_command(&self, command: &str) -> String {
        let mut stream = UnixStream::connect(&self.socket1)
            .expect("Failed to connect to Hyprland socket1");
        stream.write_all(command.as_bytes()).expect("Failed to write");
        stream.write_all(b"\n").expect("Failed to write newline");
        stream.shutdown(std::net::Shutdown::Write).expect("Failed to shutdown write");

        let mut reader = BufReader::new(&stream);
        let mut response = String::new();
        reader.read_line(&mut response).expect("Failed to read response");
        response.trim().to_string()
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
