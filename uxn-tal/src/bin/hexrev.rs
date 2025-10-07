
use std::path::Path;
use uxn_tal::hexrev::HexRev;

// fn main() -> std::io::Result<()> {
//     // hex -> bin (permissive)
//     HexRev::hex_to_bin_paths(Path::new("etc/drifblim.rom.txt"), Path::new("bin/drifblim-seed.rom"))?;

//     // hex -> bin (strict)
//     // HexRev::hex_to_bin_paths_strict(Path::new("hex.txt"), Path::new("out.bin"))?;

//     // bin -> hex (uppercase, 60 bytes/line)
//     HexRev::bin_to_hex_paths(Path::new("bin/drifblim-seed.rom"), Path::new("etc/new.hex"), true, Some(60))?;

//     Ok(())
// }

fn main() -> std::io::Result<()> {
    // Call the `main()` function defined inside the hexrev module
    uxn_tal::hexrev::main()
}