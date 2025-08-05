use uxn_tal::opcode_table::{decode_opcode, encode_opcode, get_opcode_name, verify_opcode_table};
use uxn_tal::{Assembler, AssemblerError};

fn main() -> Result<(), AssemblerError> {
    println!("=== UXN TAL Assembler - Opcode Table Verification ===\n");

    // Verify our opcode table matches the official specification
    match verify_opcode_table() {
        Ok(()) => println!("✅ Opcode table verification passed!\n"),
        Err(e) => {
            eprintln!("❌ Opcode table verification failed: {}", e);
            return Ok(());
        }
    }

    // Display the complete opcode table in the same format as the reference
    println!("UXN Opcode Reference Table (https://wiki.xxiivv.com/site/uxntal_reference.html):\n");

    // Header
    print!("    ");
    for col in 0..16 {
        print!("{:02x}  ", col);
    }
    println!();

    // Table rows
    for row in 0..16 {
        print!("{:02x}  ", row);
        for col in 0..16 {
            let opcode = (row << 4) | col;
            let name = get_opcode_name(opcode);
            print!("{:<4}", name);
        }
        println!();
    }

    println!("\n=== Testing Opcode Encoding/Decoding ===\n");

    // Test some key opcodes
    let test_cases = [
        ("BRK", 0x00, false, false, false),
        ("LIT", 0x00, false, false, true),  // 0x80
        ("LIT2", 0x00, true, false, true),  // 0xA0
        ("DEO", 0x17, false, false, false), // 0x17
        ("ADD2k", 0x18, true, false, true), // 0xB8
        ("STH2kr", 0x0F, true, true, true), // 0xEF
    ];

    for (name, base, short, ret, keep) in test_cases {
        let encoded = encode_opcode(base, short, ret, keep);
        let (decoded_base, decoded_short, decoded_ret, decoded_keep) = decode_opcode(encoded);
        let opcode_name = get_opcode_name(encoded);

        println!("Instruction: {}", name);
        println!("  Encoded: 0x{:02x}", encoded);
        println!("  Name: {}", opcode_name);
        println!(
            "  Decoded: base=0x{:02x}, short={}, return={}, keep={}",
            decoded_base, decoded_short, decoded_ret, decoded_keep
        );
        println!("  ✓ Match: {}", opcode_name == name);
        println!();
    }

    // Test assembling with various opcodes
    println!("=== Testing Assembly with Various Opcodes ===\n");

    let test_programs = [
        ("Simple LIT + DEO", "#41 #18 DEO BRK"),
        ("Stack operations", "#12 #34 SWP ADD BRK"),
        ("Short mode", "#1234 #5678 ADD2 BRK"),
        ("Keep mode", "#12 DUPk ADD BRK"),
        ("Return mode", "#12 STH ADDr BRK"),
        ("Complex", "#1234 DUP2k SWP2 ADD2r STH2kr BRK"),
    ];

    let mut assembler = Assembler::new();

    for (description, source) in test_programs {
        match assembler.assemble(source, None) {
            Ok(rom) => {
                println!("{}: ✅ ({} bytes)", description, rom.len());
                print!("  Bytes: ");
                for (i, &byte) in rom.iter().enumerate() {
                    if i > 0 {
                        print!(" ");
                    }
                    print!("{:02x}", byte);
                }
                println!();

                // Decode the instructions
                print!("  Instructions: ");
                let mut i = 0;
                while i < rom.len() {
                    let opcode = rom[i];
                    let name = get_opcode_name(opcode);
                    print!("{} ", name);

                    // Skip operands for LIT/LIT2
                    if name == "LIT" {
                        i += 2; // Skip the literal byte
                    } else if name == "LIT2" {
                        i += 3; // Skip the literal short
                    } else {
                        i += 1;
                    }
                }
                println!();
            }
            Err(e) => {
                println!("{}: ❌ {}", description, e);
            }
        }
        println!();
    }

    Ok(())
}
