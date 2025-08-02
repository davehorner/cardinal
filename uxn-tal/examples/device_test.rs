use uxn_tal::{Assembler, AssemblerError};

fn main() -> Result<(), AssemblerError> {
    // Create a ROM that exactly matches working UXN console programs
    // Based on the uxnemu.c code, console device is at 0x10-0x1f
    // Port 0x18 (0x10 + 0x08) is for console output

    let proper_console = r#"
        |0100
        
        ( Console test - exactly like working UXN programs )
        
        ( Try console device 0x10 + offset )
        #41 .Console/write DEO  ( This might not work due to syntax )
        BRK
    "#;

    // Since our assembler doesn't support device name syntax yet,
    // let's use the raw device numbers from uxnemu.c
    let device_test = r#"
        |0100
        
        ( Test console output using device 0x10 base )
        #48 #08 #10 ADD DEO  ( 'H' to device 0x18 )
        #69 #08 #10 ADD DEO  ( 'i' to device 0x18 )
        #0a #08 #10 ADD DEO  ( newline to device 0x18 )
        BRK
    "#;

    // Even simpler - direct device addressing
    let direct_test = r#"
        |0100
        #48 #18 DEO  ( 'H' directly to device 0x18 )
        #0a #18 DEO  ( newline )
        BRK
    "#;

    let mut assembler = Assembler::new();
    let rom = assembler.assemble(direct_test, None)?;
    std::fs::write("direct_test.rom", &rom)?;

    println!("Created direct_test.rom - simplest possible console output");

    // Let's also try outputting to device 0x19 (console error)
    let stderr_test = r#"
        |0100
        #45 #19 DEO  ( 'E' to stderr )
        BRK
    "#;

    let mut assembler2 = Assembler::new();
    let rom2 = assembler2.assemble(stderr_test, None)?;
    std::fs::write("stderr_test.rom", &rom2)?;

    println!("Created stderr_test.rom - tries stderr output");

    Ok(())
}
