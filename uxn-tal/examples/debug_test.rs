use uxn_tal::{Assembler, AssemblerError};

fn main() -> Result<(), AssemblerError> {
    // Test 1: Character literals
    println!("Testing character literals...");
    let source1 = r#"
        |0100
        'A 'B 'C
    "#;

    let mut assembler = Assembler::new();
    match assembler.assemble(source1, Some("debug_test.rs:char_lit".to_owned()))
    {
        Ok(rom) => {
            println!("Character test OK, ROM length: {}", rom.len());
            println!(
                "Bytes: {:02x} {:02x} {:02x}",
                rom[0x100], rom[0x101], rom[0x102]
            );
        }
        Err(e) => {
            println!("Character test failed: {}", e);
        }
    }

    // Test 2: Simple label reference
    println!("\nTesting simple label reference...");
    let source2 = r#"
        |0100
        ;data
        @data #42
    "#;

    let mut assembler2 = Assembler::new();
    match assembler2
        .assemble(source2, Some("debug_test.rs:simple_label".to_owned()))
    {
        Ok(rom) => {
            println!("Simple label test OK, ROM length: {}", rom.len());
            println!("First 6 bytes:");
            for (i, byte) in rom.iter().take(6).enumerate() {
                println!("  [{:04x}] = {:02x}", 0x100 + i, byte);
            }
            // Symbol table printing removed (symbols() method does not exist)
        }
        Err(e) => {
            println!("Simple label test failed: {}", e);
        }
    }

    // Test 3: Label reference with instruction
    println!("\nTesting label reference with instruction...");
    let source3 = r#"
        |0100 @start
        ;data LDA2
        BRK
        @data #1234
    "#;

    let mut assembler3 = Assembler::new();
    match assembler3
        .assemble(source3, Some("debug_test.rs:label_instr".to_owned()))
    {
        Ok(rom) => {
            println!(
                "Label with instruction test OK, ROM length: {}",
                rom.len()
            );
            println!("First 8 bytes:");
            for (i, byte) in rom.iter().take(8).enumerate() {
                println!("  [{:04x}] = {:02x}", 0x100 + i, byte);
            }
            // Symbol table printing removed (symbols() method does not exist)
        }
        Err(e) => {
            println!("Label with instruction test failed: {}", e);
        }
    }

    Ok(())
}
