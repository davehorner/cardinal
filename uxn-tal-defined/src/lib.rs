pub mod v1;
pub use v1::*;
pub mod emu_buxn;
pub mod emu_cuxn;
pub mod emu_uxn;

/// Decode percent-encoding. If the decoded bytes aren't valid UTF-8,
/// return the original string unchanged.
///
/// Examples:
/// - "Hello%20World%21" => "Hello World!"
/// - "%E2%9C%93" => "✓"
/// - "%ZZ" (malformed) => "%ZZ" (unchanged, because decoding would be invalid)
pub fn percent_decode_or_original(s: &str) -> String {
    // Fast path: nothing to do
    if !s.as_bytes().contains(&b'%') {
        return s.to_string();
    }

    // Decode %XX into raw bytes; copy other bytes as-is.
    let mut out = Vec::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(h), Some(l)) = (hex_val(bytes[i + 1]), hex_val(bytes[i + 2])) {
                out.push((h << 4) | l);
                i += 3;
                continue;
            }
            // Malformed escape: keep '%' literally
            out.push(b'%');
            i += 1;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }

    // Try UTF-8; on failure, return the original unchanged.
    match std::str::from_utf8(&out) {
        Ok(decoded) => decoded.to_string(),
        Err(_) => s.to_string(),
    }
}

fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}
