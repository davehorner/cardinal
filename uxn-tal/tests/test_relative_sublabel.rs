use uxn_tal::Assembler;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing relative sublabel references...");

    let content = std::fs::read_to_string("../tal/test_relative_sublabel.tal")?;
    println!("Source content:\n{}", content);

    // Assemble
    let mut assembler = Assembler::new();
    let rom = assembler.assemble(&content, Some("test_relative_sublabel.rs".to_owned()))?;

    println!("Assembly successful! ROM size: {} bytes", rom.len());

    Ok(())
}
