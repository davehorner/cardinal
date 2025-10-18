#![allow(clippy::all)]
// Chocolatal: TAL Preprocessor for uxn-tal
use std::ffi::OsStr;
//
// This module provides preprocessing for Uxn TAL assembly files, inspired by preprocess-tal.sh.
// It operates purely on &str, performing file inclusion, label/prefix expansion, lambda/loop label generation,
// path rewriting, and special token rewrites. See README.md for details and comments at the end of this file.
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum PreprocessError {
    Io(std::io::Error),
    RecursionLimit,
    Other(String),
}

pub type Result<T> = std::result::Result<T, PreprocessError>;

macro_rules! debug {
    ($($arg:tt)*) => {
        if env::var("CHOCOLATAL_DEBUG")
    .map(|v| v == "1")
    .unwrap_or(true) {
            eprintln!("[chocolatal:debug] {}", format!($($arg)*));
        }
    }
}

/// Preprocess a TAL source file, returning a preprocessed string.
/// This function operates on raw text, not tokens.
pub fn preprocess(input: &str, path: &str, root_dir: &PathBuf) -> Result<String> {
    let mut output = String::new();
    let mut stack = Vec::new(); // For lambda/loop label tracking
    let mut lambda_counter = 0;

    // // Enable debug output if env var CHOCOLATAL_DEBUG=1
    // let debug_enabled = env::var("CHOCOLATAL_DEBUG")
    //     .map(|v| v == "1")
    //     .unwrap_or(true);
    // macro_rules! debug {
    //     ($($arg:tt)*) => {
    //         if debug_enabled {
    //             eprintln!("[chocolatal:debug] {}", format!($($arg)*));
    //         }
    //     }
    // }

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
                    if nc.is_whitespace() {
                        break;
                    }
                    incl.push(nc);
                    chars.next();
                }
                tokens.push((incl, sep.clone()));
                sep.clear();
                continue;
            }
            // General: accumulate all consecutive non-whitespace as a single token
            while let Some(&nc) = chars.peek() {
                if nc.is_whitespace() {
                    break;
                }
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
                    .map(|f| {
                        if f.ends_with(".tal") {
                            &f[..f.len() - 4]
                        } else {
                            f
                        }
                    })
                    .unwrap_or("");
                let prefix_label = format!("{}{}", prefix, label);
                prefix_stack.push(current_prefix.clone());
                //current_prefix = prefix_label.clone();
                let input_dir = std::path::Path::new(path)
                    .parent()
                    .map(|p| p.to_path_buf())
                    .unwrap_or_else(|| std::path::PathBuf::from("."));
                let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
                let path_part = path_part.trim_start_matches("./").trim_start_matches('/');
                let incl_pattern_cwd = cwd.join(path_part);
                let incl_pattern_cwd_str = incl_pattern_cwd.to_str().unwrap_or("");
                if incl_pattern_cwd.exists() {
                    process_include_pattern(
                        incl_pattern_cwd_str,
                        &mut output,
                        sep,
                        &prefix_label,
                        root_dir,
                    )?;
                } else {
                    let incl_pattern = input_dir.join(path_part);
                    let incl_pattern_str = incl_pattern.to_str().unwrap_or("");
                    process_include_pattern(
                        incl_pattern_str,
                        &mut output,
                        sep,
                        &prefix_label,
                        root_dir,
                    )?;
                }
                current_prefix = prefix_stack.pop().unwrap_or_default();
                i += 1;
                continue;
            } else if prefix.is_empty() && rest.len() > 1 {
                // Plain ~file.tal include, no prefix context
                let path_part = &rest[1..];
                let input_dir = std::path::Path::new(path)
                    .parent()
                    .map(|p| p.to_path_buf())
                    .unwrap_or_else(|| std::path::PathBuf::from("."));
                let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
                let path_part = path_part.trim_start_matches("./").trim_start_matches('/');
                let incl_pattern_cwd = cwd.join(path_part);
                let incl_pattern_cwd_str = incl_pattern_cwd.to_str().unwrap_or("");
                if incl_pattern_cwd.exists() {
                    process_include_pattern(
                        incl_pattern_cwd_str,
                        &mut output,
                        sep,
                        path_part,
                        root_dir,
                    )?;
                } else {
                    let incl_pattern = input_dir.join(path_part);
                    let incl_pattern_str = incl_pattern.to_str().unwrap_or("");
                    process_include_pattern(
                        incl_pattern_str,
                        &mut output,
                        sep,
                        path_part,
                        root_dir,
                    )?;
                }
                i += 1;
                continue;
            }
        }

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
                    .map(|f| {
                        if f.ends_with(".tal") {
                            &f[..f.len() - 4]
                        } else {
                            f
                        }
                    })
                    .unwrap_or("");
                // Compose prefix+label (e.g., &dist)
                let prefix_label = format!("{}{}", prefix, label);
                // Try to resolve include path: first try cwd, then relative to the parent of the current file
                let input_dir = Path::new(path)
                    .parent()
                    .map(|p| p.to_path_buf())
                    .unwrap_or_else(|| PathBuf::from("."));
                let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
                let path_part = path_part.trim_start_matches("./").trim_start_matches('/');
                debug!(
                    "Prefix+include token: '{}', resolved path part: '{}', prefix_label: '{}'",
                    tok, path_part, prefix_label
                );
                // Try cwd first
                let incl_pattern_cwd = cwd.join(path_part);
                let incl_pattern_cwd_str = incl_pattern_cwd.to_str().unwrap_or("");
                debug!(
                    "Including file(s) with pattern1: {}",
                    incl_pattern_cwd.display()
                );
                if incl_pattern_cwd.exists() {
                    process_include_pattern(
                        incl_pattern_cwd_str,
                        &mut output,
                        sep,
                        &prefix_label,
                        root_dir,
                    )?;
                } else {
                    // Fallback: try relative to the parent of the current file
                    let incl_pattern = input_dir.join(path_part);
                    let incl_pattern_str = incl_pattern.to_str().unwrap_or("");
                    process_include_pattern(
                        incl_pattern_str,
                        &mut output,
                        sep,
                        &prefix_label,
                        root_dir,
                    )?;
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
            let input_dir = Path::new(path)
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| PathBuf::from("."));
            let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            let path_part = path_part.trim_start_matches("./").trim_start_matches('/');
            debug!(
                "Include token: '{}', resolved path part: '{}'",
                tok, path_part
            );

            // Try cwd first
            let incl_pattern_cwd = cwd.join(path_part);
            let incl_pattern_cwd_str = incl_pattern_cwd.to_str().unwrap_or("");
            debug!(
                "root:{} Including file(s) with pattern2: {}",
                root_dir.display(),
                incl_pattern_cwd.display()
            );
            if incl_pattern_cwd.exists() {
                process_include_pattern(
                    incl_pattern_cwd_str,
                    &mut output,
                    sep,
                    path_part,
                    root_dir,
                )?;
            } else {
                // Fallback: try relative to the parent of the current file
                let incl_pattern = input_dir.join(path_part);
                let incl_pattern_str = incl_pattern.to_str().unwrap_or("");
                process_include_pattern(incl_pattern_str, &mut output, sep, path_part, root_dir)?;
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
            debug!(
                "@. token found, rewriting to parent directory name: tok='{}', parent_name='{}'",
                tok, parent_name
            );
            // std::process::exit(1);
            output.push_str(&format!("@{}{}", parent_name, sep));
            i += 1;
            continue;
        }
        // /.' rewrite: replace with STH2kr <rest>
        if tok.starts_with("/.") {
            output.push_str(&format!("STH2kr {}{}", &tok[2..], sep));
            i += 1;
            continue;
        }
        // Faithful prefix/prefix-rewrite rules from preprocess-tal.sh
        // 1. &&* or [',.;_-=!?|$']&&*  => tok = tok[..tok.find("&&")] + "&" + &tok[tok.find("&&")+2..]
        if let Some(idx) = tok.find("&&") {
            let newtok = format!("{}&{}", &tok[..idx], &tok[idx + 2..]);
            output.push_str(&format!("{}{}", newtok, sep));
            i += 1;
            continue;
        }
        // 2. [',.;_-=!?|$']&* => tok = tok[..tok.find("/&")] + "&" + current_prefix + &tok[tok.find("/&")+2..]
        if let Some(idx) = tok.find("/&") {
            let newtok = format!("{}&{}{}", &tok[..idx], current_prefix, &tok[idx + 2..]);
            output.push_str(&format!("{}{}", newtok, sep));
            i += 1;
            continue;
        }
        // 3. &* => tok = tok[..tok.find("&")] + "&" + current_prefix + &tok[tok.find("&")+1..]
        if let Some(idx) = tok.find('&') {
            if idx == 0 && tok.len() > 1 {
                let newtok = format!("&{}{}", current_prefix, &tok[idx + 1..]);
                output.push_str(&format!("{}{}", newtok, sep));
                i += 1;
                continue;
            }
        }
        // 4. //* or [',.;_-=!?|$']//* => tok = tok[..tok.find("//")] + "/" + &tok[tok.find("//")+2..]
        if let Some(idx) = tok.find("//") {
            let newtok = format!("{}/{}", &tok[..idx], &tok[idx + 2..]);
            output.push_str(&format!("{}{}", newtok, sep));
            i += 1;
            continue;
        }
        // 5. [',.;_-=!?|$']/[^.]* => tok = tok[..tok.find("/")] + "/" + current_prefix + &tok[tok.find("/")+1..]
        if let Some(idx) = tok.find('/') {
            if idx > 0 && !tok[idx + 1..].starts_with('.') {
                let newtok = format!("{}/{}{}", &tok[..idx], current_prefix, &tok[idx + 1..]);
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
        // Faithful prefix/prefix-rewrite rules from preprocess-tal.sh
        // 1. &&* or [',.;_-=!?|$']&&*  => tok = tok[..tok.find("&&")] + "&" + &tok[tok.find("&&")+2..]
        if let Some(idx) = tok.find("&&") {
            let newtok = format!("{}&{}", &tok[..idx], &tok[idx + 2..]);
            output.push_str(&format!("{}{}", newtok, sep));
            i += 1;
            continue;
        }
        // 2. [',.;_-=!?|$']&* => tok = tok[..tok.find("/&")] + "&" + prefix + &tok[tok.find("/&")+2..]
        if let Some(idx) = tok.find("/&") {
            // Use prefix from context if available (here, just use prefix_label if set)
            let prefix = ""; // TODO: track prefix context if needed
            let newtok = format!("{}&{}{}", &tok[..idx], prefix, &tok[idx + 2..]);
            output.push_str(&format!("{}{}", newtok, sep));
            i += 1;
            continue;
        }
        // 3. &* => tok = tok[..tok.find("&")] + "&" + prefix + &tok[tok.find("&")+1..]
        if let Some(idx) = tok.find('&') {
            if idx == 0 && tok.len() > 1 {
                let prefix = ""; // TODO: track prefix context if needed
                let newtok = format!("&{}{}", prefix, &tok[idx + 1..]);
                output.push_str(&format!("{}{}", newtok, sep));
                i += 1;
                continue;
            }
        }
        // 4. //* or [',.;_-=!?|$']//* => tok = tok[..tok.find("//")] + "/" + &tok[tok.find("//")+2..]
        if let Some(idx) = tok.find("//") {
            let newtok = format!("{}/{}", &tok[..idx], &tok[idx + 2..]);
            output.push_str(&format!("{}{}", newtok, sep));
            i += 1;
            continue;
        }
        // 5. [',.;_-=!?|$']/[^.]* => tok = tok[..tok.find("/")] + "/" + prefix + &tok[tok.find("/")+1..]
        if let Some(idx) = tok.find('/') {
            if idx > 0 && !tok[idx + 1..].starts_with('.') {
                let prefix = ""; // TODO: track prefix context if needed
                let newtok = format!("{}/{}{}", &tok[..idx], prefix, &tok[idx + 1..]);
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

fn process_include_pattern(
    incl_pattern_str: &str,
    output: &mut String,
    sep: &String,
    _path_part: &str,
    root_dir: &PathBuf,
) -> Result<()> {
    // macro_rules! debug {
    //     ($($arg:tt)*) => {
    //         if debug_enabled {
    //             eprintln!("[chocolatal:debug] {}", format!($($arg)*));
    //         }
    //     }
    // }
    let input_path = Path::new(incl_pattern_str).to_path_buf();
    debug!(
        "root: {} Including file(s) with pattern3: {}",
        root_dir.display(),
        incl_pattern_str
    );
    if incl_pattern_str.contains('*')
        || incl_pattern_str.contains('?')
        || incl_pattern_str.contains('[')
    {
        // Split pattern into directory and filename pattern
        let (dir, file_pat) = match incl_pattern_str.rfind(std::path::MAIN_SEPARATOR) {
            Some(idx) => (&incl_pattern_str[..=idx], &incl_pattern_str[idx + 1..]),
            None => ("./", incl_pattern_str),
        };
        // Remove the trailing separator from dir, if present
        let dir_trimmed = if dir.ends_with(std::path::MAIN_SEPARATOR) && dir.len() > 1 {
            &dir[..dir.len() - 1]
        } else {
            dir
        };
        let dir_path = Path::new(dir_trimmed);

        //         let file_pat_path = root_dir.join(file_pat);
        // //debug!("Including file (from root_dir): {}", file_pat_path.display());
        // if file_pat_path.exists() && file_pat_path.is_file() {
        //     debug!("Including file (from root_dir): {}", file_pat_path.display());
        //     if file_pat_path.extension().and_then(OsStr::to_str) == Some("tal") {
        //         let incl_pre = preprocess_include_file(&root_dir, &file_pat_path)?;
        //         output.push_str(&incl_pre);
        //         output.push_str(sep);
        //     } else {
        //         hexdump(output, sep, file_pat_path)?;
        //     }
        //     return Ok(());
        // }

        let read_dir = match fs::read_dir(dir_path) {
            Ok(rd) => rd,
            Err(e) => {
                let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
                let file_pat_path = cwd.join(file_pat);
                if file_pat_path.exists() && file_pat_path.is_file() {
                    debug!("Including file (from cwd): {}", file_pat_path.display());
                    if file_pat_path.extension().and_then(OsStr::to_str) == Some("tal") {
                        let incl_pre = preprocess_include_file(&cwd, &file_pat_path)?;
                        output.push_str(&incl_pre);
                        output.push_str(sep);
                    } else {
                        hexdump(output, sep, file_pat_path)?;
                    }
                    return Ok(());
                }

                return Err(PreprocessError::Other(format!(
                    "Failed to read directory '{}': {}",
                    dir_path.display(),
                    e
                )));
            }
        };
        Ok(for entry in read_dir {
            match entry {
                Ok(entry) => {
                    let path = entry.path();
                    let fname = entry.file_name();
                    let fname_str = fname.to_str().unwrap_or("");
                    if matches_pattern(fname_str, file_pat) && path.is_file() {
                        debug!("Including file: {}", path.display());
                        if path.extension().and_then(OsStr::to_str) == Some("tal") {
                            // Try to include using cwd as current_dir first, then fallback to original logic if fails
                            let incl_pre = match preprocess_include_file(&PathBuf::from("."), &path)
                            {
                                Ok(s) => s,
                                Err(_) =>
                                //preprocess_include_file(&input_path, &path)?,
                                {
                                    match preprocess_include_file(&input_path, &path) {
                                        Ok(s) => s,
                                        Err(_) => preprocess_include_file(root_dir, &path)?,
                                    }
                                }
                            };
                            output.push_str(&incl_pre);
                            output.push_str(sep);
                        } else {
                            hexdump(output, sep, input_path.clone())?;
                        }
                    }
                }
                Err(e) => {
                    debug!("Glob error: {}", e);
                    return Err(PreprocessError::Other(e.to_string()));
                }
            }
        })
    } else {
        process_single_file(incl_pattern_str, output, sep, input_path, &root_dir)?;
        Ok(())
    }
}

fn process_single_file(
    incl_pattern_str: &str,
    output: &mut String,
    sep: &String,
    input_path: PathBuf,
    root_dir: &PathBuf,
) -> Result<()> {
    // Enable debug output if env var CHOCOLATAL_DEBUG=1
    let debug_enabled = env::var("CHOCOLATAL_DEBUG")
        .map(|v| v == "1")
        .unwrap_or(true);
    macro_rules! debug {
        ($($arg:tt)*) => {
            if debug_enabled {
                eprintln!("[chocolatal:debug] {}", format!($($arg)*));
            }
        }
    }
    debug!("Including file: {}", incl_pattern_str);
    Ok(
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
                        // Try root_dir as a fallback if not already tried
                        if let Some(ref fname) = filename {
                            if !tried_paths.contains(&root_dir.join(fname)) {
                                let root_path = root_dir.join(fname);
                                if !tried_paths.contains(&root_path) && root_path != try_path {
                                    debug!(
                                        "Trying filename in root_dir fallback: {}",
                                        root_path.display()
                                    );
                                    debug!("(was trying: {})", try_path.display());
                                    try_path = root_dir
                                        .join(try_path.file_name().unwrap_or(&OsStr::new("")));
                                    debug!("(was trying: {})", try_path.display());
                                    continue;
                                }
                            }
                        }
                        tried_paths.push(try_path.clone());
                        // Try all parent directories up to and including cwd with the same filename
                        if let Some(fname) = &filename {
                            let cwd =
                                std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
                            let mut tried_cwd = false;
                            loop {
                                let mut parent_opt = match try_path.parent() {
                                    Some(parent) => Some(parent.to_path_buf()),
                                    None => {
                                        // If no parent, try the directory itself (for root or relative files)
                                        if try_path.is_dir() {
                                            Some(try_path.clone())
                                        } else {
                                            None
                                        }
                                    }
                                };
                                if let Some(ref parent) = parent_opt {
                                    // if parent == root_dir {
                                    //     debug!("Reached root directory: {}", parent.display());
                                    //     break;

                                    // }
                                    debug!("Trying parent directory: {}", parent.display());
                                    if parent.display().to_string().is_empty() {
                                        debug!("Reached root directory (empty path).");
                                        parent_opt = Some(root_dir.clone());
                                    }
                                } else {
                                    debug!("No parent directory found.");
                                }
                                if let Some(parent) = parent_opt {
                                    // Stop if we've reached root or already tried this parent
                                    if parent == try_path || tried_paths.contains(&parent) {
                                        break;
                                    }
                                    let parent_path = parent.join(fname);
                                    if tried_paths.contains(&parent_path) {
                                        break;
                                    }
                                    debug!("Trying parent directory: {}", parent_path.display());
                                    try_path = parent_path.clone();
                                    tried_paths.push(try_path.clone());
                                    // If we've reached cwd, mark as tried
                                    if parent.canonicalize().ok() == cwd.canonicalize().ok() {
                                        tried_cwd = true;
                                        break;
                                    }
                                    continue;
                                } else {
                                    eprintln!("Reached root directory without finding file.");
                                    break;
                                }
                            }
                            // As a last resort, try just the filename in cwd if not already tried
                            if !tried_cwd {
                                let cwd_path = cwd.join(fname);
                                if !tried_paths.contains(&cwd_path) && cwd_path != try_path {
                                    debug!(
                                        "Trying filename in cwd last resort: {}",
                                        cwd_path.display()
                                    );
                                    try_path = cwd_path;
                                    continue;
                                }
                            }
                        }
                        return Err(PreprocessError::Other(format!(
                            "chocolatal::Failed to read file '{}': {}",
                            input_path.display(),
                            e
                        )));
                    }
                }
            };
            let incl_pre = preprocess(&incl_str, input_path.to_str().unwrap_or(""), &root_dir)?;
            let incl_pre = incl_pre.trim_end();
            output.push_str(&incl_pre);
            output.push_str(sep);
        } else {
            hexdump(output, sep, input_path)?;
        },
    )
}

fn hexdump(output: &mut String, sep: &String, input_path: PathBuf) -> Result<()> {
    let bytes = match fs::read(&input_path) {
        Ok(b) => b,
        Err(e) => {
            return Err(PreprocessError::Other(format!(
                "Failed to read file '{}': {}",
                input_path.display(),
                e
            )));
        }
    };
    let mut line = String::new();
    for chunk in bytes.chunks(16) {
        line.clear();
        for (j, group) in chunk.chunks(2).enumerate() {
            if j > 0 {
                line.push(' ');
            }
            for b in group {
                line.push_str(&format!("{:02x}", b));
            }
        }
        output.push_str(&line);
        output.push('\n');
    }
    output.push_str(sep);
    Ok(())
}

fn preprocess_include_file(current_dir: &PathBuf, path: &PathBuf) -> Result<String> {
    // let rel_path = path.strip_prefix(current_dir).unwrap_or(path);
    // let rel_str = rel_path.to_str().unwrap_or("");
    // let rel_str = rel_str
    //     .strip_prefix("./")
    //     .or(rel_str.strip_prefix('/'))
    //     .unwrap_or(rel_str);
    // let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    // eprintln!("Including file curr: {} (path): {} (rewritten path): {} cwd: {}", current_dir.display(), path.display(), rel_str, cwd.display());
    let incl_str = match fs::read_to_string(&path) {
        //(&rel_str) {
        Ok(s) => s,
        Err(e) => {
            return Err(PreprocessError::Other(format!(
                "chocolatal: Failed to read file '{}': {}",
                path.display(),
                e
            )));
        }
    };
    let incl_str = incl_str.trim_end();
    let incl_pre = preprocess(&incl_str, &path.display().to_string(), current_dir)?;
    Ok(incl_pre)
}

/// Runs the deluge docker container and returns the output as a String.
pub fn deluge_preprocess(_input_path: &str) -> std::io::Result<String> {
    // pub fn deluge_preprocess() -> std::io::Result<String> {
    use std::env;
    use std::process::Command;
    let cwd = env::current_dir()?;
    let volume = format!("{}:/workspace", cwd.display());
    let args = [
        "run",
        "--rm",
        "-v",
        &volume,
        "-w",
        "/workspace",
        "alpine",
        "sh",
        "./preprocess-tal.sh",
        "deluge/main.tal",
    ];
    eprintln!(
        "[deluge_preprocess:debug] Running: docker {:?}\n  cwd: {}",
        args,
        cwd.display()
    );
    let output = Command::new("docker").args(&args).output()?;
    // eprintln!("[deluge_preprocess:debug] stdout:\n{}", String::from_utf8_lossy(&output.stdout));
    // eprintln!("[deluge_preprocess:debug] stderr:\n{}", String::from_utf8_lossy(&output.stderr));
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

// Glob pattern: include all matching files
// Implement simple globbing for patterns like "*.tal" or "foo?.tal"
// Only supports '*' and '?' wildcards in the filename, not directories.

fn matches_pattern(name: &str, pattern: &str) -> bool {
    // Only supports '*' and '?' wildcards, no character classes.
    let (mut np, mut pp) = (0, 0);
    let (nlen, plen) = (name.len(), pattern.len());
    let (name, pattern) = (name.as_bytes(), pattern.as_bytes());
    let mut star = None;
    while np < nlen {
        if pp < plen && (pattern[pp] == b'?' || pattern[pp] == name[np]) {
            np += 1;
            pp += 1;
        } else if pp < plen && pattern[pp] == b'*' {
            star = Some((np, pp));
            pp += 1;
        } else if let Some((snp, spp)) = star {
            np = snp + 1;
            pp = spp + 1;
            star = Some((np, spp));
        } else {
            return false;
        }
    }
    while pp < plen && pattern[pp] == b'*' {
        pp += 1;
    }
    pp == plen
}

// --- Standalone CLI for Chocolatal Preprocessor ---
// This allows you to compile and run this file directly with rustc, for quick testing or scripting.
// Usage: rustc -o chocolatal_preprocessor src/chocolatal/mod.rs && ./chocolatal_preprocessor <input.tal> [output.tal]

#[cfg(not(test))]
#[allow(dead_code)]
fn main() {
    use std::env;

    use std::fs;
    use std::io::{self, Read, Write};
    use std::process;
    let root_dir = &std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!(
            "Usage: {} <input1.tal> [input2.tal ...] [output.tal|-]",
            args[0]
        );
        eprintln!("       If no input files or '-' is given, reads from stdin. If no output is given or '-' is given, writes to stdout.");
        process::exit(1);
    }

    // Determine output path: if last arg is not a .tal file and not '-', treat as output
    let (input_files, output_path): (Vec<&str>, Option<&str>) = {
        if args.len() == 2 {
            // One arg: could be input or '-'
            if args[1] == "-" {
                (vec!["-"], None)
            } else if args[1].ends_with(".tal") {
                (vec![&args[1]], None)
            } else {
                (vec!["-"], Some(&args[1]))
            }
        } else {
            let last = &args[args.len() - 1];
            if last == "-" || last.ends_with(".tal") {
                (args[1..].iter().map(|s| s.as_str()).collect(), None)
            } else {
                (
                    args[1..args.len() - 1].iter().map(|s| s.as_str()).collect(),
                    Some(last.as_str()),
                )
            }
        }
    };

    // Read and concatenate all input files (or stdin)
    let mut input = String::new();
    let mut input_path = String::new();
    for (i, file) in input_files.iter().enumerate() {
        let (content, path) = if *file == "-" {
            let mut buf = String::new();
            if let Err(e) = io::stdin().read_to_string(&mut buf) {
                eprintln!("Failed to read from stdin: {}", e);
                process::exit(1);
            }
            (buf, "<stdin>".to_string())
        } else {
            let content = match fs::read_to_string(file) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Failed to read {}: {}", file, e);
                    process::exit(1);
                }
            };
            (content, file.to_string())
        };
        if i > 0 {
            input.push_str("\n"); // Separate files with newline
        }
        input.push_str(&content);
        if input_path.is_empty() {
            input_path = path;
        }
    }

    match preprocess(&input, &input_path, root_dir) {
        Ok(result) => {
            match output_path {
                Some(out) if out != "-" => {
                    if let Err(e) = fs::write(out, &result) {
                        eprintln!("Failed to write {}: {}", out, e);
                        process::exit(1);
                    }
                }
                _ => {
                    // Write to stdout
                    if let Err(e) = io::stdout().write_all(result.as_bytes()) {
                        eprintln!("Failed to write to stdout: {}", e);
                        process::exit(1);
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Preprocess error: {:?}", e);
            process::exit(1);
        }
    }
}

// # Running with rust-script or as a CLI
//
// supports any number of .tal input files (or - for stdin), concatenates and preprocesses them in order, and writes to the specified output or stdout.
//
// This file is designed to be executable directly using [`rust-script`](https://rust-script.org/) or as a compiled binary.
// The shebang (`#!/usr/bin/env rust-script`) and the embedded Cargo manifest allow you to run it as a script:
//
// ```sh
// rust-script src/chocolatal/mod.rs file1.tal file2.tal ... [output.tal|-]
// ```
//
// - You can specify as many input `.tal` files as you want; they will be concatenated in order and preprocessed as one.
// - If any input is `-`, it will read from stdin at that position.
// - If the last argument is not a `.tal` file or `-`, it is treated as the output file. Otherwise, output goes to stdout.
//
// ## Example usage
//
// ```sh
// rust-script src/chocolatal/mod.rs foo.tal bar.tal baz.tal > preprocessed.tal
// rust-script src/chocolatal/mod.rs foo.tal - bar.tal output.tal
// rust-script src/chocolatal/mod.rs - output.tal
// ```
//
// ## Dependencies
//
// The embedded manifest ensures that the `glob` crate is available when running with rust-script.
//
// ## Notes
//
// - You can set the environment variable `CHOCOLATAL_DEBUG=1` to enable debug output to stderr.
// - This script can also be compiled and run as a standalone binary (see the comments at the end of the file).
/*
To build and run this file directly with rustc (bypassing Cargo):

    git clone https://github.com/rust-lang/glob.git
    # For Linux/macOS:
    rustc --crate-type=lib glob/src/lib.rs -o libglob.rlib
    rustc -L . -o chocolatal_preprocessor src/chocolatal/mod.rs --extern glob=libglob.rlib

    # For Windows (PowerShell or CMD):
    rustc --crate-type=lib glob\src\lib.rs -o libglob.rlib
    rustc -L . -o chocolatal_preprocessor.exe src\chocolatal\mod.rs --extern glob=libglob.rlib

    # Read from a file, write to stdout:
    ./chocolatal_preprocessor input.tal > output.tal

    # Read from stdin, write to stdout:
    cat input.tal | ./chocolatal_preprocessor - > output.tal

    # Read from a file, write to a file:
    ./chocolatal_preprocessor input.tal output.tal

    # Read from stdin, write to a file:
    cat input.tal | ./chocolatal_preprocessor - output.tal
*/

// MIT License - Copyright 2025 @notchoc, David Horner
