use uxn_tal::{Assembler, AssemblerError};

fn main() -> Result<(), AssemblerError> {
    // Try starting code at different addresses to see if 0x0100 is the issue

    // Test 1: Start at 0x0000 (beginning of ROM)
    let start_at_zero = r#"
        |0000
        #41 #18 DEO  ( 'A' to console )
        BRK
    "#;

    // Test 2: Start at 0x0200 (different address)
    let start_at_200 = r#"
        |0200
        #42 #18 DEO  ( 'B' to console )
        BRK
    "#;

    // Test 3: No explicit padding, let it start naturally
    let no_padding = r#"
        #43 #18 DEO  ( 'C' to console )
        BRK
    "#;

    let tests = vec![
        ("start_zero.rom", start_at_zero),
        ("start_200.rom", start_at_200),
        ("no_pad.rom", no_padding),
    ];

    for (filename, source) in tests {
        let mut assembler = Assembler::new();
        match assembler.assemble(source, Some(filename.to_owned())) {
            Ok(rom) => {
                std::fs::write(filename, &rom)?;
                println!("Created {} ({} bytes)", filename, rom.len());

                // Show where the actual code is
                for (i, &byte) in rom.iter().enumerate() {
                    if byte != 0 {
                        println!(
                            "  First non-zero byte at 0x{:04x}: 0x{:02x}",
                            i, byte
                        );
                        break;
                    }
                }
            }
            Err(e) => {
                println!("Failed to create {}: {}", filename, e);
            }
        }
    }

    println!("\nTry these ROMs with uxncli to see if starting address matters");
    Ok(())
}
