//! Local loopback server for byte-range local audio streaming, thumbnails, and single-instance forwarding.

use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::net::TcpListener;
use std::sync::OnceLock;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::platform::paths::{get_music_dir, get_songs_dir, get_thumbnails_image_dir};

static LOOPBACK_PORT: OnceLock<u16> = OnceLock::new();

pub fn get_loopback_port() -> u16 {
    *LOOPBACK_PORT.get_or_init(|| 51189)
}

fn mime_for_ext(ext: &str) -> &'static str {
    match ext.to_ascii_lowercase().as_str() {
        "m4a" | "mp4" | "aac" => "audio/mp4",
        "mp3" => "audio/mpeg",
        "flac" => "audio/flac",
        "wav" => "audio/wav",
        "webm" => "audio/webm",
        "ogg" => "audio/ogg",
        _ => "application/octet-stream",
    }
}

pub fn url_decode(s: &str) -> String {
    urlencoding::decode(s).map(|c| c.into_owned()).unwrap_or_else(|_| s.to_string())
}

/// Spawns the local loopback server on a background Tokio task.
pub fn start_loopback_server(on_deep_link: Option<Box<dyn Fn(String) + Send + Sync + 'static>>) {
    let std_listener = match TcpListener::bind("127.0.0.1:51189") {
        Ok(l) => l,
        Err(_) => match TcpListener::bind("127.0.0.1:0") {
            Ok(l) => l,
            Err(e) => {
                log::error!("[BeatFlow IPC] Failed to bind local server: {}", e);
                return;
            }
        },
    };

    let local_addr = match std_listener.local_addr() {
        Ok(a) => a,
        Err(e) => {
            log::error!("[BeatFlow IPC] Failed to get local_addr: {}", e);
            return;
        }
    };

    let port = local_addr.port();
    LOOPBACK_PORT.set(port).ok();
    log::info!("[BeatFlow IPC] Local loopback server listening on http://127.0.0.1:{}", port);

    let _ = std_listener.set_nonblocking(true);
    let deep_link_cb = on_deep_link.map(std::sync::Arc::new);

    tokio::spawn(async move {
        let listener = match tokio::net::TcpListener::from_std(std_listener) {
            Ok(l) => l,
            Err(e) => {
                log::error!("[BeatFlow IPC] tokio TcpListener error: {}", e);
                return;
            }
        };

        loop {
            if let Ok((mut socket, _)) = listener.accept().await {
                let cb = deep_link_cb.clone();
                tokio::spawn(async move {
                    let mut buf = vec![0u8; 65536];
                    let n = match socket.read(&mut buf).await {
                        Ok(n) if n > 0 => n,
                        _ => return,
                    };
                    let req_text = String::from_utf8_lossy(&buf[..n]);

                    // Handle CORS OPTIONS
                    if req_text.starts_with("OPTIONS ") {
                        let resp = "HTTP/1.1 204 No Content\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: POST, GET, OPTIONS, HEAD\r\nAccess-Control-Allow-Headers: *\r\n\r\n";
                        let _ = socket.write_all(resp.as_bytes()).await;
                        return;
                    }

                    // Handle local song streaming over loopback HTTP
                    let is_get = req_text.starts_with("GET ");
                    let is_head = req_text.starts_with("HEAD ");
                    if is_get || is_head {
                        if let Some(pos) = req_text.find(" /local/") {
                            let after = &req_text[pos + 8..];
                            let path_end = after.find(" HTTP/").unwrap_or(after.len());
                            let raw_file = &after[..path_end];
                            let clean_file = raw_file.split(['?', '#']).next().unwrap_or(raw_file);
                            let file_name = url_decode(clean_file);
                            let songs_dir = get_songs_dir();
                            let target_file = if songs_dir.join(&file_name).exists() {
                                songs_dir.join(&file_name)
                            } else {
                                get_music_dir().join(&file_name)
                            };

                            if target_file.exists() && target_file.is_file()
                                && let Ok(mut f) = fs::File::open(&target_file) {
                                    let file_size = f.metadata().map(|m| m.len()).unwrap_or(0);
                                    let ext = target_file.extension().and_then(|e| e.to_str()).unwrap_or("");
                                    let mime = mime_for_ext(ext);

                                    // Extract Range header if present
                                    let mut range_header: Option<String> = None;
                                    for line in req_text.lines() {
                                        let trimmed = line.trim();
                                        if trimmed.to_lowercase().starts_with("range:")
                                            && let Some(val) = trimmed.split(':').nth(1) {
                                                range_header = Some(val.trim().to_string());
                                                break;
                                            }
                                    }

                                    if let Some(ref range_str) = range_header
                                        && let Some(range_part) = range_str.strip_prefix("bytes=") {
                                            let parts: Vec<&str> = range_part.split('-').collect();
                                            let start: u64 = parts.first().and_then(|s| s.parse().ok()).unwrap_or(0);
                                            let requested_end: Option<u64> = parts.get(1).and_then(|s| s.parse().ok());
                                            let end = requested_end.unwrap_or(file_size.saturating_sub(1)).min(file_size.saturating_sub(1));

                                            if start <= end && start < file_size {
                                                let chunk_len = end - start + 1;
                                                let header = format!(
                                                    "HTTP/1.1 206 Partial Content\r\nContent-Type: {}\r\nContent-Range: bytes {}-{}/{}\r\nContent-Length: {}\r\nAccept-Ranges: bytes\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, HEAD, OPTIONS\r\n\r\n",
                                                    mime, start, end, file_size, chunk_len
                                                );
                                                let _ = socket.write_all(header.as_bytes()).await;
                                                if is_get && f.seek(SeekFrom::Start(start)).is_ok() {
                                                    let mut buffer = vec![0u8; chunk_len as usize];
                                                    if f.read_exact(&mut buffer).is_ok() {
                                                        let _ = socket.write_all(&buffer).await;
                                                    }
                                                }
                                                return;
                                            }
                                        }

                                    let header = format!(
                                        "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nAccept-Ranges: bytes\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, HEAD, OPTIONS\r\n\r\n",
                                        mime, file_size
                                    );
                                    let _ = socket.write_all(header.as_bytes()).await;
                                    if is_get {
                                        let mut buffer = Vec::new();
                                        if f.read_to_end(&mut buffer).is_ok() {
                                            let _ = socket.write_all(&buffer).await;
                                        }
                                    }
                                    return;
                                }

                            let resp = "HTTP/1.1 404 Not Found\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: 0\r\n\r\n";
                            let _ = socket.write_all(resp.as_bytes()).await;
                            return;
                        }

                        // Handle thumbnails
                        if let Some(pos) = req_text.find(" /thumbnail/") {
                            let after = &req_text[pos + 12..];
                            let path_end = after.find(" HTTP/").unwrap_or(after.len());
                            let raw_file = &after[..path_end];
                            let clean_file = raw_file.split(['?', '#']).next().unwrap_or(raw_file);
                            let file_name = url_decode(clean_file);
                            let target_file = get_thumbnails_image_dir().join(&file_name);
                            if target_file.exists() && target_file.is_file()
                                && let Ok(bytes) = fs::read(&target_file) {
                                    let mime = if file_name.ends_with(".png") { "image/png" } else { "image/jpeg" };
                                    let header = format!(
                                        "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nCache-Control: public, max-age=86400\r\n\r\n",
                                        mime, bytes.len()
                                    );
                                    let _ = socket.write_all(header.as_bytes()).await;
                                    if is_get {
                                        let _ = socket.write_all(&bytes).await;
                                    }
                                    return;
                                }

                            let resp = "HTTP/1.1 404 Not Found\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: 0\r\n\r\n";
                            let _ = socket.write_all(resp.as_bytes()).await;
                            return;
                        }
                    }

                    // Handle single instance forward-arg
                    if req_text.contains("/forward-arg") {
                        let body = if let Some(pos) = req_text.find("\r\n\r\n") {
                            req_text[pos + 4..].trim().to_string()
                        } else {
                            String::new()
                        };
                        if let Some(ref callback) = cb {
                            callback(body);
                        }
                        let resp = "HTTP/1.1 200 OK\r\nAccess-Control-Allow-Origin: *\r\nContent-Type: application/json\r\nContent-Length: 11\r\n\r\n{\"ok\":true}";
                        let _ = socket.write_all(resp.as_bytes()).await;
                        return;
                    }

                    let resp = "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n";
                    let _ = socket.write_all(resp.as_bytes()).await;
                });
            }
        }
    });
}
