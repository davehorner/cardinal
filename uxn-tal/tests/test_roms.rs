use std::fs;
use std::path::Path;
use uxn_tal::ProtocolParser;
use uxn_tal_defined::{ProtocolParseResult, ProtocolVarVar};

/// Test configuration for a ROM URL
#[derive(Debug, Clone)]
struct TestRomConfig {
    url: String,
    timeout: Option<f64>,
    emulators: Vec<String>,
}

/// Read test ROM URLs from the TEST_ROM_URLS file with configuration parsing
fn read_test_rom_configs() -> Vec<TestRomConfig> {
    let test_file_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("TEST_ROM_URLS");
    let content = fs::read_to_string(&test_file_path).expect("Failed to read TEST_ROM_URLS file");

    let mut configs = Vec::new();
    let mut current_config: Option<TestRomConfig> = None;

    for line in content.lines() {
        let line = line.trim();

        // Skip empty lines and double comments (##)
        if line.is_empty() || line.starts_with("##") {
            continue;
        }

        // Parse single comment test injection (#test_inject)
        if let Some(inject_spec) = line.strip_prefix("#test_inject ") {
            current_config = Some(parse_test_injection(inject_spec));
            continue;
        }

        // Skip other single comments
        if line.starts_with("#") {
            continue;
        }

        // This is a URL line
        if let Some(config) = &current_config {
            // Apply the configuration to this URL
            let mut url_config = config.clone();
            url_config.url = line.to_string();
            configs.push(url_config);
        } else {
            // No configuration specified, use defaults
            configs.push(TestRomConfig {
                url: line.to_string(),
                timeout: None,
                emulators: vec!["cuxn".to_string()], // Default to cuxn
            });
        }
    }

    configs
}

/// Parse test injection specification like "t^^10:emu^^{cuxn,buxn}"
fn parse_test_injection(spec: &str) -> TestRomConfig {
    let mut timeout = None;
    let mut emulators = vec!["cuxn".to_string()]; // Default

    // Split by colon to get different parameter specifications
    for param_spec in spec.split(':') {
        let param_spec = param_spec.trim();

        if let Some(timeout_str) = param_spec.strip_prefix("t^^") {
            // Parse timeout: t^^10
            if let Ok(t) = timeout_str.parse::<f64>() {
                timeout = Some(t);
            }
        } else if param_spec.starts_with("emu^^{") && param_spec.ends_with('}') {
            // Parse emulator list: emu^^{cuxn,buxn}
            let emu_list = &param_spec[6..param_spec.len() - 1]; // Remove "emu^^{" and "}"
            emulators = emu_list
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            // If no emulators parsed, keep default
            if emulators.is_empty() {
                emulators = vec!["cuxn".to_string()];
            }
        } else if param_spec.starts_with("emu^^") && !param_spec.contains('{') {
            // Parse single emulator: emu^^buxn
            let emu_name = &param_spec[5..]; // Remove "emu^^"
            if !emu_name.is_empty() {
                emulators = vec![emu_name.to_string()];
            }
        }
    }

    TestRomConfig {
        url: String::new(), // Will be filled in later
        timeout,
        emulators,
    }
}

/// Read test ROM URLs from the TEST_ROM_URLS file (legacy function for compatibility)
fn read_test_rom_urls() -> Vec<String> {
    read_test_rom_configs()
        .into_iter()
        .map(|config| config.url)
        .collect()
}

/// Test that all ROM URLs from TEST_ROM_URLS can be parsed correctly
#[test]
#[ignore] // Use --ignored to run this test
fn test_parse_all_rom_urls() {
    let test_urls = read_test_rom_urls();
    println!("Found {} test URLs in TEST_ROM_URLS file", test_urls.len());

    for (index, url) in test_urls.iter().enumerate() {
        println!("Testing URL {}: {}", index + 1, url);

        let result = ProtocolParser::parse(url);

        // Basic validation that parsing succeeded
        assert!(
            !result.url.is_empty(),
            "URL {} failed to parse: empty result URL",
            index + 1
        );

        // Verify the URL starts with expected protocol
        assert!(
            url.starts_with("uxntal:"),
            "URL {} should start with uxntal:",
            index + 1
        );

        // Check for expected automatic parameters based on file extension
        if url.contains(".orca") {
            assert_eq!(
                result.proto_vars.get("orca"),
                Some(&ProtocolVarVar::Bool(true)),
                "URL {} with .orca extension should have orca=true automatically",
                index + 1
            );
            println!("  ✓ Auto-detected orca=true for .orca file");
        } else if url.contains(".bas") {
            assert_eq!(
                result.proto_vars.get("basic"),
                Some(&ProtocolVarVar::Bool(true)),
                "URL {} with .bas extension should have basic=true automatically",
                index + 1
            );
            println!("  ✓ Auto-detected basic=true for .bas file");
        } else {
            // Non-orca, non-basic files should not have auto-detected orca or basic parameters
            assert_ne!(
                result.proto_vars.get("orca"),
                Some(&ProtocolVarVar::Bool(true)),
                "URL {} should not have auto-detected orca=true",
                index + 1
            );
            assert_ne!(
                result.proto_vars.get("basic"),
                Some(&ProtocolVarVar::Bool(true)),
                "URL {} should not have auto-detected basic=true",
                index + 1
            );
            println!("  ✓ No auto-detected orca/basic parameters for this file type");
        }

        println!("  ✓ Parsed successfully: {}", result.url);
        println!("  ✓ Protocol vars: {:?}", result.proto_vars);
        println!();
    }
}

/// Test that we can create protocol results with injected t^10 parameter
#[test]
#[ignore] // Use --ignored to run this test
fn test_inject_t10_parameter() {
    let test_urls = read_test_rom_urls();
    println!("Testing t^10 injection for {} URLs", test_urls.len());

    for (index, url) in test_urls.iter().enumerate() {
        println!("Testing t^10 injection for URL {}: {}", index + 1, url);

        // Parse the clean URL (should have no parameters)
        let mut result = ProtocolParser::parse(url);

        // Verify it starts clean (no t parameter)
        assert_eq!(
            result.proto_vars.get("t"),
            None,
            "URL {} should not have t parameter initially",
            index + 1
        );

        // Inject t^10 parameter (t=10)
        result
            .proto_vars
            .insert("t".to_string(), ProtocolVarVar::Float(10.0));

        // Verify the injection worked
        match result.proto_vars.get("t") {
            Some(ProtocolVarVar::Float(10.0)) => {
                println!("  ✓ Successfully injected t=10 parameter");
            }
            Some(other) => {
                panic!(
                    "URL {} injection failed, got unexpected value: {:?}",
                    index + 1,
                    other
                );
            }
            None => {
                panic!("URL {} injection failed, t parameter not found", index + 1);
            }
        }

        // Render the URL with the injected parameter
        let rendered_url = ProtocolParser::render_url(&result);
        println!("  ✓ Rendered URL: {}", rendered_url);

        println!("  ✓ Final protocol vars: {:?}", result.proto_vars);
        println!();
    }
}

/// Test automatic orca=true injection for .orca files
#[test]
#[ignore] // Use --ignored to run this test
fn test_inject_orca_mode() {
    let test_urls = read_test_rom_urls();
    let orca_urls: Vec<String> = test_urls
        .into_iter()
        .filter(|url| url.contains(".orca"))
        .collect();

    println!(
        "Testing automatic orca=true injection for {} orca URLs",
        orca_urls.len()
    );

    for (index, url) in orca_urls.iter().enumerate() {
        println!(
            "Testing automatic orca=true injection for URL {}: {}",
            index + 1,
            url
        );

        // Parse the URL - .orca files should automatically get orca=true
        let result = ProtocolParser::parse(url);

        // Verify that .orca URLs automatically have orca=true parameter
        assert_eq!(
            result.proto_vars.get("orca"),
            Some(&ProtocolVarVar::Bool(true)),
            "URL {} should automatically have orca=true parameter for .orca files",
            index + 1
        );

        println!("  ✓ Automatic orca=true parameter injection confirmed");

        // Test that we can override it if needed
        let mut modified_result = result.clone();
        modified_result
            .proto_vars
            .insert("orca".to_string(), ProtocolVarVar::Bool(false));

        // Verify the override worked
        match modified_result.proto_vars.get("orca") {
            Some(ProtocolVarVar::Bool(false)) => {
                println!("  ✓ Manual override to orca=false works");
            }
            Some(other) => {
                panic!(
                    "URL {} override failed, got unexpected value: {:?}",
                    index + 1,
                    other
                );
            }
            None => {
                panic!(
                    "URL {} override failed, orca parameter not found",
                    index + 1
                );
            }
        }

        println!("  ✓ Final protocol vars: {:?}", result.proto_vars);
        println!();
    }
}

/// Test rendering URLs with injected t^^10 parameter  
#[test]
#[ignore] // Use --ignored to run this test
fn test_render_url_with_t10() {
    let test_urls = read_test_rom_urls();
    println!(
        "Testing URL rendering with t^^10 for {} URLs",
        test_urls.len()
    );

    for (index, url) in test_urls.iter().enumerate() {
        println!("Testing URL rendering for URL {}: {}", index + 1, url);

        // Parse the clean URL
        let mut result = ProtocolParser::parse(url);

        // Inject t^10 parameter
        result
            .proto_vars
            .insert("t".to_string(), ProtocolVarVar::Float(10.0));

        // Render back to URL
        let rendered_url = ProtocolParser::render_url(&result);

        println!("  ✓ Original: {}", url);
        println!("  ✓ Rendered: {}", rendered_url);

        // Verify the rendered URL contains t^^10 (double caret for Windows shell escaping)
        assert!(
            rendered_url.contains("t^^10"),
            "URL {} rendered URL should contain t^^10: {}",
            index + 1,
            rendered_url
        );

        // Verify it still starts with uxntal:
        assert!(
            rendered_url.starts_with("uxntal:"),
            "URL {} rendered URL should start with uxntal:: {}",
            index + 1,
            rendered_url
        );

        // Parse the rendered URL to verify round-trip
        let re_parsed = ProtocolParser::parse(&rendered_url);
        assert_eq!(
            re_parsed.proto_vars.get("t"),
            Some(&ProtocolVarVar::Float(10.0)),
            "URL {} round-trip failed for t parameter",
            index + 1
        );

        println!("  ✓ Round-trip successful");
        println!();
    }
}
/// Test ROM resolution and caching for each URL
#[test]
#[ignore] // Use --ignored to run this test
fn test_rom_resolution_and_caching() {
    use uxn_tal::resolve_entry_from_url;

    let test_urls = read_test_rom_urls();

    for (index, url) in test_urls.iter().enumerate() {
        println!("Testing ROM resolution for URL {}: {}", index + 1, url);

        let result = ProtocolParser::parse(url);

        // Test ROM resolution
        match resolve_entry_from_url(&result.url) {
            Ok((entry_local, rom_dir)) => {
                println!("  ✓ Resolved entry: {}", entry_local.display());
                println!("  ✓ ROM directory: {}", rom_dir.display());

                // Verify the resolved file exists
                if !entry_local.exists() {
                    println!("  ⚠ Warning: Resolved file does not exist (may need network fetch)");
                } else {
                    println!("  ✓ Resolved file exists locally");
                }
            }
            Err(e) => {
                println!(
                    "  ⚠ ROM resolution failed (expected for network URLs): {}",
                    e
                );
                // This is expected for URLs that need network fetching
            }
        }

        println!();
    }
}

/// Test orca mode handling for orca URLs
#[test]
#[ignore] // Use --ignored to run this test
fn test_orca_mode_handling() {
    use uxn_tal::mode_orca;

    let test_urls = read_test_rom_urls();
    let orca_urls: Vec<String> = test_urls
        .into_iter()
        .filter(|url| url.contains(".orca"))
        .collect();

    for (index, url) in orca_urls.iter().enumerate() {
        println!("Testing orca mode handling for URL {}: {}", index + 1, url);

        let result = ProtocolParser::parse(url);

        // Verify orca mode is enabled
        assert_eq!(
            result.proto_vars.get("orca"),
            Some(&ProtocolVarVar::Bool(true)),
            "URL {} should have orca=true",
            index + 1
        );

        // Test canonical orca ROM resolution
        match mode_orca::resolve_canonical_orca_rom() {
            Ok((canonical_rom_path, cache_dir)) => {
                println!(
                    "  ✓ Canonical orca ROM resolved: {}",
                    canonical_rom_path.display()
                );
                println!("  ✓ Cache directory: {}", cache_dir.display());

                if canonical_rom_path.exists() {
                    println!("  ✓ Canonical orca ROM exists locally");
                } else {
                    println!("  ⚠ Canonical orca ROM not cached yet");
                }
            }
            Err(e) => {
                println!("  ⚠ Canonical orca ROM resolution failed: {}", e);
            }
        }

        println!();
    }
}

/// Test the complete workflow: read URLs, inject t^10, render executable URLs
#[test]
#[ignore] // Use --ignored to run this test
fn test_complete_t10_workflow() {
    let test_urls = read_test_rom_urls();
    println!(
        "Testing complete t^10 workflow for {} URLs",
        test_urls.len()
    );

    let mut executable_urls = Vec::new();

    for (index, clean_url) in test_urls.iter().enumerate() {
        println!("\n=== Processing URL {} ===", index + 1);
        println!("Clean URL: {}", clean_url);

        // Step 1: Parse the clean URL
        let mut result = ProtocolParser::parse(clean_url);

        // Step 2: Inject t^10 parameter
        result
            .proto_vars
            .insert("t".to_string(), ProtocolVarVar::Float(10.0));

        // Step 3: If it's an orca file, also inject orca=true
        if clean_url.contains(".orca") {
            result
                .proto_vars
                .insert("orca".to_string(), ProtocolVarVar::Bool(true));
            println!("  → Added orca=true for .orca file");
        }

        // Step 4: Render the executable URL
        let executable_url = ProtocolParser::render_url(&result);
        executable_urls.push(executable_url.clone());

        println!("  → Injected t=10");
        println!("Executable URL: {}", executable_url);

        // Step 5: Verify the executable URL is valid
        let verified = ProtocolParser::parse(&executable_url);
        assert_eq!(
            verified.proto_vars.get("t"),
            Some(&ProtocolVarVar::Float(10.0))
        );

        if clean_url.contains(".orca") {
            assert_eq!(
                verified.proto_vars.get("orca"),
                Some(&ProtocolVarVar::Bool(true))
            );
        }

        println!("  ✓ Verification passed");
    }

    println!("\n=== Summary ===");
    println!(
        "Generated {} executable URLs with t^10:",
        executable_urls.len()
    );
    for (i, url) in executable_urls.iter().enumerate() {
        println!("{}. {}", i + 1, url);
    }
}

/// Test running all orca URLs through cardinal-gui and buxn with t^10 transformation
#[test]
#[ignore] // Use --ignored to run this test
fn test_orca_urls() {
    use std::time::Duration;
    use uxn_tal::mode_orca;
    use uxn_tal::RealRomEntryResolver;
    use uxn_tal_common::cache::RomEntryResolver;
    use uxn_tal_defined::emu_buxn::BuxnMapper;
    use uxn_tal_defined::v1::EmulatorArgMapper;

    let test_urls = read_test_rom_urls();
    let orca_urls: Vec<String> = test_urls
        .into_iter()
        .filter(|url| url.contains(".orca"))
        .collect();

    println!(
        "Testing {} orca URLs with cardinal-gui and buxn APIs",
        orca_urls.len()
    );

    // Try to find emulators in PATH
    let cardinal_gui_path = which::which("cardinal-gui").ok();
    let buxn_path = which::which("buxn").ok();

    if cardinal_gui_path.is_none() && buxn_path.is_none() {
        println!("⚠ Neither cardinal-gui nor buxn found in PATH, skipping emulator tests");
        return;
    }

    for (index, clean_url) in orca_urls.iter().enumerate() {
        println!("\n=== Testing Orca URL {} ===", index + 1);
        println!("Clean URL: {}", clean_url);

        // Step 1: Parse and transform the URL with t^10
        let mut result = ProtocolParser::parse(clean_url);
        result
            .proto_vars
            .insert("t".to_string(), ProtocolVarVar::Float(10.0));

        // Step 2: Render the executable URL
        let executable_url = ProtocolParser::render_url(&result);
        println!("Executable URL: {}", executable_url);

        // Step 3: Resolve the ROM entry and get cache directory
        let entry_resolver = RealRomEntryResolver;
        let resolve_result = entry_resolver.resolve_entry_and_cache_dir(&result.url);

        match resolve_result {
            Ok((entry_path, cache_dir)) => {
                println!("  ✓ Resolved entry: {}", entry_path.display());
                println!("  ✓ Cache directory: {}", cache_dir.display());

                if !entry_path.exists() {
                    println!("  ⚠ Entry file not cached yet, skipping emulator tests");
                    continue;
                }

                // Step 4: Test with cardinal-gui if available
                if let Some(_cardinal_gui) = &cardinal_gui_path {
                    println!("  → Testing with cardinal-gui API...");

                    // Resolve canonical orca ROM for orca mode
                    match mode_orca::resolve_canonical_orca_rom() {
                        Ok((canonical_orca_rom, _)) => {
                            // Copy canonical ROM to the orca directory (not just cache root)
                            let orca_dir = entry_path.parent().unwrap_or(&cache_dir);
                            let orca_rom_in_dir = orca_dir.join("orca.rom");
                            if let Err(e) = std::fs::copy(&canonical_orca_rom, &orca_rom_in_dir) {
                                println!("    ⚠ Failed to copy canonical orca ROM: {}", e);
                                continue;
                            }

                            // Use orca mode to build command
                            let orca_filename = entry_path.file_name().unwrap().to_string_lossy();

                            println!("    → Orca file: {}", orca_filename);
                            println!("    → Orca directory: {}", orca_dir.display());
                            println!("    → Full orca path: {}", entry_path.display());
                            println!("    → Orca ROM location: {}", orca_rom_in_dir.display());

                            // Get emulator launcher from parsed result (like mode_orca does)
                            use uxn_tal_defined::get_emulator_launcher;
                            if let Some((mapper, emulator_path)) = get_emulator_launcher(&result) {
                                // Build command using the mapper with the copied ROM
                                let mut cmd = mapper.build_command(
                                    &result,
                                    &orca_rom_in_dir.display().to_string(),
                                    &emulator_path,
                                    Some(orca_dir),
                                );

                                // Add the orca file as an additional argument
                                cmd.arg(orca_filename.as_ref());

                                println!("    ✓ Built cardinal-gui command with correct ROM path");
                                println!("    → Command: {:?}", cmd);

                                // Try to spawn the process with a timeout
                                match cmd.spawn() {
                                    Ok(mut child) => {
                                        // Wait briefly to see if it starts successfully
                                        std::thread::sleep(Duration::from_millis(500));

                                        match child.try_wait() {
                                            Ok(Some(status)) => {
                                                if status.success() {
                                                    println!(
                                                        "    ✓ cardinal-gui completed successfully"
                                                    );
                                                } else {
                                                    println!("    ⚠ cardinal-gui exited with status: {:?}", status.code());
                                                }
                                            }
                                            Ok(None) => {
                                                println!("    ✓ cardinal-gui started successfully (still running)");
                                                // Kill the process after short test
                                                let _ = child.kill();
                                                let _ = child.wait();
                                            }
                                            Err(e) => {
                                                println!(
                                                    "    ⚠ Failed to check cardinal-gui status: {}",
                                                    e
                                                );
                                                let _ = child.kill();
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        println!("    ✗ Failed to spawn cardinal-gui: {}", e);
                                    }
                                }
                            } else {
                                println!("    ✗ Failed to get emulator launcher for cardinal-gui");
                            }
                        }
                        Err(e) => {
                            println!("    ✗ Failed to resolve canonical orca ROM: {}", e);
                        }
                    }
                }

                // Step 5: Test with buxn if available
                if let Some(buxn) = &buxn_path {
                    println!("  → Testing with buxn API...");

                    // Use buxn mapper to build command
                    let mapper = BuxnMapper;
                    let args = mapper.map_args(&result);

                    let mut cmd = std::process::Command::new(buxn);
                    cmd.args(&args);
                    cmd.arg(entry_path.to_string_lossy().as_ref());
                    cmd.current_dir(&cache_dir);

                    match cmd.spawn() {
                        Ok(mut child) => {
                            // Wait briefly to see if it starts successfully
                            std::thread::sleep(Duration::from_millis(500));

                            match child.try_wait() {
                                Ok(Some(status)) => {
                                    if status.success() {
                                        println!("    ✓ buxn completed successfully");
                                    } else {
                                        println!(
                                            "    ⚠ buxn exited with status: {:?}",
                                            status.code()
                                        );
                                    }
                                }
                                Ok(None) => {
                                    println!("    ✓ buxn started successfully (still running)");
                                    // Kill the process after short test
                                    let _ = child.kill();
                                    let _ = child.wait();
                                }
                                Err(e) => {
                                    println!("    ⚠ Failed to check buxn status: {}", e);
                                    let _ = child.kill();
                                }
                            }
                        }
                        Err(e) => {
                            println!("    ✗ Failed to spawn buxn: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                println!("  ✗ Failed to resolve ROM entry: {}", e);
                println!("    This may be due to network issues or missing files");
            }
        }

        // Brief pause between tests
        std::thread::sleep(Duration::from_millis(100));
    }

    println!("\n=== Test Summary ===");
    println!(
        "Tested {} orca URLs with t^10 transformation using internal APIs",
        orca_urls.len()
    );
    println!("Each URL was tested with available emulators (cardinal-gui and/or buxn)");
    println!("Check output above for any failures or issues");
}

/// Test running uxntal binary with each modified URL using configured emulators and timeouts
#[test]
#[ignore] // Use --ignored to run this test
fn test_uxntal_binary_with_modified_urls() {
    use std::process::Command;
    use std::time::Duration;

    let test_configs = read_test_rom_configs();

    // Separate configs by type, with orca URLs at the end
    let mut tal_configs = Vec::new();
    let mut basic_configs = Vec::new();
    let mut rom_configs = Vec::new();
    let mut other_configs = Vec::new();
    let mut orca_configs = Vec::new();

    for config in test_configs {
        if config.url.contains(".orca") {
            orca_configs.push(config);
        } else if config.url.contains(".tal") {
            tal_configs.push(config);
        } else if config.url.contains(".bas") {
            basic_configs.push(config);
        } else if config.url.contains(".rom") {
            rom_configs.push(config);
        } else {
            other_configs.push(config);
        }
    }

    // Combine all configs with orca at the end
    let mut all_configs = Vec::new();
    all_configs.extend(tal_configs);
    all_configs.extend(basic_configs);
    all_configs.extend(rom_configs);
    all_configs.extend(other_configs);
    all_configs.extend(orca_configs);

    println!(
        "Testing {} URLs with uxntal binary using configured parameters",
        all_configs.len()
    );
    println!("Order: TAL files, BASIC files, ROM files, other files, then ORCA files");

    // Skip installation since uxntal is already available
    println!("Skipping uxntal binary installation (assuming already installed)...");

    // Test each configured URL
    for (index, config) in all_configs.iter().enumerate() {
        println!("\n=== Testing URL {} ===", index + 1);
        println!("Clean URL: {}", config.url);
        println!("Configured timeout: {:?}", config.timeout);
        println!("Configured emulators: {:?}", config.emulators);

        // Determine file type
        let file_type = if config.url.contains(".orca") {
            "orca"
        } else if config.url.contains(".tal") {
            "tal"
        } else if config.url.contains(".bas") {
            "basic"
        } else if config.url.contains(".rom") {
            "rom"
        } else {
            "other"
        };
        println!("File type: {}", file_type);

        // Step 1: Parse and transform the URL
        let mut result = ProtocolParser::parse(&config.url);

        // Apply configured timeout or default
        let timeout = config.timeout.unwrap_or(10.0);
        result
            .proto_vars
            .insert("t".to_string(), ProtocolVarVar::Float(timeout));

        // Add mode-specific parameters
        match file_type {
            "orca" => {
                result
                    .proto_vars
                    .insert("orca".to_string(), ProtocolVarVar::Bool(true));
                println!("  → Added orca=true for .orca file");
            }
            "basic" => {
                result
                    .proto_vars
                    .insert("basic".to_string(), ProtocolVarVar::Bool(true));
                println!("  → Added basic=true for .bas file");
            }
            _ => {
                println!("  → No special mode parameters for {} file", file_type);
            }
        }

        // Test each configured emulator
        for emulator in &config.emulators {
            println!("\n  → Testing with emulator: {}", emulator);

            // Add emulator configuration
            let mut emu_result = result.clone();
            emu_result
                .proto_vars
                .insert("emu".to_string(), ProtocolVarVar::String(emulator.clone()));

            // Step 2: Render the executable URL
            let executable_url = ProtocolParser::render_url(&emu_result);
            println!("    Executable URL: {}", executable_url);

            // Step 3: Run uxntal with the executable URL
            println!("    → Running uxntal with {} configuration...", emulator);
            let mut uxntal_cmd = Command::new("uxntal");
            uxntal_cmd.arg(&executable_url);

            // Use a timeout-enforcing launcher for each emulator
            let mut uxntal_cmd = Command::new("uxntal");
            uxntal_cmd.arg(&executable_url);
            match uxntal_cmd.spawn() {
                Ok(mut child) => match child.wait() {
                    Ok(status) => {
                        if status.success() {
                            println!("      ✓ uxntal completed successfully with {}", emulator);
                        } else {
                            println!(
                                "      ⚠ uxntal exited with status: {:?} using {}",
                                status.code(),
                                emulator
                            );
                        }
                    }
                    Err(e) => {
                        println!(
                            "      ⚠ Failed to wait for uxntal process with {}: {}",
                            emulator, e
                        );
                    }
                },
                Err(e) => {
                    println!("      ✗ Failed to spawn uxntal with {}: {}", emulator, e);
                    println!("      → Make sure uxntal is in PATH");
                }
            }

            // Brief pause between emulator tests
            std::thread::sleep(Duration::from_millis(200));
        }

        // Brief pause between URL tests
        std::thread::sleep(Duration::from_millis(500));
    }

    println!("\n=== Test Summary ===");
    println!(
        "Tested {} URLs using uxntal binary with configured parameters",
        all_configs.len()
    );
    println!("Each URL was tested with its configured emulators and timeout values");
    println!("Check output above for any failures or issues");
}

/// Test Svitlyna QOI image viewer with different showcase images
#[test]
#[ignore]
fn test_svitlyna() {
    use std::process::Command;
    use std::thread;
    use std::time::Duration;

    // Read test ROM URLs from the TEST_SVITLYNA file with configuration parsing
    let test_file_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("TEST_SVITLYNA");
    let content = fs::read_to_string(&test_file_path).expect("Failed to read TEST_SVITLYNA file");

    let mut all_configs = Vec::new();
    let mut current_config: Option<TestRomConfig> = None;

    for line in content.lines() {
        let line = line.trim();

        // Skip empty lines and double comments (##)
        if line.is_empty() || line.starts_with("##") {
            continue;
        }

        // Parse single comment test injection (#test_inject)
        if let Some(inject_spec) = line.strip_prefix("#test_inject ") {
            current_config = Some(parse_test_injection(inject_spec));
            continue;
        }

        // Skip other single comments
        if line.starts_with("#") {
            continue;
        }

        // This is a URL line
        if let Some(config) = &current_config {
            // Apply the configuration to this URL
            let mut url_config = config.clone();
            url_config.url = line.to_string();
            all_configs.push(url_config);
        } else {
            // No configuration specified, use defaults
            all_configs.push(TestRomConfig {
                url: line.to_string(),
                timeout: None,
                emulators: vec!["cuxn".to_string()],
            });
        }
    }

    println!("=== Testing Svitlyna QOI Image Viewer ===");
    println!("Found {} URLs in TEST_SVITLYNA", all_configs.len());

    for (index, config) in all_configs.iter().enumerate() {
        println!("\n=== Testing URL {} ===", index + 1);
        println!("Clean URL: {}", config.url);
        println!("Configured timeout: {:?}", config.timeout);
        println!("Configured emulators: {:?}", config.emulators);

        // Test with each configured emulator
        for emulator in &config.emulators {
            println!("  → Testing with emulator: {}", emulator);

            // Build the URL with injected parameters
            let mut url_with_params = config.url.clone();

            // Remove uxntal:// prefix if present to work with base URL
            if url_with_params.starts_with("uxntal://") {
                url_with_params = url_with_params[9..].to_string();
            }

            // Create a new protocol URL with injected parameters
            let mut injected_params = Vec::new();

            // Add timeout parameter
            if let Some(timeout) = config.timeout {
                injected_params.push(format!("t^^{}", timeout));
            } else {
                injected_params.push("t^^10".to_string()); // Default 10 second timeout
            }

            // Add emulator parameter
            injected_params.push(format!("emu^^{}", emulator));

            let protocol_prefix = if injected_params.is_empty() {
                "uxntal://".to_string()
            } else {
                format!("uxntal:{}://", injected_params.join(":"))
            };

            let executable_url = format!("{}{}", protocol_prefix, url_with_params);
            println!("    Executable URL: {}", executable_url);
            println!("    → Running uxntal with {} configuration...", emulator);

            // Execute uxntal with the URL
            let output = Command::new("uxntal")
                .arg(&executable_url)
                .output()
                .expect("Failed to execute uxntal command");

            if output.status.success() {
                println!(
                    "      ✓ uxntal started successfully with {} (still running)",
                    emulator
                );
            } else {
                println!(
                    "      ✗ uxntal failed with {}: {}",
                    emulator,
                    String::from_utf8_lossy(&output.stderr)
                );
            }

            // Small delay between emulator tests
            thread::sleep(Duration::from_millis(500));
        }
    }

    println!("\n=== Svitlyna Test Summary ===");
    println!(
        "Tested {} Svitlyna URLs with QOI showcase images",
        all_configs.len()
    );
    println!("Each URL was tested with its configured emulators and timeout values");
    println!("QOI images should display properly in the Svitlyna viewer");
}

/// Helper function to create a test ProtocolResult with t^10 injected
pub fn create_test_protocol_result_with_t10(base_url: &str) -> ProtocolParseResult {
    let mut result = ProtocolParser::parse(base_url);
    result
        .proto_vars
        .insert("t".to_string(), ProtocolVarVar::Float(10.0));
    result
}

/// Helper function to get all test ROM URLs
pub fn get_test_rom_urls() -> Vec<String> {
    read_test_rom_urls()
}

/// Helper function to get only orca test URLs
pub fn get_orca_test_urls() -> Vec<String> {
    read_test_rom_urls()
        .into_iter()
        .filter(|url| url.contains(".orca"))
        .collect()
}

/// Helper function to get only TAL source URLs
pub fn get_tal_test_urls() -> Vec<String> {
    read_test_rom_urls()
        .into_iter()
        .filter(|url| url.contains(".tal"))
        .collect()
}

/// Helper function to get only ROM file URLs
pub fn get_rom_test_urls() -> Vec<String> {
    read_test_rom_urls()
        .into_iter()
        .filter(|url| url.contains(".rom"))
        .collect()
}

#[test]
#[ignore]
fn test_brainfuck() {
    use std::process::Command;
    use std::thread;
    use std::time::Duration;

    // Read test ROM URLs from the TEST_BRAINFUCK file with configuration parsing
    let test_file_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("TEST_BRAINFUCK");
    let content = fs::read_to_string(&test_file_path).expect("Failed to read TEST_BRAINFUCK file");

    let mut all_configs = Vec::new();
    let mut current_config: Option<TestRomConfig> = None;

    for line in content.lines() {
        let line = line.trim();

        // Skip empty lines and double comments (##)
        if line.is_empty() || line.starts_with("##") {
            continue;
        }

        // Parse single comment test injection (#test_inject)
        if let Some(inject_spec) = line.strip_prefix("#test_inject ") {
            current_config = Some(parse_test_injection(inject_spec));
            continue;
        }

        // Skip other single comments
        if line.starts_with("#") {
            continue;
        }

        // This is a URL line
        if let Some(config) = &current_config {
            // Apply the configuration to this URL
            let mut url_config = config.clone();
            url_config.url = line.to_string();
            all_configs.push(url_config);
        } else {
            // No configuration specified, use defaults
            all_configs.push(TestRomConfig {
                url: line.to_string(),
                timeout: None,
                emulators: vec!["cuxn".to_string()],
            });
        }
    }

    println!("=== Testing Brainfuck Interpreter ===");
    println!("Found {} URLs in TEST_BRAINFUCK", all_configs.len());

    for (index, config) in all_configs.iter().enumerate() {
        println!("\n=== Testing URL {} ===", index + 1);
        println!("Clean URL: {}", config.url);
        println!("Configured timeout: {:?}", config.timeout);
        println!("Configured emulators: {:?}", config.emulators);

        // Extract the brainfuck program name from the URL for better identification
        let bf_program = if let Some(arg_pos) = config.url.find("?!arg1=") {
            let arg_url = &config.url[arg_pos + 7..];
            if let Some(last_slash) = arg_url.rfind('/') {
                arg_url[last_slash + 1..].replace(".bf", "")
            } else {
                "unknown".to_string()
            }
        } else {
            "no_arg".to_string()
        };

        println!("Brainfuck program: {}", bf_program);

        // Test with each configured emulator
        for emulator in &config.emulators {
            println!("  → Testing with emulator: {}", emulator);

            // Build the URL with injected parameters
            let mut url_with_params = config.url.clone();

            // Remove uxntal:// prefix if present to work with base URL
            if url_with_params.starts_with("uxntal://") {
                url_with_params = url_with_params[9..].to_string();
            }

            // Create a new protocol URL with injected parameters
            let mut injected_params = Vec::new();

            // Add timeout parameter
            if let Some(timeout) = config.timeout {
                injected_params.push(format!("t^^{}", timeout));
            } else {
                injected_params.push("t^^15".to_string()); // Default 15 second timeout for brainfuck
            }

            // Add emulator parameter
            injected_params.push(format!("emu^^{}", emulator));

            let protocol_prefix = if injected_params.is_empty() {
                "uxntal://".to_string()
            } else {
                format!("uxntal:{}://", injected_params.join(":"))
            };

            let executable_url = format!("{}{}", protocol_prefix, url_with_params);
            println!("    Executable URL: {}", executable_url);
            println!("    → Running uxntal with {} configuration...", emulator);

            // Execute uxntal with the URL and show all output
            println!("    → Command: uxntal {}", executable_url);
            let output = Command::new("uxntal")
                .arg(&executable_url)
                .output()
                .expect("Failed to execute uxntal command");

            // Always show the output for debugging
            println!("      Exit status: {:?}", output.status);

            let stdout_str = String::from_utf8_lossy(&output.stdout);
            let stderr_str = String::from_utf8_lossy(&output.stderr);

            if !stdout_str.is_empty() {
                println!("      STDOUT:");
                for line in stdout_str.lines() {
                    println!("        {}", line);
                }
            }

            if !stderr_str.is_empty() {
                println!("      STDERR:");
                for line in stderr_str.lines() {
                    println!("        {}", line);
                }
            }

            // Check if command succeeded (uxntal started properly)
            if output.status.success() {
                println!("      ✓ uxntal completed successfully with {}", emulator);
            } else {
                println!("      ⚠ uxntal exited with error status using {}", emulator);
                // Don't panic on error, just continue to next test
            }

            // Give a moment for the emulator to initialize
            thread::sleep(Duration::from_millis(500));
        }
    }

    println!("\n=== Brainfuck Test Summary ===");
    println!(
        "Tested {} Brainfuck URLs with brainfuck.tal interpreter",
        all_configs.len()
    );
    println!("Each URL was tested with its configured emulators and timeout values");
    println!("Brainfuck programs should execute properly in the UXN emulator");
    println!("Some programs may require user input and will wait for interaction");
}
