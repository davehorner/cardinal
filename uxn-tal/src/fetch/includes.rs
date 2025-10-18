use regex::Regex;
use std::path::Path;

pub fn parse_includes(tal: &str) -> Vec<String> {
    let mut buf = String::new();
    for line in tal.lines() {
        let t = line.trim_start();
        if t.starts_with(';') {
            continue;
        }
        buf.push_str(line);
        buf.push('\n');
    }

    let re_tilde = Regex::new(r#"(?m)(~/?[A-Za-z0-9_\-./]+\.tal)"#).unwrap();
    let re_quote =
        Regex::new(r#"(?m)(?:^|\s)(?:\|?include|#include)\s*["']([^"']+\.tal)["']"#).unwrap();
    let re_bare = Regex::new(r#"(?m)(^|\s)([A-Za-z0-9_\-./]+\.tal)"#).unwrap();

    let mut out = Vec::new();
    for c in re_tilde.captures_iter(&buf) {
        out.push(c[1].to_string());
    }
    for c in re_quote.captures_iter(&buf) {
        out.push(c[1].to_string());
    }
    for c in re_bare.captures_iter(&buf) {
        out.push(c[2].to_string());
    }

    out.retain(|p| p.ends_with(".tal"));
    out.sort();
    out.dedup();
    out
}

pub fn resolve_include(curr_repo_rel: &str, inc: &str) -> String {
    // For remote providers, always resolve ~ and ~/ includes relative to the current file's repo path
    if let Some(s) = inc.strip_prefix("~/") {
        let base = Path::new(curr_repo_rel).parent().unwrap_or(Path::new(""));
        let inc_path = Path::new(s);
        // If the first segment of inc matches the parent dir, don't join
        if let (Some(base_name), Some(first)) = (base.file_name(), inc_path.iter().next()) {
            if first == base_name {
                return s.replace('\\', "/");
            }
        }
        let joined = base.join(s);
        return joined.to_string_lossy().replace('\\', "/");
    }
    if let Some(s) = inc.strip_prefix('~') {
        let base = Path::new(curr_repo_rel).parent().unwrap_or(Path::new(""));
        let s = s.trim_start_matches('/');
        let inc_path = Path::new(s);
        if let (Some(base_name), Some(first)) = (base.file_name(), inc_path.iter().next()) {
            if first == base_name {
                return s.replace('\\', "/");
            }
        }
        let joined = base.join(s);
        return joined.to_string_lossy().replace('\\', "/");
    }
    if inc.starts_with('/') {
        return inc.trim_start_matches('/').to_string();
    }
    let base = Path::new(curr_repo_rel).parent().unwrap_or(Path::new(""));
    let inc_path = Path::new(inc);
    if let (Some(base_name), Some(first)) = (base.file_name(), inc_path.iter().next()) {
        if first == base_name {
            return inc.replace('\\', "/");
        }
    }
    let joined = base.join(inc);
    joined.to_string_lossy().replace('\\', "/")
}

pub fn repo_entry_guesses() -> &'static [&'static str] {
    &[
        "src/main.tal",
        "main.tal",
        "src/drifblim.tal",
        "drifblim.tal",
        "index.tal",
    ]
}
