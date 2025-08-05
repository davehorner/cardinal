use std::env;
use std::fs;
use uxn_tal::{Assembler, AssemblerError};

/// Extract line number from error message
fn extract_line_number(error_message: &str) -> Option<usize> {
    // Look for "line X:" pattern in error message
    if let Some(start) = error_message.find("line ") {
        let after_line = &error_message[start + 5..];
        if let Some(end) = after_line.find(':') {
            after_line[..end].parse().ok()
        } else {
            None
        }
    } else {
        None
    }
}

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
        path: input_file.to_owned(),
        position: 0,
        source_line: String::new(),
    })?;

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
            let error_string = e.to_string();
            println!("Assembly failed: {}", error_string);

            // Extract line number from error message and show the line content
            if let Some(line_num) = extract_line_number(&error_string) {
                let lines: Vec<&str> = source.lines().collect();
                if line_num > 0 && line_num <= lines.len() {
                    println!("  --> {}", lines[line_num - 1]);
                } else {
                    println!("  --> (line {} not found)", line_num);
                }
            } else {
                println!("  (could not extract line number from: '{}')", error_string);
            }

            std::process::exit(1);
        }
    }

    Ok(())
}
