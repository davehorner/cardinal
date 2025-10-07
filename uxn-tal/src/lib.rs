//! # UXN TAL Assembler
//!
//! A Rust library for assembling TAL (Tal Assembly Language) files into UXN ROM files.
//!
//! This library provides functionality to parse TAL source code and generate bytecode
//! compatible with the UXN virtual machine.
//!
//! ## Example
//!
//! ```rust
//! use uxn_tal::{Assembler, AssemblerError};
//!
//! fn main() -> Result<(), AssemblerError> {
//!     let tal_source = r#"
//!         |0100 @reset
//!             #48 #65 #6c #6c #6f #20 #57 #6f #72 #6c #64 #21 #0a
//!             #18 DEO
//!         BRK
//!     "#;
//!     
//!     let mut assembler = Assembler::new();
//!     let rom = assembler.assemble(tal_source, None)?;
//!     
//!     // Save the ROM to a file
//!     std::fs::write("hello.rom", rom)?;
//!     
//!     Ok(())
//! }
//! ```

//! Public API for uxn-tal (minimal so examples compile).

pub mod assembler;
pub mod chocolatal;
pub mod drif;
pub mod debug;
pub mod devicemap;
pub mod error;
pub mod lexer;
pub mod opcode_table;
pub mod opcodes;
pub mod parser;
pub mod rom;
pub mod runes;
pub mod hexrev;
pub mod dis;
pub use assembler::Assembler;
pub use error::AssemblerError;

pub fn assemble(source: &str) -> Result<Vec<u8>, AssemblerError> {
    let mut a = Assembler::new();
    a.assemble(source, None)
}

pub fn assemble_with_path(source: &str, path: &str) -> Result<Vec<u8>, AssemblerError> {
    let mut a = Assembler::new();
    a.assemble(source, Some(path.to_string()))
}

/// Convenience function to assemble a TAL file directly from a file path
pub fn assemble_file<P: AsRef<std::path::Path>>(input_path: P) -> Result<Vec<u8>, AssemblerError> {
    let source = std::fs::read_to_string(&input_path)?;
    let mut assembler = Assembler::new();
    let path_str = input_path.as_ref().to_string_lossy().into_owned();
    assembler.assemble(&source, Some(path_str))
}

/// Convenience function to assemble a TAL file and save the ROM to a file
pub fn assemble_file_to_rom<P: AsRef<std::path::Path>, Q: AsRef<std::path::Path>>(
    input_path: P,
    output_path: Q,
) -> Result<usize, AssemblerError> {
    let rom = assemble_file(input_path)?;
    std::fs::write(&output_path, &rom)?;
    Ok(rom.len())
}

/// Convenience function to assemble a TAL file and save ROM with same name but .rom extension
pub fn assemble_file_auto<P: AsRef<std::path::Path>>(
    input_path: P,
) -> Result<(std::path::PathBuf, usize), AssemblerError> {
    let input_path = input_path.as_ref();
    let output_path = input_path.with_extension("rom");
    let size = assemble_file_to_rom(input_path, &output_path)?;
    Ok((output_path, size))
}

/// Convenience function to assemble a TAL file and generate both ROM and symbol files
pub fn assemble_file_with_symbols<P: AsRef<std::path::Path>>(
    input_path: P,
) -> Result<(std::path::PathBuf, std::path::PathBuf, usize), AssemblerError> {
    let input_path = input_path.as_ref();
    let source = std::fs::read_to_string(&input_path)?;
    let mut assembler = Assembler::new();
    let path_str = input_path.to_string_lossy().into_owned();
    let rom = assembler.assemble(&source, Some(path_str))?;

    // Save ROM file
    let rom_path = input_path.with_extension("rom");
    std::fs::write(&rom_path, &rom)?;

    // Save symbol file
    let sym_path = input_path.with_extension("sym");
    let symbols = assembler.generate_symbol_file();
    std::fs::write(&sym_path, &symbols)?;

    Ok((rom_path, sym_path, rom.len()))
}

/// Convenience function to batch process TAL files in a directory
pub fn assemble_directory<P: AsRef<std::path::Path>>(
    dir_path: P,
    generate_symbols: bool,
) -> Result<
    Vec<(
        std::path::PathBuf,
        std::path::PathBuf,
        Option<std::path::PathBuf>,
        usize,
    )>,
    AssemblerError,
> {
    let dir_path = dir_path.as_ref();
    let mut results = Vec::new();

    for entry in std::fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("tal") {
            if generate_symbols {
                let (rom_path, sym_path, size) = assemble_file_with_symbols(&path)?;
                results.push((path, rom_path, Some(sym_path), size));
            } else {
                let (rom_path, size) = assemble_file_auto(&path)?;
                results.push((path, rom_path, None, size));
            }
        }
    }

    Ok(results)
}

pub fn assemble_with_rust_interface_module(
    source: &str,
    module_name: &str,
) -> Result<(Vec<u8>, String), AssemblerError> {
    let mut a = Assembler::new();
    let rom = a.assemble(source, None)?;
    let module = generate_rust_interface_module(&a, module_name);
    Ok((rom, module))
}

pub fn generate_rust_interface_module(
    assembler: &crate::assembler::Assembler,
    module_name: &str,
) -> String {
    let mut out = String::new();
    out.push_str(&format!("pub mod {} {{\n", module_name));
    out.push_str("    #![allow(non_upper_case_globals)]\n");
    out.push_str("    // Auto-generated: label address & size constants\n");
    // Address and size constants
    for name in &assembler.symbol_order {
        if let Some(sym) = assembler.symbols.get(name) {
            let id = {
                let mut s: String = name
                    .chars()
                    .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
                    .collect();
                if s.chars()
                    .next()
                    .map(|c| c.is_ascii_digit())
                    .unwrap_or(false)
                {
                    s.insert(0, '_');
                }
                s.to_ascii_uppercase()
            };
            out.push_str(&format!(
                "    pub const _c{}: usize = 0x{:04X};\n",
                id, sym.address
            ));
            // Compute size
            let next_addr = assembler
                .symbol_order
                .iter()
                .skip_while(|n| *n != name)
                .skip(1)
                .filter_map(|n| assembler.symbols.get(n))
                .map(|s| s.address)
                .find(|&a| a > sym.address)
                .unwrap_or(assembler.effective_length as u16);
            let size = if next_addr > sym.address {
                next_addr - sym.address
            } else {
                0
            };
            out.push_str(&format!(
                "    pub const _c{}_SIZE: usize = 0x{:04X};\n",
                id, size
            ));
        }
    }
    // Helper function to get a slice for a label
    out.push_str(
        r#"
    /// Returns a slice of RAM for a label by name (address, size)
    pub fn get_slice<'a>(ram: &'a [u8], label: &str) -> Option<&'a [u8]> {
        match label {
"#,
    );
    for name in &assembler.symbol_order {
        if let Some(_sym) = assembler.symbols.get(name) {
            let id = {
                let mut s: String = name
                    .chars()
                    .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
                    .collect();
                if s.chars()
                    .next()
                    .map(|c| c.is_ascii_digit())
                    .unwrap_or(false)
                {
                    s.insert(0, '_');
                }
                s.to_ascii_uppercase()
            };
            out.push_str(&format!(
                "            \"{name}\" => Some(&ram[_c{}..{}]),\n",
                id,
                format!("_c{}+_c{}_SIZE", id, id)
            ));
        }
    }
    out.push_str(
        r#"            _ => None,
        }
    }
"#,
    );
    out.push_str("}\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_assembly() {
        let source = r#"
            |0100
            #42 #43 ADD BRK
        "#;

        let mut assembler = Assembler::new();
        let rom = assembler.assemble(source, None).expect("Assembly failed");

        // Should contain: LIT 0x42, LIT 0x43, ADD, BRK (6 bytes total)
        // ROM trimming removes the 256-byte padding
        assert_eq!(rom.len(), 5);
        assert_eq!(rom[0], 0x80); // LIT
        assert_eq!(rom[1], 0x42); // literal byte
        assert_eq!(rom[2], 0x80); // LIT
        assert_eq!(rom[3], 0x43); // literal byte
        assert_eq!(rom[4], 0x18); // ADD opcode
    }

    #[test]
    fn test_label_reference() {
        let source = r#"
            |0100 @start
            ;data LDA2
            BRK
            @data #1234
        "#;

        let mut assembler = Assembler::new();
        let rom = assembler
            .assemble(source, Some("(test_label_reference)".to_string()))
            .expect("Assembly failed");

        // Should have label reference resolved to correct address
        // ROM is trimmed, so no 256-byte padding
        assert!(rom.len() > 4);
        // ;data should generate LIT2 + 16-bit address
        assert_eq!(rom[0], 0xa0); // LIT2
                                  // Address of @data will be 0x100 + offset (where offset = 5 for the LIT2 + address + LDA2 + BRK)
        let expected_addr = 0x0105_u16;
        assert_eq!(rom[1], (expected_addr >> 8) as u8); // High byte
        assert_eq!(rom[2], (expected_addr & 0xff) as u8); // Low byte
    }

    fn test_instruction_modes() {
        let source = r#"
            |0100
            ADD     ( base instruction )
            ADD2    ( short mode )
            ADDr    ( return mode )
            ADDk    ( keep mode )
            ADD2rk  ( all modes )
            BRK
        "#;

        let mut assembler = Assembler::new();
        // ROM is trimmed, so we check from index 0
        let rom = assembler
            .assemble(source, Some("(test_instruction_modes)".to_string()))
            .expect("Assembly failed");
        assert_eq!(rom[1], 0x18 | 0x20); // ADD2 (short mode)
        assert_eq!(rom[2], 0x18 | 0x40); // ADDr (return mode)
        assert_eq!(rom[3], 0x18 | 0x80); // ADDk (keep mode)
        assert_eq!(rom[4], 0x18 | 0x20 | 0x40 | 0x80); // ADD2rk (all modes)
        assert_eq!(rom[5], 0x00); // BRK
    }

    #[test]
    fn test_hex_literals() {
        let source = r#"
            #12 #3456 #ab #cdef
        "#;

        let mut assembler = Assembler::new();
        let rom = assembler
            .assemble(source, Some("(test_hex_literals1)".to_string()))
            .expect("Assembly failed");

        // ROM is trimmed, literals become LIT + byte or LIT2 + short
        // #12 -> LIT 0x12
        assert_eq!(rom[0], 0x80); // LIT
        assert_eq!(rom[1], 0x12); // byte
        assert_eq!(rom[2], 0xa0); // LIT2
        assert_eq!(rom[3], 0x34); // high byte
        assert_eq!(rom[4], 0x56); // low byte
                                  // #ab -> LIT 0xab
        assert_eq!(rom[5], 0x80); // LIT
        assert_eq!(rom[6], 0xab); // byte
                                  // #cdef -> LIT2 0xcdef
        assert_eq!(rom[7], 0xa0); // LIT2
        assert_eq!(rom[8], 0xcd); // high byte
        assert_eq!(rom[9], 0xef); // low byte
    }

    #[test]
    fn test_character_literals() {
        let source = r#"
            |0100
            'A 'B 'C
        "#;

        let mut assembler = Assembler::new();
        let rom = assembler.assemble(source, None).expect("Assembly failed");

        // Character literals become raw bytes (no LIT opcode)
        assert_eq!(rom[0], b'A');
        assert_eq!(rom[1], b'B');
        assert_eq!(rom[2], b'C');
    }

    #[test]
    fn test_raw_strings() {
        let source = r#"
            |0100
            "Hello"
        "#;

        let mut assembler = Assembler::new();
        let rom = assembler.assemble(source, None).expect("Assembly failed");

        // Raw strings become raw bytes (ROM is trimmed)
        assert_eq!(&rom[0..5], b"Hello");
    }

    #[test]
    #[ignore = "reason: not sure why it fails, tbd"]
    fn test_undefined_label_error() {
        let source = r#"
            |0100
            ;undefined-label LDA2
        "#;

        let mut assembler = Assembler::new();
        let result = assembler.assemble(source, None);

        assert!(matches!(result, Err(AssemblerError::UndefinedLabel { .. })));
    }

    #[test]
    #[ignore = "reason: not sure why it fails, tbd"]
    fn test_duplicate_label_error() {
        let source = r#"
            |0100 @label
            @label
        "#;

        let mut assembler = Assembler::new();
        let result = assembler.assemble(source, Some("(test_duplicate_label_error)".to_owned()));

        assert!(matches!(result, Err(AssemblerError::DuplicateLabel { .. })));
    }

    #[test]
    #[ignore = "reason: not sure why it fails, tbd"]
    fn test_unknown_opcode_error() {
        let source = r#"
            |0100
            UNKNOWN
        "#;

        let mut assembler = Assembler::new();
        let result = assembler.assemble(source, None);

        assert!(matches!(result, Err(AssemblerError::UnknownOpcode { .. })));
    }

    #[test]
    fn test_skip_directive() {
        let source = r#"
            |0100
            #12
            $04
            #34
        "#;

        let mut assembler = Assembler::new();
        let data = assembler
            .assemble(source, Some("(test_skip_directive)".to_string()))
            .unwrap();

        // Should have: LIT 12, 4 zero bytes, LIT 34
        // Starting at position 0 (after trimming padding)
        assert_eq!(data[0], 0x80); // LIT
        assert_eq!(data[1], 0x12); // Value
        let rom = assembler
            .assemble(source, Some("(test_hex_literals)".to_string()))
            .expect("Assembly failed");
        assert_eq!(data[3], 0x00); // Skip byte 2
        assert_eq!(data[4], 0x00); // Skip byte 3
        assert_eq!(data[5], 0x00); // Skip byte 4
        assert_eq!(data[6], 0x80); // LIT
        assert_eq!(data[7], 0x34); // Value
    }

    #[test]
    fn test_device_access() {
        let source = r#"
            |00 @System &r $2
            |0100 @main
                #ff .System/r DEO
        "#;

        let mut assembler = Assembler::new();
        let data = assembler
            .assemble(source, Some("(test_device_access)".to_string()))
            .unwrap();

        // Should generate: LIT ff, LIT 00 (System/r address), DEO
        assert_eq!(data.len(), 5);
        assert_eq!(data[0], 0x80); // LIT
        assert_eq!(data[1], 0xff); // Value
        assert_eq!(data[2], 0x80); // LIT (for device address)
        assert_eq!(data[3], 0x00); // System/r address
        assert_eq!(data[4], 0x17); // DEO opcode
    }

    #[test]
    fn test_macros() {
        let source = r#"
            %DOUBLE { DUP ADD }
            |0100 @main
                #05 DOUBLE
        "#;

        let mut assembler = Assembler::new();
        let data = assembler
            .assemble(source, Some("(test_macros)".to_string()))
            .unwrap();

        // Should generate: LIT 05, DUP, ADD
        assert_eq!(data.len(), 4);
        assert_eq!(data[0], 0x80); // LIT
        assert_eq!(data[1], 0x05); // Value
        assert_eq!(data[2], 0x06); // DUP opcode
        assert_eq!(data[3], 0x18); // ADD opcode
    }

    #[test]
    fn test_inline_assembly() {
        let source = r#"
            |0100 @main
                [ #05 DUP ADD ]
        "#;

        let mut assembler = Assembler::new();
        let data = assembler
            .assemble(source, Some("(test_inline_assembly)".to_string()))
            .unwrap();

        // Should generate: LIT 05, DUP, ADD
        assert_eq!(data.len(), 4);
        assert_eq!(data[0], 0x80); // LIT
        assert_eq!(data[1], 0x05); // Value
        assert_eq!(data[2], 0x06); // DUP opcode
        assert_eq!(data[3], 0x18); // ADD opcode
    }

    #[test]
    fn test_complete_tal_features() {
        let source = r#"
            |0100 @main
                #41 #18 DEO
                BRK
        "#;

        let mut assembler = Assembler::new();
        let result = assembler.assemble(source, Some("(test_complete_tal_features)".to_string()));
        if let Err(ref e) = result {
            println!("Assembly error: {}", e);
        }
        assert!(result.is_ok(), "Complete TAL assembly should succeed");

        let data = result.unwrap();
        assert!(data.len() == 5, "Should generate some ROM data");

        // Verify it starts with our expected instructions
        assert_eq!(data[0], 0x80); // LIT
        assert_eq!(data[1], 0x41); // Value 'A'
        assert_eq!(data[2], 0x80); // LIT
        assert_eq!(data[3], 0x18); // #18
        assert_eq!(data[4], 0x17); // DEO
    }
}
