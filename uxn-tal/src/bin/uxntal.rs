use std::collections::VecDeque;
// use std::path;
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::exit,
};
// use base64::Engine;
use uxn_tal::resolve_entry_from_url;
use uxn_tal::bkend_uxn::{ensure_docker_uxn_image, ensure_uxn_repo};
use uxn_tal::bkend_uxn38::{ensure_docker_uxn38_image, ensure_uxn38_repo};
use uxn_tal::bkend_buxn::{ensure_buxn_repo, ensure_docker_buxn_image};
use uxn_tal::chocolatal;
use uxn_tal::debug;
use uxn_tal::bkend_drif::ensure_drifblim_repo;
use uxn_tal::dis_uxndis::ensure_uxndis_repo;
use uxn_tal::{Assembler, AssemblerError};
use std::process::Command;
use std::io::Write;
// use std::fs::File;
// use std::hash::{Hasher, Hash};
// use std::collections::hash_map::DefaultHasher;
// use std::time::Duration;
fn main() {
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

    if args.len() > 0 {
        let raw_url = &args[0];
        if raw_url == "uxntal:"
            || raw_url == "uxntal:/"
            || raw_url == "uxntal://"
            || raw_url == "uxntal:///"
        {
                // #[cfg(feature = "uses_gui")]
                // {
                //     println!("Starting GUI...");
                //     if let Err(e) = gui::start_gui().await {
                //         eprintln!("GUI error: {}", e);
                //         log::error!("GUI error: {}", e);
                //         std::process::exit(1);
                //     }
                // }

                // #[cfg(not(feature = "uses_gui"))]
                // {
                //     eprintln!("GUI support is not enabled. Rebuild with the `uses_gui` feature.");
                //     log::error!("GUI support is not enabled. Rebuild with the `uses_gui` feature.");
                //     std::process::exit(1);
                // }
            std::process::exit(0);
        }

        if raw_url.starts_with("uxntal://") {


let (entry_local, rom_dir) = match resolve_entry_from_url(raw_url) {
    Ok(v) => v,
    Err(e) => { eprintln!("Failed to resolve uxntal URL: {}", e); std::process::exit(1); }
};
println!("Resolved entry: {}", entry_local.display());

run_after_assembly = Some("cardinal-gui".to_owned());
run_after_cwd = Some(rom_dir.clone());

// Replace args[0] so the rest compiles the correct file
args[0] = entry_local
    .strip_prefix(r"\\?\")
    .unwrap_or(&entry_local)
    .display()
    .to_string();


            // println!("Handling uxntal:// URL: {}", raw_url);
            // let rebuilt_url = if let Some(rebuilt_url) = extract_target_from_uxntal(raw_url) {
            //     println!("Received URL: {:?}", rebuilt_url);
            //     rebuilt_url
            // } else {
            //     eprintln!("Malformed uxntal URL: {}", raw_url);
            //     std::process::exit(1);
            // };
            // // Rebuild the URL from the uxntal:// scheme.
            // // Example: uxntal://https///wiki.xxiivv.com/etc/cccc.tal.txt -> https://wiki.xxiivv.com/etc/cccc.tal.txt
            // // let path_part = &raw_url[9..];
            // // let rebuilt_url = if path_part.starts_with("https///") {
            // //     format!("https://{}", &path_part[8..])
            // // } else if path_part.starts_with("http///") {
            // //     format!("http://{}", &path_part[7..])
            // // } else if path_part.starts_with("file///") {
            // //     format!("file://{}", &path_part[7..])
            // // } else if path_part.starts_with("https//") {
            // //     format!("https://{}", &path_part[7..])
            // // } else if path_part.starts_with("http//") {
            // //     format!("http://{}", &path_part[6..])
            // // } else if path_part.starts_with("file//") {
            // //     format!("file://{}", &path_part[7..])
            // // } else {
            // //     // fallback: treat as a normal path or URL
            // //     path_part.to_string()
            // // };
        
            // // log::debug!("Received URL: {}", raw_url);
            // println!("Received URL: {:?}", rebuilt_url);
            // // let status = Command::new("e_window")
            // //     .arg(&format!("--title={}", rebuilt_url))
            // //     .arg(&rebuilt_url)
            // //     .status();
            // // match status {
            // //     Ok(s) if s.success() => {
            // //         println!("e_window launched successfully.");
            // //     }
            // //     Ok(s) => {
            // //         eprintln!("e_window exited with status: {}", s);
            // //     }
            // //     Err(e) => {
            // //         eprintln!("Failed to launch e_window: {}", e);
            // //     }
            // // }
            // // Compute a hash of the URL for directory naming
            // fn hash_url(url: &str) -> u64 {
            //     let mut hasher = DefaultHasher::new();
            //     url.hash(&mut hasher);
            //     hasher.finish()
            // }
        
            // // Extract filename from URL, fallback to "downloaded.tal"
            // fn filename_from_url(url: &str) -> String {
            //     url.split('/')
            //         .last()
            //         .and_then(|s| if s.is_empty() { None } else { Some(s) })
            //         .unwrap_or("downloaded.tal")
            //         .to_string()
            // }
        
            // // Download the file from the URL
            // fn download_url(url: &str, dest: &Path) -> Result<(), Box<dyn std::error::Error>> {
            //     let resp = reqwest::blocking::get(url)?;
            //     if resp.status() == reqwest::StatusCode::FORBIDDEN {
            //         // Try using curl if available
            //         let status = Command::new("curl")
            //             .arg("-L")
            //             .arg("-o")
            //             .arg(dest)
            //             .arg(url)
            //             .status();
            //         match status {
            //             Ok(s) if s.success() => return Ok(()),
            //             Ok(s) => return Err(format!("curl exited with status: {}", s).into()),
            //             Err(e) => return Err(format!("Failed to run curl: {}", e).into()),
            //         }
            //     }
            //     if !resp.status().is_success() {
            //         return Err(format!("Failed to download: HTTP {}", resp.status()).into());
            //     }
            //     let bytes = resp.bytes()?;
            //     let mut file = File::create(dest)?;
            //     file.write_all(&bytes)?;
            //     Ok(())
            // }
        

            // let url = &rebuilt_url;
            // let hash = hash_url(url);
            // let fname = filename_from_url(url);
            // let roms_dir = uxn_tal::paths::uxntal_roms_get_path().unwrap_or_else(|| PathBuf::from(".uxntal/roms"));
            // let rom_dir = roms_dir.join(format!("{}", hash));
            // // let status = Command::new("e_window")
            // //     .arg(&format!("--title={}", rom_dir.display()))
            // //     .arg(&rebuilt_url)
            // //     .status();
            // fs::create_dir_all(&rom_dir).map_err(|e| simple_err(&rom_dir, &format!("failed to create dir: {e}")))?;
            // let file_path = rom_dir.join(&fname);
        
            // if !file_path.exists() {
            //     println!("Downloading {} to {}", url, file_path.display());
            //     if let Err(e) = download_url(url, &file_path) {
            //         eprintln!("Download error: {}", e);
            //         let status = Command::new("e_window")
            //             .arg(&format!("--title={}", e))
            //             .arg(&rebuilt_url)
            //             .status();
            //         exit(1);
            //     }
            //     let url_file_path = file_path.with_extension("url");
            //     let url_file_contents = format!(
            //         "[InternetShortcut]\nURL={}\n",
            //         url
            //     );
            //     if let Err(e) = fs::write(&url_file_path, url_file_contents) {
            //         eprintln!("Failed to write .url file: {}", e);
            //     }
            // } else {
            //     println!("File already downloaded: {}", file_path.display());
            // }
            // run_after_assembly = Some(
            //     "cardinal-gui".to_owned()
            // );
            // run_after_cwd = Some(rom_dir.clone());
            // // Replace args[0] with the downloaded file path and continue as if user input that path
            // args[0] = file_path.strip_prefix(r"\\?\").unwrap_or(&file_path).display().to_string();
        }
    }
    println!("args: {:?}", args);
    if args.len() > 0 && args[0] == "--register" {
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
    let canon_input_p;
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
                return Err(simple_err(
                    std::path::Path::new(raw_input),
                    "input file not found (tried direct, +.tal, multi-root recursive scan)",
                ));
            }
        };
        // Canonical (or absolute fallback) before chdir so later paths remain valid
        canon_input_p = input_path
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
        } else if let Some(parent) = canon_input_p.parent() {
            if let Err(e) = std::env::set_current_dir(parent) {
                if want_verbose {
                    eprintln!("warning: failed to chdir to {}: {e}", parent.display());
                }
            } else if want_verbose {
                eprintln!("Changed working directory to {}", parent.display());
            }
        }
        source = fs::read_to_string(&canon_input_p)
            .map_err(|e| simple_err(&canon_input_p, &format!("failed to read: {e}")))?;
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
    let canon_input = canon_input_str.strip_prefix(r"\\?\").unwrap_or(&canon_input_str);
    if want_verbose {
        eprintln!("Resolved input : {}", canon_input);
        eprintln!("Output ROM     : {}", rom_path);
        if rust_iface.is_some() {
            eprintln!("Rust interface : enabled");
        }
    }

    let processed_src = if pre {
        match chocolatal::preprocess(&source, &canon_input, &root_dir) {
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
        let rel_path = match canon_input_p.strip_prefix(std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))) {
            Ok(p) => p.display().to_string(),
            Err(_) => canon_input_p.display().to_string(),
        };
        eprintln!("Relative path to input: {}", rel_path);
        let res = dbg.assemble_and_compare( &rel_path,&processed_src, true);
        return res.map(|_| ());
    }

    if run_after_assembly.is_some() {
        if rom_path_p.exists() {
            println!("ROM already exists at {}", rom_path);
        } else {
            let rom = asm.assemble(&processed_src, Some(canon_input.to_owned()))?;
            fs::write(&rom_path, &rom)
                .map_err(|e| simple_err(Path::new(rom_path), &format!("failed to write rom: {e}")))?;
            if want_verbose {
                eprintln!("Wrote ROM ({} bytes)", rom.len());
            } else {
                println!("{} ({} bytes)", rom_path, rom.len());
            }
        }
    } else {
        let rom = asm.assemble(&processed_src, Some(canon_input.to_owned()))?;
        fs::write(&rom_path, &rom)
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

    if let Some(cmd) = run_after_assembly {
        let cmd_name = cmd.clone();
        let path_to_emu = which::which(&cmd_name).map_err(|_| simple_err(Path::new("."), &format!("{cmd_name} not found in PATH")))?;
        println!("Running post-assembly command: {}", cmd);
        let dir_str = run_after_cwd
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| std::env::current_dir().map(|p| p.display().to_string()).unwrap_or_else(|_| ".".to_string()));
        println!("In directory: {}", dir_str);

        let status = Command::new(path_to_emu)
            .arg(&rom_path)
            .current_dir(run_after_cwd.unwrap_or_else(|| PathBuf::from(".")))
            .status();

        match status {
            Ok(s) if s.success() => {
                println!("Ran post-assembly command: {}", cmd);
            }
            Ok(s) => {
                eprintln!("Post-assembly command exited with status: {}", s);
            }
            Err(e) => {
                eprintln!("Failed to run post-assembly command: {}", e);
            }
        }
    }
    // Remove intermediate file unless --no-intermediate is set
    // if !no_intermediate {
    //     let _ = fs::remove_file(&pre_path);
    // }

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
    --r, --root[=DIR]     Set root directory for includes (default: current dir)
    --register            Register uxntal as a file handler (Windows only)
    --r, --root[=DIR]     Set root directory for includes (default: current dir)
    --register            Register uxntal as a file handler (Windows only)
    --help, -h            Show this help

Behavior:
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

#[cfg(target_os = "macos")]
fn register_protocol_per_user() -> std::io::Result<()> {
    use std::fs::{self, File};
    use std::io::{Write, stdin, stdout};
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
    let swift_app_delegate = format!(r#"
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
"#, bin_path = uxntal_path.display());
    let mut f = File::create(&app_delegate_file)?;
    f.write_all(swift_app_delegate.as_bytes())?;

    // Write main.swift
    let swift_main = r#"
import Cocoa

let delegate = AppDelegate()
NSApplication.shared.delegate = delegate
_ = NSApplicationMain(CommandLine.argc, CommandLine.unsafeArgv)
"#;
    let mut f = File::create(&main_file)?;
    f.write_all(swift_main.as_bytes())?;

    // Write Info.plist with correct version
    let plist = format!(r#"<?xml version="1.0" encoding="UTF-8"?>
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
"#, version=version, app_name=app_name);
    let mut f = File::create(&plist_file)?;
    f.write_all(plist.as_bytes())?;

    // Compile Swift app with both files
    let status = Command::new("xcrun")
        .args([
            "swiftc",
            "-o", &format!("{}/{}", temp_dir.display(), app_name),
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
    fs::copy(
        temp_dir.join(app_name),
        macos_dir.join(app_name),
    )?;
    fs::copy(&plist_file, app_contents.join("Info.plist"))?;

    // Move to ~/Applications as uxntal.app
    let user_app = PathBuf::from(format!("{}/Applications/uxntal.app", home));
    if user_app.exists() {
        println!("An existing uxntal.app was found at {}.", user_app.display());
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
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
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
    let mut mime_file = File::create(&mime_file_path)?;
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
    let mut desktop_file = File::create(&desktop_file_path)?;
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
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
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
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Failed to run cargo install",
        ));
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
