//! UXN opcodes and instruction definitions

use crate::error::{AssemblerError, Result};
use crate::opcode_table::{create_instruction_map, encode_opcode};
use std::collections::HashMap;

/// UXN opcode definitions
pub struct Opcodes {
    opcodes: HashMap<String, u8>,
}

impl Opcodes {
    pub fn new() -> Self {
        Self {
            opcodes: create_instruction_map(),
        }
    }

    /// Get the base opcode for an instruction
    pub fn get_opcode(&self, instruction: &str) -> Result<u8> {
        self.opcodes
            .get(instruction)
            .copied()
            .ok_or_else(|| AssemblerError::UnknownOpcode {
                opcode: instruction.to_string(),
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
