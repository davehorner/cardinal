/// Runs all TAL heuristics and prints the results for the given source bytes (lossy, strips replacements).
pub fn print_all_tal_heuristics_bytes(bytes: &[u8]) {
    match std::str::from_utf8(bytes) {
        Ok(s) => {
            println!("[heuristics] Input is valid UTF-8");
            print_all_tal_heuristics(s);
        }
        Err(e) => {
            let lossy = String::from_utf8_lossy(bytes);
            let replacement_count = lossy.chars().filter(|&c| c == '\u{fffd}').count();
            println!("[heuristics] Input is NOT valid UTF-8: {e}");
            println!("[heuristics] {} replacement characters would be present if lossy conversion was used.", replacement_count);
            let mut tal_source = lossy.to_string();
            tal_source.retain(|c| c != '\u{fffd}');
            print_all_tal_heuristics(&tal_source);
        }
    }
}
use crate::lexer::{Lexer, Token};
use std::collections::HashMap;
/// Runs all TAL heuristics and prints the results for the given source.
pub fn print_all_tal_heuristics(tal_source: &str) {
    println!("--- TAL Heuristics ---");
    println!("Proportional font: {}", uses_proportional_font(tal_source));
    println!("Has manifest: {}", has_manifest(tal_source));
    println!("Has theme: {}", has_theme(tal_source));
    println!("Has snarf: {}", has_snarf(tal_source));
    println!("Has metadata: {}", has_metadata(tal_source));
    println!("Resolution: {:?}", get_resolution(tal_source));
    println!("File types: {:?}", get_file_types(tal_source));
    println!("Expects a path: {}", heuristic_expects_a_path(tal_source));
    println!("Uses console: {}", heuristic_uses_console(tal_source));
    println!("Uses GUI: {}", heuristic_uses_gui(tal_source));
    let stats = analyze_tal_source_stats(tal_source);
    println!("Stats: {:?}", stats);
}

/// Returns true if the TAL source meta block contains a proportional font setting.
pub fn uses_proportional_font(tal_source: &str) -> bool {
    let mut lexer = Lexer::new(tal_source.to_string(), None);
    let tokens = match lexer.tokenize() {
        Ok(t) => t,
        Err(_) => return false,
    };
    let mut in_meta = false;
    let mut last_key: Option<String> = None;
    for t in &tokens {
        match &t.token {
            Token::LabelDef(_, label) if label == "meta" => {
                in_meta = true;
                continue;
            }
            Token::LabelDef(_, _) if in_meta => break, // End of meta block
            Token::SublabelDef(key) | Token::Word(key) if in_meta && key.starts_with('&') => {
                last_key = Some(key.trim_start_matches('&').to_string());
            }
            Token::RawString(val) | Token::Word(val) if in_meta && last_key.is_some() => {
                let key = last_key.take().unwrap();
                if (key == "font" || key == "font_type" || key == "font-family")
                    && val.to_lowercase().contains("proportional")
                {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

/// Returns true if the TAL source has a manifest block.
pub fn has_manifest(tal_source: &str) -> bool {
    heuristic_manifest_lexer(tal_source).is_some()
}

/// Returns true if the TAL source meta block contains 'theme'.
pub fn has_theme(tal_source: &str) -> bool {
    tal_source.contains("@theme/")
}

/// Returns true if the TAL source meta block contains 'snarf'.
pub fn has_snarf(tal_source: &str) -> bool {
    heuristic_meta_data_lexer(tal_source).is_some_and(|m| m.contains_key("snarf"))
}

/// Returns true if the TAL source meta block contains 'metadata'.
pub fn has_metadata(tal_source: &str) -> bool {
    heuristic_meta_data_lexer(tal_source).is_some_and(|m| m.contains_key("metadata"))
}

/// Returns the screen resolution (width, height) if found.
pub fn get_resolution(tal_source: &str) -> Option<(u32, u32)> {
    heuristic_screen_dimensions(tal_source)
}

/// Returns a list of file types (e.g., txt, bin, icn) found in includes or meta.
pub fn get_file_types(tal_source: &str) -> Vec<String> {
    let mut types = Vec::new();
    // Check includes
    for inc in crate::lexer::extract_includes_from_lexer(tal_source, None) {
        if let Some(ext) = inc.split('.').next_back() {
            types.push(ext.to_string());
        }
    }
    // Check meta
    if let Some(meta) = heuristic_meta_data_lexer(tal_source) {
        for v in meta.values() {
            if v == "txt" || v == "bin" || v == "icn" {
                types.push(v.clone());
            }
        }
    }
    types
}
#[derive(Debug, Clone)]
pub struct ManifestEntry {
    pub flags: Option<String>,
    pub shortcut: Option<String>,
    pub command: Option<String>,
    pub label: Option<String>,
    pub section: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Manifest {
    pub entries: Vec<ManifestEntry>,
}

/// Uses the TAL lexer to extract all manifest entries from TAL source.
pub fn heuristic_manifest_lexer(tal_source: &str) -> Option<Manifest> {
    let mut lexer = Lexer::new(tal_source.to_string(), None);
    let tokens = match lexer.tokenize() {
        Ok(t) => t,
        Err(_) => return None,
    };
    let mut in_manifest = false;
    let mut current_section: Option<String> = None;
    let mut entries = Vec::new();
    let mut entry: Option<ManifestEntry> = None;
    for t in &tokens {
        match &t.token {
            Token::LabelDef(_, label) if label == "manifest" => {
                in_manifest = true;
                continue;
            }
            Token::LabelDef(_, _) if in_manifest => {
                // End of manifest block
                break;
            }
            Token::Newline if in_manifest => {
                // Push entry if complete
                if let Some(e) = entry.take() {
                    entries.push(e);
                }
            }
            Token::Word(w) if in_manifest && w.chars().all(|c| c.is_ascii_digit()) => {
                // Flags (e.g., 04, 01)
                if entry.is_none() {
                    entry = Some(ManifestEntry {
                        flags: Some(w.clone()),
                        shortcut: None,
                        command: None,
                        label: None,
                        section: current_section.clone(),
                    });
                } else if let Some(ref mut e) = entry {
                    e.flags = Some(w.clone());
                }
            }
            Token::CharLiteral(c) if in_manifest => {
                // Shortcut (e.g., 'n)
                if let Some(ref mut e) = entry {
                    e.shortcut = Some(c.to_string());
                }
            }
            Token::Word(w) if in_manifest && w.starts_with(':') => {
                // Command (e.g., :file-new)
                if let Some(ref mut e) = entry {
                    e.command = Some(w.clone());
                }
            }
            Token::RawString(s) if in_manifest => {
                // Label (e.g., "New)
                if let Some(ref mut e) = entry {
                    e.label = Some(s.clone());
                }
            }
            Token::Word(w) if in_manifest && w.starts_with('"') => {
                // Section or label (e.g., "Application)
                let section = w.trim_start_matches('"').to_string();
                current_section = Some(section.clone());
                if let Some(ref mut e) = entry {
                    e.section = Some(section);
                }
            }
            _ => {}
        }
    }
    // Push last entry if still open
    if let Some(e) = entry {
        entries.push(e);
    }
    if !entries.is_empty() {
        Some(Manifest { entries })
    } else {
        None
    }
}
/// Uses the TAL lexer to extract all @dict blocks and their key-value pairs from TAL source.
pub fn heuristic_dicts_lexer(
    tal_source: &str,
) -> Vec<(String, std::collections::HashMap<String, String>)> {
    let mut lexer = Lexer::new(tal_source.to_string(), None);
    let tokens = match lexer.tokenize() {
        Ok(t) => t,
        Err(_) => return vec![],
    };
    let mut dicts = Vec::new();
    let mut current_dict: Option<String> = None;
    let mut current_map = std::collections::HashMap::new();
    let mut last_key: Option<String> = None;
    for t in &tokens {
        match &t.token {
            Token::LabelDef(_, label) if label.starts_with("dict/") => {
                if let Some(dict_name) = current_dict.take() {
                    if !current_map.is_empty() {
                        dicts.push((dict_name, current_map.clone()));
                        current_map.clear();
                    }
                }
                current_dict = Some(label.trim_start_matches("dict/").to_string());
            }
            Token::SublabelDef(key) | Token::Word(key)
                if current_dict.is_some() && key.starts_with('&') =>
            {
                last_key = Some(key.trim_start_matches('&').to_string());
            }
            Token::RawString(val) | Token::Word(val)
                if current_dict.is_some() && last_key.is_some() =>
            {
                current_map.insert(last_key.take().unwrap(), val.clone());
            }
            Token::LabelDef(_, _) if current_dict.is_some() => {
                if let Some(dict_name) = current_dict.take() {
                    if !current_map.is_empty() {
                        dicts.push((dict_name, current_map.clone()));
                        current_map.clear();
                    }
                }
            }
            _ => {}
        }
    }
    // Push last dict if still open
    if let Some(dict_name) = current_dict.take() {
        if !current_map.is_empty() {
            dicts.push((dict_name, current_map));
        }
    }
    dicts
}

/// Uses the TAL lexer to extract the @meta block and its key-value pairs from TAL source.
pub fn heuristic_meta_data_lexer(tal_source: &str) -> Option<HashMap<String, String>> {
    let mut lexer = Lexer::new(tal_source.to_string(), None);
    let tokens = match lexer.tokenize() {
        Ok(t) => t,
        Err(_) => return None,
    };
    let mut meta = HashMap::new();
    let mut in_meta = false;
    let mut last_key: Option<String> = None;
    for t in &tokens {
        match &t.token {
            Token::LabelDef(_, label) if label == "meta" => {
                in_meta = true;
                continue;
            }
            Token::LabelDef(_, _) if in_meta => break, // End of meta block
            Token::SublabelDef(key) | Token::Word(key) if in_meta && key.starts_with('&') => {
                last_key = Some(key.trim_start_matches('&').to_string());
            }
            Token::RawString(val) | Token::Word(val) if in_meta && last_key.is_some() => {
                meta.insert(last_key.take().unwrap(), val.clone());
            }
            _ => {}
        }
    }
    if !meta.is_empty() {
        Some(meta)
    } else {
        None
    }
}
/// Attempts to extract the @meta block and its key-value pairs from TAL source.
/// Returns a map of meta keys to their values if found.
pub fn heuristic_meta_data(tal_source: &str) -> Option<std::collections::HashMap<String, String>> {
    use std::collections::HashMap;
    let mut meta_map = HashMap::new();
    // Find the @meta block
    let meta_start = tal_source.find("@meta");
    if let Some(start) = meta_start {
        // Find the end of the block (next @ or end of file)
        let rest = &tal_source[start..];
        let end = rest.find("@").filter(|&e| e > 0).unwrap_or(rest.len());
        let meta_block = &rest[..end];
        // Parse lines like '&key ( value )' or '&key "value"
        for line in meta_block.lines() {
            let line = line.trim();
            if let Some(stripped) = line.strip_prefix('&') {
                let parts: Vec<&str> = stripped.splitn(2, ['(', '"']).collect();
                if parts.len() == 2 {
                    let key = parts[0].trim().to_string();
                    let value = parts[1]
                        .trim_matches(|c| c == ')' || c == '"' || c == ' ')
                        .to_string();
                    meta_map.insert(key, value);
                }
            }
        }
        if !meta_map.is_empty() {
            return Some(meta_map);
        }
    }
    None
}
use regex::Regex;

/// Attempts to extract screen width and height from TAL source.
/// Returns (width, height) if found, otherwise None.
pub fn heuristic_screen_dimensions(tal_source: &str) -> Option<(u32, u32)> {
    // Example matches: #0270 .Screen/width DEO2, #0180 .Screen/height DEO2
    let width_re = Regex::new(r"#([0-9a-fA-F]{4})\s+\.Screen/width DEO2").ok()?;
    let height_re = Regex::new(r"#([0-9a-fA-F]{4})\s+\.Screen/height DEO2").ok()?;
    let width = width_re
        .captures(tal_source)
        .and_then(|cap| u32::from_str_radix(&cap[1], 16).ok());
    let height = height_re
        .captures(tal_source)
        .and_then(|cap| u32::from_str_radix(&cap[1], 16).ok());
    match (width, height) {
        (Some(w), Some(h)) => Some((w, h)),
        _ => None,
    }
}
/// Returns true if the TAL source contains direct or indirect file access, indicating it expects a file/path.
pub fn heuristic_expects_a_path(tal_source: &str) -> bool {
    // Direct file access
    if tal_source.contains(".File/name DEO2") || tal_source.contains(".File/read DEO2") {
        return true;
    }
    // Indirect file access via file/<load>
    if tal_source.contains("file/<load>") {
        return true;
    }
    // Handler setup that leads to file access (e.g., ;src/on-console .Console/vector DEO2)
    if tal_source.contains(";src/on-console .Console/vector DEO2") {
        return true;
    }
    false
}

/// Returns true if the TAL source uses the console (input/output)
pub fn heuristic_uses_console(tal_source: &str) -> bool {
    tal_source.contains(".Console/vector")
        || tal_source.contains("input/<listen>")
        || tal_source.contains(";input/on-console .Console/vector DEO2")
        || tal_source.contains(".Console/read DEI")
}

/// Returns true if the TAL source uses the GUI (screen)
pub fn heuristic_uses_gui(tal_source: &str) -> bool {
    tal_source.contains(".Screen/vector")
        || tal_source.contains(".Screen/width DEO2")
        || tal_source.contains(".Screen/height DEO2")
        || tal_source.contains(".Screen/x DEO2")
        || tal_source.contains(".Screen/y DEO2")
        || tal_source.contains(".Screen/pixel DEO")
        || tal_source.contains(".Screen/sprite DEO")
}

// Heuristics and statistics for TAL source and ROM analysis
#[derive(Debug, Default, Clone)]
pub struct ProbeTalStats {
    pub num_includes: usize,
    pub num_labels: usize,
    pub num_vectors: usize,
    pub num_comments: usize,
    pub num_bytes: usize,
    pub num_words: usize,
    pub num_lines: usize,
    // Add more as needed
}

pub fn analyze_tal_source_stats(tal_source: &str) -> ProbeTalStats {
    ProbeTalStats {
        num_lines: tal_source.lines().count(),
        num_includes: tal_source.matches("@include").count(),
        num_labels: tal_source.matches("@").count(),
        num_vectors: tal_source.matches("/vector").count(),
        num_comments: tal_source.matches("(").count(),
        num_words: tal_source.split_whitespace().count(),
        num_bytes: tal_source.len(),
    }
}
// Trait for assembler integration with optional ProbeTal

#[derive(Debug, Clone)]
pub struct ProbeTalResult {
    pub characteristics: ProbeTalCharacteristics,
    pub rom_path: Option<std::path::PathBuf>,
    pub rom_data: Vec<u8>,
    pub symbols: Vec<u8>, // These are the actual symbols after the ROM
                          // Add more fields as needed (e.g., stats, metadata)
}

pub trait AssemblerWithProbe {
    fn assemble_with_probe<P: ProbeTal + ?Sized>(
        &self,
        tal_source: &str,
        probe: Option<&P>,
        rom_path: Option<&std::path::Path>,
    ) -> ProbeTalResult;
}
// Interface for probe analysis results for TAL/UXN ROMs
#[derive(Debug, Default, Clone)]
pub struct ProbeTalCharacteristics {
    pub expects_a_path: bool,
    pub uses_console: bool,
    pub uses_gui: bool,
    pub static_notes: Vec<String>,
}

pub trait ProbeTal {
    fn analyze(&self, rom_bytes: &[u8]) -> ProbeTalCharacteristics;
    fn analyze_path(&self, path: &std::path::Path) -> ProbeTalCharacteristics;
    fn analyze_source(&self, tal_source: &str) -> ProbeTalCharacteristics;
}
