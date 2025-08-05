use std::env;
use std::fs;
use std::path::Path;
use uxn_tal::{Assembler, AssemblerError};

fn main() -> Result<(), AssemblerError> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <input.tal>", args[0]);
        std::process::exit(1);
    }

    let input_file = &args[1];
    let source = fs::read_to_string(input_file).map_err(|e| AssemblerError::SyntaxError {
        line: 0,
        message: format!("Failed to read file {}: {}", input_file, e),
        path: input_file.clone(),
        position: 0,
        source_line: String::new(),
    })?;

    println!("Source file contents:");
    for (i, line) in source.lines().enumerate() {
        println!("{:2}: {}", i + 1, line);
    }
    println!("--- End of file ---");

    let mut assembler = Assembler::new();
    match assembler.assemble(&source, Some(input_file.clone())) {
        Ok(rom) => {
            println!("Assembly successful!");
            println!("ROM size: {} bytes", rom.len());
            let output_file = input_file.as_str().replace(".tal", ".rom");

            // Ensure the output directory exists
            let output_path = Path::new(&output_file);
            if let Some(parent) = output_path.parent() {
                std::fs::create_dir_all(parent).ok();
            }
            println!("Writing ROM to {}", output_file);
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

            // Write symbols to a .sym file
            let sym_output_file = input_file.as_str().replace(".tal", ".sym");

            // Ensure the output directory exists
            let sym_output_path = Path::new(&sym_output_file);
            if let Some(parent) = sym_output_path.parent() {
                std::fs::create_dir_all(parent).ok();
            }

            let symbols = assembler.generate_symbol_file_binary();
            fs::write(&sym_output_file, &symbols).map_err(|e| AssemblerError::SyntaxError {
                line: 0,
                message: format!("Failed to write symbol file {}: {}", sym_output_file, e),
                path: sym_output_file.clone(),
                position: 0,
                source_line: String::new(),
            })?;
            println!("Created {} ({} symbols)", sym_output_file, symbols.len());

            let sym_txt_file = input_file.as_str().replace(".tal", ".sym.txt");

            // Ensure the output directory exists
            let sym_txt_output_path = Path::new(&sym_txt_file);
            if let Some(parent) = sym_txt_output_path.parent() {
                std::fs::create_dir_all(parent).ok();
            }

            let sym_txt = assembler.generate_symbol_file_txt();
            fs::write(&sym_txt_file, &sym_txt).map_err(|e| AssemblerError::SyntaxError {
                line: 0,
                message: format!("Failed to write symbol text file {}: {}", sym_txt_file, e),
                path: sym_txt_file.clone(),
                position: 0,
                source_line: String::new(),
            })?;
            println!("Created {} ({} bytes)", sym_txt_file, sym_txt.len());

            // Try debug comparison, but don't fail if it doesn't work
            println!("\nAttempting debug comparison with WSL uxnasm...");
            match uxn_tal::DebugAssembler::default().assemble_and_compare(&input_file, &source) {
                Ok(debug_result) => {
                    println!("Debug assembly result:");
                    println!("Rust ROM path: {}", debug_result.rust_rom_path);
                    println!("WSL ROM path: {}", debug_result.wsl_rom_path);

                    println!("First line difference in disassembly:");
                    debug_result
                        .first_line_difference_dis
                        .as_ref()
                        .map(|(line, rust, wsl)| {
                            println!("Line {}: Rust = {}, WSL = {}", line, rust, wsl);
                        });
                }
                Err(e) => {
                    println!("Debug comparison failed (this is OK): {}", e);
                    println!("The main assembly was successful - this error is just from the WSL comparison tool.");
                }
            }
        }
        Err(e) => {
            println!("Assembly failed: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}
