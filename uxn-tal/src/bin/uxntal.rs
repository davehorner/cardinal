use std::{env, fs, path::{PathBuf, Path}, process::exit};
use uxn_tal::{Assembler, AssemblerError};
use std::collections::VecDeque;
use uxn_tal::debug;
fn main() {
    if let Err(e) = real_main() {
        eprintln!("error: {e}");
        exit(1);
    }
}

fn real_main() -> Result<(), AssemblerError> {
    let mut args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() || needs_help(&args) {
        print_usage();
        if args.is_empty() { exit(1); }
        return Ok(());
    }

    // Flags
    let mut want_version = false;
    let mut want_verbose = false;
    let mut want_cmp = false;
    let mut rust_iface: Option<String> = None; // module name (None => not requested)

    // Collect positional (non-flag) args after flag parsing
    let mut positional: Vec<String> = Vec::new();

    for a in args.drain(..) {
        if a == "--version" || a == "-V" {
            want_version = true;
        } else if a == "--verbose" || a == "-v" {
            want_verbose = true;
        } else if a == "--cmp" {
            want_cmp = true;
        } else if a.starts_with("--rust-interface") {
            // Forms:
            //   --rust-interface
            //   --rust-interface=ModuleName
            if let Some(eq) = a.find('=') {
                let name = a[eq + 1 ..].trim();
                if !name.is_empty() {
                    rust_iface = Some(name.to_string());
                } else {
                    rust_iface = Some("symbols".to_string());
                }
            } else {
                rust_iface = Some("symbols".to_string());
            }
        } else if a.starts_with('-') {
            eprintln!("unknown flag: {a}");
            print_usage();
            exit(2);
        } else {
            positional.push(a);
        }
    }

    if want_version {
        println!("uxntal {} (library)", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    if positional.is_empty() {
        eprintln!("missing input file");
        print_usage();
        exit(2);
    }

    // NEW: robust resolution of the input path (expanded search roots)
    let raw_input = &positional[0];
    let input_path = match resolve_input_path(raw_input) {
        Some(p) => p,
        None => {
            return Err(simple_err(
                std::path::Path::new(raw_input),
                "input file not found (tried direct, +.tal, multi-root recursive scan)",
            ));
        }
    };
    // Canonical (or absolute fallback) before chdir so later paths remain valid
    let canon_input = input_path
        .canonicalize()
        .unwrap_or_else(|_| input_path.clone());

    // Compute rom path (absolute) before changing directory
    let rom_path = if positional.len() > 1 {
        let supplied = PathBuf::from(&positional[1]);
        if supplied.is_absolute() {
            supplied
        } else {
            std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")).join(supplied)
        }
    } else {
        // If no explicit output, derive sibling .rom next to input file
        canon_input.with_extension("rom")
    };

    // Change current working directory to the input file's directory (for relative includes)
    if let Some(parent) = canon_input.parent() {
        if let Err(e) = std::env::set_current_dir(parent) {
            if want_verbose {
                eprintln!("warning: failed to chdir to {}: {e}", parent.display());
            }
        } else if want_verbose {
            eprintln!("Changed working directory to {}", parent.display());
        }
    }

    let source = fs::read_to_string(&canon_input)
        .map_err(|e| simple_err(&canon_input, &format!("failed to read: {e}")))?;

    if want_verbose {
        eprintln!("Resolved input : {}", canon_input.display());
        eprintln!("Output ROM     : {}", rom_path.display());
        if rust_iface.is_some() {
            eprintln!("Rust interface : enabled");
        }
    }

    let mut asm = Assembler::new();

    // --- ADD: cmp mode ---
    if want_cmp {
        // Use DebugAssembler from the debug module
        let mut dbg = debug::DebugAssembler::default();
        let res = dbg.assemble_and_compare(
            &source,
            &canon_input.display().to_string()
            ,true
        );
        return res.map(|_| ());
    }

    let rom = asm.assemble(&source, Some(canon_input.display().to_string()))?;
    fs::write(&rom_path, &rom)
        .map_err(|e| simple_err(&rom_path, &format!("failed to write rom: {e}")))?;

    if want_verbose {
        eprintln!("Wrote ROM ({} bytes)", rom.len());
    } else {
        println!("{} ({} bytes)", rom_path.display(), rom.len());
    }

    if let Some(module_name) = rust_iface {
        let mod_src = asm.generate_rust_interface_module(&module_name);
        let iface_path = rom_path.with_extension("rom.symbols.rs");
        fs::write(&iface_path, mod_src)
            .map_err(|e| simple_err(&iface_path, &format!("failed to write rust interface: {e}")))?;
        if want_verbose {
            eprintln!("Wrote Rust interface module: {}", iface_path.display());
        } else {
            println!("{}", iface_path.display());
        }
    }

    Ok(())
}

fn needs_help(args: &[String]) -> bool {
    args.iter().any(|a| a == "--help" || a == "-h" || a == "help")
}

fn print_usage() {
    eprintln!(
"Usage:
  uxntal [flags] <input.tal> [output.rom]

Flags:
  --version, -V         Show version and exit
  --verbose, -v         Verbose output
  --rust-interface[=M]  Emit Rust symbols module (default module name: symbols)
  --cmp                 Compare disassembly for all backends
  --help, -h            Show this help

Behavior:
  If output.rom omitted, use input path with .rom extension.
  Rust interface file path: <output>.rom.symbols.rs"
    );
}

fn simple_err(path: &std::path::Path, msg: &str) -> AssemblerError {
    AssemblerError::SyntaxError {
        path: path.display().to_string(),
        line: 0,
        position: 0,
        message: msg.to_string(),
        source_line: String::new(),
    }
}

// UPDATED: multi-root recursive search
fn resolve_input_path(arg: &str) -> Option<PathBuf> {
    use std::collections::VecDeque;
    let direct = PathBuf::from(arg);
    if direct.exists() {
        return Some(direct);
    }
    if direct.extension().is_none() {
        let with_ext = direct.with_extension("tal");
        if with_ext.exists() {
            return Some(with_ext);
        }
    }

    // Candidate roots to scan (dedup later)
    let mut roots: Vec<PathBuf> = Vec::new();
    if let Ok(cwd) = std::env::current_dir() {
        roots.push(cwd);
    }
    roots.push(PathBuf::from(env!("CARGO_MANIFEST_DIR"))); // crate root
    if let Some(parent) = Path::new(env!("CARGO_MANIFEST_DIR")).parent() {
        roots.push(parent.to_path_buf());
        if let Some(grand) = parent.parent() {
            roots.push(grand.to_path_buf());
        }
    }

    // Deduplicate
    roots.sort();
    roots.dedup();

    // Only do recursive scan if filename has no path separators
    if arg.contains('/') || arg.contains('\\') {
        return None;
    }

    for root in roots {
        if !root.is_dir() { continue; }
        if let Some(found) = recursive_find(&root, arg, 12_000) {
            return Some(found);
        }
        // Try arg.tal variation
        let alt = format!("{arg}.tal");
        if let Some(found) = recursive_find(&root, &alt, 12_000) {
            return Some(found);
        }
    }
    None
}

fn recursive_find(root: &Path, needle: &str, cap: usize) -> Option<PathBuf> {
    let mut q = VecDeque::new();
    q.push_back(root.to_path_buf());
    let mut visited = 0usize;
    while let Some(dir) = q.pop_front() {
        if visited >= cap { break; }
        visited += 1;
        let rd = fs::read_dir(&dir).ok()?;
        for entry in rd.flatten() {
            let p = entry.path();
            if p.is_dir() {
                if q.len() < 4096 { q.push_back(p); }
                continue;
            }
            if let Some(name) = p.file_name().and_then(|s| s.to_str()) {
                if name == needle {
                    return Some(p);
                }
            }
        }
    }
    None
}
