#![cfg_attr(windows, windows_subsystem = "windows")]
use std::collections::VecDeque;
// use std::path;
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::exit,
};
// use base64::Engine;
use std::io::Write;
use std::process::Command;
use uxn_tal::bkend_buxn::{ensure_buxn_repo, ensure_docker_buxn_image};
use uxn_tal::bkend_drif::ensure_drifblim_repo;
use uxn_tal::bkend_uxn::{ensure_docker_uxn_image, ensure_uxn_repo};
use uxn_tal::bkend_uxn38::{ensure_docker_uxn38_image, ensure_uxn38_repo};
use uxn_tal::chocolatal;
use uxn_tal::debug;
use uxn_tal::dis_uxndis::ensure_uxndis_repo;
use uxn_tal::util::RealRomCache;
use uxn_tal::util::{pause_for_windows, pause_on_error};
use uxn_tal::{Assembler, AssemblerError};

#[cfg(windows)]
unsafe fn show_console() {
    unsafe {
        // AllocConsole returns nonzero on success
        if winapi::um::consoleapi::AllocConsole() != 0 {
            // Optionally, redirect stdout/stderr to the new console
            use std::fs::File;
            use std::io::Write;

            let _ = File::create("CONOUT$").map(|mut f| {
                let _ = writeln!(f, "Debug console attached.");
            });
        }
    }
}

#[cfg(not(windows))]
unsafe fn show_console() {
    // no-op stub for non-Windows
}

#[cfg(windows)]
fn maybe_show_console() {
    use std::env;
    if env::args().any(|arg| arg == "--debug")
        || env::args().next().is_none()
        || env::args().any(|a| a == "--help")
    {
        unsafe {
            show_console();
        }
    }
}

#[cfg(not(windows))]
fn maybe_show_console() {}

fn main() {
    maybe_show_console();
    if let Err(e) = real_main() {
        eprintln!("error: {e}");
        pause_on_error();
        exit(1);
    }
}

fn real_main() -> Result<(), AssemblerError> {
    let rom_cache = RealRomCache;
    let mut args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        unsafe { show_console() };
        print_usage();
        return Ok(());
    }
    let root_dir = &std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    println!("root dir: {:?}", root_dir);
    let mut pre = false;
    let mut preprocess_only = false;
    let mut want_version = false;
    let mut want_verbose = false;
    let mut want_cmp = false;
    let mut want_cmp_pp = false;
    let mut want_stdin = false;
    let mut drif_mode = false;
    let mut rust_iface: Option<String> = None; // module name (None => not requested)
    let mut use_root: Option<PathBuf> = None; // root name (None => not requested)
    let mut run_after_assembly: Option<String> = None; // command to run after assembly
    let mut run_after_cwd: Option<PathBuf> = None; // directory to run the command in (None => current dir)

    // Collect positional (non-flag) args after flag parsing
    let mut positional: Vec<String> = Vec::new();

    // Store emulator flags for use after assembly
    let emulator_flags: Vec<String> = Vec::new();
    #[allow(unused_mut)]
    let mut should_show_console = false;
    if !args.is_empty() {
        // Look for uxntal:// URL in any of the arguments
        let uxntal_url_pos = args.iter().position(|arg| arg.starts_with("uxntal:"));
        if let Some(pos) = uxntal_url_pos {
            let raw_url = &args[pos];
            use uxn_tal::ProtocolParser;
            use uxn_tal_defined::get_emulator_launcher;

            let result = ProtocolParser::parse(raw_url);
            #[cfg(windows)]
            {
                use uxn_tal_defined::ProtocolVarVar;
                if matches!(
                    result.proto_vars.get("debug"),
                    Some(ProtocolVarVar::Bool(true))
                ) {
                    should_show_console = true;
                    unsafe {
                        show_console();
                    }
                } else {
                    should_show_console = false;
                }
            }
            println!("uxntal {}", raw_url);
            println!("[DEBUG] parsed uxntal protocol result: {:?}", result);
            // Select emulator and get path
            let (mapper, emulator_path) = match get_emulator_launcher(&result, &rom_cache) {
                Some(pair) => pair,
                None => {
                    eprintln!("No suitable emulator found in PATH.");
                    pause_on_error();
                    std::process::exit(1);
                }
            };
            // Resolve ROM path and working directory
            let actual_url = &result.url;
            let (entry_local, rom_dir) = match uxn_tal::resolve_entry_from_url(actual_url) {
                Ok(v) => {
                    println!(
                        "[DEBUG] resolve_entry_from_url succeeded: entry_local={}, rom_dir={}",
                        v.0.display(),
                        v.1.display()
                    );
                    v
                }
                Err(e) => {
                    eprintln!("Failed to resolve uxntal URL: {}", e);
                    pause_on_error();
                    std::process::exit(1);
                }
            };
            let rom_path = entry_local
                .strip_prefix(r"\\?\")
                .unwrap_or(&entry_local)
                .display()
                .to_string();
            println!("[DEBUG] rom_path after processing: {}", rom_path);
            let cmd = mapper.build_command(&result, &rom_path, &emulator_path);
            let emulator_args: Vec<String> = cmd
                .get_args()
                .map(|a| a.to_string_lossy().to_string())
                .collect();
            println!("[DEBUG] Emulator command: {:?}", cmd);
            println!("[DEBUG] Emulator args: {:?}", emulator_args);
            // Save for use after assembly
            run_after_assembly = Some(emulator_path.display().to_string());
            run_after_cwd = Some(rom_dir.clone());

            // Remove the uxntal URL from args and replace with rom_path
            args.remove(pos);
            // Insert rom_path at the beginning (after any flags)
            let mut new_args = Vec::new();
            let mut added_rom_path = false;
            for arg in args.iter() {
                if arg.starts_with('-') {
                    new_args.push(arg.clone());
                } else if !added_rom_path {
                    new_args.push(rom_path.clone());
                    added_rom_path = true;
                    break;
                } else {
                    new_args.push(arg.clone());
                }
            }
            if !added_rom_path {
                new_args.push(rom_path.clone());
            }
            args = new_args;
        }
    }
    if should_show_console {
        unsafe { show_console() };
    }
    println!("args: {:?}", args);
    if !args.is_empty() && args[0] == "--register" {
        register_protocol_per_user()?;
        println!("You need to `cargo install e_window cardinal-gui`. Ctrl+c to exit, or press return to run the install.");
        print!("Press Enter to continue...");
        std::io::stdout().flush().ok();
        let _ = std::io::stdin().read_line(&mut String::new());
        let status = Command::new("cargo")
            .args(["install", "e_window", "cardinal-gui"])
            .status();
        match status {
            Ok(s) if s.success() => {
                println!("Successfully ran: cargo install e_window cardinal-gui");
            }
            Ok(s) => {
                eprintln!("cargo install exited with status: {}", s);
            }
            Err(e) => {
                eprintln!("Failed to run cargo install: {}", e);
            }
        }
        pause_on_error();
        return Ok(());
    }
    if !args.is_empty() && args[0] == "--unregister" {
        unregister_protocol_per_user()?;
        println!("Unregistered uxntal:// protocol handler for current user.");
        pause_on_error();
        return Ok(());
    }

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
        } else if a == "--debug" {
            // Accept --debug as a valid flag (no-op here, handled elsewhere)
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
        } else if a == "--drif" || a == "--drifblim" {
            drif_mode = true;
        } else if a.starts_with('-') {
            eprintln!("unknown flag: {a}");
            print_usage();
            pause_on_error();
            exit(2);
        } else {
            positional.push(a);
        }
    }
    if want_cmp_pp {
        if positional.is_empty() {
            eprintln!("missing input file for --cmp-pp");
            print_usage();
            pause_on_error();
            exit(2);
        }
        // Use canonical path resolution as in normal mode
        let raw_input = &positional[0];
        let input_path = match resolve_input_path(raw_input) {
            Some(p) => p,
            None => {
                return Err(AssemblerError::FileReadError {
                    path: raw_input.to_string(),
                    message:
                        "input file not found (tried direct, +.tal, multi-root recursive scan)"
                            .to_string(),
                });
            }
        };
        // Call debug::compare_preprocessors and exit
        if let Err(e) = debug::compare_preprocessors(&input_path.display().to_string(), root_dir) {
            eprintln!("compare_preprocessors error: {e}");
            pause_on_error();
            exit(1);
        }
        exit(0);
    }

    if want_version {
        println!("uxntal {} (library)", env!("CARGO_PKG_VERSION"));
        pause_for_windows();
        return Ok(());
    }

    let mut source = String::new();
    let canon_input_p;
    let mut input_from_stdin = false;
    let mut input_is_rom = false;
    let mut input_is_orca = false;
    if want_stdin || (!positional.is_empty() && positional[0] == "/dev/stdin") {
        // Read from stdin
        use std::io::{self, Read};

        io::stdin()
            .read_to_string(&mut source)
            .map_err(|e| AssemblerError::FileReadError {
                path: "/dev/stdin".to_string(),
                message: format!("failed to read from stdin: {e}"),
            })?;
        canon_input_p = PathBuf::from("/dev/stdin");
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
                return Err(AssemblerError::FileReadError {
                    path: raw_input.to_string(),
                    message: "input file not found (tried direct, +.tal, +.rom, multi-root recursive scan)".to_string(),
                });
            }
        };
        // Canonical (or absolute fallback) before chdir so later paths remain valid
        canon_input_p = input_path
            .canonicalize()
            .unwrap_or_else(|_| input_path.clone());

        // Detect .rom or .orca extension
        if let Some(ext) = canon_input_p.extension() {
            if ext == "rom" {
                input_is_rom = true;
            } else if ext == "orca" {
                input_is_orca = true;
            }
        }

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
        } else if let Some(parent) = canon_input_p.parent() {
            if let Err(e) = std::env::set_current_dir(parent) {
                if want_verbose {
                    eprintln!("warning: failed to chdir to {}: {e}", parent.display());
                }
            } else if want_verbose {
                eprintln!("Changed working directory to {}", parent.display());
            }
        }
        // All TAL file reading is handled by the byte-based + UTF-8 validation logic below.
        // Write the source to a sibling file with .src.tal extension
        // let mut src_path = canon_input.clone();
        // src_path.set_extension("src.tal");
        // fs::write(&src_path, &source)
        //     .map_err(|e| simple_err(&src_path, &format!("failed to write source: {e}")))?;
    }

    // Compute rom path (absolute) before changing directory
    let rom_path_p = if positional.len() > 1 {
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
        canon_input_p.with_extension("rom")
    };

    let rom_path_str = rom_path_p.display().to_string();
    let rom_path = rom_path_str.strip_prefix(r"\\?\").unwrap_or(&rom_path_str);
    let canon_input_str = canon_input_p.display().to_string();
    let canon_input = canon_input_str
        .strip_prefix(r"\\?\")
        .unwrap_or(&canon_input_str);
    if want_verbose {
        eprintln!("Resolved input : {}", canon_input);
        eprintln!("Output ROM     : {}", rom_path);
        if rust_iface.is_some() {
            eprintln!("Rust interface : enabled");
        }
    }

    let processed_src = if !input_is_rom && !input_is_orca {
        // Read as bytes, check for BOM, then validate as UTF-8
        let bytes = match std::fs::read(&canon_input_p) {
            Ok(mut b) => {
                // Detect and skip UTF-8 BOM (0xEF,0xBB,0xBF) or single-byte BOMs (0xFF, 0xFE)
                let skip = if b.starts_with(&[0xEF, 0xBB, 0xBF]) {
                    3
                } else if b.starts_with(&[0xFF, 0xFE]) || b.starts_with(&[0xFE, 0xFF]) {
                    2
                } else if b.starts_with(&[0xFF]) || b.starts_with(&[0xFE]) {
                    1
                } else {
                    0
                };
                if skip > 0 {
                    eprintln!("[debug] Skipping BOM of length {}", skip);
                }
                b.drain(0..skip);
                b
            }
            Err(e) => {
                use std::io::ErrorKind;
                if e.kind() == ErrorKind::InvalidData {
                    // Try to read as bytes and report UTF-8 error with diagnostics
                    match std::fs::read(&canon_input_p) {
                        Ok(mut bytes) => {
                            // Detect and skip BOM as above
                            let skip = if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
                                3
                            } else if bytes.starts_with(&[0xFF, 0xFE])
                                || bytes.starts_with(&[0xFE, 0xFF])
                            {
                                2
                            } else if bytes.starts_with(&[0xFF]) || bytes.starts_with(&[0xFE]) {
                                1
                            } else {
                                0
                            };
                            if skip > 0 {
                                eprintln!("[debug] Skipping BOM of length {}", skip);
                            }
                            bytes.drain(0..skip);
                            match std::str::from_utf8(&bytes) {
                                Ok(_) => {
                                    // Should not happen, but fallback
                                    return Err(AssemblerError::FileReadError {
                                        path: canon_input_p.display().to_string(),
                                        message: format!("failed to read: {e}"),
                                    });
                                }
                                Err(utf8err) => {
                                    return Err(report_utf8_error_with_context(
                                        &canon_input_p,
                                        &bytes,
                                        utf8err.valid_up_to(),
                                        &utf8err,
                                    ));
                                }
                            }
                        }
                        Err(e2) => {
                            // Could not read bytes at all
                            return Err(AssemblerError::FileReadError {
                                path: canon_input_p.display().to_string(),
                                message: format!("failed to read: {e2}"),
                            });
                        }
                    }
                } else {
                    return Err(AssemblerError::FileReadError {
                        path: canon_input_p.display().to_string(),
                        message: format!("failed to read: {e}"),
                    });
                }
            }
        };
        eprintln!("[debug] Attempting to parse as UTF-8");
        std::io::stderr().flush().ok();
        match std::str::from_utf8(&bytes) {
            Ok(tal_source) => {
                eprintln!("[debug] UTF-8 parse OK");
                std::io::stderr().flush().ok();
                use uxn_tal::probe_tal::print_all_tal_heuristics;
                print_all_tal_heuristics(tal_source);
                if pre {
                    match chocolatal::preprocess(tal_source, canon_input, root_dir) {
                        Ok(s) => s,
                        Err(e) => {
                            eprintln!("Preprocessor error: {:?}", e);
                            std::process::exit(1);
                        }
                    }
                } else {
                    tal_source.to_string()
                }
            }
            Err(e) => {
                eprintln!(
                    "[debug] UTF-8 parse failed: valid_up_to={} error={:?}",
                    e.valid_up_to(),
                    e
                );
                std::io::stderr().flush().ok();
                use uxn_tal::probe_tal::print_all_tal_heuristics_bytes;
                print_all_tal_heuristics_bytes(&bytes);
                return Err(report_utf8_error_with_context(
                    &canon_input_p,
                    &bytes,
                    e.valid_up_to(),
                    &e,
                ));
            }
        }
    } else {
        eprintln!("[debug] input_is_rom, skipping text read");
        std::io::stderr().flush().ok();
        String::new()
    };

    if preprocess_only && !input_is_rom && !input_is_orca {
        print!("{}", processed_src);
        pause_for_windows();
        std::process::exit(0);
    } else if preprocess_only && input_is_rom {
        eprintln!("Cannot preprocess a .rom file");
        pause_for_windows();
        std::process::exit(1);
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
        println!("{:?}", ensure_drifblim_repo());
        let _ = ensure_uxndis_repo();
        let _ = ensure_buxn_repo();
        let _ = ensure_docker_buxn_image();
        let _ = ensure_uxn38_repo();
        let _ = ensure_docker_uxn38_image();
        let _ = ensure_uxn_repo();
        let _ = ensure_docker_uxn_image();
        // Use DebugAssembler from the debug module with drif mode if enabled
        let dbg = if drif_mode {
            debug::DebugAssembler::with_drif_mode(true)
        } else {
            debug::DebugAssembler::default()
        };
        let rel_path = match canon_input_p
            .strip_prefix(std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
        {
            Ok(p) => p.display().to_string(),
            Err(_) => canon_input_p.display().to_string(),
        };
        eprintln!("Relative path to input: {}", rel_path);
        let res = dbg.assemble_and_compare(&rel_path, &processed_src, true);
        return res.map(|_| ());
    }

    if input_is_orca {
        // For .orca files, launch emulator with canonical orca ROM resolved via util (cache-aware)
        if let Some(ref _cmd) = run_after_assembly {
            if let Some(raw_url) = std::env::args().nth(1) {
                if raw_url.starts_with("uxntal:") {
                    use uxn_tal::util::resolve_canonical_orca_rom;
                    use uxn_tal::ProtocolParser;
                    use uxn_tal_defined::get_emulator_launcher;
                    let result = ProtocolParser::parse(&raw_url);
                    if let Some((mapper, emulator_path)) =
                        get_emulator_launcher(&result, &rom_cache)
                    {
                        // Always use the canonical orca ROM path, not the per-file cache
                        let (canonical_orca_rom, _canonical_cache_dir) =
                            match resolve_canonical_orca_rom() {
                                Ok(pair) => pair,
                                Err(e) => {
                                    eprintln!("Failed to resolve canonical orca.rom: {e}");
                                    pause_on_error();
                                    return Ok(());
                                }
                            };
                        // Set working directory to the directory containing the .orca file
                        let orca_dir = canon_input_p.parent().unwrap_or_else(|| Path::new("."));
                        // Compute relative path for canonical ROM from this directory
                        let rel_rom = match canonical_orca_rom.strip_prefix(orca_dir) {
                            Ok(rel) => rel.display().to_string(),
                            Err(_) => canonical_orca_rom.display().to_string(),
                        };
                        let rel_orca = match canon_input_p.strip_prefix(orca_dir) {
                            Ok(rel) => rel.display().to_string(),
                            Err(_) => canon_input_p.display().to_string(),
                        };
                        let mut cmd = mapper.build_command(&result, &rel_rom, &emulator_path);
                        // Insert the .orca file as the second argument (relative path)
                        cmd.arg(&rel_orca);
                        cmd.current_dir(orca_dir);
                        println!("[DEBUG] Spawning emulator (orca): {:?}", cmd);
                        match cmd.spawn() {
                            Ok(_) => {
                                println!(
                                    "Launched emulator: {}",
                                    cmd.get_program().to_string_lossy()
                                );
                            }
                            Err(e) => {
                                eprintln!("Failed to spawn emulator: {}", e);
                                pause_on_error();
                            }
                        }
                        return Ok(());
                    }
                }
            }
        }
        // Fallback: just print info if not protocol
        println!("[INFO] .orca file detected: {}", canon_input_p.display());
        return Ok(());
    } else if input_is_rom {
        // If input is .rom, just copy to output if needed
        if rom_path_p.exists() {
            println!("ROM already exists at {}", rom_path);
        } else {
            fs::copy(&canon_input_p, &rom_path_p).map_err(|e| {
                simple_err(Path::new(rom_path), &format!("failed to copy rom: {e}"))
            })?;
            if want_verbose {
                eprintln!(
                    "Copied ROM ({} bytes)",
                    fs::metadata(&rom_path_p).map(|m| m.len()).unwrap_or(0)
                );
            } else {
                println!(
                    "{} ({} bytes)",
                    rom_path,
                    fs::metadata(&rom_path_p).map(|m| m.len()).unwrap_or(0)
                );
            }
        }
    } else if run_after_assembly.is_some() {
        if rom_path_p.exists() {
            println!("ROM already exists at {}", rom_path);
        } else {
            let rom = match asm.assemble(&processed_src, Some(canon_input.to_owned())) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("Assembly error: {e}");
                    // pause_on_error();
                    return Err(e);
                }
            };
            fs::write(rom_path, &rom).map_err(|e| {
                simple_err(Path::new(rom_path), &format!("failed to write rom: {e}"))
            })?;
            if want_verbose {
                eprintln!("Wrote ROM ({} bytes)", rom.len());
            } else {
                println!("{} ({} bytes)", rom_path, rom.len());
            }
        }
    } else {
        let rom = match asm.assemble(&processed_src, Some(canon_input.to_owned())) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Assembly error: {e}");
                // pause_on_error();
                return Err(e);
            }
        };
        fs::write(rom_path, &rom)
            .map_err(|e| simple_err(Path::new(rom_path), &format!("failed to write rom: {e}")))?;

        if want_verbose {
            eprintln!("Wrote ROM ({} bytes)", rom.len());
        } else {
            println!("{} ({} bytes)", rom_path, rom.len());
        }
    }

    if let Some(module_name) = rust_iface {
        let mod_src = uxn_tal::generate_rust_interface_module(&asm, &module_name);
        let iface_path = rom_path_p.with_extension("rom.symbols.rs");
        fs::write(&iface_path, mod_src).map_err(|e| {
            simple_err(&iface_path, &format!("failed to write rust interface: {e}"))
        })?;
        if want_verbose {
            eprintln!("Wrote Rust interface module: {}", iface_path.display());
        } else {
            println!("{}", iface_path.display());
        }
    }

    if let Some(ref cmd) = run_after_assembly {
        // If protocol URL is present, use protocol handler for argument construction
        if let Some(raw_url) = std::env::args().nth(1) {
            if raw_url.starts_with("uxntal:") {
                use uxn_tal::ProtocolParser;
                use uxn_tal_defined::get_emulator_launcher;
                let result = ProtocolParser::parse(&raw_url);
                if let Some((mapper, emulator_path)) = get_emulator_launcher(&result, &rom_cache) {
                    // Always use the actual output ROM path for the emulator
                    let rom_path = rom_path_p.to_string_lossy().to_string();
                    let mut cmd = mapper.build_command(&result, &rom_path, &emulator_path);
                    cmd.current_dir(run_after_cwd.clone().unwrap_or_else(|| PathBuf::from(".")));
                    println!("[DEBUG] Spawning emulator: {:?}", cmd);
                    match cmd.spawn() {
                        Ok(_) => {
                            println!("Launched emulator: {}", cmd.get_program().to_string_lossy());
                        }
                        Err(e) => {
                            eprintln!("Failed to spawn emulator: {}", e);
                            pause_on_error();
                        }
                    }
                    return Ok(());
                }
            }
        }
        // Fallback: legacy path if not protocol URL
        #[cfg(not(all(target_family = "wasm", target_os = "unknown")))]
        {
            let cmd_name = cmd.clone();
            let path_to_emu = which::which(&cmd_name)
                .or_else(|_| {
                    if let Ok(home) = std::env::var("HOME") {
                        let p = PathBuf::from(format!("{}/.cargo/bin/{}", home, &cmd_name));
                        if p.exists() {
                            Ok(p)
                        } else {
                            Err(())
                        }
                    } else {
                        Err(())
                    }
                })
                .map_err(|_| {
                    simple_err(
                        Path::new("."),
                        &format!("{cmd_name} not found in PATH or ~/.cargo/bin"),
                    )
                })?;
            println!("Running post-assembly command: {}", cmd);
            let dir_str = run_after_cwd
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| {
                    std::env::current_dir()
                        .map(|p| p.display().to_string())
                        .unwrap_or_else(|_| ".".to_string())
                });
            println!("In directory: {}", dir_str);
            let mut cmd_args = emulator_flags.clone();
            cmd_args.push(rom_path.to_string());
            match Command::new(path_to_emu)
                .args(&cmd_args)
                .current_dir(run_after_cwd.clone().unwrap_or_else(|| PathBuf::from(".")))
                .spawn()
            {
                Ok(_) => {
                    println!("Launched emulator: {}", cmd);
                }
                Err(e) => {
                    eprintln!("Failed to spawn emulator: {}", e);
                    pause_on_error();
                }
            }
        }
        #[cfg(all(target_family = "wasm", target_os = "unknown"))]
        {
            eprintln!("which::which and post-assembly command execution are not available in browser WASM");
        }
    }

    // After assembly, if run_after_assembly is set, launch the emulator with the correct flags and ROM path
    if let (Some(_emulator_cmd), Some(run_cwd)) =
        (run_after_assembly.clone(), run_after_cwd.clone())
    {
        if let Some(raw_url) = env::args().nth(1) {
            if raw_url.starts_with("uxntal:") {
                use uxn_tal::ProtocolParser;
                use uxn_tal_defined::get_emulator_launcher;
                let result = ProtocolParser::parse(&raw_url);
                if let Some((mapper, emulator_path)) = get_emulator_launcher(&result, &rom_cache) {
                    let rom_path = args.first().cloned().unwrap_or_default();
                    let mut cmd = mapper.build_command(&result, &rom_path, &emulator_path);
                    cmd.current_dir(run_cwd);
                    println!("[DEBUG] Spawning emulator: {:?}", cmd);
                    match cmd.spawn() {
                        Ok(_) => {
                            println!("Launched emulator: {}", cmd.get_program().to_string_lossy());
                        }
                        Err(e) => eprintln!("Failed to launch emulator: {}", e),
                    }
                    return Ok(());
                }
            }
        }
    }
    // Remove intermediate file unless --no-intermediate is set
    // if !no_intermediate {
    //     let _ = fs::remove_file(&pre_path);
    // }
    pause_on_error();
    Ok(())
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
    --cmp-pp              Compare preprocessor output (Rust vs deluge)
    --pre                 Enable preprocessing
    --preprocess          Print preprocessed output and exit
    --drif, --drifblim    Enable drifblim-compatible mode (optimizations, reference resolution)
    --debug               Enable debug output
    --r, --root[=DIR]     Set root directory for includes (default: current dir)
    --register            Register uxntal as a file handler (Windows only)
    --unregister          Unregister uxntal as a file handler (Windows only)
    --help, -h            Show this help

Behavior:
    If output.rom omitted, use input path with .rom extension, or 'out.rom' if reading from stdin.
    You can also pass /dev/stdin as the input filename to read from stdin.
    Rust interface file path: <output>.rom.symbols.rs
    See README.md for more protocol and flag examples."
    );
    pause_on_error();
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
    // Try .tal and .rom extensions if not present
    if direct.extension().is_none() {
        let with_tal = direct.with_extension("tal");
        if with_tal.exists() {
            return Some(with_tal);
        }
        let with_rom = direct.with_extension("rom");
        if with_rom.exists() {
            return Some(with_rom);
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
        // Try arg.tal and arg.rom variations
        let alt_tal = format!("{arg}.tal");
        if let Some(found) = recursive_find(&root, &alt_tal, 12_000) {
            return Some(found);
        }
        let alt_rom = format!("{arg}.rom");
        if let Some(found) = recursive_find(&root, &alt_rom, 12_000) {
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

#[cfg(target_os = "macos")]
fn register_protocol_per_user() -> std::io::Result<()> {
    use std::io::{stdin, stdout, Write};
    use std::path::PathBuf;
    use std::process::Command;

    // Check for Xcode command line tools (xcrun and swiftc)
    if which::which("xcrun").is_err() || which::which("swiftc").is_err() {
        eprintln!("Error: Xcode command line tools are required to register the protocol handler.");
        eprintln!("Please install them with: xcode-select --install");
        return Ok(());
    }

    // Find the correct uxntal binary in PATH
    let uxntal_path = which::which("uxntal").expect("Could not find uxntal in PATH");
    let version = env!("CARGO_PKG_VERSION");

    let home = std::env::var("HOME").unwrap();
    let temp_dir = PathBuf::from(format!("{}/.uxntal_swift_launcher", home));
    let app_delegate_file = temp_dir.join("AppDelegate.swift");
    let main_file = temp_dir.join("main.swift");
    let plist_file = temp_dir.join("Info.plist");
    let app_name = "uxntal-launcher";
    let app_bundle = temp_dir.join(format!("{app_name}.app"));

    // Clean temp dir
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir)?;

    // Write AppDelegate.swift (no @main)
    let swift_app_delegate = format!(
        r#"
import Cocoa

class AppDelegate: NSObject, NSApplicationDelegate {{
    func application(_ application: NSApplication, open urls: [URL]) {{
        for url in urls {{
            let task = Process()
            task.launchPath = "{bin_path}"
            task.arguments = [url.absoluteString]
            task.launch()
        }}
        NSApp.terminate(nil)
    }}
    func applicationDidFinishLaunching(_ notification: Notification) {{
        NSApp.terminate(nil)
    }}
}}
"#,
        bin_path = uxntal_path.display()
    );
    let mut f = std::fs::File::create(&app_delegate_file)?;
    f.write_all(swift_app_delegate.as_bytes())?;

    // Write main.swift
    let swift_main = r#"
import Cocoa

let delegate = AppDelegate()
NSApplication.shared.delegate = delegate
_ = NSApplicationMain(CommandLine.argc, CommandLine.unsafeArgv)
"#;
    let mut f = std::fs::File::create(&main_file)?;
    f.write_all(swift_main.as_bytes())?;

    // Write Info.plist with correct version
    let plist = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key>
    <string>uxntal-launcher</string>
    <key>CFBundleIdentifier</key>
    <string>uxntal.uxn-tal.launcher</string>
    <key>CFBundleVersion</key>
    <string>{version}</string>
    <key>CFBundleShortVersionString</key>
    <string>{version}</string>
    <key>CFBundleExecutable</key>
    <string>{app_name}</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleURLTypes</key>
    <array>
        <dict>
            <key>CFBundleURLName</key>
            <string>UXNTAL Protocol</string>
            <key>CFBundleURLSchemes</key>
            <array>
                <string>uxntal</string>
            </array>
        </dict>
    </array>
</dict>
</plist>
"#,
        version = version,
        app_name = app_name
    );
    let mut f = std::fs::File::create(&plist_file)?;
    f.write_all(plist.as_bytes())?;

    // Compile Swift app with both files
    let status = Command::new("xcrun")
        .args([
            "swiftc",
            "-o",
            &format!("{}/{}", temp_dir.display(), app_name),
            main_file.to_str().unwrap(),
            app_delegate_file.to_str().unwrap(),
        ])
        .status()?;
    if !status.success() {
        eprintln!("Failed to compile Swift launcher. Is Xcode command line tools installed?");
        return Ok(());
    }

    // Create .app bundle structure
    let app_contents = app_bundle.join("Contents");
    let macos_dir = app_contents.join("MacOS");
    fs::create_dir_all(&macos_dir)?;
    fs::copy(temp_dir.join(app_name), macos_dir.join(app_name))?;
    fs::copy(&plist_file, app_contents.join("Info.plist"))?;

    // Move to ~/Applications as uxntal.app
    let user_app = PathBuf::from(format!("{}/Applications/uxntal.app", home));
    if user_app.exists() {
        println!(
            "An existing uxntal.app was found at {}.",
            user_app.display()
        );
        print!("Do you want to remove it and create a new one? [y/N]: ");
        stdout().flush().ok();
        let mut answer = String::new();
        stdin().read_line(&mut answer).ok();
        if answer.trim().eq_ignore_ascii_case("y") {
            fs::remove_dir_all(&user_app)?;
            println!("Removed old uxntal.app.");
        } else {
            println!("Aborted by user.");
            return Ok(());
        }
    }
    fs::rename(&app_bundle, &user_app)?;

    println!("Created uxntal.app at {}", user_app.display());
    println!("Double click uxntal.app in ~/Applications to register the uxntal:// protocol.");
    let _ = Command::new("open")
        .arg(format!("{}/Applications", home))
        .status();

    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn register_protocol_per_user() -> std::io::Result<()> {
    #[cfg(windows)]
    {
        let exe = std::env::current_exe()?.display().to_string();
        // Important: NO backslash-doubling, just quote the path.
        let cmd = format!(r#""{}" "%1""#, exe);

        // Base key
        let status1 = Command::new("reg")
            .args([
                "add",
                r"HKCU\Software\Classes\uxntal",
                "/ve",
                "/t",
                "REG_SZ",
                "/d",
                "URL:UXNTAL Protocol",
                "/f",
            ])
            .status()?;
        // Mark as URL protocol
        let status2 = Command::new("reg")
            .args([
                "add",
                r"HKCU\Software\Classes\uxntal",
                "/v",
                "URL Protocol",
                "/t",
                "REG_SZ",
                "/d",
                "",
                "/f",
            ])
            .status()?;
        // Open command
        let status3 = Command::new("reg")
            .args([
                "add",
                r"HKCU\Software\Classes\uxntal\shell\open\command",
                "/ve",
                "/t",
                "REG_SZ",
                "/d",
                &cmd,
                "/f",
            ])
            .status()?;

        if status1.success() && status2.success() && status3.success() {
            println!("Registered uxntal:// protocol for current user on Windows.");
        } else {
            eprintln!(
                "Failed: {:?} {:?} {:?}",
                status1.code(),
                status2.code(),
                status3.code()
            );
            return Err(std::io::Error::other(
                "Failed to register protocol on Windows",
            ));
        }
        Ok(())
    }

    #[cfg(unix)]
    {
        // Get the path to the executable
        let exe = std::env::current_exe()?.display().to_string();

        // Define the paths for .desktop and MIME files
        let home_dir = std::env::var("HOME").map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("HOME environment variable not found: {}", e),
            )
        })?;
        let desktop_file_path = format!("{}/.local/share/applications/uxntal.desktop", home_dir);
        let mime_file_path = format!(
            "{}/.local/share/mime/packages/x-scheme-handler-uxntal.xml",
            home_dir
        );

        // Create the parent directory for the MIME file if it doesn't exist
        if let Some(parent) = std::path::Path::new(&mime_file_path).parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Create the MIME type XML file
        let mime_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<mime-info xmlns="http://www.freedesktop.org/standards/shared-mime-info">
    <mime-type type="x-scheme-handler/uxntal">
        <comment>UXNTAL Protocol</comment>
        <glob pattern="uxntal://*"/>
    </mime-type>
</mime-info>
"#;
        let mut mime_file = std::fs::File::create(&mime_file_path)?;
        mime_file.write_all(mime_content.as_bytes())?;

        // Create the .desktop file content
        let desktop_content = format!(
            r#"[Desktop Entry]
Name=UXNTAL Handler
Exec={} %u
Type=Application
Terminal=false
MimeType=x-scheme-handler/uxntal;
NoDisplay=true
"#,
            exe
        );

        // Create the parent directory for the .desktop file if it doesn't exist
        if let Some(parent) = std::path::Path::new(&desktop_file_path).parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Write the .desktop file
        let mut desktop_file = std::fs::File::create(&desktop_file_path)?;
        desktop_file.write_all(desktop_content.as_bytes())?;

        // Register the MIME type with xdg-mime
        let status1 = Command::new("xdg-mime")
            .args(["install", "--mode", "user", &mime_file_path])
            .status()?;

        // Associate the .desktop file with the MIME type
        let status2 = Command::new("xdg-mime")
            .args(["default", "uxntal.desktop", "x-scheme-handler/uxntal"])
            .status()?;

        // Update the desktop database
        let status3 = Command::new("update-desktop-database")
            .arg(format!("{}/.local/share/applications", home_dir))
            .status()?;

        if status1.success() && status2.success() && status3.success() {
            println!("Registered uxntal:// protocol for current user on Ubuntu.");
        } else {
            eprintln!(
            "Failed: xdg-mime install status: {:?}, xdg-mime default status: {:?}, update-desktop-database status: {:?}",
            status1.code(),
            status2.code(),
            status3.code()
        );
            // Clean up created files in case of failure
            let _ = std::fs::remove_file(&desktop_file_path);
            let _ = std::fs::remove_file(&mime_file_path);
            return Err(std::io::Error::other(
                "Failed to register protocol on Ubuntu",
            ));
        }

        // Install dependencies (e_window and cardinal-gui)
        println!("You need to `cargo install e_window cardinal-gui`. Ctrl+C to exit, or press Enter to run the install.");
        print!("Press Enter to continue...");
        std::io::stdout().flush()?;
        let _ = std::io::stdin().read_line(&mut String::new())?;
        let status = Command::new("cargo")
            .args(["install", "e_window", "cardinal-gui"])
            .status()?;
        if status.success() {
            println!("Successfully ran: cargo install e_window cardinal-gui");
        } else {
            eprintln!("cargo install exited with status: {:?}", status.code());
            return Err(std::io::Error::other("Failed to run cargo install"));
        }

        Ok(())
    }

    #[cfg(not(any(windows, unix)))]
    {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Protocol registration is only supported on Windows and Unix-like systems (e.g., Ubuntu)",
        ))
    }
}

#[allow(dead_code)]
fn unregister_protocol_per_user() -> std::io::Result<()> {
    #[cfg(windows)]
    {
        // Remove registry keys for uxntal protocol
        let status1 = Command::new("reg")
            .args(["delete", r"HKCU\Software\Classes\uxntal", "/f"])
            .status()?;
        if status1.success() {
            println!("Removed registry keys for uxntal protocol.");
        } else {
            eprintln!("Failed to remove registry keys for uxntal protocol.");
        }
        Ok(())
    }
    #[cfg(target_os = "macos")]
    {
        use std::env;
        use std::fs;
        use std::process::Command;
        let home = env::var("HOME").unwrap();
        let temp_dir = format!("{}/.uxntal_swift_launcher", home);
        let app_bundle = format!("{}/uxntal-launcher.app", temp_dir);
        let user_app = format!("{}/Applications/uxntal.app", home);
        // Remove both the temp .app bundle and the user Applications .app
        let _ = fs::remove_dir_all(&app_bundle);
        let _ = fs::remove_dir_all(&temp_dir);
        let _ = fs::remove_dir_all(&user_app);
        // Unregister with lsregister
        let _ = Command::new("/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister")
            .args(["-u", &user_app])
            .status();
        println!("Removed uxntal.app from ~/Applications, uxntal-launcher.app, and temp files. Ran lsregister to unregister protocol handler.");
        Ok(())
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        // Remove .desktop and MIME files, update desktop database
        use std::env;
        use std::fs;
        let home_dir = env::var("HOME").map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("HOME environment variable not found: {}", e),
            )
        })?;
        let desktop_file_path = format!("{}/.local/share/applications/uxntal.desktop", home_dir);
        let mime_file_path = format!(
            "{}/.local/share/mime/packages/x-scheme-handler-uxntal.xml",
            home_dir
        );
        let _ = fs::remove_file(&desktop_file_path);
        let _ = fs::remove_file(&mime_file_path);
        let _ = Command::new("update-desktop-database")
            .arg(format!("{}/.local/share/applications", home_dir))
            .status();
        println!("Removed uxntal.desktop and MIME handler files.");
        Ok(())
    }
    #[cfg(not(any(windows, unix)))]
    {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Protocol unregistration is only supported on Windows, macOS, and Unix-like systems",
        ))
    }
}

// fn extract_target_from_uxntal(raw_url: &str) -> Option<String> {
//     use std::borrow::Cow;

//     fn pct_decode(s: &str) -> String {
//         percent_encoding::percent_decode_str(s)
//             .decode_utf8()
//             .unwrap_or(Cow::from(s))
//             .into_owned()
//     }

//     fn qs_get<'a>(query: &'a str, key: &str) -> Option<String> {
//         // query w/o leading '?', split on '&', look for key=...
//         for pair in query.split('&') {
//             let mut it = pair.splitn(2, '=');
//             let k = it.next().unwrap_or("");
//             let v = it.next().unwrap_or("");
//             if k.eq_ignore_ascii_case(key) {
//                 return Some(pct_decode(v));
//             }
//         }
//         None
//     }

//     if !raw_url.starts_with("uxntal:") {
//         return None;
//     }

//     // Strip scheme and any number of leading slashes (uxntal:, uxntal:/, uxntal://, etc.)
//     let mut s = raw_url.trim_start_matches("uxntal:").trim_start_matches('/');

//     // 1) open router: support both "open?..." and "open/?..."
//     // Normalize an optional single slash before '?'
//     // Examples we accept:
//     //   open?url=ENC
//     //   open/?url=ENC
//     if s.starts_with("open") {
//         // split off path and query
//         let (path, rest) = if let Some(qpos) = s.find('?') {
//             (&s[..qpos], &s[qpos + 1..])
//         } else {
//             (s, "")
//         };
//         // path could be "open" or "open/"
//         if path == "open" || path == "open/" {
//             if let Some(v) = qs_get(rest, "url") {
//                 return Some(v);
//             }
//         }
//         // fallthrough if no url param
//     }

//     // 2) Base64 form: uxntal://b64,<payload>  (URL_SAFE_NO_PAD)
//     if let Some(rest) = s.strip_prefix("b64,") {
//         if let Ok(bytes) = base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(rest) {
//             if let Ok(strv) = String::from_utf8(bytes) {
//                 return Some(strv);
//             }
//         }
//     }

//     // 3) Slash-mangled http(s)/file
//     // Normalize common mangles:
//     //  - https///example.com -> https://example.com
//     //  - https//example.com  -> https://example.com
//     //  - http///example.com  -> http://example.com
//     //  - http//example.com   -> http://example.com
//     //  - file///C:/path      -> file://C:/path
//     for (bad, good, cut) in [
//         ("https///", "https://", 8usize),
//         ("http///",  "http://",  7usize),
//         ("file///",  "file://",  7usize),
//         ("https//",  "https://", 7usize),
//         ("http//",   "http://",  6usize),
//         ("file//",   "file://",  7usize),
//     ] {
//         if s.starts_with(bad) {
//             return Some(format!("{}{}", good, &s[cut..]));
//         }
//     }

//     // 4) Percent-encoded whole URL after scheme
//     if s.contains("%2F") || s.contains("%3A") || s.contains('%') {
//         let dec = pct_decode(s);
//         if dec.starts_with("http://") || dec.starts_with("https://") || dec.starts_with("file://") {
//             return Some(dec);
//         }
//     }

//     // 5) Plain pass-through if it already looks like a URL
//     if s.starts_with("http://") || s.starts_with("https://") || s.starts_with("file://") {
//         return Some(s.to_string());
//     }

//     // 6) Fallback: as-is
//     Some(s.to_string())
// }

/// Scan bytes for UTF-8 error and return AssemblerError::Utf8Error with detailed info
fn report_utf8_error_with_context(
    path: &std::path::Path,
    bytes: &[u8],
    err_offset: usize,
    err: &dyn std::fmt::Display,
) -> AssemblerError {
    let (mut line, mut col) = (1, 1);
    let mut last_line_start = 0;
    for (i, &b) in bytes.iter().enumerate().take(err_offset) {
        if b == b'\n' {
            line += 1;
            last_line_start = i + 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    let bad_byte = bytes.get(err_offset).copied();
    let source_line = {
        let start = last_line_start;
        let end = bytes.len().min(
            bytes
                .iter()
                .skip(start)
                .position(|&b| b == b'\n')
                .map(|p| start + p)
                .unwrap_or(bytes.len()),
        );
        String::from_utf8_lossy(&bytes[start..end]).to_string()
    };
    let (char_repr, hex_repr) = match bad_byte {
        Some(b) => {
            let c = if b.is_ascii_graphic() || b == b' ' {
                (b as char).to_string()
            } else {
                format!("\\x{:02X}", b)
            };
            (c, format!("0x{:02X}", b))
        }
        None => ("<EOF>".to_string(), "<EOF>".to_string()),
    };
    AssemblerError::Utf8Error {
        path: path.display().to_string(),
        line,
        position: col,
        message: format!(
            "failed to read: invalid UTF-8 at byte {offset} (line {line}, col {col}): found {char_repr} ({hex_repr}): {err}",
            offset=err_offset, line=line, col=col, char_repr=char_repr, hex_repr=hex_repr, err=err
        ),
        source_line,
    }
}
