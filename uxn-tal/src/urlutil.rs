use std::borrow::Cow;

pub fn extract_target_from_uxntal(raw_url: &str) -> Option<String> {
    fn pct_decode(s: &str) -> String {
        percent_encoding::percent_decode_str(s)
            .decode_utf8()
            .unwrap_or(Cow::from(s))
            .into_owned()
    }
    fn qs_get<'a>(query: &'a str, key: &str) -> Option<String> {
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
    if !raw_url.starts_with("uxntal:") { return None; }

    let mut s = raw_url.trim_start_matches("uxntal:").trim_start_matches('/');

    if s.starts_with("open") {
        let (path, rest) = if let Some(qpos) = s.find('?') { (&s[..qpos], &s[qpos + 1..]) } else { (s, "") };
        if path == "open" || path == "open/" {
            if let Some(v) = qs_get(rest, "url") { return Some(v); }
        }
    }
    // 2) Base64 form: uxntal://b64,<payload>  (URL_SAFE_NO_PAD)
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
        ("http///",  "http://",  7usize),
        ("file///",  "file://",  7usize),
        ("https//",  "https://", 7usize),
        ("http//",   "http://",  6usize),
        ("file//",   "file://",  7usize),
    ] {
        if s.starts_with(bad) { return Some(format!("{}{}", good, &s[cut..])); }
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
