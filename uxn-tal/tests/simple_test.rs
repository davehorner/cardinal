use uxn_tal::{Assembler, AssemblerError};

fn main() -> Result<(), AssemblerError> {
    let source = r#"#41 #18 DEO BRK"#;

    let mut assembler = Assembler::new();
    match assembler.assemble(source, Some("simple_test.rs".to_owned())) {
        Ok(rom) => {
            std::fs::write("our_simple.rom", &rom)?;
            println!("Created our_simple.rom ({} bytes)", rom.len());

            // Show the bytes
            print!("Bytes: ");
            for (i, &byte) in rom.iter().enumerate() {
                if i > 0 {
                    print!(" ");
                }
                print!("{:02x}", byte);
            }
            println!();
        }
        Err(e) => {
            println!("Failed: {}", e);
        }
    }

    Ok(())
}
