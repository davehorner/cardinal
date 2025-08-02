use uxn_tal::{Assembler, AssemblerError};

fn main() -> Result<(), AssemblerError> {
    // Try to create a ROM that will definitely produce some kind of output
    // if UXN is working at all

    // Test all possible console-related devices
    let all_devices_test = r#"
        |0100
        
        ( Try writing 'A' to various device ports )
        #41 #10 DEO  ( device 0x10 )
        #42 #11 DEO  ( device 0x11 )
        #43 #12 DEO  ( device 0x12 )
        #44 #18 DEO  ( device 0x18 )
        #45 #19 DEO  ( device 0x19 )
        
        ( Force system halt with different pattern )
        #ff #00 DEO  ( try to write to system device )
        
        BRK
    "#;

    let mut assembler = Assembler::new();
    let rom = assembler.assemble(all_devices_test, Default::default())?;
    std::fs::write("all_devices_test.rom", &rom)?;

    println!("Created all_devices_test.rom");
    println!("This tries multiple device ports: 0x10, 0x11, 0x12, 0x18, 0x19");

    // Also create a version that exits immediately to test basic execution
    let immediate_exit = r#"
        |0100
        BRK
    "#;

    let mut assembler2 = Assembler::new();
    let rom2 = assembler2.assemble(immediate_exit, Default::default())?;
    std::fs::write("immediate_exit.rom", &rom2)?;

    println!("Created immediate_exit.rom - should exit immediately");

    // Create a test that tries to crash/error to see if we get any response
    let error_test = r#"
        |0100
        #ff DEO  ( try invalid instruction - missing device )
        BRK
    "#;

    let mut assembler3 = Assembler::new();
    let rom3 = assembler3.assemble(error_test, Default::default())?;
    std::fs::write("error_test.rom", &rom3)?;

    println!("Created error_test.rom - should cause an error");

    Ok(())
}
