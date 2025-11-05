use crate::util::pause_on_error;
use std::path::PathBuf;
use std::time::Duration;
use uxn_tal_defined::v1::{ProtocolParseResult, ProtocolQueryVarVar, ProtocolVarVar};
use uxn_tal_defined::EmulatorLauncher;

/// Download an HTTP URL to a temporary file and return the local path
/// Returns the original string if it doesn't start with @http
pub fn resolve_arg_url(arg_value: &str, cache_dir: Option<&std::path::Path>) -> String {
    println!("[DEBUG] resolve_arg_url called with: {}", arg_value);
    // Only process arguments that start with @http
    if arg_value.starts_with("@http://") || arg_value.starts_with("@https://") {
        let url_to_check = &arg_value[1..]; // Remove the @ prefix
        println!("[DEBUG] Processing @http URL: {}", url_to_check);

        match download_url_to_cache(url_to_check, cache_dir) {
            Ok(local_path) => {
                println!("[DEBUG] Downloaded and saved to: {}", local_path.display());
                // Return just the filename, not the full path
                let filename = local_path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("downloaded_file");
                println!("[DEBUG] Returning filename: {}", filename);
                filename.to_string()
            }
            Err(e) => {
                eprintln!("Warning: Failed to download {}: {}", url_to_check, e);
                eprintln!("Using original URL as argument");
                arg_value.to_string()
            }
        }
    } else {
        println!("[DEBUG] Not an @http URL, returning as-is: {}", arg_value);
        // Not a @http URL, return as-is
        arg_value.to_string()
    }
}

/// Download a URL to the cache directory
fn download_url_to_cache(
    url: &str,
    cache_dir: Option<&std::path::Path>,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    // Use the provided cache_dir or create a temp directory
    let target_dir = if let Some(dir) = cache_dir {
        dir.to_path_buf()
    } else {
        std::env::temp_dir()
    };

    // Extract filename from URL
    let filename = url
        .rsplit('/')
        .find(|s| !s.is_empty())
        .unwrap_or("downloaded_file");

    let local_path = target_dir.join(filename);

    // Skip download if file already exists
    if local_path.exists() {
        println!("Using cached file: {}", local_path.display());
        return Ok(local_path);
    }

    // Download the file
    println!(
        "Downloading !arg1 file: {} -> {}",
        url,
        local_path.display()
    );

    let bytes = crate::fetch::downloader::http_get(url)?;
    crate::fetch::downloader::write_bytes(&local_path, &bytes)?;

    println!("Downloaded !arg1 file successfully");
    Ok(local_path)
}

/// Spawn emulator with common error handling and debug output
pub fn spawn_emulator_with_logging(
    mut cmd: std::process::Command,
    verbose: bool,
) -> Result<(), String> {
    if verbose {
        println!("[DEBUG] Spawning emulator: {:?}", cmd);
    }

    // Show the full command with arguments
    let program = cmd.get_program().to_string_lossy();
    let args: Vec<String> = cmd
        .get_args()
        .map(|a| a.to_string_lossy().to_string())
        .collect();
    if args.is_empty() {
        println!("Launched emulator: {}", program);
    } else {
        println!("Launched emulator: {} {}", program, args.join(" "));
    }

    match cmd.spawn() {
        Ok(_) => Ok(()),
        Err(e) => {
            let error_msg = format!("Failed to spawn emulator: {}", e);
            eprintln!("{}", error_msg);
            pause_on_error();
            Err(error_msg)
        }
    }
}

/// Spawn emulator with stdin support and common error handling and debug output
pub fn spawn_emulator_with_stdin_and_logging(
    mut cmd: std::process::Command,
    result: &ProtocolParseResult,
    verbose: bool,
) -> Result<(), String> {
    if verbose {
        println!("[DEBUG] Spawning emulator: {:?}", cmd);
    }

    // Check for !stdin bang variable and prepare stdin content
    let stdin_content = if let Some(stdin_var) = result.bang_vars.get("stdin") {
        match &stdin_var.value {
            ProtocolQueryVarVar::String(stdin_value) => {
                println!("Found !stdin variable: {}", stdin_value);
                match fetch_stdin_content(stdin_value) {
                    Ok(mut content) => {
                        println!("Successfully fetched {} bytes for stdin", content.len());
                        // Always append Ctrl+Z (0x1A) to explicitly send EOF
                        content.push(0x1A);
                        Some(content)
                    }
                    Err(e) => {
                        eprintln!("Failed to fetch stdin content: {}", e);
                        return Err(format!("Failed to fetch stdin content: {}", e));
                    }
                }
            }
            _ => {
                eprintln!("!stdin variable is not a string");
                return Err("!stdin variable must be a string".to_string());
            }
        }
    } else {
        None
    };

    // Configure stdin pipe if we have content to send
    if stdin_content.is_some() {
        cmd.stdin(std::process::Stdio::piped());
    }

    // Show the full command with arguments
    let program = cmd.get_program().to_string_lossy();
    let args: Vec<String> = cmd
        .get_args()
        .map(|a| a.to_string_lossy().to_string())
        .collect();
    if args.is_empty() {
        println!("Launched emulator: {}", program);
    } else {
        println!("Launched emulator: {} {}", program, args.join(" "));
    }

    match cmd.spawn() {
        Ok(mut child) => {
            // Handle stdin piping if we have content
            if let Some(content) = stdin_content {
                if let Some(mut stdin) = child.stdin.take() {
                    println!("Piping {} bytes to emulator stdin", content.len());
                    std::thread::spawn(move || {
                        use std::io::Write;
                        // Longer sleep to ensure emulator is ready for input
                        std::thread::sleep(std::time::Duration::from_millis(200));
                        if let Err(e) = stdin.write_all(&content) {
                            eprintln!("Failed to write to stdin: {}", e);
                        }
                        if let Err(e) = stdin.flush() {
                            eprintln!("Failed to flush stdin: {}", e);
                        }
                        // stdin is automatically closed when it goes out of scope
                    });
                } else {
                    eprintln!("Failed to get stdin handle for emulator");
                }
            }

            Ok(())
        }
        Err(e) => {
            let error_msg = format!("Failed to spawn emulator: {}", e);
            eprintln!("{}", error_msg);
            pause_on_error();
            Err(error_msg)
        }
    }
}

/// Spawn emulator with timeout support
/// For emulators that support native --timeout (like cuxn), just spawn normally
/// For emulators that don't support --timeout (like buxn, uxn), spawn and kill after timeout
pub fn spawn_emulator_with_timeout(
    launcher: &dyn EmulatorLauncher,
    result: &ProtocolParseResult,
    mut cmd: std::process::Command,
    emulator_path: &std::path::Path,
    verbose: bool,
) -> Result<(), String> {
    if verbose {
        println!("[DEBUG] Spawning emulator with timeout support: {:?}", cmd);
    }

    // Check for !stdin bang variable and prepare stdin content
    let stdin_content = if let Some(stdin_var) = result.bang_vars.get("stdin") {
        match &stdin_var.value {
            ProtocolQueryVarVar::String(stdin_value) => {
                println!("Found !stdin variable: {}", stdin_value);
                match fetch_stdin_content(stdin_value) {
                    Ok(mut content) => {
                        println!("Successfully fetched {} bytes for stdin", content.len());
                        // Always append Ctrl+Z (0x1A) to explicitly send EOF
                        content.push(0x1A);
                        Some(content)
                    }
                    Err(e) => {
                        eprintln!("Failed to fetch stdin content: {}", e);
                        return Err(format!("Failed to fetch stdin content: {}", e));
                    }
                }
            }
            _ => {
                eprintln!("!stdin variable is not a string");
                return Err("!stdin variable must be a string".to_string());
            }
        }
    } else {
        None
    };

    // Configure stdin pipe if we have content to send
    if stdin_content.is_some() {
        cmd.stdin(std::process::Stdio::piped());
    }

    // Get timeout value from protocol result (either t or timeout parameter)
    let timeout_seconds = get_timeout_from_result(result);

    if launcher.timeout_follow_forceexit(emulator_path) {
        if let Some(timeout_seconds) = timeout_seconds {
            // Emulator doesn't support native timeout, so we need to manually kill it
            let timeout_duration = Duration::from_secs_f32(timeout_seconds);

            match cmd.spawn() {
                Ok(mut child) => {
                    // Handle stdin piping if we have content
                    if let Some(content) = stdin_content {
                        if let Some(mut stdin) = child.stdin.take() {
                            println!("Piping {} bytes to emulator stdin", content.len());
                            std::thread::spawn(move || {
                                use std::io::Write;
                                // Longer sleep to ensure emulator is ready for input
                                std::thread::sleep(std::time::Duration::from_millis(200));
                                if let Err(e) = stdin.write_all(&content) {
                                    eprintln!("Failed to write to stdin: {}", e);
                                }
                                if let Err(e) = stdin.flush() {
                                    eprintln!("Failed to flush stdin: {}", e);
                                }
                                // stdin is automatically closed when it goes out of scope
                            });
                        } else {
                            eprintln!("Failed to get stdin handle for emulator");
                        }
                    }

                    // Show the full command with arguments
                    let program = cmd.get_program().to_string_lossy();
                    let args: Vec<String> = cmd
                        .get_args()
                        .map(|a| a.to_string_lossy().to_string())
                        .collect();
                    if args.is_empty() {
                        println!(
                            "Launched emulator: {} (manual timeout: {:.1}s)",
                            program, timeout_seconds
                        );
                    } else {
                        println!(
                            "Launched emulator: {} {} (manual timeout: {:.1}s)",
                            program,
                            args.join(" "),
                            timeout_seconds
                        );
                    }

                    // Wait for the child process or timeout, whichever comes first
                    let timeout_val = timeout_seconds;
                    let handle = std::thread::spawn(move || {
                        let start_time = std::time::Instant::now();

                        loop {
                            // Check if the process has exited naturally
                            match child.try_wait() {
                                Ok(Some(_status)) => {
                                    // Process exited naturally
                                    return;
                                }
                                Ok(None) => {
                                    // Process is still running, check if we've hit timeout
                                    if start_time.elapsed() >= timeout_duration {
                                        // Timeout reached, kill the process
                                        if let Err(e) = child.kill() {
                                            eprintln!("Failed to kill emulator process: {}", e);
                                        } else {
                                            println!(
                                                "Emulator process killed after {:.1}s timeout",
                                                timeout_val
                                            );
                                        }
                                        return;
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Error checking process status: {}", e);
                                    return;
                                }
                            }

                            // Sleep briefly before checking again
                            std::thread::sleep(Duration::from_millis(100));
                        }
                    });

                    // Join the timeout thread to ensure we wait for the process to complete or timeout
                    if let Err(e) = handle.join() {
                        eprintln!("Timeout thread panicked: {:?}", e);
                    }

                    Ok(())
                }
                Err(e) => {
                    let error_msg = format!("Failed to spawn emulator: {}", e);
                    eprintln!("{}", error_msg);
                    pause_on_error();
                    Err(error_msg)
                }
            }
        } else {
            // Emulator supports native timeout or no timeout specified, use normal spawn
            spawn_emulator_with_stdin_and_logging(cmd, result, verbose)
        }
    } else {
        // Emulator supports native timeout or no timeout specified, use normal spawn
        spawn_emulator_with_stdin_and_logging(cmd, result, verbose)
    }
}

/// Extract timeout value from ProtocolParseResult
/// Returns Some(seconds) if timeout is specified, None otherwise
fn get_timeout_from_result(result: &ProtocolParseResult) -> Option<f32> {
    // Check protocol variables first (t or timeout)
    if let Some(timeout_var) = result
        .proto_vars
        .get("t")
        .or(result.proto_vars.get("timeout"))
    {
        match timeout_var {
            ProtocolVarVar::Float(f) => return Some(*f as f32),
            ProtocolVarVar::Int(i) => return Some(*i as f32),
            _ => {}
        }
    }

    // Check query variables
    if let Some(timeout_query) = result
        .query_vars
        .get("t")
        .or(result.query_vars.get("timeout"))
    {
        match &timeout_query.value {
            ProtocolQueryVarVar::Float(f) => return Some(*f as f32),
            ProtocolQueryVarVar::Int(i) => return Some(*i as f32),
            ProtocolQueryVarVar::String(s) => {
                if let Ok(f) = s.parse::<f32>() {
                    return Some(f);
                }
            }
            _ => {}
        }
    }

    None
}

/// Fetch content for stdin from a bang variable
/// Supports @http/https URLs and direct content
pub fn fetch_stdin_content(stdin_value: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    if stdin_value.starts_with("@http://") || stdin_value.starts_with("@https://") {
        let url = &stdin_value[1..]; // Remove the @ prefix
        println!("Downloading stdin content from: {}", url);
        crate::fetch::downloader::http_get(url)
    } else {
        // Direct content, just return as bytes
        Ok(stdin_value.as_bytes().to_vec())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FileType {
    Rom,
    Orca,
    Basic,
    Tal,
}

/// Detect file type based on extension
pub fn detect_file_type(path: &std::path::Path) -> FileType {
    let path_str = path.to_string_lossy();

    // Check for .tal.txt files first
    if path_str.ends_with(".tal.txt") {
        return FileType::Tal;
    }

    match path.extension().and_then(|ext| ext.to_str()) {
        Some("rom") => FileType::Rom,
        Some("orca") => FileType::Orca,
        Some("bas") => FileType::Basic,
        Some("tal") => FileType::Tal,
        _ => FileType::Tal,
    }
}
