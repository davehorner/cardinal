//! Chocolatal: TAL Preprocessor for uxn-tal
//
// This module provides preprocessing for Uxn TAL assembly files, inspired by preprocess-tal.sh.
// It operates purely on &str, performing file inclusion, label/prefix expansion, lambda/loop label generation,
// path rewriting, and special token rewrites. See README.md for details.

use std::fs;
use std::path::{Path, PathBuf};

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
    let current_dir = Path::new(path).parent().map(|p| p.to_path_buf()).unwrap_or_else(|| PathBuf::from("."));

    // Split input into tokens and separators (whitespace, newlines)
    let mut token = String::new();
    let mut sep = String::new();
    let mut chars = input.chars().peekable();
    let mut tokens = Vec::new();
    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            if !token.is_empty() {
                tokens.push((token.clone(), sep.clone()));
                token.clear();
                sep.clear();
            }
            sep.push(c);
            chars.next();
        } else {
            if !sep.is_empty() && !token.is_empty() {
                tokens.push((token.clone(), sep.clone()));
                token.clear();
                sep.clear();
            }
            token.push(c);
            chars.next();
        }
    }
    if !token.is_empty() || !sep.is_empty() {
        tokens.push((token, sep));
    }

    let mut i = 0;
    while i < tokens.len() {
        let (ref tok, ref sep) = tokens[i];
        // File inclusion: ~file.tal
        if tok.starts_with('~') && tok.ends_with(".tal") {
            let incl_path = current_dir.join(&tok[1..]);
            let incl_str = fs::read_to_string(&incl_path).map_err(PreprocessError::Io)?;
            let incl_pre = preprocess(&incl_str, incl_path.to_str().unwrap_or(""))?;
            output.push_str(&incl_pre);
            output.push_str(sep);
            i += 1;
            continue;
        }
        // Lambda/loop label generation: '>{'
        if tok == ">{" {
            lambda_counter += 1;
            stack.push(lambda_counter);
            output.push_str(&format!("&ʎ{}{}", lambda_counter, sep));
            i += 1;
            continue;
        }
        // Lambda/loop label close: |} ?} or }
        if (tok == "|}" || tok == "?}" || tok == "}") && !stack.is_empty() {
            let n = stack.pop().unwrap();
            output.push_str(&format!("&ʎ{}{}", n, sep));
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
