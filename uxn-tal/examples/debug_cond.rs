use uxn_tal::{Assembler, AssemblerError};

fn main() -> Result<(), AssemblerError> {
    let source = "|0100\n#01 ? { #02 }\nBRK".to_string();

    println!("Testing source:");
    for (line_num, line) in source.lines().enumerate() {
        println!("Line {}: '{}'", line_num + 1, line);
        for (char_num, ch) in line.chars().enumerate() {
            println!("  Char {}: '{}' ({})", char_num + 1, ch, ch as u32);
        }
    }

    let mut assembler = Assembler::new();
    // Provide default options as the second argument
    match assembler.assemble(&source, None) {
        Ok(rom) => {
            println!("Assembly successful! ROM size: {} bytes", rom.len());
        }
        Err(e) => {
            println!("Assembly failed: {}", e);
            return Err(e);
        }
    }

    Ok(())
}
