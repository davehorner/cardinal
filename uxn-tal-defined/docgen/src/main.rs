use uxn_tal_defined::v1::{all_emulator_kinds, emulator_kind_name, EmulatorKind};
use std::fs;
use std::path::Path;
use uxn_tal_defined::ProtocolQueryType;
 use uxn_tal_defined::BANG_VARS;
 use uxn_tal_defined::ProtocolVarType;
 use uxn_tal_defined::PROTOCOL_VARS;
/// Run this file as a binary to generate README.md from README.template
fn main() {
    // Make lookup relative to the source file location
    // Print current working directory for debugging
    let cwd = std::env::current_dir().unwrap();
    println!("Current working directory: {}", cwd.display());

    // Edit README.md in crate root

    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let readme_path1 = crate_root.join("README.md");
    let readme_path2 = crate_root.parent().unwrap().join("README.md");
    let (readme_path, mut readme) = if readme_path1.exists() {
        (readme_path1.clone(), fs::read_to_string(&readme_path1).expect("read README.md in manifest dir"))
    } else if readme_path2.exists() {
        (readme_path2.clone(), fs::read_to_string(&readme_path2).expect("read README.md in parent dir"))
    } else {
        panic!("README.md not found in manifest or parent directory");
    };

    // Replace between ## Protocol Section Variables and next ##
    let protocol_table = table_for_protocol_vars();
    readme = replace_section(
        &readme,
        "## Protocol Section Variables",
        &protocol_table,
    );

    // Replace between ## Bang Query Variables and next ##
    let bang_table = table_for_bang_vars();
    readme = replace_section(
        &readme,
        "## Bang Query Variables",
        &bang_table,
    );

        // Insert emulator compatibility matrix
    let matrix_intro = "This table shows which protocol/bang variables affect the command-line arguments for each supported emulator. An `X` means the variable is mapped to CLI args for that emulator.\n\n";
    let compat_matrix = generate_emu_compat_matrix();
    println!("\n---\n[docgen] Generated Emulator Compatibility Matrix:\n{}{}\n---\n", matrix_intro, compat_matrix);
    let override_note = "**Note:** If both a protocol variable (e.g. `x`) and a bang variable (e.g. `!x`) are provided, the protocol variable typically takes precedence and overrides the bang variable.";
    let matrix_section = if compat_matrix.trim().is_empty() {
        format!("{}(No compatibility data available)\n", matrix_intro)
    } else {
        format!("{}{}\n\n{}\n", matrix_intro, compat_matrix, override_note)
    };
    readme = replace_section(
        &readme,
        "## Emulator Compatibility Matrix",
        &matrix_section,
    );

    fs::write(&readme_path, &readme).expect("write README.md");
    println!("README.md updated at {}", readme_path.display());
    // Debug: print first 20 lines of the README after writing
    let debug_lines: Vec<_> = readme.lines().take(20).collect();
    println!("[docgen] First 20 lines of written README.md:\n{}\n---", debug_lines.join("\n"));
}

/// Replace the section in `text` that starts with `section_header` and ends at the next '##', with `replacement`.
fn replace_section(text: &str, section_header: &str, replacement: &str) -> String {
    let mut out = String::new();
    let mut lines = text.lines();
    let mut in_section = false;
    let mut replaced = false;
    while let Some(line) = lines.next() {
        if !in_section && line.trim() == section_header {
            // Write the header
            out.push_str(line);
            out.push('\n');
            out.push('\n');
            // Write the replacement
            out.push_str(replacement);
            if !replacement.ends_with('\n') {
                out.push('\n');
            }
            out.push('\n');
            // Skip lines until next section or EOF, skipping blank/whitespace lines
            in_section = true;
            replaced = true;
            while let Some(next_line) = lines.next() {
                if next_line.trim().is_empty() {
                    continue;
                }
                if next_line.trim_start().starts_with("##") && next_line.trim() != section_header {
                    out.push_str(next_line);
                    out.push('\n');
                    in_section = false;
                    break;
                }
            }
        } else if !in_section {
            out.push_str(line);
            out.push('\n');
        }
    }
    if !replaced {
        // If the section header was not found, just return the original text
        text.to_string()
    } else {
        out
    }
}
/// Shared doc generation logic for protocol variables
use std::fmt::Write;

pub fn generate_protocol_readme() -> String {
    let mut doc = String::new();
    writeln!(doc, "# uxntal Protocol Specification\n").unwrap();
    writeln!(doc, "\nThis module documents and implements the `uxntal://` protocol handler for launching TAL/ROM files via URL.\n").unwrap();
    writeln!(doc, "## Protocol Format\n").unwrap();
    writeln!(doc, "URLs are parsed as:\n").unwrap();
    writeln!(doc, "    ```text\n    uxntal:var1:var2^val2:var3^^val3://actual_url\n    ```\n").unwrap();
    writeln!(doc, "- Variables are separated by `:`\n- Key-value pairs use `^` or `^^` as separators (double `^^` for Windows shell escaping)\n- The actual TAL/ROM file URL is after the `://`\n").unwrap();
    writeln!(doc, "## Supported Variables\n").unwrap();
    writeln!(doc, "- `emu`    : Select emulator backend (`buxn`, `cuxn`, `uxn`). Example: `emu^^buxn`\n- `widget` : Enable widget mode (transparent, no decorations, always-on-top). Example: `widget`\n- `ontop`  : Control always-on-top (`ontop^false` disables it in widget mode)\n- `debug`  : Enable debug console (Windows only). Example: `debug`\n").unwrap();
    writeln!(doc, "## Examples\n").unwrap();
    writeln!(doc, "- `uxntal:emu^uxn://https://wiki.xxiivv.com/etc/catclock.tal.txt` launches catclock in `uxnemu` emulator\n- `uxntal:widget://https://wiki.xxiivv.com/etc/catclock.tal.txt` passes `--widget` flag to emulator\n- `uxntal:widget:ontop^false://...` disables always-on-top in widget mode\n- `uxntal:widget:debug://...` enables debug console (Windows only)\n").unwrap();
    writeln!(doc, "## Notes\n").unwrap();
    writeln!(doc, "- Only the variables above are supported for protocol invocation; arbitrary input is restricted for security.\n- The actual ROM/TAL file is always specified after the `://`.\n- See README for more details and security notes.\n").unwrap();

    // Protocol variables table
    writeln!(doc, "## Protocol Section Variables\n").unwrap();
    writeln!(doc, "| Name | Type | Description | Example |").unwrap();
    writeln!(doc, "|------|------|-------------|---------|").unwrap();
    for var in PROTOCOL_VARS {
        let typ = match &var.var_type {
            ProtocolVarType::Bool => "Bool",
            ProtocolVarType::String => "String",
            ProtocolVarType::Int => "Int",
            ProtocolVarType::Float => "Float",
            ProtocolVarType::Enum(variants) => {
                let joined = variants.join(", ");
                doc.push_str(&format!("Enum ({})", joined));
                continue;
            }
        };
        writeln!(doc, "| `{}` | {} | {} | `{}` |", var.name, typ, var.description, var.example).unwrap();
    }

    // Bang variables table
    writeln!(doc, "\n## Bang Query Variables\n").unwrap();
    writeln!(doc, "| Name | Type | Description | Example |").unwrap();
    writeln!(doc, "|------|------|-------------|---------|").unwrap();
    for var in BANG_VARS {
        let typ = match &var.var_type {
            ProtocolQueryType::Bool => "Bool",
            ProtocolQueryType::String => "String",
            ProtocolQueryType::Int => "Int",
            ProtocolQueryType::Float => "Float",
            ProtocolQueryType::Enum(variants) => {
                let joined = variants.join(", ");
                doc.push_str(&format!("Enum ({})", joined));
                continue;
            }
        };
        writeln!(doc, "| `{}` | {} | {} | `{}` |", var.name, typ, var.description, var.example).unwrap();
    }

    writeln!(doc, "\n## Example URL\n").unwrap();
    writeln!(doc, "```text\nuxntal:emu^^buxn:widget://https://example.com/rom.tal?!fit=cover&!timeout=60\n```
").unwrap();
    doc
}

pub fn table_for_protocol_vars() -> String {
    let mut out = String::from("| Name | Type | Description | Example |\n|------|------|-------------|---------|\n");
    for var in PROTOCOL_VARS {
        let typ = match &var.var_type {
            ProtocolVarType::Bool => "Bool",
            ProtocolVarType::String => "String",
            ProtocolVarType::Int => "Int",
            ProtocolVarType::Float => "Float",
            ProtocolVarType::Enum(variants) => {
                let joined = variants.join(", ");
                out.push_str(&format!("| `{}` | Enum ({}) | {} | `{}` |\n", var.name, joined, var.description, var.example));
                continue;
            }
        };
        out.push_str(&format!("| `{}` | {} | {} | `{}` |\n", var.name, typ, var.description, var.example));
    }
    out
}

pub fn table_for_bang_vars() -> String {
    let mut out = String::from("| Name | Type | Description | Example |\n|------|------|-------------|---------|\n");
    for var in BANG_VARS {
        let typ = match &var.var_type {
            ProtocolQueryType::Bool => "Bool",
            ProtocolQueryType::String => "String",
            ProtocolQueryType::Int => "Int",
            ProtocolQueryType::Float => "Float",
            ProtocolQueryType::Enum(variants) => {
                let joined = variants.join(", ");
                out.push_str(&format!("| `{}` | Enum ({}) | {} | `{}` |\n", var.name, joined, var.description, var.example));
                continue;
            }
        };
        out.push_str(&format!("| `{}` | {} | {} | `{}` |\n", var.name, typ, var.description, var.example));
    }
    out
}

/// Generate a compatibility matrix for protocol/bang vars vs emulators
fn generate_emu_compat_matrix() -> String {
    use uxn_tal_defined::{PROTOCOL_VARS, BANG_VARS};
    let emus = all_emulator_kinds();
    let mut out = String::new();
    // Header
    out.push_str("| Variable | ");
    for &emu in emus {
        out.push_str(emulator_kind_name(emu));
        out.push_str(" | ");
    }
    out.push_str("Example | Type ");
    out.push_str("\n|---|");
    for _ in emus { out.push_str("---|"); }
    out.push_str("---|---|\n");
    use std::collections::HashSet;
    let proto_names: HashSet<_> = PROTOCOL_VARS.iter().map(|v| v.name).collect();
    let bang_names: HashSet<_> = BANG_VARS.iter().map(|v| v.name.as_ref()).collect();
    // Protocol vars
    for var in PROTOCOL_VARS {
        let is_both = bang_names.contains(var.name);
        let type_str = if is_both { "both" } else { "proto" };
        let var_col = if is_both {
            format!("`{}`/`!{}`", var.name, var.name)
        } else {
            format!("`{}`", var.name)
        };
        out.push_str(&format!("| {} |", var_col));
        for &emu in emus {
            let (affects, _) = emu_var_affects_cli(emu, &var.name);
            if affects {
                out.push_str(" X | ");
            } else {
                out.push_str("   | ");
            }
        }
        out.push_str(&format!(" {} | {} |\n", var.example, type_str));
    }
    // Bang vars
    for var in BANG_VARS {
        let is_both = proto_names.contains(var.name.as_ref());
        // Only print bang-only rows (proto rows already printed above)
        if is_both { continue; }
        let type_str = "bang";
        out.push_str(&format!("| `!{}` |", var.name));
        for &emu in emus {
            let (affects, _) = emu_var_affects_cli(emu, &var.name);
            if affects {
                out.push_str(" X | ");
            } else {
                out.push_str("   | ");
            }
        }
        out.push_str(&format!(" {} | {} |\n", var.example, type_str));
    }
    out
}

/// Returns (affects_cli, example) for a given emulator and variable name
fn emu_var_affects_cli(emu: EmulatorKind, var: &str) -> (bool, &'static str) {
    match emu {
        EmulatorKind::Buxn => match var {
            "orca" | "basic" | "emu" | "arg1" | "stdin" => (true, ""),
            _ => (false, ""),
        },
        EmulatorKind::Uxn => match var {
            "orca" | "basic" | "emu" | "arg1" | "stdin" => (true, ""),
            _ => (false, ""),
        },
        EmulatorKind::Cuxn => match var {
            "widget" | "ontop" | "debug" | "emu" | "orca" | "basic" | "transparent" | "timeout" | "t" | "efx" | "efxmode" | "x" | "y" | "w" | "h" | "fit" | "theme" | "scale" | "opacity" | "borderless" | "fullscreen" | "vsync" | "keep_focus" | "screenshot" | "openwindow" | "close" | "refresh" | "monitor" | "id" | "arg1" | "stdin" => (true, ""),
            _ => (false, ""),
        },
    }
}