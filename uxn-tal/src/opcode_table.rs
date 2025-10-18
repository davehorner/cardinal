//! UXN Opcode Reference Table
//!
//! This module contains the complete UXN opcode table as specified in the official
//! UXN Tal Reference: <https://wiki.xxiivv.com/site/uxntal_reference.html>
//!
//! The table maps each opcode byte (0x00-0xFF) to its corresponding instruction
//! with mode flags applied.

use std::collections::HashMap;

/// Complete UXN opcode reference table
///
/// This table represents the full 256-entry opcode map where each byte
/// corresponds to a specific instruction with mode flags applied.
///
/// Mode flags are encoded in the opcode byte as follows:
/// - Bit 7 (0x80): Keep mode (k)
/// - Bit 6 (0x40): Return mode (r)
/// - Bit 5 (0x20): Short mode (2)
/// - Bits 4-0: Base instruction (0-31)
pub const UXN_OPCODE_TABLE: &[(u8, &str)] = &[
    // Row 0x00: Base instructions
    (0x00, "BRK"),
    (0x01, "INC"),
    (0x02, "POP"),
    (0x03, "NIP"),
    (0x04, "SWP"),
    (0x05, "ROT"),
    (0x06, "DUP"),
    (0x07, "OVR"),
    (0x08, "EQU"),
    (0x09, "NEQ"),
    (0x0A, "GTH"),
    (0x0B, "LTH"),
    (0x0C, "JMP"),
    (0x0D, "JCN"),
    (0x0E, "JSR"),
    (0x0F, "STH"),
    // Row 0x10: Base instructions continued
    (0x10, "LDZ"),
    (0x11, "STZ"),
    (0x12, "LDR"),
    (0x13, "STR"),
    (0x14, "LDA"),
    (0x15, "STA"),
    (0x16, "DEI"),
    (0x17, "DEO"),
    (0x18, "ADD"),
    (0x19, "SUB"),
    (0x1A, "MUL"),
    (0x1B, "DIV"),
    (0x1C, "AND"),
    (0x1D, "ORA"),
    (0x1E, "EOR"),
    (0x1F, "SFT"),
    // Row 0x20: Short mode (2) instructions
    (0x20, "JCI"),
    (0x21, "INC2"),
    (0x22, "POP2"),
    (0x23, "NIP2"),
    (0x24, "SWP2"),
    (0x25, "ROT2"),
    (0x26, "DUP2"),
    (0x27, "OVR2"),
    (0x28, "EQU2"),
    (0x29, "NEQ2"),
    (0x2A, "GTH2"),
    (0x2B, "LTH2"),
    (0x2C, "JMP2"),
    (0x2D, "JCN2"),
    (0x2E, "JSR2"),
    (0x2F, "STH2"),
    // Row 0x30: Short mode continued
    (0x30, "LDZ2"),
    (0x31, "STZ2"),
    (0x32, "LDR2"),
    (0x33, "STR2"),
    (0x34, "LDA2"),
    (0x35, "STA2"),
    (0x36, "DEI2"),
    (0x37, "DEO2"),
    (0x38, "ADD2"),
    (0x39, "SUB2"),
    (0x3A, "MUL2"),
    (0x3B, "DIV2"),
    (0x3C, "AND2"),
    (0x3D, "ORA2"),
    (0x3E, "EOR2"),
    (0x3F, "SFT2"),
    // Row 0x40: Return mode (r) instructions
    (0x40, "JMI"),
    (0x41, "INCr"),
    (0x42, "POPr"),
    (0x43, "NIPr"),
    (0x44, "SWPr"),
    (0x45, "ROTr"),
    (0x46, "DUPr"),
    (0x47, "OVRr"),
    (0x48, "EQUr"),
    (0x49, "NEQr"),
    (0x4A, "GTHr"),
    (0x4B, "LTHr"),
    (0x4C, "JMPr"),
    (0x4D, "JCNr"),
    (0x4E, "JSRr"),
    (0x4F, "STHr"),
    // Row 0x50: Return mode continued
    (0x50, "LDZr"),
    (0x51, "STZr"),
    (0x52, "LDRr"),
    (0x53, "STRr"),
    (0x54, "LDAr"),
    (0x55, "STAr"),
    (0x56, "DEIr"),
    (0x57, "DEOr"),
    (0x58, "ADDr"),
    (0x59, "SUBr"),
    (0x5A, "MULr"),
    (0x5B, "DIVr"),
    (0x5C, "ANDr"),
    (0x5D, "ORAr"),
    (0x5E, "EORr"),
    (0x5F, "SFTr"),
    // Row 0x60: Return + Short mode (2r) instructions
    (0x60, "JSI"),
    (0x61, "INC2r"),
    (0x62, "POP2r"),
    (0x63, "NIP2r"),
    (0x64, "SWP2r"),
    (0x65, "ROT2r"),
    (0x66, "DUP2r"),
    (0x67, "OVR2r"),
    (0x68, "EQU2r"),
    (0x69, "NEQ2r"),
    (0x6A, "GTH2r"),
    (0x6B, "LTH2r"),
    (0x6C, "JMP2r"),
    (0x6D, "JCN2r"),
    (0x6E, "JSR2r"),
    (0x6F, "STH2r"),
    // Row 0x70: Return + Short mode continued
    (0x70, "LDZ2r"),
    (0x71, "STZ2r"),
    (0x72, "LDR2r"),
    (0x73, "STR2r"),
    (0x74, "LDA2r"),
    (0x75, "STA2r"),
    (0x76, "DEI2r"),
    (0x77, "DEO2r"),
    (0x78, "ADD2r"),
    (0x79, "SUB2r"),
    (0x7A, "MUL2r"),
    (0x7B, "DIV2r"),
    (0x7C, "AND2r"),
    (0x7D, "ORA2r"),
    (0x7E, "EOR2r"),
    (0x7F, "SFT2r"),
    // Row 0x80: Keep mode (k) instructions
    (0x80, "LIT"),
    (0x81, "INCk"),
    (0x82, "POPk"),
    (0x83, "NIPk"),
    (0x84, "SWPk"),
    (0x85, "ROTk"),
    (0x86, "DUPk"),
    (0x87, "OVRk"),
    (0x88, "EQUk"),
    (0x89, "NEQk"),
    (0x8A, "GTHk"),
    (0x8B, "LTHk"),
    (0x8C, "JMPk"),
    (0x8D, "JCNk"),
    (0x8E, "JSRk"),
    (0x8F, "STHk"),
    // Row 0x90: Keep mode continued
    (0x90, "LDZk"),
    (0x91, "STZk"),
    (0x92, "LDRk"),
    (0x93, "STRk"),
    (0x94, "LDAk"),
    (0x95, "STAk"),
    (0x96, "DEIk"),
    (0x97, "DEOk"),
    (0x98, "ADDk"),
    (0x99, "SUBk"),
    (0x9A, "MULk"),
    (0x9B, "DIVk"),
    (0x9C, "ANDk"),
    (0x9D, "ORAk"),
    (0x9E, "EORk"),
    (0x9F, "SFTk"),
    // Row 0xA0: Keep + Short mode (2k) instructions
    (0xA0, "LIT2"),
    (0xA1, "INC2k"),
    (0xA2, "POP2k"),
    (0xA3, "NIP2k"),
    (0xA4, "SWP2k"),
    (0xA5, "ROT2k"),
    (0xA6, "DUP2k"),
    (0xA7, "OVR2k"),
    (0xA8, "EQU2k"),
    (0xA9, "NEQ2k"),
    (0xAA, "GTH2k"),
    (0xAB, "LTH2k"),
    (0xAC, "JMP2k"),
    (0xAD, "JCN2k"),
    (0xAE, "JSR2k"),
    (0xAF, "STH2k"),
    // Row 0xB0: Keep + Short mode continued
    (0xB0, "LDZ2k"),
    (0xB1, "STZ2k"),
    (0xB2, "LDR2k"),
    (0xB3, "STR2k"),
    (0xB4, "LDA2k"),
    (0xB5, "STA2k"),
    (0xB6, "DEI2k"),
    (0xB7, "DEO2k"),
    (0xB8, "ADD2k"),
    (0xB9, "SUB2k"),
    (0xBA, "MUL2k"),
    (0xBB, "DIV2k"),
    (0xBC, "AND2k"),
    (0xBD, "ORA2k"),
    (0xBE, "EOR2k"),
    (0xBF, "SFT2k"),
    // Row 0xC0: Keep + Return mode (kr) instructions
    (0xC0, "LITr"),
    (0xC1, "INCkr"),
    (0xC2, "POPkr"),
    (0xC3, "NIPkr"),
    (0xC4, "SWPkr"),
    (0xC5, "ROTkr"),
    (0xC6, "DUPkr"),
    (0xC7, "OVRkr"),
    (0xC8, "EQUkr"),
    (0xC9, "NEQkr"),
    (0xCA, "GTHkr"),
    (0xCB, "LTHkr"),
    (0xCC, "JMPkr"),
    (0xCD, "JCNkr"),
    (0xCE, "JSRkr"),
    (0xCF, "STHkr"),
    // Row 0xD0: Keep + Return mode continued
    (0xD0, "LDZkr"),
    (0xD1, "STZkr"),
    (0xD2, "LDRkr"),
    (0xD3, "STRkr"),
    (0xD4, "LDAkr"),
    (0xD5, "STAkr"),
    (0xD6, "DEIkr"),
    (0xD7, "DEOkr"),
    (0xD8, "ADDkr"),
    (0xD9, "SUBkr"),
    (0xDA, "MULkr"),
    (0xDB, "DIVkr"),
    (0xDC, "ANDkr"),
    (0xDD, "ORAkr"),
    (0xDE, "EORkr"),
    (0xDF, "SFTkr"),
    // Row 0xE0: Keep + Return + Short mode (2kr) instructions
    (0xE0, "LIT2r"),
    (0xE1, "INC2kr"),
    (0xE2, "POP2kr"),
    (0xE3, "NIP2kr"),
    (0xE4, "SWP2kr"),
    (0xE5, "ROT2kr"),
    (0xE6, "DUP2kr"),
    (0xE7, "OVR2kr"),
    (0xE8, "EQU2kr"),
    (0xE9, "NEQ2kr"),
    (0xEA, "GTH2kr"),
    (0xEB, "LTH2kr"),
    (0xEC, "JMP2kr"),
    (0xED, "JCN2kr"),
    (0xEE, "JSR2kr"),
    (0xEF, "STH2kr"),
    // Row 0xF0: Keep + Return + Short mode continued
    (0xF0, "LDZ2kr"),
    (0xF1, "STZ2kr"),
    (0xF2, "LDR2kr"),
    (0xF3, "STR2kr"),
    (0xF4, "LDA2kr"),
    (0xF5, "STA2kr"),
    (0xF6, "DEI2kr"),
    (0xF7, "DEO2kr"),
    (0xF8, "ADD2kr"),
    (0xF9, "SUB2kr"),
    (0xFA, "MUL2kr"),
    (0xFB, "DIV2kr"),
    (0xFC, "AND2kr"),
    (0xFD, "ORA2kr"),
    (0xFE, "EOR2kr"),
    (0xFF, "SFT2kr"),
];

/// Base instruction names (without mode flags)
pub const BASE_INSTRUCTIONS: &[&str] = &[
    "BRK", "INC", "POP", "NIP", "SWP", "ROT", "DUP", "OVR", // 0x00-0x07
    "EQU", "NEQ", "GTH", "LTH", "JMP", "JCN", "JSR", "STH", // 0x08-0x0F
    "LDZ", "STZ", "LDR", "STR", "LDA", "STA", "DEI", "DEO", // 0x10-0x17
    "ADD", "SUB", "MUL", "DIV", "AND", "ORA", "EOR", "SFT", // 0x18-0x1F
];

/// Create a hashmap from instruction name to base opcode value
pub fn create_instruction_map() -> HashMap<String, u8> {
    let mut map = HashMap::new();
    for (index, &instruction) in BASE_INSTRUCTIONS.iter().enumerate() {
        map.insert(instruction.to_string(), index as u8);
        map.insert(instruction.to_ascii_lowercase(), index as u8);
    }
    map
}

/// Get the complete opcode name for a given byte value
pub fn get_opcode_name(opcode: u8) -> &'static str {
    UXN_OPCODE_TABLE[opcode as usize].1
}

/// Decode an opcode byte into its components
pub fn decode_opcode(opcode: u8) -> (u8, bool, bool, bool) {
    let base = opcode & 0x1F; // Bits 0-4: base instruction
    let short_mode = (opcode & 0x20) != 0; // Bit 5: short mode
    let return_mode = (opcode & 0x40) != 0; // Bit 6: return mode
    let keep_mode = (opcode & 0x80) != 0; // Bit 7: keep mode

    (base, short_mode, return_mode, keep_mode)
}

/// Encode an opcode from base instruction and mode flags
pub fn encode_opcode(base: u8, short_mode: bool, return_mode: bool, keep_mode: bool) -> u8 {
    let mut opcode = base & 0x1F;
    if short_mode {
        opcode |= 0x20;
    }
    if return_mode {
        opcode |= 0x40;
    }
    if keep_mode {
        opcode |= 0x80;
    }
    opcode
}

/// Verify that our opcode table matches the official specification
pub fn verify_opcode_table() -> Result<(), String> {
    // Check that we have exactly 256 entries
    if UXN_OPCODE_TABLE.len() != 256 {
        return Err(format!(
            "Expected 256 opcodes, found {}",
            UXN_OPCODE_TABLE.len()
        ));
    }

    // Check that opcodes are in order
    for (i, &(opcode, _)) in UXN_OPCODE_TABLE.iter().enumerate() {
        if opcode != i as u8 {
            return Err(format!(
                "Opcode at index {} should be 0x{:02x}, found 0x{:02x}",
                i, i, opcode
            ));
        }
    }

    // Check special opcodes
    let special_checks = [
        (0x00, "BRK"),
        (0x80, "LIT"),
        (0xA0, "LIT2"),
        (0x17, "DEO"),
        (0x20, "JCI"),
        (0x40, "JMI"),
        (0x60, "JSI"),
    ];

    for &(opcode, expected_name) in &special_checks {
        let actual_name = get_opcode_name(opcode);
        if actual_name != expected_name {
            return Err(format!(
                "Opcode 0x{:02x} should be '{}', found '{}'",
                opcode, expected_name, actual_name
            ));
        }
    }

    println!("✓ UXN opcode table verification passed");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opcode_table_verification() {
        verify_opcode_table().unwrap();
    }

    #[test]
    fn test_opcode_encoding() {
        // Test LIT (0x80)
        assert_eq!(encode_opcode(0x00, false, false, true), 0x80);

        // Test LIT2 (0xA0)
        assert_eq!(encode_opcode(0x00, true, false, true), 0xA0);

        // Test DEO (0x17)
        assert_eq!(encode_opcode(0x17, false, false, false), 0x17);

        // Test ADD2k (0xB8)
        assert_eq!(encode_opcode(0x18, true, false, true), 0xB8);
    }

    #[test]
    fn test_opcode_decoding() {
        // Test LIT (0x80)
        let (base, short, ret, keep) = decode_opcode(0x80);
    assert_eq!(base, 0x00);
    assert!(!short);
    assert!(!ret);
    assert!(keep);

        // Test ADD2kr (0xF8)
        let (base, short, ret, keep) = decode_opcode(0xF8);
    assert_eq!(base, 0x18);
    assert!(short);
    assert!(ret);
    assert!(keep);
    }

    #[test]
    fn test_instruction_names() {
        assert_eq!(get_opcode_name(0x00), "BRK");
        assert_eq!(get_opcode_name(0x80), "LIT");
        assert_eq!(get_opcode_name(0x17), "DEO");
        assert_eq!(get_opcode_name(0xB8), "ADD2k");
        assert_eq!(get_opcode_name(0xFF), "SFT2kr");
    }
}
