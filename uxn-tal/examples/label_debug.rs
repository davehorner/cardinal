use uxn_tal::{Assembler, AssemblerError};

fn main() -> Result<(), AssemblerError> {
    let source = r#"
        |0100 @start
        ;data LDA2
        BRK
        @data #1234
    "#;

    let mut assembler = Assembler::new();
    let rom = assembler.assemble(source, Some("label_debug.rs".to_owned()))?;

    println!("ROM bytes ({} total):", rom.len());
    for (i, &byte) in rom.iter().enumerate() {
        print!("{:02x} ", byte);
        if (i + 1) % 8 == 0 {
            println!();
        }
    }
    if rom.len() % 8 != 0 {
        println!();
    }

    println!("\nDecoded:");
    println!("rom[0] = 0x{:02x} (should be LIT2 = 0xa0)", rom[0]);
    println!("rom[1] = 0x{:02x} (address high byte)", rom[1]);
    println!("rom[2] = 0x{:02x} (address low byte)", rom[2]);

    let addr = ((rom[1] as u16) << 8) | (rom[2] as u16);
    println!("Address = 0x{:04x}", addr);

    Ok(())
}
