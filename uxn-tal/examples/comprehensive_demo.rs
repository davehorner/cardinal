use std::{fs, io::BufRead};
use uxn_tal::{assemble_directory, assemble_file_with_symbols, Assembler};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔨 UXN TAL Assembler Demo");
    println!("========================\n");

    // Create some demo TAL files to show our assembler in action
    create_demo_files()?;

    // Demo 1: Single file assembly with symbols
    println!("📝 Demo 1: Single File Assembly");
    let (rom_path, sym_path, size) =
        assemble_file_with_symbols("demo_hello.tal")?;
    println!("✅ Assembled {} bytes to {}", size, rom_path.display());
    println!("📍 Generated symbols to {}", sym_path.display());

    let symbols = fs::read_to_string(&sym_path)?;
    println!("Symbols:");
    for line in symbols.lines() {
        println!("  {:?}", line);
    }
    println!();

    // Demo 2: Batch assembly
    println!("📂 Demo 2: Batch Assembly");
    let results = assemble_directory(".", true)?;

    for (tal_path, rom_path, sym_path, size) in &results {
        if tal_path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("demo_")
        {
            println!(
                "✅ {} -> {} ({} bytes)",
                tal_path.file_name().unwrap().to_string_lossy(),
                rom_path.file_name().unwrap().to_string_lossy(),
                size
            );
            if let Some(sym_path) = sym_path {
                println!(
                    "  + {}",
                    sym_path.file_name().unwrap().to_string_lossy()
                );
            }
        }
    }

    // Demo 3: Show assembler flexibility
    println!("\n🔧 Demo 3: Manual Assembly");
    let tal_code = r#"
( Counter example with $ padding )
|0100 @main
    #00 
    &loop
        INC DUP 
        #0a EQU ,end JCN
        ,loop JMP
    &end BRK
    
@data $1
"#;

    let mut assembler = Assembler::new();
    let rom = assembler.assemble(tal_code, None)?;
    println!("Generated {} bytes from inline TAL code", rom.len());

    let symbols = assembler.generate_symbol_file();
    println!("Extracted symbols:");
    for line in symbols.lines() {
        println!("  {:?}", line);
    }

    // Show binary symbol format too
    let binary_symbols = assembler.generate_symbol_file_binary();
    println!("Binary symbol data: {} bytes", binary_symbols.len());
    // If you want to print lines, try converting to String (if valid UTF-8)
    if let Ok(symbols_str) = String::from_utf8(binary_symbols.clone()) {
        println!("Binary symbol lines:");
        for line in symbols_str.lines() {
            println!("  {}", line);
        }
    } else {
        println!("Binary symbol data is not valid UTF-8, cannot print lines.");
    }

    // Clean up demo files
    cleanup_demo_files()?;

    println!("\n🎉 Demo complete! The assembler supports:");
    println!("  ✅ All UXN opcodes with mode flags (2, r, k)");
    println!("  ✅ Hex literals (#12, #1234)");
    println!("  ✅ Character literals ('A')");
    println!("  ✅ Labels (@main) and sublabels (&loop)");
    println!("  ✅ Label references (;main, ,loop)");
    println!("  ✅ Padding directives (|0100) and skip bytes ($10)");
    println!("  ✅ Symbol file generation (text and binary formats)");
    println!("  ✅ Batch processing with ergonomic API");

    Ok(())
}

fn create_demo_files() -> Result<(), Box<dyn std::error::Error>> {
    let hello_tal = r#"
( Hello World Example )
|0100 @main
    #48 #65 #6c #6c #6f #20 #57 #6f #72 #6c #64
    BRK

@data $10
"#;

    let counter_tal = r#"
( Simple Counter with Skip )
|0100 @main
    #00 
    &loop
        INC DUP
        #05 EQU 
        ,done JCN
        ,loop JMP
    &done BRK

@value $1
@buffer $10
"#;

    let echo_tal = r#"
( Echo example with character literals )
|0100 @main
    'H 'e 'l 'l 'o
    BRK
"#;

    fs::write("demo_hello.tal", hello_tal)?;
    fs::write("demo_counter.tal", counter_tal)?;
    fs::write("demo_echo.tal", echo_tal)?;

    Ok(())
}

fn cleanup_demo_files() -> Result<(), Box<dyn std::error::Error>> {
    let patterns = ["demo_hello", "demo_counter", "demo_echo"];

    for pattern in &patterns {
        if let Ok(entries) = fs::read_dir(".") {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                if name_str.starts_with(pattern) {
                    let _ = fs::remove_file(entry.path());
                }
            }
        }
    }

    Ok(())
}
