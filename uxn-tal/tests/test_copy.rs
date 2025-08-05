use std::env;
use std::fs;
use uxn_tal::{Assembler, AssemblerError};

fn main() -> Result<(), AssemblerError> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <input.tal>", args[0]);
        std::process::exit(1);
    }

    let input_file = &args[1];
    let tal_path = format!("../tal/{}", input_file);
    let source = fs::read_to_string(&tal_path).map_err(|e| AssemblerError::SyntaxError {
        line: 0,
        message: format!("Failed to read file {}: {}", tal_path, e),
        path: tal_path.clone(),
        position: 0,
        source_line: String::new(),
    })?;

    println!("Source file contents:");
    for (i, line) in source.lines().enumerate() {
        println!("{:2}: {}", i + 1, line);
    }
    println!("--- End of file ---");

    let mut assembler = Assembler::new();
    match assembler.assemble(&source, Some(input_file.to_owned())) {
        Ok(rom) => {
            let output_file = input_file.replace(".tal", ".rom");
            fs::write(&output_file, &rom).map_err(|e| AssemblerError::SyntaxError {
                line: 0,
                message: format!("Failed to write ROM file {}: {}", output_file, e),
                path: output_file.clone(),
                position: 0,
                source_line: String::new(),
            })?;

            println!("Created {} ({} bytes)", output_file, rom.len());

            // Show the first few bytes
            print!("Bytes: ");
            for (i, &byte) in rom.iter().take(20).enumerate() {
                if i > 0 {
                    print!(" ");
                }
                print!("{:02x}", byte);
            }
            if rom.len() > 20 {
                print!(" ...");
            }
            println!();
        }
        Err(e) => {
            println!("Assembly failed: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}
