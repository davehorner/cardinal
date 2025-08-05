use std::fs;
use std::path::Path;
use uxn_tal::{Assembler, AssemblerError};

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

    println!("\n📊 Summary:");
    println!("===========");
    println!("Total files: {}", tal_files.len());
    println!(
        "Successful: {} ({:.1}%)",
        successful,
        (successful as f64 / tal_files.len() as f64) * 100.0
    );
    println!(
        "Failed: {} ({:.1}%)",
        failed,
        (failed as f64 / tal_files.len() as f64) * 100.0
    );
    println!("Total ROM size: {} bytes", total_size);

    if failed > 0 {
        println!("\n⚠️  Failed files may need additional TAL features or contain syntax errors.");
    } else {
        println!("\n🎉 All TAL files assembled successfully!");
    }

    Ok(())
}

fn find_tal_files(dir: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut tal_files = Vec::new();
    find_tal_files_recursive(Path::new(dir), &mut tal_files)?;
    tal_files.sort();
    Ok(tal_files)
}

fn find_tal_files_recursive(
    dir: &Path,
    tal_files: &mut Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                // Skip target directories and hidden directories
                if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                    if dir_name.starts_with('.') || dir_name == "target" {
                        continue;
                    }
                }
                find_tal_files_recursive(&path, tal_files)?;
            } else if let Some(extension) = path.extension() {
                if extension == "tal" {
                    if let Some(path_str) = path.to_str() {
                        tal_files.push(path_str.to_string());
                    }
                }
            }
        }
    }
    Ok(())
}

fn assemble_file(file_path: &str) -> Result<usize, String> {
    let source =
        fs::read_to_string(file_path).map_err(|e| format!("Failed to read file: {}", e))?;

    let mut assembler = Assembler::new();
    match assembler.assemble(&source, Some(file_path.to_string())) {
        Ok(rom) => {
            // Save the ROM file
            let rom_path = file_path.replace(".tal", ".rom");
            if let Err(e) = fs::write(&rom_path, &rom) {
                return Err(format!("Failed to write ROM: {}", e));
            }
            Ok(rom.len())
        }
        Err(e) => {
            // Extract line information from the error if available
            let error_msg = match &e {
                AssemblerError::SyntaxError {
                    line,
                    message,
                    path,
                    position,
                    source_line,
                } => {
                    format!(
                        "{}:{}:{}: {}\n    {}",
                        path, line, position, message, source_line
                    )
                }
                _ => {
                    format!("{}: {}", file_path, e)
                }
            };
            Err(error_msg)
        }
    }
}
