use std::collections::VecDeque;
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::exit,
};
use uxn_tal::chocolatal;
use uxn_tal::debug;
use uxn_tal::drif::ensure_drifblim_repo;
use uxn_tal::dis::ensure_uxndis_repo;
use uxn_tal::{Assembler, AssemblerError};
use std::process::Command;

fn main() {
    println!("{:?}", ensure_drifblim_repo());
    ensure_uxndis_repo();
    if let Err(e) = real_main() {
        eprintln!("error: {e}");
        exit(1);
    }
}

fn real_main() -> Result<(), AssemblerError> {
    let mut args: Vec<String> = env::args().skip(1).collect();
    let root_dir = &std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    println!("root dir: {:?}", root_dir);
    let mut pre = false;
    let mut preprocess_only = false;
    let mut no_intermediate = false;
    let mut want_version = false;
    let mut want_verbose = false;
    let mut want_cmp = false;
    let mut want_cmp_pp = false;
    let mut want_stdin = false;
    let mut drif_mode = false;
    let mut rust_iface: Option<String> = None; // module name (None => not requested)
    let mut use_root: Option<PathBuf> = None; // root name (None => not requested)

    // Collect positional (non-flag) args after flag parsing
    let mut positional: Vec<String> = Vec::new();

    for a in args.drain(..) {
        if a == "--version" || a == "-V" {
            want_version = true;
        } else if a == "--verbose" || a == "-v" {
            want_verbose = true;
        } else if a == "--cmp" {
            want_cmp = true;
        } else if a == "--cmp-pp" {
            want_cmp_pp = true;
        } else if a == "--stdin" {
            want_stdin = true;
        } else if a.starts_with("--rust-interface") {
            // Forms:
            //   --rust-interface
            //   --rust-interface=ModuleName
            if let Some(eq) = a.find('=') {
                let name = a[eq + 1..].trim();
                if !name.is_empty() {
                    rust_iface = Some(name.to_string());
                } else {
                    rust_iface = Some("symbols".to_string());
                }
            } else {
                rust_iface = Some("symbols".to_string());
            }


        } else if a.starts_with("--r") {
            // Forms:
            //   --root
            //   --root=RootName
            if let Some(eq) = a.find('=') {
                let name = a[eq + 1..].trim();
                if !name.is_empty() {
                    use_root = Some(PathBuf::from(name));
                } else {
                    use_root = Some(root_dir.clone());
                }
            } else {
                use_root = Some(root_dir.clone());
            }
        } else if a == "--pre" {
            pre = true;
        } else if a == "--preprocess" {
            preprocess_only = true;
        } else if a == "--no-intermediate" {
            no_intermediate = true;
        } else if a == "--drif" || a == "--drifblim" {
            drif_mode = true;
        } else if a.starts_with('-') {
            eprintln!("unknown flag: {a}");
            print_usage();
            exit(2);
        } else {
            positional.push(a);
        }
    }
    if want_cmp_pp {
        if positional.is_empty() {
            eprintln!("missing input file for --cmp-pp");
            print_usage();
            exit(2);
        }
        // Use canonical path resolution as in normal mode
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
        // Call debug::compare_preprocessors and exit
        if let Err(e) = debug::compare_preprocessors(&input_path.display().to_string(), &root_dir) {
            eprintln!("compare_preprocessors error: {e}");
            exit(1);
        }
        exit(0);
    }

    if want_version {
        println!("uxntal {} (library)", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    let mut source = String::new();
    let mut canon_input = PathBuf::new();
    let mut input_from_stdin = false;
    if want_stdin || (!positional.is_empty() && positional[0] == "/dev/stdin") {
        // Read from stdin
        use std::io::{self, Read};

        io::stdin().read_to_string(&mut source).map_err(|e| {
            simple_err(
                Path::new("/dev/stdin"),
                &format!("failed to read from stdin: {e}"),
            )
        })?;
        canon_input = PathBuf::from("/dev/stdin");
        input_from_stdin = true;
    } else {
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
        canon_input = input_path
            .canonicalize()
            .unwrap_or_else(|_| input_path.clone());

        // Change current working directory to the input file's directory (for relative includes)
        if use_root.is_some() {
            if let Some(root) = &use_root {
                if let Err(e) = std::env::set_current_dir(root) {
                    if want_verbose {
                        eprintln!("warning: failed to chdir to {}: {e}", root.display());
                    }
                } else if want_verbose {
                    eprintln!("Changed working directory to {}", root.display());
                }
            }
        } else if let Some(parent) = canon_input.parent() {
            if let Err(e) = std::env::set_current_dir(parent) {
                if want_verbose {
                    eprintln!("warning: failed to chdir to {}: {e}", parent.display());
                }
            } else if want_verbose {
                eprintln!("Changed working directory to {}", parent.display());
            }
        }
        source = fs::read_to_string(&canon_input)
            .map_err(|e| simple_err(&canon_input, &format!("failed to read: {e}")))?;
        // Write the source to a sibling file with .src.tal extension
        // let mut src_path = canon_input.clone();
        // src_path.set_extension("src.tal");
        // fs::write(&src_path, &source)
        //     .map_err(|e| simple_err(&src_path, &format!("failed to write source: {e}")))?;
        }

    // Compute rom path (absolute) before changing directory
    let rom_path = if positional.len() > 1 {
        let supplied = PathBuf::from(&positional[1]);
        if supplied.is_absolute() {
            supplied
        } else {
            std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join(supplied)
        }
    } else if input_from_stdin {
        // If reading from stdin and no output specified, default to "out.rom"
        PathBuf::from("out.rom")
    } else {
        // If no explicit output, derive sibling .rom next to input file
        canon_input.with_extension("rom")
    };

    if want_verbose {
        eprintln!("Resolved input : {}", canon_input.display());
        eprintln!("Output ROM     : {}", rom_path.display());
        if rust_iface.is_some() {
            eprintln!("Rust interface : enabled");
        }
    }

    let processed_src = if pre {
        match chocolatal::preprocess(&source, &canon_input.display().to_string(), &root_dir) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Preprocessor error: {:?}", e);
                std::process::exit(1);
            }
        }
    } else {
        source.clone()
    };

    if preprocess_only {
        print!("{}", processed_src);
        std::process::exit(0);
    }

    // // Write preprocessed output to .pre.tal in cwd
    // let mut pre_path = PathBuf::from(&canon_input);
    // pre_path.set_extension("pre.tal");
    // if let Err(e) = fs::write(&pre_path, &preprocessed) {
    //     eprintln!(
    //         "Failed to write intermediate file {}: {}",
    //         pre_path.display(),
    //         e
    //     );
    //     std::process::exit(1);
    // }

    // // Use the intermediate file for assembly
    // let pre_source = match fs::read_to_string(&pre_path) {
    //     Ok(s) => s,
    //     Err(e) => {
    //         eprintln!(
    //             "Failed to read intermediate file {}: {}",
    //             pre_path.display(),
    //             e
    //         );
    //         std::process::exit(1);
    //     }
    // };

    let mut asm = if drif_mode {
        Assembler::with_drif_mode(true)
    } else {
        Assembler::new()
    };

    // --- ADD: cmp mode ---
    if want_cmp {
        // Use DebugAssembler from the debug module with drif mode if enabled
        let dbg = if drif_mode {
            debug::DebugAssembler::with_drif_mode(true)
        } else {
            debug::DebugAssembler::default()
        };
        let rel_path = match canon_input.strip_prefix(std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))) {
            Ok(p) => p.display().to_string(),
            Err(_) => canon_input.display().to_string(),
        };
        eprintln!("Relative path to input: {}", rel_path);
        let res = dbg.assemble_and_compare( &rel_path,&processed_src, true);
        return res.map(|_| ());
    }

    let rom = asm.assemble(&processed_src, Some(canon_input.display().to_string()))?;
    fs::write(&rom_path, &rom)
        .map_err(|e| simple_err(&rom_path, &format!("failed to write rom: {e}")))?;

    if want_verbose {
        eprintln!("Wrote ROM ({} bytes)", rom.len());
    } else {
        println!("{} ({} bytes)", rom_path.display(), rom.len());
    }

    if let Some(module_name) = rust_iface {
        let mod_src = uxn_tal::generate_rust_interface_module(&asm, &module_name);
        let iface_path = rom_path.with_extension("rom.symbols.rs");
        fs::write(&iface_path, mod_src).map_err(|e| {
            simple_err(&iface_path, &format!("failed to write rust interface: {e}"))
        })?;
        if want_verbose {
            eprintln!("Wrote Rust interface module: {}", iface_path.display());
        } else {
            println!("{}", iface_path.display());
        }
    }

    // Remove intermediate file unless --no-intermediate is set
    // if !no_intermediate {
    //     let _ = fs::remove_file(&pre_path);
    // }

    Ok(())
}

fn needs_help(args: &[String]) -> bool {
    args.iter()
        .any(|a| a == "--help" || a == "-h" || a == "help")
}

fn print_usage() {
    eprintln!(
        "Usage:
    uxntal [flags] <input.tal|/dev/stdin> [output.rom]

Flags:
    --version, -V         Show version and exit
    --verbose, -v         Verbose output
    --rust-interface[=M]  Emit Rust symbols module (default module name: symbols)
    --cmp                 Compare disassembly for all backends
    --stdin               Read input.tal from stdin
    --cmp-pp             Compare preprocessor output (Rust vs deluge)
    --no-pre              Disable preprocessing
    --preprocess          Print preprocessed output and exit
    --drif, --drifblim    Enable drifblim-compatible mode (optimizations, reference resolution)
    --help, -h            Show this help

Behavior:
    By default, input is preprocessed using chocolatal.
    If output.rom omitted, use input path with .rom extension, or 'out.rom' if reading from stdin.
    You can also pass /dev/stdin as the input filename to read from stdin.
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
        if !root.is_dir() {
            continue;
        }
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
        if visited >= cap {
            break;
        }
        visited += 1;
        let rd = fs::read_dir(&dir).ok()?;
        for entry in rd.flatten() {
            let p = entry.path();
            if p.is_dir() {
                if q.len() < 4096 {
                    q.push_back(p);
                }
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
