//! UXN opcodes and instruction definitions

use crate::error::Result;
use crate::opcode_table::{create_instruction_map, encode_opcode};
use std::collections::HashMap;

/// UXN opcode definitions
pub struct Opcodes {
    #[allow(dead_code)]
    opcodes: HashMap<String, u8>,
}

impl Opcodes {
    pub fn new() -> Self {
        Self {
            opcodes: create_instruction_map(),
        }
    }

    /// Get the base opcode for an instruction
    pub fn get_opcode(&self, name: &str) -> Result<u8> {
        // Reference ops table, same order as uxnasm.c
        const OPS: [&str; 32] = [
            "LIT", "INC", "POP", "NIP", "SWP", "ROT", "DUP", "OVR", "EQU", "NEQ", "GTH", "LTH",
            "JMP", "JCN", "JSR", "STH", "LDZ", "STZ", "LDR", "STR", "LDA", "STA", "DEI", "DEO",
            "ADD", "SUB", "MUL", "DIV", "AND", "ORA", "EOR", "SFT",
        ];

        let name = name.trim();
        for (i, &base) in OPS.iter().enumerate() {
            if name.len() < 3 {
                continue;
            }
            if &name[..3].to_ascii_uppercase() != base {
                continue;
            }
            let mut opcode = i as u8;
            let mut m = 3;
            // LIT always sets keep bit
            if i == 0 {
                opcode |= 1 << 7;
            }
            let chars: Vec<char> = name.chars().collect();
            while m < chars.len() {
                match chars[m] {
                    '2' => opcode |= 1 << 5,
                    'r' => opcode |= 1 << 6,
                    'k' => opcode |= 1 << 7,
                    _ => {
                        return Err(crate::error::AssemblerError::SyntaxError {
                            path: "".to_string(),
                            line: 0,
                            position: 0,
                            message: format!(
                                "Invalid mode flag '{}' in opcode '{}'",
                                chars[m], name
                            ),
                            source_line: "".to_string(),
                        })
                    }
                }
                m += 1;
            }
            return Ok(opcode);
        }
        // Special case for BRK (not in ops table, always 0x00)
        if name.eq_ignore_ascii_case("BRK") {
            return Ok(0x00);
        }
        Err(crate::error::AssemblerError::SyntaxError {
            path: "".to_string(),
            line: 0,
            position: 0,
            message: format!("Unknown opcode '{}'", name),
            source_line: "".to_string(),
        })
    }

    /// Apply mode flags to an opcode
    pub fn apply_modes(opcode: u8, short_mode: bool, return_mode: bool, keep_mode: bool) -> u8 {
        encode_opcode(opcode, short_mode, return_mode, keep_mode)
    }
}

impl Default for Opcodes {
    fn default() -> Self {
        Self::new()
    }
}
