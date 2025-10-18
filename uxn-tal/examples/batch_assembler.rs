// use std::fs;
use std::path::Path;
use uxn_tal::{Assembler, AssemblerError};

// Find all .tal files recursively in the workspace
fn find_tal_files(root: &str) -> Result<Vec<String>, std::io::Error> {
    let mut files = Vec::new();
    for entry in walkdir::WalkDir::new(root) {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("tal") {
            files.push(path.to_string_lossy().to_string());
        }
    }
    Ok(files)
}

// Assemble a single TAL file and return ROM size or error
fn assemble_file(file_path: &str) -> Result<usize, AssemblerError> {
    let source = std::fs::read_to_string(file_path)?;
    let mut assembler = Assembler::new();
    // Always use the canonical path for error context
    let path = std::fs::canonicalize(file_path)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| file_path.to_string());
    let rom = assembler.assemble(&source, Some(path))?;
    Ok(rom.len())
}

// Assemble a TAL file and write symbol file, returning ROM and symbol file paths and size
fn assemble_file_with_symbols(
    path: &Path,
) -> Result<(std::path::PathBuf, std::path::PathBuf, usize), AssemblerError> {
    let source = std::fs::read_to_string(path)?;
    let mut assembler = Assembler::new();
    let canonical = std::fs::canonicalize(path)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| path.to_string_lossy().to_string());
    let rom = assembler.assemble(&source, Some(canonical))?;
    let rom_path = path.with_extension("rom");
    let sym_path = path.with_extension("sym");
    std::fs::write(&rom_path, &rom)?;
    let sym_txt = assembler.generate_symbol_file();
    std::fs::write(&sym_path, sym_txt)?;
    Ok((rom_path, sym_path, rom.len()))
}

// Assemble a TAL file and write ROM, returning ROM path and size
fn assemble_file_auto(path: &Path) -> Result<(std::path::PathBuf, usize), AssemblerError> {
    let source = std::fs::read_to_string(path)?;
    let mut assembler = Assembler::new();
    let canonical = std::fs::canonicalize(path)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| path.to_string_lossy().to_string());
    let rom = assembler.assemble(&source, Some(canonical))?;
    let rom_path = path.with_extension("rom");
    std::fs::write(&rom_path, &rom)?;
    Ok((rom_path, rom.len()))
}

// Directory containing demo TAL files (update as needed)
const DEMOS_DIR: &str = "../tal";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔨 UXN TAL Batch Assembler Test");
    println!("Testing all TAL files in the project");
    println!("====================================");

    let tal_files = find_tal_files(".")?;
    let mut successful = 0;
    let mut failed = 0;
    let mut total_size = 0;
    // NEW: collect failed builds
    struct Failure {
        file: String,
        reason: String,
        lines: Vec<String>,
    }
    let mut failures: Vec<Failure> = Vec::new();

    for file_path in &tal_files {
        print!("Assembling {}... ", file_path);

        match assemble_file(file_path) {
            Ok(size) => {
                println!("✅ Success ({} bytes)", size);
                successful += 1;
                total_size += size;
            }
            Err(e) => {
                let msg = format!("{e}");
                println!("❌ Failed: {}", first_line(&msg));
                failed += 1;
                failures.push(Failure {
                    file: file_path.to_string(),
                    reason: first_line(&msg).to_string(),
                    lines: last_n_nonempty_lines(&msg, 3),
                });
            }
        }
    }

    // Check if --symbols flag is provided
    let generate_symbols = std::env::args().any(|arg| arg == "--symbols" || arg == "-s");

    if generate_symbols {
        println!("🔍 Symbol file generation enabled\n");
    }

    // Read directory and process all .tal files
    if let Ok(entries) = std::fs::read_dir(DEMOS_DIR) {
        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            // Skip if not a .tal file
            if path.extension().and_then(|s| s.to_str()) != Some("tal") {
                continue;
            }

            let filename = path.file_name().unwrap().to_string_lossy();
            print!("📝 Assembling {}... ", filename);

            if generate_symbols {
                match assemble_file_with_symbols(&path) {
                    Ok((rom_path, sym_path, size)) => {
                        println!(
                            "✅ {} bytes -> {} + {}",
                            size,
                            rom_path.file_name().unwrap().to_string_lossy(),
                            sym_path.file_name().unwrap().to_string_lossy()
                        );
                        successful += 1;
                        total_size += size;
                    }
                    Err(AssemblerError::Io(e)) => {
                        let msg = format!("{e}");
                        println!("❌ IO error: {}", first_line(&msg));
                        failed += 1;
                        failures.push(Failure {
                            file: path.display().to_string(),
                            reason: "IO error".into(),
                            lines: last_n_nonempty_lines(&msg, 3),
                        });
                    }
                    Err(e) => {
                        let msg = format!("{e}");
                        println!("❌ Assembly error: {}", first_line(&msg));
                        failed += 1;
                        failures.push(Failure {
                            file: path.display().to_string(),
                            reason: first_line(&msg).to_string(),
                            lines: last_n_nonempty_lines(&msg, 3),
                        });
                    }
                }
            } else {
                match assemble_file_auto(&path) {
                    Ok((rom_path, size)) => {
                        println!(
                            "✅ {} bytes -> {}",
                            size,
                            rom_path.file_name().unwrap().to_string_lossy()
                        );
                        successful += 1;
                        total_size += size;
                    }
                    Err(AssemblerError::Io(e)) => {
                        let msg = format!("{e}");
                        println!("❌ IO error: {}", first_line(&msg));
                        failed += 1;
                        failures.push(Failure {
                            file: path.display().to_string(),
                            reason: "IO error".into(),
                            lines: last_n_nonempty_lines(&msg, 3),
                        });
                    }
                    Err(e) => {
                        let msg = format!("{e}");
                        println!("❌ Assembly error: {}", first_line(&msg));
                        failed += 1;
                        failures.push(Failure {
                            file: path.display().to_string(),
                            reason: first_line(&msg).to_string(),
                            lines: last_n_nonempty_lines(&msg, 3),
                        });
                    }
                }
            }
        }
    } else {
        println!(
            "ℹ️ DEMOS_DIR '{}' does not exist, skipping demo assembly.",
            DEMOS_DIR
        );
    }

    println!("\n📊 Results:");
    println!("  ✅ Successfully assembled: {} files", successful);
    println!("  ❌ Failed: {} files", failed);
    println!("  📦 Total ROM size: {} bytes", total_size);

    // NEW: failure table
    if !failures.is_empty() {
        println!("\n🔎 Failure Summary");
        println!("{:<4} {:<60} {}", "#", "File", "Reason");
        println!("{}", "-".repeat(100));
        for (i, f) in failures.iter().enumerate() {
            println!(
                "{:<4} {:<60} {}",
                i + 1,
                truncate(&f.file, 60),
                truncate(&f.reason, 30),
            );
            if f.lines.is_empty() {
                println!("      -");
            } else {
                for line in &f.lines {
                    println!("      {}", line);
                }
            }
        }
    }

    if failed > 0 {
        println!("\n💡 Some files may use features not yet implemented in our assembler.");
    }

    Ok(())
}

// fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
//     if let Ok(rd) = fs::read_dir(dir) {
//         for e in rd.flatten() {
//             let p = e.path();
//             if p.is_dir() {
//                 collect(&p, out);
//             } else if p.extension().map(|x| x == "tal").unwrap_or(false) {
//                 out.push(p);
//             }
//         }
//     }
// }

// NEW helpers
fn first_line(s: &str) -> &str {
    s.lines().next().unwrap_or(s)
}
fn last_n_nonempty_lines(s: &str, n: usize) -> Vec<String> {
    let mut lines: Vec<String> = s
        .lines()
        .map(|l| l.trim_end().to_string())
        .filter(|l| !l.is_empty())
        .collect();
    if lines.len() > n {
        lines = lines.split_off(lines.len() - n);
    }
    lines
}
fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else if max > 3 {
        format!("{}...", &s[..max - 3])
    } else {
        s[..max].to_string()
    }
}
