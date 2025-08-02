use uxn_tal::rom::Rom;

fn main() {
    let mut rom = Rom::new();

    // Pad to 0x100
    rom.pad_to(0x100).unwrap();

    // Write some data
    rom.write_byte(0x41).unwrap(); // 'A'
    rom.write_byte(0x42).unwrap(); // 'B'

    println!("ROM size: {}", rom.len());
    println!("ROM position: 0x{:04x}", rom.position());

    let data = rom.data();
    println!("Data length: {}", data.len());

    if data.len() > 0x100 {
        println!("Data at 0x100: 0x{:02x}", data[0x100]);
        println!("Data at 0x101: 0x{:02x}", data[0x101]);
    }

    // Now test write_short_at
    rom.write_short_at(0x100, 0x1234).unwrap();

    let data2 = rom.data();
    if data2.len() > 0x100 {
        println!("After write_short_at:");
        println!("Data at 0x100: 0x{:02x}", data2[0x100]);
        println!("Data at 0x101: 0x{:02x}", data2[0x101]);
    }
}
