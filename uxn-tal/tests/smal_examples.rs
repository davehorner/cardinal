// Integration test: compile all SMAL examples using the uxn-tal assembler API.
// This test only runs when the crate is built with `--features uses_uxnsmal`.

use std::{error::Error, fs, path::PathBuf};
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};
use std::process::{Command, Stdio};

#[cfg(feature = "uses_uxnsmal")]
#[test]
fn compile_all_smal_examples() -> Result<(), Box<dyn Error>> {
    // Locate the examples directory by walking up from this crate's manifest
    // and checking for a sibling `uxnsmal/uxnsmal/examples` directory.
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut examples_dir: Option<PathBuf> = None;
    let mut cur = manifest.as_path();
    while let Some(parent) = cur.parent() {
        // 1) common layout: <repo>/uxnsmal/uxnsmal/examples
        let candidate = parent.join("uxnsmal/uxnsmal/examples");
        if candidate.exists() {
            examples_dir = Some(candidate);
            break;
        }
        // 2) some checkouts may have only uxnsal/examples or different nesting; scan children for a matching structure
        if let Ok(rd) = std::fs::read_dir(parent) {
            for ent in rd.flatten() {
                if ent.file_name().to_string_lossy().to_lowercase().starts_with("uxnsmal") {
                    let alt = ent.path().join("examples");
                    if alt.exists() {
                        examples_dir = Some(alt);
                        break;
                    }
                    let nested = ent.path().join("uxnsmal/examples");
                    if nested.exists() {
                        examples_dir = Some(nested);
                        break;
                    }
                }
            }
            if examples_dir.is_some() {
                break;
            }
        }
        cur = parent;
    }
    let examples_dir = examples_dir.expect("examples dir not found: searched upward from manifest for ux nsmal examples");

    // Prepare report directory and markdown report at crate manifest root
    let report_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("smal-report");
    fs::create_dir_all(&report_dir)?;
    let report_path = report_dir.join("report.md");
        // Compute workspace root (two levels above this crate's manifest: .../smol-cardinal)
        let workspace_root = manifest.parent().and_then(|p| p.parent()).map(|p| p.to_path_buf()).unwrap_or_else(|| manifest.clone());
    let mut report = fs::File::create(&report_path)?;
    writeln!(report, "# SMAL Examples Report")?;
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    writeln!(report, "Generated: {} (epoch seconds)\n", now)?;
    for entry in fs::read_dir(&examples_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("smal") {
            println!("Compiling SMAL example: {}", path.display());
            // Use the uxn-tal helper to assemble and write ROM+SYM.
            let (rom_path, sym_path, _size) = uxn_tal::assemble_file_with_symbols(&path)?;
            assert!(rom_path.exists(), "ROM not written for {}", path.display());
            assert!(sym_path.exists(), "SYM not written for {}", path.display());

            // Validate heuristics vs runtime (if a CLI emulator is available).
            use uxn_tal::probe_tal::{heuristic_uses_console, heuristic_uses_gui};
            use std::ffi::OsStr;
            use std::time::Duration;
            use crc32fast::Hasher;
            let src = fs::read_to_string(&path).unwrap_or_default();
            let predicts_console = heuristic_uses_console(&src);
            let predicts_gui = heuristic_uses_gui(&src);

            // Per-example artifact placeholders
            let stem = path.file_stem().and_then(OsStr::to_str).unwrap_or("example");
            let mut stdout_file_opt: Option<PathBuf> = None;
            let mut stderr_file_opt: Option<PathBuf> = None;
            let mut screenshot_file_opt: Option<PathBuf> = None;
            let mut screenshot_crc_opt: Option<u32> = None;

            // Prefer `cardinal-cli` for runtime validation; fallback to `uxnemu` or `uxncli`.
            let emulator = which::which("cardinal-cli")
                .or_else(|_| which::which("uxnemu"))
                .or_else(|_| which::which("uxncli"));
                if let Ok(emulator_path) = emulator {
                    println!("Found emulator: {} — running briefly to observe output", emulator_path.display());
                    let example_dir = report_dir.join("examples").join(stem);
                    let captures_dir = example_dir.join("captures");
                    std::fs::create_dir_all(&captures_dir)?;
                    let stdout_file = example_dir.join(format!("{}-stdout.txt", stem));
                    let stderr_file = example_dir.join(format!("{}-stderr.txt", stem));
                    let stdout_f = std::fs::File::create(&stdout_file)?;
                    let stderr_f = std::fs::File::create(&stderr_file)?;

                    let mut child = Command::new(&emulator_path)
                        .arg(rom_path.display().to_string())
                        .stdout(Stdio::from(stdout_f))
                        .stderr(Stdio::from(stderr_f))
                        .spawn()?;
                    // Let it run briefly to produce any console output, then kill.
                    std::thread::sleep(std::time::Duration::from_millis(700));
                    let _ = child.kill();
                    let _ = child.wait();
                    // Read files to determine length
                    let stdout_len = std::fs::metadata(&stdout_file).map(|m| m.len()).unwrap_or(0) as usize;
                    let stderr_len = std::fs::metadata(&stderr_file).map(|m| m.len()).unwrap_or(0) as usize;
                    stdout_file_opt = Some(stdout_file.clone());
                    stderr_file_opt = Some(stderr_file.clone());

                if predicts_console {
                    let strict = std::env::var("SMAL_TEST_STRICT").unwrap_or_default() == "1";
                    if stdout_len == 0 && stderr_len == 0 {
                        let msg = format!(
                            "Predicts console but emulator produced no output for {}",
                            path.display()
                        );
                        if strict {
                            panic!("{}", msg);
                        } else {
                            eprintln!("WARN: {} (set SMAL_TEST_STRICT=1 to make this a failure)", msg);
                        }
                    }
                }
                }
                if predicts_gui {
                    // If heuristics predict GUI, also try to spawn `cardinal-gui` so examples actually show.
                    println!("Predicts GUI: {} — CLI run is best-effort; attempting to spawn GUI", predicts_gui);

                    // Determine GUI timeout: prefer explicit `SMAL_GUI_TIMEOUT_MS`,
                    // else short timeout in strict/CI mode, longer for local runs.
                    let gui_timeout_ms = std::env::var("SMAL_GUI_TIMEOUT_MS").ok()
                        .and_then(|s| s.parse::<u64>().ok())
                        .unwrap_or_else(|| {
                            if std::env::var("SMAL_TEST_STRICT").unwrap_or_default() == "1" {
                                700
                            } else {
                                5000
                            }
                        });

                    // Optionally capture a screenshot if requested via `SMAL_GUI_CAPTURE=1`.
                    // Default to capturing for GUI-predicted examples so reports include screenshots.
                    let capture = std::env::var("SMAL_GUI_CAPTURE").unwrap_or_default() == "1" || predicts_gui;

                    // Prefer a workspace-built `cardinal-gui` if present in `cardinal/cardinal-gui/target/debug`.
                    let workspace_gui = {
                        let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
                        if let Some(cardinal_dir) = p.parent() {
                            let mut candidate = cardinal_dir.join("cardinal-gui").join("target").join("debug").join("cardinal-gui");
                            if cfg!(windows) {
                                candidate = candidate.with_extension("exe");
                            }
                            if candidate.exists() {
                                Some(candidate)
                            } else {
                                None
                            }
                        } else { None }
                    };

                    match workspace_gui.or_else(|| which::which("cardinal-gui").ok()) {
                        Some(gui_path) => {
                            println!("Also spawning GUI: {}", gui_path.display());

                            if capture {
                                // Attempt to spawn `cardinal-gui` with the capture flag; if this fails or
                                // no screenshot is produced, fall back to a normal spawn.
                                let stem = path.file_stem().and_then(OsStr::to_str).unwrap_or("example");
                                let example_dir = report_dir.join("examples").join(stem);
                                let captures_dir = example_dir.join("captures");
                                std::fs::create_dir_all(&captures_dir)?;
                                let screenshot = captures_dir.join("screenshot.png");

                                let mut cmd = std::process::Command::new(&gui_path);
                                cmd.arg("--capture-screenshot");
                                cmd.arg(&screenshot);
                                cmd.arg(rom_path.display().to_string());

                                match cmd.spawn() {
                                    Ok(mut gui_child) => {
                                        std::thread::sleep(Duration::from_millis(gui_timeout_ms));
                                        let _ = gui_child.kill();
                                        let _ = gui_child.wait();
                                        if screenshot.exists() {
                                            let data = std::fs::read(&screenshot)?;
                                            let mut h = Hasher::new();
                                            h.update(&data);
                                            let crc = h.finalize();
                                            screenshot_file_opt = Some(screenshot.clone());
                                            screenshot_crc_opt = Some(crc);
                                            println!("Captured screenshot: {} (crc32=0x{:08x})", screenshot.display(), crc);
                                        } else {
                                            eprintln!("WARN: capture requested but screenshot not created: {}", screenshot.display());
                                            // Fallback: spawn normally to at least show the GUI briefly
                                            let mut gui_child = std::process::Command::new(&gui_path)
                                                .arg(rom_path.display().to_string())
                                                .spawn()?;
                                            std::thread::sleep(Duration::from_millis(gui_timeout_ms));
                                            let _ = gui_child.kill();
                                            let _ = gui_child.wait();
                                            println!("Spawned and stopped cardinal-gui for {}", path.display());
                                        }
                                    }
                                    Err(_) => {
                                        eprintln!("WARN: failed to spawn cardinal-gui with capture flag; spawning normally");
                                        let mut gui_child = std::process::Command::new(&gui_path)
                                            .arg(rom_path.display().to_string())
                                            .spawn()?;
                                        std::thread::sleep(Duration::from_millis(gui_timeout_ms));
                                        let _ = gui_child.kill();
                                        let _ = gui_child.wait();
                                        println!("Spawned and stopped cardinal-gui for {}", path.display());
                                    }
                                }
                            } else {
                                let mut gui_child = std::process::Command::new(&gui_path)
                                    .arg(rom_path.display().to_string())
                                    .spawn()?;
                                std::thread::sleep(Duration::from_millis(gui_timeout_ms));
                                let _ = gui_child.kill();
                                let _ = gui_child.wait();
                                println!("Spawned and stopped cardinal-gui for {}", path.display());
                            }
                        }
                        None => {
                            eprintln!("WARN: cardinal-gui not found — GUI examples may not display");
                        }
                    }
                } else {
                // No CLI emulator found; for GUI-predicted examples try to spawn cardinal-gui instead.
                if predicts_gui {
                    let gui = which::which("cardinal-gui");
                    match gui {
                        Ok(gui_path) => {
                            println!("Found GUI runner: {} — spawning", gui_path.display());
                            use std::process::Command;
                            let mut child = Command::new(&gui_path)
                                .arg(rom_path.display().to_string())
                                .spawn()?;
                            // Let it run briefly to ensure it starts, then kill.
                            std::thread::sleep(std::time::Duration::from_millis(700));
                            let _ = child.kill();
                            let _ = child.wait();
                            println!("Spawned and stopped cardinal-gui for {}", path.display());
                        }
                        Err(_) => {
                            let strict = std::env::var("SMAL_TEST_STRICT").unwrap_or_default() == "1";
                            let msg = format!(
                                "No GUI runner found (cardinal-gui); skipping GUI runtime validation for {}",
                                path.display()
                            );
                            if strict {
                                panic!("{} (set SMAL_TEST_STRICT=0 to allow missing GUI runner)", msg);
                            } else {
                                eprintln!("WARN: {} (set SMAL_TEST_STRICT=1 to make this a failure)", msg);
                            }
                        }
                    }
                } else {
                    let strict = std::env::var("SMAL_TEST_STRICT").unwrap_or_default() == "1";
                    let msg = format!(
                        "No CLI emulator found (tried cardinal-cli, uxnemu, uxncli); skipping runtime validation for {}",
                        path.display()
                    );
                    if strict {
                        panic!("{} (set SMAL_TEST_STRICT=0 to allow missing emulator)", msg);
                    } else {
                        eprintln!("WARN: {} (set SMAL_TEST_STRICT=1 to make this a failure)", msg);
                    }
                }
            }

            // Append a section to the markdown report for this example (inside the smal branch so
            // all referenced variables are in scope). Use workspace-relative paths where possible.
            let rel = |p: &PathBuf| -> String {
                p.strip_prefix(&workspace_root)
                    .or_else(|_| p.strip_prefix(&report_dir))
                    .map(|s| s.display().to_string())
                    .unwrap_or_else(|_| p.display().to_string())
            };

            writeln!(report, "## {}\n", stem)?;
            writeln!(report, "- Path: {}", rel(&path))?;
            writeln!(report, "- ROM: {}", rel(&rom_path))?;
            writeln!(report, "- SYM: {}", rel(&sym_path))?;
            writeln!(report, "- Predicts console: {}", predicts_console)?;
            writeln!(report, "- Predicts GUI: {}", predicts_gui)?;

            // Inline small stdout/stderr outputs (under 24 lines); otherwise reference file.
                if let Some(p) = stdout_file_opt {
                let relp = rel(&p);
                let txt = fs::read_to_string(&p).unwrap_or_default();
                let nlines = txt.lines().count();
                if nlines > 0 && nlines <= 25 {
                    writeln!(report, "- stdout (inlined, {} lines):", nlines)?;
                    writeln!(report, "```")?;
                    writeln!(report, "{}", txt)?;
                    writeln!(report, "```")?;
                } else {
                    writeln!(report, "- stdout: {}", relp)?;
                }
            }

            if let Some(p) = stderr_file_opt {
                let relp = rel(&p);
                let txt = fs::read_to_string(&p).unwrap_or_default();
                let nlines = txt.lines().count();
                if nlines > 0 && nlines < 24 {
                    writeln!(report, "- stderr (inlined, {} lines):", nlines)?;
                    writeln!(report, "```")?;
                    writeln!(report, "{}", txt)?;
                    writeln!(report, "```")?;
                } else {
                    writeln!(report, "- stderr: {}", relp)?;
                }
            }

            if let Some(p) = screenshot_file_opt {
                writeln!(report, "- screenshot: {}", rel(&p))?;
                // Inline image for report viewers that render markdown
                writeln!(report, "![{}]({})", stem, rel(&p))?;
                if let Some(crc) = screenshot_crc_opt {
                    writeln!(report, "  - screenshot_crc32: 0x{:08x}", crc)?;
                }
            }

            writeln!(report, "\n---\n")?;
        }
    }
    Ok(())
}

// If the feature is not enabled, provide a harmless no-op test so CI doesn't fail to compile the test file.
#[cfg(not(feature = "uses_uxnsmal"))]
#[test]
fn compile_all_smal_examples_not_enabled() {
    eprintln!("SMAL feature not enabled; skipping SMAL example compilation tests.");
}
