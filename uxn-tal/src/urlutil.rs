use std::collections::HashMap;

/// Parse a uxntal: URL into a HashMap of key-value pairs using ^ as separator.
/// Example: uxntal:widget:emu^buxn:foo^bar -> {"widget": "true", "emu": "buxn", "foo": "bar"}
pub fn parse_uxntal_url_to_map(raw_url: &str) -> (HashMap<String, String>, String) {
    let mut map = HashMap::new();
    if !raw_url.starts_with("uxntal:") {
        return (map, String::new());
    }
    // Only trim the "uxntal:" prefix, do NOT trim leading slashes, so protocol is preserved
    let s = raw_url.trim_start_matches("uxntal:");
    // Split once at the first "://" or ":" before a URL (//...)
    // Split at the first occurrence of //
    let (kv_part, url_part) = if let Some(idx) = s.find("//") {
        (&s[..idx], &s[idx..])
    } else {
        (s, "")
    };

    // Parse key-value pairs
    use percent_encoding::percent_decode_str;
    for part in kv_part.split(':') {
        if part.is_empty() {
            continue;
        }
        let decoded = percent_decode_str(part).decode_utf8_lossy();
        // Find the first occurrence of ^ or ^^
        let mut split_idx = None;
        let mut sep_len = 0;
        if let Some(idx) = decoded.find("^^") {
            split_idx = Some(idx);
            sep_len = 2;
        } else if let Some(idx) = decoded.find('^') {
            split_idx = Some(idx);
            sep_len = 1;
        }
        if let Some(idx) = split_idx {
            let key = &decoded[..idx];
            let value = &decoded[idx + sep_len..];
            map.insert(key.to_string(), value.to_string());
        } else {
            map.insert(decoded.to_string(), String::new());
        }
    }
    // Remove leading double slash from url if present (for uxntal:// and uxntal:widget://)
    // Only trim // if the URL starts with //https:// or //http://, otherwise leave as-is
    let url = if url_part.starts_with("//https://")
        || url_part.starts_with("//http://")
        || url_part.starts_with("//file://")
    {
        url_part.trim_start_matches("//").to_string()
    } else {
        url_part.to_string()
    };
    (map, url)
}

pub fn extract_target_from_uxntal(url: &str) -> Option<String> {
    use std::borrow::Cow;
    fn pct_decode(s: &str) -> String {
        percent_encoding::percent_decode_str(s)
            .decode_utf8()
            .unwrap_or(Cow::from(s))
            .into_owned()
    }
    fn qs_get(query: &str, key: &str) -> Option<String> {
        for pair in query.split('&') {
            let mut it = pair.splitn(2, '=');
            let k = it.next().unwrap_or("");
            let v = it.next().unwrap_or("");
            if k.eq_ignore_ascii_case(key) {
                return Some(pct_decode(v));
            }
        }
        None
    }
    let s = url;
    // Handle open?url=... and b64, cases as before
    if s.starts_with("open") {
        let (path, rest) = if let Some(qpos) = s.find('?') {
            (&s[..qpos], &s[qpos + 1..])
        } else {
            (s, "")
        };
        if path == "open" || path == "open/" {
            if let Some(v) = qs_get(rest, "url") {
                return Some(v);
            }
        }
    }
    if let Some(rest) = s.strip_prefix("b64,") {
        use base64::Engine;
        if let Ok(bytes) = base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(rest) {
            if let Ok(strv) = String::from_utf8(bytes) {
                return Some(strv);
            }
        }
    }
    for (bad, good, cut) in [
        ("https///", "https://", 8usize),
        ("http///", "http://", 7usize),
        ("file///", "file://", 7usize),
        ("https//", "https://", 7usize),
        ("http//", "http://", 6usize),
        ("file//", "file://", 7usize),
    ] {
        if s.starts_with(bad) {
            return Some(format!("{}{}", good, &s[cut..]));
        }
    }

    if s.contains('%') {
        let dec = pct_decode(s);
        if dec.starts_with("http://") || dec.starts_with("https://") || dec.starts_with("file://") {
            return Some(dec);
        }
    }
    if s.starts_with("http://") || s.starts_with("https://") || s.starts_with("file://") {
        return Some(s.to_string());
    }
    Some(s.to_string())
}
