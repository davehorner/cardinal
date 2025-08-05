use std::process::Command;
use uxn_tal::{Assembler, AssemblerError};

fn main() -> Result<(), AssemblerError> {
    let tal_source = r#"
        ( Simple working hello world )
        #48 #18 DEO  ( H )
        #65 #18 DEO  ( e )
        #6c #18 DEO  ( l )
        #6c #18 DEO  ( l )
        #6f #18 DEO  ( o )
        #20 #18 DEO  ( space )
        #57 #18 DEO  ( W )
        #6f #18 DEO  ( o )
        #72 #18 DEO  ( r )
        #6c #18 DEO  ( l )
        #64 #18 DEO  ( d )
        #21 #18 DEO  ( ! )
        #0a #18 DEO  ( newline )
        BRK
    "#;

    let mut assembler = Assembler::new();
    match assembler.assemble(tal_source, Some("working_hello.rs".to_owned())) {
        Ok(rom) => {
            println!("Successfully assembled {} bytes", rom.len());

            // Save to file
            std::fs::write("working_hello.rom", &rom)?;
            println!("ROM saved to working_hello.rom");
        }
        Err(e) => {
            eprintln!("Assembly failed: {}", e);
        }
    }
    // Write the TAL source to a temporary file
    let temp_path = "working_hello.tal";
    std::fs::write(&temp_path, tal_source)?;
    println!("TAL source written to {:?}", temp_path);

    // Run WSL uxnasm on the TAL source file
    let output_path = "working_hello_wsl.rom";
    let status = Command::new("wsl")
        .arg("uxnasm")
        .arg(&temp_path)
        .arg(&output_path)
        .status()?;

    if status.success() {
        println!("WSL uxnasm succeeded, output at {:?}", output_path);
    } else {
        eprintln!("WSL uxnasm failed with status: {}", status);
    }

    // Compare the two ROM files byte-by-byte
    let rust_rom = std::fs::read("working_hello.rom")?;
    let wsl_rom = std::fs::read(&output_path)?;

    if rust_rom == wsl_rom {
        println!("ROM outputs are identical.");
    } else {
        println!("ROM outputs differ:");
        let min_len = rust_rom.len().min(wsl_rom.len());
        for i in 0..min_len {
            if rust_rom[i] == wsl_rom[i] {
                println!("Byte {}: equal ({:02x})", i, rust_rom[i]);
            } else {
                println!(
                    "Byte {}: Rust = {:02x}, WSL = {:02x}",
                    i, rust_rom[i], wsl_rom[i]
                );
            }
        }
        let min_len = rust_rom.len().min(wsl_rom.len());
        for i in 0..min_len {
            if rust_rom[i] != wsl_rom[i] {
                println!(
                    "Byte {}: Rust = {:02x}, WSL = {:02x}",
                    i, rust_rom[i], wsl_rom[i]
                );
            }
        }
        if rust_rom.len() != wsl_rom.len() {
            println!(
                "ROM sizes differ: Rust = {}, WSL = {}",
                rust_rom.len(),
                wsl_rom.len()
            );
            if rust_rom.len() > wsl_rom.len() {
                println!("Extra bytes in Rust ROM:");
                for i in wsl_rom.len()..rust_rom.len() {
                    println!("Byte {}: {:02x}", i, rust_rom[i]);
                }
            } else {
                println!("Extra bytes in WSL ROM:");
                for i in rust_rom.len()..wsl_rom.len() {
                    println!("Byte {}: {:02x}", i, wsl_rom[i]);
                }
            }
        }
    }

    // Run both ROMs using uxncli via WSL and compare their outputs
    let run_rom = |rom_path: &str| -> Result<String, std::io::Error> {
        let output = Command::new("wsl").arg("uxncli").arg(rom_path).output()?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    };

    let rust_output = run_rom("working_hello.rom")?;
    let wsl_output = run_rom(&output_path)?;

    println!("--- Rust ROM output ---\n{}", rust_output);
    println!("--- WSL ROM output ---\n{}", wsl_output);

    if rust_output == wsl_output {
        println!("ROM runtime outputs are identical.");
    } else {
        println!("ROM runtime outputs differ.");
    }
    Ok(())
}
