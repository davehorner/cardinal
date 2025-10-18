use uxn_tal::{Assembler, AssemblerError};

fn main() -> Result<(), AssemblerError> {
    // Minimal test - just try to output 'A' using standard console device
    let minimal_source = r#"
        |0100
        #41 #10 DEO  ( Output 'A' to console device 0x10 )
        BRK
    "#;

    let mut assembler = Assembler::new();
    let rom = assembler.assemble(minimal_source, Some("(minimal)".to_owned()))?;
    std::fs::write("minimal.rom", &rom)?;

    println!("Created minimal.rom ({} bytes)", rom.len());
    println!("This ROM should output the letter 'A' and then halt.");
    println!();
    println!("ROM contents at 0x100:");
    println!("  0x100: 0x{:02x} (should be 0x41 = 'A')", rom[0]);
    println!(
        "  0x101: 0x{:02x} (should be 0x10 = console device)",
        rom[1]
    );
    println!(
        "  0x102: 0x{:02x} (should be 0x17 = DEO instruction)",
        rom[2]
    );
    println!(
        "  0x103: 0x{:02x} (should be 0x00 = BRK instruction)",
        rom[3]
    );
    println!();
    println!("Try: uxnemu minimal.rom");

    Ok(())
}
