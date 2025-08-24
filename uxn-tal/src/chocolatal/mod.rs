//! Chocolatal: TAL Preprocessor for uxn-tal
//
// This module provides preprocessing for Uxn TAL assembly files, inspired by preprocess-tal.sh.
// It operates purely on &str, performing file inclusion, label/prefix expansion, lambda/loop label generation,
// path rewriting, and special token rewrites. See README.md for details.

use std::fs;
use std::path::{Path, PathBuf};
use glob::glob;
use std::env;

use crate::debug;

#[derive(Debug)]
pub enum PreprocessError {
    Io(std::io::Error),
    RecursionLimit,
    Other(String),
}

pub type Result<T> = std::result::Result<T, PreprocessError>;

/// Preprocess a TAL source file, returning a preprocessed string.
/// This function operates on raw text, not tokens.
pub fn preprocess(input: &str, path: &str) -> Result<String> {
    let mut output = String::new();
    let mut stack = Vec::new(); // For lambda/loop label tracking
    let mut lambda_counter = 0;

    // Enable debug output if env var CHOCOLATAL_DEBUG=1
    let debug_enabled = env::var("CHOCOLATAL_DEBUG").map(|v| v == "1").unwrap_or(true);
    macro_rules! debug {
        ($($arg:tt)*) => {
            if debug_enabled {
                eprintln!("[chocolatal:debug] {}", format!($($arg)*));
            }
        }
    }

    // Split input into tokens and separators (whitespace, newlines), preserving all separators
    let mut token = String::new();
    let mut sep = String::new();
    let mut chars = input.chars().peekable();
    let mut tokens = Vec::new();
    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            sep.push(c);
            chars.next();
        } else {
            if !sep.is_empty() || !token.is_empty() {
                if !token.is_empty() {
                    tokens.push((token.clone(), sep.clone()));
                    token.clear();
                    sep.clear();
                } else if !sep.is_empty() {
                    // Handle leading whitespace as a "token" (for perfect preservation)
                    tokens.push((String::new(), sep.clone()));
                    sep.clear();
                }
            }
            // Special case: if token is empty and c is '~', try to grab the whole include token
            if token.is_empty() && c == '~' {
                let mut incl = String::new();
                incl.push('~');
                chars.next();
                while let Some(&nc) = chars.peek() {
                    if nc.is_whitespace() { break; }
                    incl.push(nc);
                    chars.next();
                }
                tokens.push((incl, sep.clone()));
                sep.clear();
                continue;
            }
            // General: accumulate all consecutive non-whitespace as a single token
            while let Some(&nc) = chars.peek() {
                if nc.is_whitespace() { break; }
                token.push(nc);
                chars.next();
            }
            tokens.push((token.clone(), sep.clone()));
            token.clear();
            sep.clear();
        }
    }
    // Push any trailing token and/or separator
    if !token.is_empty() || !sep.is_empty() {
        tokens.push((token, sep));
    }

    let mut i = 0;
    let mut prefix_stack: Vec<String> = Vec::new();
    let mut current_prefix: String = String::new();
    while i < tokens.len() {
        let (ref tok, ref sep) = tokens[i];

        // If we just did a prefix+include, push the prefix for the included block
        // (This is a simplification: in the shell, prefix is set for the duration of the include)
        // We use a stack to allow nested includes
        if let Some(idx) = tok.find('~') {
            let (prefix, rest) = tok.split_at(idx);
            if !prefix.is_empty() && rest.len() > 1 {
                let path_part = &rest[1..];
                let label = std::path::Path::new(path_part)
                    .file_name()
                    .and_then(|f| f.to_str())
                    .map(|f| if f.ends_with(".tal") { &f[..f.len()-4] } else { f })
                    .unwrap_or("");
                let prefix_label = format!("{}{}", prefix, label);
                prefix_stack.push(current_prefix.clone());
                current_prefix = prefix_label.clone();
                let input_dir = std::path::Path::new(path).parent().map(|p| p.to_path_buf()).unwrap_or_else(|| std::path::PathBuf::from("."));
                let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
                let path_part = path_part.trim_start_matches("./").trim_start_matches('/');
                let incl_pattern_cwd = cwd.join(path_part);
                let incl_pattern_cwd_str = incl_pattern_cwd.to_str().unwrap_or("");
                if incl_pattern_cwd.exists() {
                    process_include_pattern(incl_pattern_cwd_str, &mut output, sep, &prefix_label, debug_enabled)?;
                } else {
                    let incl_pattern = input_dir.join(path_part);
                    let incl_pattern_str = incl_pattern.to_str().unwrap_or("");
                    process_include_pattern(incl_pattern_str, &mut output, sep, &prefix_label, debug_enabled)?;
                }
                current_prefix = prefix_stack.pop().unwrap_or_default();
                i += 1;
                continue;
            } else if prefix.is_empty() && rest.len() > 1 {
                // Plain ~file.tal include, no prefix context
                let path_part = &rest[1..];
                let input_dir = std::path::Path::new(path).parent().map(|p| p.to_path_buf()).unwrap_or_else(|| std::path::PathBuf::from("."));
                let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
                let path_part = path_part.trim_start_matches("./").trim_start_matches('/');
                let incl_pattern_cwd = cwd.join(path_part);
                let incl_pattern_cwd_str = incl_pattern_cwd.to_str().unwrap_or("");
                if incl_pattern_cwd.exists() {
                    process_include_pattern(incl_pattern_cwd_str, &mut output, sep, path_part, debug_enabled)?;
                } else {
                    let incl_pattern = input_dir.join(path_part);
                    let incl_pattern_str = incl_pattern.to_str().unwrap_or("");
                    process_include_pattern(incl_pattern_str, &mut output, sep, path_part, debug_enabled)?;
                }
                i += 1;
                continue;
            }
        }

        // Faithful prefix/prefix-rewrite rules from preprocess-tal.sh
        // 1. &&* or [',.;_-=!?|$']&&*  => tok = tok[..tok.find("&&")] + "&" + &tok[tok.find("&&")+2..]
        if let Some(idx) = tok.find("&&") {
            let newtok = format!("{}&{}", &tok[..idx], &tok[idx+2..]);
            output.push_str(&format!("{}{}", newtok, sep));
            i += 1;
            continue;
        }
        // 2. [',.;_-=!?|$']&* => tok = tok[..tok.find("/&")] + "&" + current_prefix + &tok[tok.find("/&")+2..]
        if let Some(idx) = tok.find("/&") {
            let newtok = format!("{}&{}{}", &tok[..idx], current_prefix, &tok[idx+2..]);
            output.push_str(&format!("{}{}", newtok, sep));
            i += 1;
            continue;
        }
        // 3. &* => tok = tok[..tok.find("&")] + "&" + current_prefix + &tok[tok.find("&")+1..]
        if let Some(idx) = tok.find('&') {
            if idx == 0 && tok.len() > 1 {
                let newtok = format!("&{}{}", current_prefix, &tok[idx+1..]);
                output.push_str(&format!("{}{}", newtok, sep));
                i += 1;
                continue;
            }
        }
        // 4. //* or [',.;_-=!?|$']//* => tok = tok[..tok.find("//")] + "/" + &tok[tok.find("//")+2..]
        if let Some(idx) = tok.find("//") {
            let newtok = format!("{}/{}", &tok[..idx], &tok[idx+2..]);
            output.push_str(&format!("{}{}", newtok, sep));
            i += 1;
            continue;
        }
        // 5. [',.;_-=!?|$']/[^.]* => tok = tok[..tok.find("/")] + "/" + current_prefix + &tok[tok.find("/")+1..]
        if let Some(idx) = tok.find('/') {
            if idx > 0 && !tok[idx+1..].starts_with('.') {
                let newtok = format!("{}/{}{}", &tok[..idx], current_prefix, &tok[idx+1..]);
                output.push_str(&format!("{}{}", newtok, sep));
                i += 1;
                continue;
            }
        }
        // 6. /[^.]* => tok = tok[..tok.find("/")] + "/" + current_prefix + &tok[tok.find("/")+1..]
        if tok.starts_with('/') && tok.len() > 1 && !tok[1..].starts_with('.') {
            let newtok = format!("/{}{}", current_prefix, &tok[1..]);
            output.push_str(&format!("{}{}", newtok, sep));
            i += 1;
            continue;
        }
        //debug!("Token: '{}', Sep: '{}'", tok, sep);

        // Prefix+include: <prefix>~file.tal (e.g., &~dist.tal)
        if let Some(idx) = tok.find('~') {
            let (prefix, rest) = tok.split_at(idx);
            if !prefix.is_empty() && rest.len() > 1 {
                let path_part = &rest[1..];
                if path_part.trim().is_empty() {
                    return Err(PreprocessError::Other(format!(
                        "Syntax error at {}:{}: Empty include path\nSource: ~",
                        path,
                        input[..input.find('~').unwrap_or(0)].lines().count() + 1
                    )));
                }
                // Use filename as label, strip .tal only if present
                let label = Path::new(path_part)
                    .file_name()
                    .and_then(|f| f.to_str())
                    .map(|f| if f.ends_with(".tal") { &f[..f.len()-4] } else { f })
                    .unwrap_or("");
                // Compose prefix+label (e.g., &dist)
                let prefix_label = format!("{}{}", prefix, label);
                // Try to resolve include path: first try cwd, then relative to the parent of the current file
                let input_dir = Path::new(path).parent().map(|p| p.to_path_buf()).unwrap_or_else(|| PathBuf::from("."));
                let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
                let path_part = path_part.trim_start_matches("./").trim_start_matches('/');
                debug!("Prefix+include token: '{}', resolved path part: '{}', prefix_label: '{}'", tok, path_part, prefix_label);
                // Try cwd first
                let incl_pattern_cwd = cwd.join(path_part);
                let incl_pattern_cwd_str = incl_pattern_cwd.to_str().unwrap_or("");
                debug!("Including file(s) with pattern: {}", incl_pattern_cwd.display());
                if incl_pattern_cwd.exists() {
                    process_include_pattern(incl_pattern_cwd_str, &mut output, sep, &prefix_label, debug_enabled)?;
                } else {
                    // Fallback: try relative to the parent of the current file
                    let incl_pattern = input_dir.join(path_part);
                    let incl_pattern_str = incl_pattern.to_str().unwrap_or("");
                    process_include_pattern(incl_pattern_str, &mut output, sep, &prefix_label, debug_enabled)?;
                }
                i += 1;
                continue;
            }
        }
        // File inclusion: ~file.tal or ~*.tal (glob)
        if tok.starts_with('~') {
            let path_part = &tok[1..];
            if path_part.trim().is_empty() {
                return Err(PreprocessError::Other(format!(
                    "Syntax error at {}:{}: Empty include path\nSource: ~",
                    path,
                    // Try to get line number (approximate)
                    input[..input.find('~').unwrap_or(0)].lines().count() + 1
                )));
            }
            // Try to resolve include path: first try cwd, then relative to the parent of the current file
            let input_dir = Path::new(path).parent().map(|p| p.to_path_buf()).unwrap_or_else(|| PathBuf::from("."));
            let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            let path_part = path_part.trim_start_matches("./").trim_start_matches('/');
            debug!("Include token: '{}', resolved path part: '{}'", tok, path_part);

            // Try cwd first
            let incl_pattern_cwd = cwd.join(path_part);
            let incl_pattern_cwd_str = incl_pattern_cwd.to_str().unwrap_or("");
            debug!("Including file(s) with pattern: {}", incl_pattern_cwd.display());
            if incl_pattern_cwd.exists() {
                process_include_pattern(incl_pattern_cwd_str, &mut output, sep, path_part, debug_enabled)?;
            } else {
                // Fallback: try relative to the parent of the current file
                let incl_pattern = input_dir.join(path_part);
                let incl_pattern_str = incl_pattern.to_str().unwrap_or("");
                process_include_pattern(incl_pattern_str, &mut output, sep, path_part, debug_enabled)?;
            }
            i += 1;
            continue;
        }
        // Lambda/loop label generation: '>{'
        if tok == ">{" {
            lambda_counter += 1;
            stack.push(lambda_counter);
            debug!("Lambda open: ʎ{} (stack: {:?})", lambda_counter, stack);
            output.push_str(&format!("&ʎ{}{}", lambda_counter, sep));
            i += 1;
            continue;
        }
        // Lambda/loop label close: |} ?} or }
        if (tok == "|}" || tok == "?}" || tok == "}") && !stack.is_empty() {
            let n = stack.pop().unwrap();
            debug!("Lambda close: ʎ{} (stack: {:?})", n, stack);
            output.push_str(&format!("&ʎ{}{}", n, sep));
            i += 1;
            continue;
        }
        // @. rewrite: replace with @<parent directory name>
        if tok == "@." {

            // Get the parent directory name (not the file stem)
            let parent_name = Path::new(path)
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|s| s.to_str())
            .unwrap_or("");
                    debug!("@. token found, rewriting to parent directory name: tok='{}', parent_name='{}'", tok, parent_name);
            // std::process::exit(1);
            output.push_str(&format!("@{}{}", parent_name, sep));
            i += 1;
            continue;
        }
        // /.' rewrite: replace with STH2kr <rest>
        if tok.starts_with("/." ) {
            output.push_str(&format!("STH2kr {}{}", &tok[2..], sep));
            i += 1;
            continue;
        }
        // Faithful prefix/prefix-rewrite rules from preprocess-tal.sh
        // 1. &&* or [',.;_-=!?|$']&&*  => tok = tok[..tok.find("&&")] + "&" + &tok[tok.find("&&")+2..]
        if let Some(idx) = tok.find("&&") {
            let newtok = format!("{}&{}", &tok[..idx], &tok[idx+2..]);
            output.push_str(&format!("{}{}", newtok, sep));
            i += 1;
            continue;
        }
        // 2. [',.;_-=!?|$']&* => tok = tok[..tok.find("/&")] + "&" + prefix + &tok[tok.find("/&")+2..]
        if let Some(idx) = tok.find("/&") {
            // Use prefix from context if available (here, just use prefix_label if set)
            let prefix = ""; // TODO: track prefix context if needed
            let newtok = format!("{}&{}{}", &tok[..idx], prefix, &tok[idx+2..]);
            output.push_str(&format!("{}{}", newtok, sep));
            i += 1;
            continue;
        }
        // 3. &* => tok = tok[..tok.find("&")] + "&" + prefix + &tok[tok.find("&")+1..]
        if let Some(idx) = tok.find('&') {
            if idx == 0 && tok.len() > 1 {
                let prefix = ""; // TODO: track prefix context if needed
                let newtok = format!("&{}{}", prefix, &tok[idx+1..]);
                output.push_str(&format!("{}{}", newtok, sep));
                i += 1;
                continue;
            }
        }
        // 4. //* or [',.;_-=!?|$']//* => tok = tok[..tok.find("//")] + "/" + &tok[tok.find("//")+2..]
        if let Some(idx) = tok.find("//") {
            let newtok = format!("{}/{}", &tok[..idx], &tok[idx+2..]);
            output.push_str(&format!("{}{}", newtok, sep));
            i += 1;
            continue;
        }
        // 5. [',.;_-=!?|$']/[^.]* => tok = tok[..tok.find("/")] + "/" + prefix + &tok[tok.find("/")+1..]
        if let Some(idx) = tok.find('/') {
            if idx > 0 && !tok[idx+1..].starts_with('.') {
                let prefix = ""; // TODO: track prefix context if needed
                let newtok = format!("{}/{}{}", &tok[..idx], prefix, &tok[idx+1..]);
                output.push_str(&format!("{}{}", newtok, sep));
                i += 1;
                continue;
            }
        }
        // 6. /[^.]* => tok = tok[..tok.find("/")] + "/" + prefix + &tok[tok.find("/")+1..]
        if tok.starts_with('/') && tok.len() > 1 && !tok[1..].starts_with('.') {
            let prefix = ""; // TODO: track prefix context if needed
            let newtok = format!("/{}{}", prefix, &tok[1..]);
            output.push_str(&format!("{}{}", newtok, sep));
            i += 1;
            continue;
        }
        // TODO: Add more token rewrites and path/prefix handling as needed
        output.push_str(tok);
        output.push_str(sep);
        i += 1;
    }
    Ok(output)
}

fn process_include_pattern(incl_pattern_str: &str, output: &mut String, sep: &String, path_part: &str, debug_enabled: bool) -> Result<()> {
    macro_rules! debug {
        ($($arg:tt)*) => {
            if debug_enabled {
                eprintln!("[chocolatal:debug] {}", format!($($arg)*));
            }
        }
    }
    let input_path = Path::new(incl_pattern_str).to_path_buf();
    debug!("Including file(s) with pattern: {}", incl_pattern_str);
    Ok(if incl_pattern_str.contains('*') || incl_pattern_str.contains('?') || incl_pattern_str.contains('[') {
        // Glob pattern: include all matching files
        for entry in glob(incl_pattern_str).map_err(|e| PreprocessError::Other(e.to_string()))? {
            match entry {
                Ok(path) => {
                    if path.is_file() {
                        debug!("Including file: {}", path.display());
                        if path.extension().map(|e| e == "tal").unwrap_or(false) {
                            // Try to include using cwd as current_dir first, then fallback to original logic if fails
                            let incl_pre = match preprocess_include_file(&PathBuf::from("."), &path) {
                                Ok(s) => s,
                                Err(_) => preprocess_include_file(&input_path, &path)?,
                            };
                            output.push_str(&incl_pre);
                            output.push_str(sep);
                        } else {
                            // Not .tal: hex dump as in shell script
                            let bytes = match fs::read(&path) {
                                Ok(b) => b,
                                Err(e) => {
                                    return Err(PreprocessError::Other(format!(
                                        "Failed to read file '{}': {}",
                                        path.display(), e
                                    )));
                                }
                            };
                            let mut line = String::new();
                            for chunk in bytes.chunks(16) {
                                line.clear();
                                for (j, group) in chunk.chunks(2).enumerate() {
                                    if j > 0 { line.push(' '); }
                                    for b in group {
                                        line.push_str(&format!("{:02x}", b));
                                    }
                                }
                                output.push_str(&line);
                                output.push('\n');
                            }
                            output.push_str(sep);
                        }
                    }
                }
                Err(e) => {
                    debug!("Glob error: {}", e);
                    return Err(PreprocessError::Other(e.to_string()))
                },
            }
        }
    } else {
        // Single file include
        debug!("Including file: {}", incl_pattern_str);
        if input_path.extension().map(|e| e == "tal").unwrap_or(false) {
            // .tal: preprocess as text
            // Try to read the file, if not found, try the same filename in all parent directories up to and including cwd
            let filename = input_path.file_name().map(|f| f.to_os_string());
            let mut tried_paths = Vec::new();
            let mut try_path = input_path.clone();
            let incl_str = loop {
                match fs::read_to_string(&try_path) {
                    Ok(s) => break s,
                    Err(e) => {
                        tried_paths.push(try_path.clone());
                        // Try parent directory with same filename
                        if let Some(parent) = try_path.parent() {
                            if let Some(fname) = &filename {
                                let parent_path = parent.join(fname);
                                // Stop if we've already tried this path or reached root
                                if tried_paths.contains(&parent_path) || parent_path == try_path {
                                    // As a last resort, try just the filename in cwd if not already tried
                                    let cwd_path = PathBuf::from(fname);
                                    if !tried_paths.contains(&cwd_path) && cwd_path != try_path {
                                        debug!("Trying filename in cwd: {}", cwd_path.display());
                                        try_path = cwd_path;
                                        continue;
                                    }
                                    return Err(PreprocessError::Other(format!(
                                        "chocolatal::Failed to read file '{}': {}",
                                        input_path.display(), e
                                    )));
                                }
                                debug!("Trying parent directory: {}", parent_path.display());
                                try_path = parent_path;
                                continue;
                            }
                        }
                        // As a last resort, try just the filename in cwd if not already tried
                        if let Some(fname) = &filename {
                            let cwd_path = PathBuf::from(fname);
                            if !tried_paths.contains(&cwd_path) && cwd_path != try_path {
                                debug!("Trying filename in cwd: {}", cwd_path.display());
                                try_path = cwd_path;
                                continue;
                            }
                        }
                        return Err(PreprocessError::Other(format!(
                            "chocolatal::Failed to read file '{}': {}",
                            input_path.display(), e
                        )));
                    }
                }
            };
            let incl_pre = preprocess(&incl_str, input_path.to_str().unwrap_or(""))?;
            let incl_pre = incl_pre.trim_end();
            output.push_str(&incl_pre);
            output.push_str(sep);
        } else {
            // Not .tal: hex dump as in shell script
            let bytes = match fs::read(&input_path) {
                Ok(b) => b,
                Err(e) => {
                    return Err(PreprocessError::Other(format!(
                        "Failed to read file '{}': {}",
                        input_path.display(), e
                    )));
                }
            };
            let mut line = String::new();
            for chunk in bytes.chunks(16) {
                line.clear();
                for (j, group) in chunk.chunks(2).enumerate() {
                    if j > 0 { line.push(' '); }
                    for b in group {
                        line.push_str(&format!("{:02x}", b));
                    }
                }
                output.push_str(&line);
                output.push('\n');
            }
            output.push_str(sep);
        }
    })
}

fn preprocess_include_file(current_dir: &PathBuf, path: &PathBuf) -> Result<String> {
    let mut rel_path = path.strip_prefix(current_dir).unwrap_or(path);
    let rel_str = rel_path.to_str().unwrap_or("");
    let rel_str = rel_str.strip_prefix("./").or(rel_str.strip_prefix('/')).unwrap_or(rel_str);
    eprintln!("Including file (rewritten path): {}", rel_str);
    let incl_str = match fs::read_to_string(&rel_str) {
        Ok(s) => s,
        Err(e) => {

            return Err(PreprocessError::Other(format!(
                "chocolatal: Failed to read file '{}': {}",
                rel_str, e
            )));
        }
    };
    let incl_str = incl_str.trim_end();
    let incl_pre = preprocess(&incl_str, rel_str)?;
    Ok(incl_pre)
}


/// Runs the deluge docker container and returns the output as a String.
pub fn deluge_preprocess(input_path: &str) -> std::io::Result<String> {
// pub fn deluge_preprocess() -> std::io::Result<String> {
    use std::env;
    use std::process::Command;
    let cwd = env::current_dir()?;
    let volume = format!("{}:/workspace", cwd.display());
    let args = ["run", "--rm", "-v", &volume, "-w", "/workspace", "alpine", "sh", "./preprocess-tal.sh","deluge/main.tal"];
    eprintln!("[deluge_preprocess:debug] Running: docker {:?}\n  cwd: {}", args, cwd.display());
    let output = Command::new("docker")
        .args(&args)
        .output()?;
    // eprintln!("[deluge_preprocess:debug] stdout:\n{}", String::from_utf8_lossy(&output.stdout));
    // eprintln!("[deluge_preprocess:debug] stderr:\n{}", String::from_utf8_lossy(&output.stderr));
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
fn resolve_include_path(pattern: &str, cwd: &std::path::Path, input_dir: &std::path::Path) -> Option<std::path::PathBuf> {
    let candidate_cwd = cwd.join(pattern);
    if candidate_cwd.exists() {
        return Some(candidate_cwd);
    }
    let candidate_input = input_dir.join(pattern);
    if candidate_input.exists() {
        return Some(candidate_input);
    }
    None
}