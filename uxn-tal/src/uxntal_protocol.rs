/**
 # uxntal Protocol Specification

 This module documents and implements the `uxntal://` protocol handler for launching TAL/ROM files via URL.

 ## Protocol Format

 URLs are parsed as:

    ```text
    uxntal:var1:var2^val2:var3^^val3://actual_url
    ```

 - Variables are separated by `:`
 - Key-value pairs use `^` or `^^` as separators (double `^^` for Windows shell escaping)
 - The actual TAL/ROM file URL is after the `://`

 ## Supported Variables

 - `emu`    : Select emulator backend (`buxn`, `cuxn`, `uxn`). Example: `emu^^buxn`
 - `widget` : Enable widget mode (transparent, no decorations, always-on-top). Example: `widget`
 - `ontop`  : Control always-on-top (`ontop^false` disables it in widget mode)
 - `debug`  : Enable debug console (Windows only). Example: `debug`

 ## Examples

 - `uxntal:emu^uxn://https://wiki.xxiivv.com/etc/catclock.tal.txt` launches catclock in `uxnemu` emulator
 - `uxntal:widget://https://wiki.xxiivv.com/etc/catclock.tal.txt` passes `--widget` flag to emulator
 - `uxntal:widget:ontop^false://...` disables always-on-top in widget mode
 - `uxntal:widget:debug://...` enables debug console (Windows only)

 ## Notes

 - Only the variables above are supported for protocol invocation; arbitrary input is restricted for security.
 - The actual ROM/TAL file is always specified after the `://`.
 - See README for more details and security notes.
*/
use std::collections::HashMap;

/// Type of protocol variable
#[derive(Debug, Clone)]
pub enum ProtocolVarType {
    Bool,
    String,
    Enum(&'static [&'static str]),
    Int,
    Float,
}

/// Protocol variable specification
#[derive(Debug, Clone)]
pub struct ProtocolVar {
    pub name: &'static str,
    pub description: &'static str,
    pub example: &'static str,
    pub var_type: ProtocolVarType,
}

/// Supported protocol variables
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
];

/// Supported !bang query parameters (for debugchrome and similar protocols)
#[derive(Debug, Clone)]
pub struct ProtocolQueryVar {
    pub name: &'static str,
    pub description: &'static str,
    pub example: &'static str,
    pub var_type: ProtocolVarType,
}

pub static BANG_VARS: &[ProtocolQueryVar] = &[
    ProtocolQueryVar {
        name: "fit",
        description: "Fit mode for ROM display (none, contain, cover, stretch)",
        example: "!fit=cover",
        var_type: ProtocolVarType::String,
    },
    ProtocolQueryVar {
        name: "timeout",
        description: "Timeout in seconds before the emulator exits (alias: t)",
        example: "!timeout=60",
        var_type: ProtocolVarType::Float,
    },
    ProtocolQueryVar {
        name: "t",
        description: "Timeout in seconds before the emulator exits (alias for timeout)",
        example: "!t=60",
        var_type: ProtocolVarType::Float,
    },
    ProtocolQueryVar {
        name: "x",
        description: "Window X position (pixels or percent or complex)",
        example: "!x=100",
        var_type: ProtocolVarType::String,
    },
    ProtocolQueryVar {
        name: "y",
        description: "Window Y position (pixels or percent or complex)",
        example: "!y=100",
        var_type: ProtocolVarType::String,
    },
    ProtocolQueryVar {
        name: "w",
        description: "Window width (pixels or percent or complex)",
        example: "!w=800",
        var_type: ProtocolVarType::String,
    },
    ProtocolQueryVar {
        name: "h",
        description: "Window height (pixels or percent or complex)",
        example: "!h=600",
        var_type: ProtocolVarType::String,
    },
];

/// Parsed protocol variable value
#[derive(Debug, Clone)]
pub enum ProtocolVarVar {
    Bool(bool),
    String(String),
    Enum(&'static str),
    Int(i64),
    Float(f64),
}

/// Result of protocol parsing
#[derive(Debug, Clone)]
pub struct ProtocolParseResult {
    pub raw: HashMap<String, String>,
    pub proto_vars: HashMap<String, ProtocolVarVar>,
    pub query_vars: HashMap<String, ProtocolVarVar>,
    pub url: String,
    pub protocol: String, // the uxntal protocol portion (before //url)
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
        let mut raw_map = HashMap::new();
        let mut proto_vars_map = HashMap::new();
        let mut query_vars_map = HashMap::new();
        if !raw_url.starts_with("uxntal:") {
            return ProtocolParseResult {
                raw: raw_map,
                proto_vars: HashMap::new(),
                query_vars: HashMap::new(),
                url: String::new(),
                protocol: String::new(),
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
        let protocol = kv_part.to_string();
        for part in kv_part.split(':') {
            if part.is_empty() {
                continue;
            }
            let decoded = percent_decode_str(part).decode_utf8_lossy();
            let mut split_idx = None;
            let mut sep_len = 0;
            if let Some(idx) = decoded.find("^^") {
                split_idx = Some(idx);
                sep_len = 2;
            } else if let Some(idx) = decoded.find('^') {
                split_idx = Some(idx);
                sep_len = 1;
            }
            let (key, value) = if let Some(idx) = split_idx {
                (&decoded[..idx], &decoded[idx + sep_len..])
            } else {
                (decoded.as_ref(), "true") // treat as bool flag if no value
            };
            println!(
                "[ProtocolParser::parse] parsed key='{}' value='{}'",
                key, value
            );
            raw_map.insert(key.to_string(), value.to_string());
            // Validate against PROTOCOL_VARS
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
                            query_vars_map.insert(key.to_string(), ProtocolVarVar::Int(i));
                        }
                    }
                    ProtocolVarType::Float => {
                        let v = value.trim().replace('%', "");
                        if let Ok(f) = v.parse::<f64>() {
                            query_vars_map.insert(key.to_string(), ProtocolVarVar::Float(f));
                        }
                    }
                }
            }
            // Validate against BANG_VARS
            if let Some(var_def) = BANG_VARS.iter().find(|v| v.name == key) {
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
                            query_vars_map.insert(key.to_string(), ProtocolVarVar::Bool(b));
                        }
                    }
                    ProtocolVarType::Enum(valids) => {
                        let val = value.trim();
                        if let Some(&valid) = valids.iter().find(|&&v| v.eq_ignore_ascii_case(val))
                        {
                            query_vars_map.insert(key.to_string(), ProtocolVarVar::Enum(valid));
                        }
                    }
                    ProtocolVarType::String => {
                        if !value.is_empty() {
                            query_vars_map
                                .insert(key.to_string(), ProtocolVarVar::String(value.to_string()));
                        }
                    }
                    ProtocolVarType::Int => {
                        if let Ok(i) = value.trim().parse::<i64>() {
                            query_vars_map.insert(key.to_string(), ProtocolVarVar::Int(i));
                        }
                    }
                    ProtocolVarType::Float => {
                        let v = value.trim().replace('%', "");
                        if let Ok(f) = v.parse::<f64>() {
                            query_vars_map.insert(key.to_string(), ProtocolVarVar::Float(f));
                        }
                    }
                }
            }
        }
        let url = if url_part.starts_with("//https://")
            || url_part.starts_with("//http://")
            || url_part.starts_with("//file://")
        {
            // Remove all but one protocol prefix (e.g., //https://https://... -> https://...)
            let mut s = url_part.trim_start_matches("//");
            if s.starts_with("https://https://") {
                s = &s[8..];
            } else if s.starts_with("http://http://") {
                s = &s[7..];
            }
            s.to_string()
        } else if let Some(stripped) = url_part.strip_prefix("//https//") {
            format!("https://{}", stripped)
        } else if let Some(stripped) = url_part.strip_prefix("//http//") {
            format!("http://{}", stripped)
        } else if let Some(stripped) = url_part.strip_prefix("//file//") {
            format!("file://{}", stripped)
        } else {
            url_part.to_string()
        };
        ProtocolParseResult {
            raw: raw_map,
            proto_vars: proto_vars_map,
            query_vars: query_vars_map,
            url,
            protocol,
        }
    }

    /// Extract the actual target URL from a parsed uxntal URL
    pub fn extract_target(url: &str) -> Option<String> {
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

// This module is intended to be extended with per-emulator mapping modules (see emu_cuxn.rs, emu_uxn.rs, emu_buxn.rs)

use percent_encoding::percent_decode_str;

/// EmulatorMapperFactory: returns the correct EmulatorArgMapper for a ProtocolParseResult
/// Trait for checking if an emulator binary is available in PATH
pub trait EmulatorPathCheck {
    /// Returns Some(path) if the emulator binary is found in PATH, None otherwise
    fn is_available_in_path(result: &ProtocolParseResult) -> Option<std::path::PathBuf>;
}

/// EmulatorMapperFactory: returns the correct EmulatorArgMapper and path for a ProtocolParseResult
pub fn get_emulator_launcher(
    result: &ProtocolParseResult,
) -> Option<(Box<dyn EmulatorLauncher>, std::path::PathBuf)> {
    match result.proto_vars.get("emu") {
        Some(ProtocolVarVar::Enum("buxn")) => {
            if let Some(path) = crate::emu_buxn::BuxnMapper::is_available_in_path(result) {
                Some((Box::new(crate::emu_buxn::BuxnMapper), path))
            } else {
                None
            }
        }
        Some(ProtocolVarVar::Enum("uxn")) => {
            if let Some(path) = crate::emu_uxn::UxnMapper::is_available_in_path(result) {
                Some((Box::new(crate::emu_uxn::UxnMapper), path))
            } else {
                None
            }
        }
        _ => {
            if let Some(path) = crate::emu_cuxn::CuxnMapper::is_available_in_path(result) {
                Some((Box::new(crate::emu_cuxn::CuxnMapper), path))
            } else {
                None
            }
        }
    }
}

/// EmulatorMapperFactory: returns the correct EmulatorArgMapper for a ProtocolParseResult
pub fn get_emulator_mapper(
    result: &ProtocolParseResult,
) -> Option<(Box<dyn EmulatorArgMapper>, std::path::PathBuf)> {
    match result.proto_vars.get("emu") {
        Some(ProtocolVarVar::Enum("buxn")) => {
            if let Some(path) = crate::emu_buxn::BuxnMapper::is_available_in_path(result) {
                Some((Box::new(crate::emu_buxn::BuxnMapper), path))
            } else {
                None
            }
        }
        Some(ProtocolVarVar::Enum("uxn")) => {
            if let Some(path) = crate::emu_uxn::UxnMapper::is_available_in_path(result) {
                Some((Box::new(crate::emu_uxn::UxnMapper), path))
            } else {
                None
            }
        }
        _ => {
            if let Some(path) = crate::emu_cuxn::CuxnMapper::is_available_in_path(result) {
                Some((Box::new(crate::emu_cuxn::CuxnMapper), path))
            } else {
                None
            }
        }
    }
}

/// Trait for launching an emulator with the correct command and arguments
pub trait EmulatorLauncher {
    /// Build a std::process::Command for this emulator, given the protocol parse result and ROM path
    fn build_command(
        &self,
        result: &ProtocolParseResult,
        rom_path: &str,
        emulator_path: &std::path::Path,
    ) -> std::process::Command;
}
