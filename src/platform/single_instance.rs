//! Single-instance management and argument forwarding.

use std::io::Write;
use std::net::TcpStream;
use std::time::Duration;

pub const DEFAULT_PORT: u16 = 51189;

/// Checks if another instance is already running on the loopback port.
/// If present, forwards `initial_arg` to that instance and returns `true`.
pub fn check_and_forward(initial_arg: &str) -> bool {
    if let Ok(mut stream) = TcpStream::connect(("127.0.0.1", DEFAULT_PORT)) {
        let _ = stream.set_write_timeout(Some(Duration::from_millis(800)));
        let body = initial_arg.as_bytes();
        let req = format!(
            "POST /forward-arg HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            DEFAULT_PORT,
            body.len(),
            initial_arg
        );
        let _ = stream.write_all(req.as_bytes());
        log::info!("[BeatFlow IPC] Another instance is already running. Forwarded argument.");
        return true;
    }
    false
}
