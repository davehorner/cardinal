use std::{borrow::Cow, fmt};
use uxn_tal_common::cache::RomCache;
// This module is intended to be extended with per-emulator mapping modules (see emu_cuxn.rs, emu_uxn.rs, emu_buxn.rs)
use std::collections::HashMap;

use crate::{emu_buxn, emu_cuxn, emu_uxn, percent_decode_or_original};

impl ProtocolQueryVarVar {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            ProtocolQueryVarVar::String(s) => Some(s),
            ProtocolQueryVarVar::Enum(e) => Some(e),
            ProtocolQueryVarVar::Invalid(s) => Some(s),
            _ => None,
        }
    }
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            ProtocolQueryVarVar::Int(i) => Some(*i),
            ProtocolQueryVarVar::String(s) => s.parse().ok(),
            _ => None,
        }
    }
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            ProtocolQueryVarVar::Float(f) => Some(*f),
            ProtocolQueryVarVar::String(s) => s.parse().ok(),
            _ => None,
        }
    }
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            ProtocolQueryVarVar::Bool(b) => Some(*b),
            ProtocolQueryVarVar::String(s) => match s.to_ascii_lowercase().as_str() {
                "1" | "true" | "yes" | "on" => Some(true),
                "0" | "false" | "no" | "off" => Some(false),
                _ => None,
            },
            _ => None,
        }
    }
}

impl fmt::Display for ProtocolQueryVarVar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProtocolQueryVarVar::Bool(b) => write!(f, "{}", b),
            ProtocolQueryVarVar::String(s) => write!(f, "{}", s),
            ProtocolQueryVarVar::Enum(e) => write!(f, "{}", e),
            ProtocolQueryVarVar::Int(i) => write!(f, "{}", i),
            ProtocolQueryVarVar::Float(fl) => write!(f, "{}", fl),
            ProtocolQueryVarVar::Invalid(s) => write!(f, "Invalid({})", s),
        }
    }
}

impl ProtocolVarVar {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            ProtocolVarVar::String(s) => Some(s),
            _ => None,
        }
    }
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            ProtocolVarVar::Int(i) => Some(*i),
            ProtocolVarVar::String(s) => s.parse().ok(),
            _ => None,
        }
    }
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            ProtocolVarVar::Float(f) => Some(*f),
            ProtocolVarVar::String(s) => s.parse().ok(),
            _ => None,
        }
    }
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            ProtocolVarVar::Bool(b) => Some(*b),
            ProtocolVarVar::String(s) => match s.to_ascii_lowercase().as_str() {
                "1" | "true" | "yes" | "on" => Some(true),
                "0" | "false" | "no" | "off" => Some(false),
                _ => None,
            },
            _ => None,
        }
    }
}

impl fmt::Display for ProtocolVarVar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProtocolVarVar::Bool(b) => write!(f, "{}", b),
            ProtocolVarVar::String(s) => write!(f, "{}", s),
            ProtocolVarVar::Enum(e) => write!(f, "{}", e),
            ProtocolVarVar::Int(i) => write!(f, "{}", i),
            ProtocolVarVar::Float(fl) => write!(f, "{}", fl),
            ProtocolVarVar::Invalid(s) => write!(f, "Invalid({})", s),
        }
    }
}

// Shared protocol variable types and statics for both lib.rs and build.rs
// use std::borrow::Cow; // already imported above if needed

#[derive(Debug, Clone, PartialEq)]
pub enum ProtocolQueryVarVar {
    Bool(bool),
    String(String),
    Enum(&'static str),
    Int(i64),
    Float(f64),
    Invalid(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProtocolVarVar {
    Bool(bool),
    String(String),
    Enum(&'static str),
    Int(i64),
    Float(f64),
    Invalid(String),
}

#[derive(Debug, Clone)]
pub enum ProtocolVarType {
    Bool,
    String,
    Enum(&'static [&'static str]),
    Int,
    Float,
}

#[derive(Debug, Clone)]
pub struct ProtocolVar {
    pub name: &'static str,
    pub description: &'static str,
    pub example: &'static str,
    pub var_type: ProtocolVarType,
}

pub static PROTOCOL_VARS: &[ProtocolVar] = &[
    ProtocolVar {
        name: "transparent",
        description: "Transparent color for widget/background (hex RGB, e.g. ffffff for white)",
        example: "transparent^ff00ff",
        var_type: ProtocolVarType::String,
    },
    ProtocolVar {
        name: "scale",
        description: "Scale factor for the window (float)",
        example: "scale^2.0",
        var_type: ProtocolVarType::Float,
    },
    ProtocolVar {
        name: "fit",
        description: "Fit mode for ROM display (none, contain, cover, stretch)",
        example: "fit^cover",
        var_type: ProtocolVarType::String,
    },
    ProtocolVar {
        name: "timeout",
        description: "Timeout in seconds before the emulator exits (alias: t)",
        example: "timeout^60",
        var_type: ProtocolVarType::Float,
    },
    ProtocolVar {
        name: "t",
        description: "Timeout in seconds before the emulator exits (alias for timeout)",
        example: "t^60",
        var_type: ProtocolVarType::Float,
    },
    ProtocolVar {
        name: "emu",
        description: "Select emulator backend",
        example: "emu^^buxn",
        var_type: ProtocolVarType::Enum(&["buxn", "cuxn", "uxn"]),
    },
    ProtocolVar {
        name: "widget",
        description: "Enable widget mode (transparent, no decorations, always-on-top)",
        example: "widget",
        var_type: ProtocolVarType::Bool,
    },
    ProtocolVar {
        name: "ontop",
        description: "Control always-on-top (true/false)",
        example: "ontop^false",
        var_type: ProtocolVarType::Bool,
    },
    ProtocolVar {
        name: "debug",
        description: "Enable debug console (Windows only)",
        example: "debug",
        var_type: ProtocolVarType::Bool,
    },
    ProtocolVar {
        name: "efx",
        description: "Effect name or identifier for emulator (string)",
        example: "efx^invert",
        var_type: ProtocolVarType::String,
    },
    ProtocolVar {
        name: "efxmode",
        description: "Effect mode for emulator (string)",
        example: "efxmode^blend",
        var_type: ProtocolVarType::String,
    },
    ProtocolVar {
        name: "orca",
        description: "Orca mode: run the orca ROM with the given .orca file. Automatically set if the URL ends with .orca.",
        example: "orca",
        var_type: ProtocolVarType::Bool,
    },
];

#[derive(Debug, Clone)]
pub enum ProtocolQueryType {
    Bool,
    String,
    Enum(&'static [&'static str]),
    Int,
    Float,
}

#[derive(Debug, Clone)]
pub struct ProtocolQueryVar {
    pub name: Cow<'static, str>,
    pub description: Cow<'static, str>,
    pub example: Cow<'static, str>,
    pub var_type: ProtocolQueryType,
    pub value: ProtocolQueryVarVar, // DUMMY for static tables, real value for runtime
}

pub static BANG_VARS: &[ProtocolQueryVar] = &[
    ProtocolQueryVar {
        name: Cow::Borrowed("fit"),
        description: Cow::Borrowed("Fit mode for ROM display (none, contain, cover, stretch)"),
        example: Cow::Borrowed("!fit=cover"),
        var_type: ProtocolQueryType::Enum(&["none", "contain", "cover", "stretch"]),
        value: ProtocolQueryVarVar::String(String::new()),
    },
    ProtocolQueryVar {
        name: Cow::Borrowed("timeout"),
        description: Cow::Borrowed("Timeout in seconds before the emulator exits (alias: t)"),
        example: Cow::Borrowed("!timeout=60"),
        var_type: ProtocolQueryType::Float,
        value: ProtocolQueryVarVar::String(String::new()),
    },
    ProtocolQueryVar {
        name: Cow::Borrowed("t"),
        description: Cow::Borrowed(
            "Timeout in seconds before the emulator exits (alias for timeout)",
        ),
        example: Cow::Borrowed("!t=60"),
        var_type: ProtocolQueryType::Float,
        value: ProtocolQueryVarVar::String(String::new()),
    },
    ProtocolQueryVar {
        name: Cow::Borrowed("x"),
        description: Cow::Borrowed("Window X position (pixels or percent or complex)"),
        example: Cow::Borrowed("!x=100"),
        var_type: ProtocolQueryType::String,
        value: ProtocolQueryVarVar::String(String::new()),
    },
    ProtocolQueryVar {
        name: Cow::Borrowed("y"),
        description: Cow::Borrowed("Window Y position (pixels or percent or complex)"),
        example: Cow::Borrowed("!y=100"),
        var_type: ProtocolQueryType::String,
        value: ProtocolQueryVarVar::String(String::new()),
    },
    ProtocolQueryVar {
        name: Cow::Borrowed("w"),
        description: Cow::Borrowed("Window width (pixels or percent or complex)"),
        example: Cow::Borrowed("!w=800"),
        var_type: ProtocolQueryType::String,
        value: ProtocolQueryVarVar::String(String::new()),
    },
    ProtocolQueryVar {
        name: Cow::Borrowed("h"),
        description: Cow::Borrowed("Window height (pixels or percent or complex)"),
        example: Cow::Borrowed("!h=600"),
        var_type: ProtocolQueryType::String,
        value: ProtocolQueryVarVar::String(String::new()),
    },
];

/// Result of protocol parsing
#[derive(Debug, Clone)]
pub struct ProtocolParseResult {
    pub raw: HashMap<String, String>,
    pub proto_vars: HashMap<String, ProtocolVarVar>,
    pub query_vars: HashMap<String, ProtocolQueryVar>,
    pub bang_vars: HashMap<String, ProtocolQueryVar>,
    pub url: String,
    pub protocol: String,     // the uxntal protocol portion (before //url)
    pub query_string: String, // the full query string, including '?' if present
}

impl ProtocolParseResult {
    /// Get a validated variable by name
    pub fn get(&self, name: &str) -> Option<&ProtocolVarVar> {
        self.proto_vars.get(name)
    }
}

/// ProtocolParser provides parsing and extraction for uxntal URLs
pub struct ProtocolParser;

impl ProtocolParser {
    /// Parse a uxntal: URL into a ProtocolParseResult with raw and validated maps
    pub fn parse(raw_url: &str) -> ProtocolParseResult {
        let mut raw_map: HashMap<String, String> = HashMap::new();
        let mut proto_vars_map: HashMap<String, ProtocolVarVar> = HashMap::new();
        let mut query_vars_map: HashMap<String, ProtocolQueryVar> = HashMap::new();
        let mut bang_vars_map: HashMap<String, ProtocolQueryVar> = HashMap::new();
        if !raw_url.starts_with("uxntal:") {
            return ProtocolParseResult {
                raw: raw_map,
                proto_vars: HashMap::new(),
                query_vars: HashMap::new(),
                bang_vars: HashMap::new(),
                url: String::new(),
                protocol: String::new(),
                query_string: String::new(),
            };
        }
        let s = raw_url.trim_start_matches("uxntal:");
        // Accept both uxntal:...://url and uxntal:...//url (with or without trailing colon)
        let (kv_part, url_part) = if let Some(idx) = s.find("//") {
            // If the part before // ends with ':', trim it
            let (kv, url) = (&s[..idx], &s[idx..]);
            let kv = if kv.ends_with(':') {
                kv.strip_suffix(':').unwrap()
            } else {
                kv
            };
            (kv, url)
        } else {
            (s, "")
        };
        // The protocol field should be the full valid uxntal:...:// prefix
        let protocol = format!("uxntal:{}://", kv_part);
        // Parse protocol section variables (always proto_vars, even if they start with '!')
        for part in kv_part.split(':') {
            if part.is_empty() {
                continue;
            }
            // If the part contains '=', split on that first (before percent-decoding)
            let (key, value) = if let Some(eq_idx) = part.find('=') {
                let (k, v) = part.split_at(eq_idx);
                let k_decoded = percent_decode_or_original(k);
                let v_decoded = percent_decode_or_original(&v[1..]);
                (k_decoded.to_string(), v_decoded.to_string())
            } else {
                let decoded = percent_decode_or_original(part);
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
                    (
                        decoded[..idx].to_string(),
                        decoded[idx + sep_len..].to_string(),
                    )
                } else {
                    (decoded.to_string(), "true".to_string())
                }
            };
            raw_map.insert(key.to_string(), value.to_string());
            if let Some(var_def) = PROTOCOL_VARS.iter().find(|v| v.name == key) {
                match &var_def.var_type {
                    ProtocolVarType::Bool => {
                        let val = value.trim().to_ascii_lowercase();
                        let parsed = match val.as_str() {
                            "1" | "true" | "yes" | "on" => Some(true),
                            "0" | "false" | "no" | "off" => Some(false),
                            "" => Some(true),
                            _ => None,
                        };
                        if let Some(b) = parsed {
                            proto_vars_map.insert(key.to_string(), ProtocolVarVar::Bool(b));
                        }
                    }
                    ProtocolVarType::Enum(valids) => {
                        let val = value.trim();
                        if let Some(&valid) = valids.iter().find(|&&v| v.eq_ignore_ascii_case(val))
                        {
                            proto_vars_map.insert(key.to_string(), ProtocolVarVar::Enum(valid));
                        }
                    }
                    ProtocolVarType::String => {
                        if !value.is_empty() {
                            proto_vars_map
                                .insert(key.to_string(), ProtocolVarVar::String(value.to_string()));
                        }
                    }
                    ProtocolVarType::Int => {
                        if let Ok(i) = value.trim().parse::<i64>() {
                            proto_vars_map.insert(key.to_string(), ProtocolVarVar::Int(i));
                        }
                    }
                    ProtocolVarType::Float => {
                        let v = value.trim().replace('%', "");
                        if let Ok(f) = v.parse::<f64>() {
                            proto_vars_map.insert(key.to_string(), ProtocolVarVar::Float(f));
                        }
                    }
                }
            } else {
                // Store unknown protocol section vars as string
                proto_vars_map.insert(key.to_string(), ProtocolVarVar::String(value.to_string()));
            }
        }
        // Parse query string for bang vars and query vars BEFORE url normalization
        let (url_for_normalization, query_string) = if let Some(qmark) = url_part.find('?') {
            let raw_query_string = &url_part[qmark..];
            // Normalize &amp; to &
            let normalized_query_string = raw_query_string.replace("&amp;", "&");
            let query = &normalized_query_string[1..]; // skip the '?'
            for pair in query.split('&') {
                if pair.is_empty() {
                    continue;
                }
                let mut it = pair.splitn(2, '=');
                let k = it.next().unwrap_or("");
                if k.is_empty() {
                    continue;
                }
                let v = it.next().unwrap_or("");
                // Percent-decode key and value
                let k_decoded = percent_decode_or_original(k);
                let v_decoded = percent_decode_or_original(v);
                if k_decoded.starts_with('!') {
                    if let Some(key) = k_decoded.strip_prefix('!') {
                        if let Some(var_def) = BANG_VARS.iter().find(|bv| bv.name == key) {
                            let value = match &var_def.var_type {
                                ProtocolQueryType::Bool => {
                                    let val = v_decoded.trim().to_ascii_lowercase();
                                    let parsed = match val.as_str() {
                                        "1" | "true" | "yes" | "on" => Some(true),
                                        "0" | "false" | "no" | "off" => Some(false),
                                        "" => Some(true),
                                        _ => None,
                                    };
                                    if let Some(b) = parsed {
                                        ProtocolQueryVarVar::Bool(b)
                                    } else {
                                        ProtocolQueryVarVar::String(v_decoded.clone())
                                    }
                                }
                                ProtocolQueryType::Enum(valids) => {
                                    let val = v_decoded.trim();
                                    if let Some(&valid) =
                                        valids.iter().find(|&&v| v.eq_ignore_ascii_case(val))
                                    {
                                        ProtocolQueryVarVar::Enum(valid)
                                    } else {
                                        ProtocolQueryVarVar::Invalid(v_decoded.clone())
                                    }
                                }
                                ProtocolQueryType::String => {
                                    ProtocolQueryVarVar::String(v_decoded.clone())
                                }
                                ProtocolQueryType::Int => {
                                    if let Ok(i) = v_decoded.trim().parse::<i64>() {
                                        ProtocolQueryVarVar::Int(i)
                                    } else {
                                        ProtocolQueryVarVar::String(v_decoded.clone())
                                    }
                                }
                                ProtocolQueryType::Float => {
                                    let v = v_decoded.trim().replace('%', "");
                                    if let Ok(f) = v.parse::<f64>() {
                                        ProtocolQueryVarVar::Float(f)
                                    } else {
                                        ProtocolQueryVarVar::String(v_decoded.clone())
                                    }
                                }
                            };
                            bang_vars_map.insert(
                                key.to_string(),
                                ProtocolQueryVar {
                                    name: var_def.name.clone(),
                                    description: var_def.description.clone(),
                                    example: var_def.example.clone(),
                                    var_type: var_def.var_type.clone(),
                                    value,
                                },
                            );
                        } else {
                            bang_vars_map.insert(
                                key.to_string(),
                                ProtocolQueryVar {
                                    name: Cow::Owned(key.to_string()),
                                    description: Cow::Borrowed(""),
                                    example: Cow::Borrowed(""),
                                    var_type: ProtocolQueryType::String,
                                    value: ProtocolQueryVarVar::String(v_decoded.clone()),
                                },
                            );
                        }
                    }
                } else {
                    // Standard query var
                    query_vars_map.insert(
                        k_decoded.to_string(),
                        ProtocolQueryVar {
                            name: Cow::Owned(k_decoded.clone()),
                            description: Cow::Borrowed(""),
                            example: Cow::Borrowed(""),
                            var_type: ProtocolQueryType::String,
                            value: ProtocolQueryVarVar::String(v_decoded.clone()),
                        },
                    );
                }
            }
            (&url_part[..qmark], normalized_query_string)
        } else {
            (url_part, String::new())
        };
        let url = if url_for_normalization.starts_with("//https://")
            || url_for_normalization.starts_with("//http://")
            || url_for_normalization.starts_with("//file://")
        {
            // Remove all but one protocol prefix (e.g., //https://https://... -> https://...)
            let mut s = url_for_normalization.trim_start_matches("//");
            if s.starts_with("https://https://") {
                s = &s[8..];
            } else if s.starts_with("http://http://") {
                s = &s[7..];
            }
            s.to_string()
        } else if let Some(stripped) = url_for_normalization.strip_prefix("//https//") {
            format!("https://{}", stripped)
        } else if let Some(stripped) = url_for_normalization.strip_prefix("//http//") {
            format!("http://{}", stripped)
        } else if let Some(stripped) = url_for_normalization.strip_prefix("//file//") {
            format!("file://{}", stripped)
        } else {
            url_for_normalization.to_string()
        };

        // If the url ends with .orca, set orca=true in proto_vars_map
        if url.trim().to_ascii_lowercase().ends_with(".orca") {
            proto_vars_map.insert("orca".to_string(), ProtocolVarVar::Bool(true));
        }
        ProtocolParseResult {
            raw: raw_map,
            proto_vars: proto_vars_map,
            query_vars: query_vars_map,
            bang_vars: bang_vars_map,
            url,
            protocol,
            query_string,
        }
    }

    /// Extract the actual target URL from a parsed uxntal URL
    pub fn extract_target(url: &str) -> Option<String> {
        fn pct_decode(s: &str) -> String {
            percent_decode_or_original(s)
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
        let s = if s.starts_with("//https://")
            || s.starts_with("//http://")
            || s.starts_with("//file://")
        {
            s.trim_start_matches("//").to_string()
        } else if let Some(stripped) = s.strip_prefix("//https//") {
            format!("https://{}", stripped)
        } else if let Some(stripped) = s.strip_prefix("//http//") {
            format!("http://{}", stripped)
        } else if let Some(stripped) = s.strip_prefix("//file//") {
            format!("file://{}", stripped)
        } else {
            s.to_string()
        };
        if s.starts_with("open") {
            let (path, rest) = if let Some(qpos) = s.find('?') {
                (&s[..qpos], &s[qpos + 1..])
            } else {
                (&s[..], "")
            };
            if path == "open" || path == "open/" {
                if let Some(v) = qs_get(rest, "url") {
                    return Some(v);
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
            let dec = pct_decode(&s);
            if dec.starts_with("http://")
                || dec.starts_with("https://")
                || dec.starts_with("file://")
            {
                return Some(dec);
            }
        }
        if s.starts_with("http://") || s.starts_with("https://") || s.starts_with("file://") {
            return Some(s.to_string());
        }
        Some(s.to_string())
    }
}

/// Emulator argument mapping trait
pub trait EmulatorArgMapper {
    fn map_args(&self, result: &ProtocolParseResult) -> Vec<String>;
}

/// EmulatorMapperFactory: returns the correct EmulatorArgMapper for a ProtocolParseResult
/// Trait for checking if an emulator binary is available in PATH
pub trait EmulatorPathCheck {
    /// Returns Some(path) if the emulator binary is found in PATH, None otherwise
    fn is_available_in_path(result: &ProtocolParseResult) -> Option<std::path::PathBuf>;
}

/// EmulatorMapperFactory: returns the correct EmulatorArgMapper and path for a ProtocolParseResult
pub fn get_emulator_launcher<'a>(
    result: &ProtocolParseResult,
    rom_cache: &'a dyn RomCache,
) -> Option<(Box<dyn EmulatorLauncher<'a> + 'a>, std::path::PathBuf)> {
    match result.proto_vars.get("emu") {
        Some(ProtocolVarVar::Enum("buxn")) => {
            if let Some(path) = emu_buxn::BuxnMapper::is_available_in_path() {
                Some((
                    Box::new(emu_buxn::BuxnMapper { rom_cache })
                        as Box<dyn EmulatorLauncher<'a> + 'a>,
                    path,
                ))
            } else {
                None
            }
        }
        Some(ProtocolVarVar::Enum("uxn")) => {
            if let Some(path) = emu_uxn::UxnMapper::is_available_in_path() {
                Some((
                    Box::new(emu_uxn::UxnMapper { rom_cache })
                        as Box<dyn EmulatorLauncher<'a> + 'a>,
                    path,
                ))
            } else {
                None
            }
        }
        _ => {
            if let Some(path) = emu_cuxn::CuxnMapper::is_available_in_path(result) {
                Some((
                    Box::new(emu_cuxn::CuxnMapper { rom_cache })
                        as Box<dyn EmulatorLauncher<'a> + 'a>,
                    path,
                ))
            } else {
                None
            }
        }
    }
}

/// EmulatorMapperFactory: returns the correct EmulatorArgMapper for a ProtocolParseResult
pub fn get_emulator_mapper<'a>(
    result: &ProtocolParseResult,
    rom_cache: &'a dyn RomCache,
) -> Option<(Box<dyn EmulatorArgMapper + 'a>, Option<std::path::PathBuf>)> {
    match result.proto_vars.get("emu") {
        Some(ProtocolVarVar::Enum("buxn")) => Some((
            Box::new(emu_buxn::BuxnMapper { rom_cache }),
            emu_buxn::BuxnMapper::is_available_in_path(),
        )),
        Some(ProtocolVarVar::Enum("uxn")) => Some((
            Box::new(emu_uxn::UxnMapper { rom_cache }),
            emu_uxn::UxnMapper::is_available_in_path(),
        )),
        _ => Some((
            Box::new(emu_cuxn::CuxnMapper { rom_cache }),
            emu_cuxn::CuxnMapper::is_available_in_path(result),
        )),
    }
}

/// Trait for launching an emulator with the correct command and arguments
pub trait EmulatorLauncher<'a> {
    /// Build a std::process::Command for this emulator, given the protocol parse result and ROM path
    fn build_command(
        &self,
        result: &ProtocolParseResult,
        rom_path: &str,
        emulator_path: &std::path::Path,
    ) -> std::process::Command;
}
