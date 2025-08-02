use uxn_tal::{Assembler, AssemblerError};

fn main() -> Result<(), AssemblerError> {
    // Try different approaches to console output
    println!("Generating test ROMs...");

    // Test 1: Original approach (console port 0x18)
    let test1 = r#"
        |0100
        'H #18 DEO
        BRK
    "#;

    // Test 2: Try console port 0x10 (sometimes used)
    let test2 = r#"
        |0100  
        'H #10 DEO
        BRK
    "#;

    // Test 3: Try different device addressing
    let test3 = r#"
        |0100
        'H #00 #18 DEO2
        BRK  
    "#;

    let tests = vec![
        ("test1_port18.rom", test1),
        ("test2_port10.rom", test2),
        ("test3_deo2.rom", test3),
    ];

    for (filename, source) in tests {
        let mut assembler = Assembler::new();
        match assembler.assemble(source, Some(filename.to_owned())) {
            Ok(rom) => {
                std::fs::write(filename, &rom)?;
                println!("Generated {}", filename);
            }
            Err(e) => {
                println!("Failed to generate {}: {}", filename, e);
            }
        }
    }

    println!("\nTry running these different ROM files with your emulator to see which works.");
    Ok(())
}
