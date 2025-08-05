use uxn_tal::Assembler;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing hyphen identifiers and raw address refs...");

    let content = std::fs::read_to_string("../tal/test_hyphen.tal")?;
    println!("Source content:\n{}", content);

    // Assemble
    let mut assembler = Assembler::new();
    let rom = assembler.assemble(&content, Some("test_hyphen.rs".to_owned()))?;

    println!("Assembly successful! ROM size: {} bytes", rom.len());

    Ok(())
}
