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

    for file_path in &tal_files {
        print!("Assembling {}... ", file_path);

        match assemble_file(file_path) {
            Ok(size) => {
                println!("✅ Success ({} bytes)", size);
                successful += 1;
                total_size += size;
            }
            Err(e) => {
                println!("❌ Failed: {}", e);
                failed += 1;
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
                        println!("❌ IO error: {}", e);
                        failed += 1;
                    }
                    Err(e) => {
                        println!("❌ Assembly error: {}", e);
                        failed += 1;
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
                        println!("❌ IO error: {}", e);
                        failed += 1;
                    }
                    Err(e) => {
                        println!("❌ Assembly error: {}", e);
                        failed += 1;
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

    if failed > 0 {
        println!("\n💡 Some files may use features not yet implemented in our assembler.");
        println!("   This is normal for a TAL assembler implementation.");
    }

    Ok(())
}
