//! Uxn disassembler
// (
// https://github.com/Liorst4/uxn-disassembler/tree/main
// Copyright © 2025 David Horner
// Copyright © 2022 Lior Stern

// Permission is hereby granted, free of charge, to any person obtaining
// a copy of this software and associated documentation files (the
// “Software”), to deal in the Software without restriction, including
// without limitation the rights to use, copy, modify, merge, publish,
// distribute, sublicense, and/or sell copies of the Software, and to
// permit persons to whom the Software is furnished to do so, subject to
// the following conditions:

// The above copyright notice and this permission notice shall be
// included in all copies or substantial portions of the Software.

// THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND,
// EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
// MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
// IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
// CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT,
// TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE
// SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
// )

const SHORT_MODE_MASK: u8 = 0x20;
const RETURN_MODE_MASK: u8 = 0x40;
const KEEP_MODE_MASK: u8 = 0x80;
const OPCODE_MASK: u8 = 0x1F;

const OPCODE_NAMES: [&str; 32] = [
    "LIT", "INC", "POP", "NIP", "SWP", "ROT", "DUP", "OVR", "EQU", "NEQ",
    "GTH", "LTH", "JMP", "JCN", "JSR", "STH", "LDZ", "STZ", "LDR", "STR",
    "LDA", "STA", "DEI", "DEO", "ADD", "SUB", "MUL", "DIV", "AND", "ORA",
    "EOR", "SFT",
];

/// A disassembled Uxn instruction
pub struct DisassembledInstr {
    /// Address of the instruction
    pub addr: usize,
    /// Opcode byte
    pub opcode: u8,
    /// Mnemonic string
    pub mnemonic: &'static str,
    /// Keep flag
    pub keep: bool,
    /// Return flag
    pub ret: bool,
    /// Short flag
    pub short: bool,
    /// Optional literal value
    pub literal: Option<u16>,
    /// Raw bytes of the instruction (max 4)
    pub raw_bytes: [u8; 4],
    /// Length of the instruction in bytes
    pub raw_len: usize,
}

/// Disassemble a Uxn ROM, calling `callback` for each instruction
pub fn disassemble<F>(rom: &[u8], _disassemble_to_byte: usize, mut callback: F)
where
    F: FnMut(DisassembledInstr),
{
    let mut i = 0;
    while i < rom.len() {
        let instr = rom[i];
        let opcode = instr & OPCODE_MASK;
        let keep = (instr & KEEP_MODE_MASK) != 0;
        let ret = (instr & RETURN_MODE_MASK) != 0;
        let short = (instr & SHORT_MODE_MASK) != 0;
        let addr = i + 0x100;

        if opcode == 0x00 {
            // LIT instruction
            if short {
                if i + 2 >= rom.len() {
                    let mut raw = [0u8; 4];
                    let len = rom.len() - i;
                    raw[..len].copy_from_slice(&rom[i..(len + i)]);
                    callback(DisassembledInstr {
                        addr,
                        opcode,
                        mnemonic: "LIT2",
                        keep,
                        ret,
                        short,
                        literal: None,
                        raw_bytes: raw,
                        raw_len: len,
                    });
                    break;
                }
                let value = u16::from_be_bytes([rom[i + 1], rom[i + 2]]);
                callback(DisassembledInstr {
                    addr,
                    opcode,
                    mnemonic: "LIT2",
                    keep,
                    ret,
                    short,
                    literal: Some(value),
                    raw_bytes: [instr, rom[i + 1], rom[i + 2], 0],
                    raw_len: 3,
                });
                i += 3;
            } else {
                if i + 1 >= rom.len() {
                    let mut raw = [0u8; 4];
                    let len = rom.len() - i;
                    raw[..len].copy_from_slice(&rom[i..(len + i)]);
                    callback(DisassembledInstr {
                        addr,
                        opcode,
                        mnemonic: "LIT",
                        keep,
                        ret,
                        short,
                        literal: None,
                        raw_bytes: raw,
                        raw_len: len,
                    });
                    break;
                }
                callback(DisassembledInstr {
                    addr,
                    opcode,
                    mnemonic: "LIT",
                    keep,
                    ret,
                    short,
                    literal: Some(rom[i + 1] as u16),
                    raw_bytes: [instr, rom[i + 1], 0, 0],
                    raw_len: 2,
                });
                i += 2;
            }
        } else {
            callback(DisassembledInstr {
                addr,
                opcode,
                mnemonic: OPCODE_NAMES[opcode as usize],
                keep,
                ret,
                short,
                literal: None,
                raw_bytes: [instr, 0, 0, 0],
                raw_len: 1,
            });
            i += 1;
        }
    }
}

#[allow(dead_code)]
fn write_literal_prefix(
    buf: &mut [u8],
    short: bool,
    _keep: bool,
    ret: bool,
) -> usize {
    let mut idx = 0;
    if !ret {
        buf[idx] = b'#';
        idx += 1;
    } else {
        let lit = b"LIT";
        buf[idx..idx + 3].copy_from_slice(lit);
        idx += 3;
        if short {
            buf[idx] = b'2';
            idx += 1;
        }
        buf[idx] = b'r';
        idx += 1;
        buf[idx] = b' ';
        idx += 1;
    }
    idx
}

#[allow(dead_code)]
fn write_literal_postfix(buf: &mut [u8], short: bool, ret: bool) -> usize {
    let mut idx = 0;
    if short && ret {
        buf[idx] = b'\t';
        idx += 1;
    }
    idx
}
