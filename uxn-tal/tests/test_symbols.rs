use uxn_tal::{assemble_file_with_symbols, Assembler};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Test 1: Direct symbol generation
    println!("=== Test 1: Symbol Generation ===");

    let source = r#"
|0100 @main
    #48 #65 #6c #6c #6f
    &loop
        #20 #57 #6f #72 #6c #64
        #21 #0a
        ,loop JMP
    @data
        $10
    @end
        BRK
"#;

    let mut assembler = Assembler::new();
    let rom = assembler.assemble(source, Some("test_symbols.rs".to_owned()))?;
    let symbols = assembler.generate_symbol_file();

    println!("Generated ROM: {} bytes", rom.len());
    println!("Symbol file contents:");
    println!("{:?}", symbols);

    // Test 2: File-based symbol generation
    println!("\n=== Test 2: File-based Assembly ===");

    std::fs::write("../tal/test_symbols.tal", source)?;

    let (rom_path, sym_path, size) = assemble_file_with_symbols("../tal/test_symbols.tal")?;
    println!("Generated ROM: {} ({} bytes)", rom_path.display(), size);
    println!("Generated symbols: {}", sym_path.display());

    let sym_content = std::fs::read_to_string(&sym_path)?;
    println!("Symbol file contents:");
    println!("{}", sym_content);

    // Clean up
    std::fs::remove_file("../tal/test_symbols.tal")?;
    std::fs::remove_file(&rom_path)?;
    std::fs::remove_file(&sym_path)?;

    println!("\n✅ All tests passed!");

    Ok(())
}
