use uxn_tal::Assembler;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing blank.tal specifically...");

    let content = std::fs::read_to_string("../tal/blank.tal")?;

    // Assemble
    let mut assembler = Assembler::new();
    let rom = assembler.assemble(&content, Some("test_blank.rs".to_owned()))?;

    println!("Assembly successful! ROM size: {} bytes", rom.len());

    Ok(())
}
