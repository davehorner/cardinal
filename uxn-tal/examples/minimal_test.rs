use uxn_tal::Assembler;

fn main() {
    // This should put 0x0102 (address of data) at position 0x0100
    let source = r#"
        |0100
        ;data
        @data #42
    "#;

    let mut assembler = Assembler::new();
    let rom = assembler
        .assemble(source, Some("minimal_test".to_owned()))
        .expect("Assembly failed");

    println!("ROM length: {}", rom.len());
    println!("Expected: data reference at 0x100, value #42 at 0x102");

    if rom.len() > 0x102 {
        println!("rom[0x100] = 0x{:02x} (should be 0x01)", rom[0x100]);
        println!("rom[0x101] = 0x{:02x} (should be 0x02)", rom[0x101]);
        println!("rom[0x102] = 0x{:02x} (should be 0x42)", rom[0x102]);

        let addr = ((rom[0x100] as u16) << 8) | (rom[0x101] as u16);
        println!("Address stored: 0x{:04x}", addr);
    }
}
