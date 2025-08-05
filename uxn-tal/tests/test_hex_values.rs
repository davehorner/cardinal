use uxn_tal::Assembler;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing hex byte sequences...");

    let content = std::fs::read_to_string("../tal/test_hex_values.tal")?;
    println!("Source content:\n{}", content);

    // Assemble
    let mut assembler = Assembler::new();
    let rom = assembler.assemble(&content, Some("test_hex_values.rs".to_owned()))?;

    println!("Assembly successful! ROM size: {} bytes", rom.len());

    // Print the hex bytes to verify they're correct
    println!("ROM contents: {:02x?}", rom);

    Ok(())
}
