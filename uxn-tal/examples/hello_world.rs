// Example: Simple "Hello World" TAL program assembly

use uxn_tal::{Assembler, AssemblerError};

fn main() -> Result<(), AssemblerError> {
    let tal_source = r#"
        ( Simple Hello World Program )
        
        |0100 @reset
            #48 #65 #6c #6c #6f #20 #57 #6f #72 #6c #64 #21 #0a
            #18 DEO
        BRK
    "#;

    let mut assembler = Assembler::new();
    match assembler.assemble(tal_source, Some("hello_world.rs".to_owned())) {
        Ok(rom) => {
            println!("Successfully assembled {} bytes", rom.len());

            // Save to file
            std::fs::write("hello.rom", &rom)?;
            println!("ROM saved to hello.rom");

            // Print first few bytes for verification
            println!("First 16 bytes:");
            for (i, byte) in rom.iter().take(16).enumerate() {
                if i % 8 == 0 {
                    print!("\n{:04x}: ", i);
                }
                print!("{:02x} ", byte);
            }
            println!();

            Ok(())
        }
        Err(e) => {
            eprintln!("Assembly failed: {}", e);
            Err(e)
        }
    }
}
