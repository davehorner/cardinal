use std::path::Path;
use std::fs;
use uxn_tal::assemble_file_with_symbols;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let workspace_root = Path::new("..").canonicalize()?;
    let sample_path = workspace_root.join("sample.smal");
    let sample = r#"; simple SMAL program
.const hello "Hello, world!"
.data 0x00 0x01 0x02
"#;
    fs::write(&sample_path, sample)?;
    println!("Wrote sample.smal to {}", sample_path.display());

    let (rom_path, sym_path, size) = uxn_tal::assemble_file_with_symbols(&sample_path)?;
    let rom = std::fs::read(&rom_path)?;
    println!("Wrote ROM to {} ({} bytes)", rom_path.display(), rom.len());

    let sym = std::fs::read(&sym_path)?;
    println!("Wrote SYM to {} ({} bytes)", sym_path.display(), sym.len());

    Ok(())
}
