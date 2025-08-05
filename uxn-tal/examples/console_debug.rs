use uxn_tal::{Assembler, AssemblerError};

fn main() -> Result<(), AssemblerError> {
    // Based on official UXN console examples
    // Console device is at 0x10-0x1f
    // 0x18 is console/write port
    let console_test = r#"
        |0100
        
        ( Test different console approaches )
        
        ( Approach 1: Direct console write )
        #41 #18 DEO  ( 'A' to console port 0x18 )
        
        ( Approach 2: Try stderr console )  
        #42 #19 DEO  ( 'B' to console port 0x19 )
        
        ( Approach 3: Standard output )
        #43 #18 DEO  ( 'C' to console port 0x18 )
        
        ( Add newline )
        #0a #18 DEO
        
        BRK
    "#;

    let mut assembler = Assembler::new();
    let rom = assembler.assemble(console_test, Some("console_debug.rs".to_owned()))?;
    std::fs::write("console_test.rom", &rom)?;

    println!("Created console_test.rom");
    println!("This should output 'ABC' followed by newline");

    // Also create a version that matches known working TAL programs
    let working_example = r#"
        |0100
        
        ( Hello World - standard console pattern )
        ;hello print-string
        BRK
        
        @print-string ( str* -- )
            &loop
                LDAk #18 DEO
                INC2
                LDAk ,&loop JCN
            POP2
            JMP2r
            
        @hello "Hello! 00
    "#;

    // This might fail due to unsupported syntax, but let's try a simpler version
    let simple_hello = r#"
        |0100
        #48 #18 DEO  ( H )
        #65 #18 DEO  ( e )  
        #6c #18 DEO  ( l )
        #6c #18 DEO  ( l )
        #6f #18 DEO  ( o )
        #21 #18 DEO  ( ! )
        #0a #18 DEO  ( newline )
        BRK
    "#;

    let mut assembler2 = Assembler::new();
    let rom2 = assembler2.assemble(
        simple_hello,
        Some("console_debug.rs:simple_hello".to_owned()),
    )?;
    std::fs::write("simple_hello.rom", &rom2)?;

    println!("Created simple_hello.rom");
    println!("This should output 'Hello!'");

    Ok(())
}
