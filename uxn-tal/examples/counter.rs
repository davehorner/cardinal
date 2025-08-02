// Example: Counter program that demonstrates various TAL features

use uxn_tal::{Assembler, AssemblerError};

fn main() -> Result<(), AssemblerError> {
    let tal_source = r#"
        ( Simple Counter Program )
        
        |0100 @reset
            #30 #18 DEO ( print '0' )
            #31 #18 DEO ( print '1' )
            #32 #18 DEO ( print '2' )
            #0a #18 DEO ( newline )
        BRK
    "#;

    let mut assembler = Assembler::new();
    match assembler.assemble(tal_source, Some("counter.rs".to_owned())) {
        Ok(rom) => {
            println!(
                "Successfully assembled counter program: {} bytes",
                rom.len()
            );
            std::fs::write("counter.rom", &rom)?;
            println!("ROM saved to counter.rom");
            Ok(())
        }
        Err(e) => {
            eprintln!("Assembly failed: {}", e);
            Err(e)
        }
    }
}
