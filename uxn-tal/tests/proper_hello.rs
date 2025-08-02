use uxn_tal::{Assembler, AssemblerError};

fn main() -> Result<(), AssemblerError> {
    // Create a proper UXN program with reset vector
    let tal_source = r#"
        |0100 ( -> )
        
        ( Hello World program )
        @reset
            ( Set up console output )
            'H #18 DEO
            'e #18 DEO  
            'l #18 DEO
            'l #18 DEO
            'o #18 DEO
            #20 #18 DEO  ( space )
            'W #18 DEO
            'o #18 DEO
            'r #18 DEO
            'l #18 DEO
            'd #18 DEO
            '! #18 DEO
            #0a #18 DEO  ( newline )
            
        BRK

        |FFE0 ( -> )
        ;&reset #0100 STA2  ( set reset vector )
        
        &reset ;reset JMP2
    "#;

    let mut assembler = Assembler::new();
    match assembler.assemble(tal_source, Default::default()) {
        Ok(rom) => {
            println!(
                "Successfully assembled proper UXN program: {} bytes",
                rom.len()
            );
            std::fs::write("proper_hello.rom", &rom)?;
            println!("ROM saved to proper_hello.rom");

            // Show the reset vector area
            if rom.len() >= 0xFFE2 {
                println!(
                    "Reset vector at 0xFFE0: {:02x} {:02x}",
                    rom[0xFFE0], rom[0xFFE1]
                );
            }

            Ok(())
        }
        Err(e) => {
            eprintln!("Assembly failed: {}", e);
            Err(e)
        }
    }
}
