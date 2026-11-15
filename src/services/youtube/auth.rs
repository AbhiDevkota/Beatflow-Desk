//! YouTube Music authentication, cookie management, and authorization hashes.

use std::fs;
use serde_json::Value;
use crate::platform::paths::get_yt_auth_path;

#[allow(clippy::needless_range_loop)]
pub fn sha1_hex(data: &[u8]) -> String {
    let mut h0: u32 = 0x67452301;
    let mut h1: u32 = 0xEFCDAB89;
    let mut h2: u32 = 0x98BADCFE;
    let mut h3: u32 = 0x10325476;
    let mut h4: u32 = 0xC3D2E1F0;

    let mut msg = data.to_vec();
    let bit_len = (msg.len() as u64) * 8;
    msg.push(0x80);
    while (msg.len() % 64) != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in msg.as_chunks::<64>().0 {
        let mut w = [0u32; 80];
        for i in 0..16 {
            w[i] = u32::from_be_bytes(chunk[i * 4..(i + 1) * 4].try_into().unwrap());
        }
        for i in 16..80 {
            w[i] = (w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16]).rotate_left(1);
        }
        let mut a = h0;
        let mut b = h1;
        let mut c = h2;
        let mut d = h3;
        let mut e = h4;
        for i in 0..80 {
            let (f, k) = match i {
                0..=19 => ((b & c) | ((!b) & d), 0x5A827999),
                20..=39 => (b ^ c ^ d, 0x6ED9EBA1),
                40..=59 => ((b & c) | (b & d) | (c & d), 0x8F1BBCDC),
                _ => (b ^ c ^ d, 0xCA62C1D6),
            };
            let temp = a.rotate_left(5).wrapping_add(f).wrapping_add(e).wrapping_add(k).wrapping_add(w[i]);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = temp;
        }
        h0 = h0.wrapping_add(a);
        h1 = h1.wrapping_add(b);
        h2 = h2.wrapping_add(c);
        h3 = h3.wrapping_add(d);
        h4 = h4.wrapping_add(e);
    }
    format!("{:08x}{:08x}{:08x}{:08x}{:08x}", h0, h1, h2, h3, h4)
}

pub fn compute_youtube_authorization(cookies: &str, origin: &str) -> Option<String> {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let get_cookie_val = |target: &str| -> Option<&str> {
        for part in cookies.split(';') {
            let trimmed = part.trim();
            if let Some(rest) = trimmed.strip_prefix(target)
                && let Some(val) = rest.strip_prefix('=') {
                    return Some(val.trim());
                }
        }
        None
    };

    let schemes = [
        ("SAPISIDHASH", "SAPISID"),
        ("SAPISID1PHASH", "__Secure-1PAPISID"),
        ("SAPISID3PHASH", "__Secure-3PAPISID"),
    ];

    let mut parts = Vec::new();
    for (scheme, cookie_name) in schemes {
        if let Some(secret) = get_cookie_val(cookie_name)
            && !secret.is_empty() {
                let to_hash = format!("{} {} {}", ts, secret, origin);
                let hash = sha1_hex(to_hash.as_bytes());
                parts.push(format!("{} {}_{}", scheme, ts, hash));
            }
    }

    if parts.is_empty() {
        None
    } else {
        Some(parts.join(" "))
    }
}

pub fn load_yt_auth() -> Option<Value> {
    let path = get_yt_auth_path();
    if path.exists()
        && let Ok(content) = fs::read_to_string(&path)
            && let Ok(val) = serde_json::from_str::<Value>(&content) {
                let has_cookies = val
                    .get("cookies")
                    .and_then(|c| c.as_str())
                    .map(|c| c.contains("SAPISID=") || c.contains("__Secure-3PAPISID=") || c.contains("__Secure-1PAPISID="))
                    .unwrap_or(false);
                if has_cookies {
                    return Some(val);
                }
            }
    None
}

pub fn save_yt_auth(val: &Value) {
    let has_cookies = val
        .get("cookies")
        .and_then(|c| c.as_str())
        .map(|c| c.contains("SAPISID=") || c.contains("__Secure-3PAPISID=") || c.contains("__Secure-1PAPISID="))
        .unwrap_or(false);
    if !has_cookies {
        log::warn!("[BeatFlow Auth] Refusing to save yt_auth without valid SAPISID cookie");
        return;
    }
    let path = get_yt_auth_path();
    if let Ok(content) = serde_json::to_string_pretty(val) {
        let _ = fs::write(&path, content);
    }
}
