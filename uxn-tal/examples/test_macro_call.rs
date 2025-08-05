use uxn_tal::Assembler;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing macro call functionality...");

    let content = std::fs::read_to_string("../tal/test_macro_call.tal")?;
    println!("Source content:\n{}", content);

    // Assemble
    let mut assembler = Assembler::new();
    let rom = assembler.assemble(&content, Some("test_macro_call.rs".to_owned()))?;

    println!("Assembly successful! ROM size: {} bytes", rom.len());

    Ok(())
}
