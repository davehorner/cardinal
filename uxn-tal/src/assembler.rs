//! Main assembler implementation

use crate::devicemap::{parse_device_maps, Device, DeviceField};
use crate::error::{AssemblerError, Result};
use crate::lexer::{Lexer, TokenWithPos};
use crate::opcodes::Opcodes;
use crate::parser::{AstNode, Parser};
use crate::rom::Rom;
use std::collections::HashMap;
use std::fs;

/// Macro definition
#[derive(Debug, Clone)]
pub struct Macro {
    pub name: String,
    pub body: Vec<AstNode>,
}

/// Symbol table entry
#[derive(Debug, Clone)]
pub struct Symbol {
    pub address: u16,
    pub is_sublabel: bool,
    pub parent_label: Option<String>,
}

/// TAL assembler
pub struct Assembler {
    opcodes: Opcodes,
    symbols: HashMap<String, Symbol>,
    macros: HashMap<String, Macro>,
    current_label: Option<String>,
    references: Vec<Reference>,
    device_map: HashMap<String, Device>, // device name -> Device
    line_number: usize,
    position_in_line: usize,
    effective_length: usize, // Track effective length like uxnasm.c
    lambda_counter: u16, // Add lambda counter as a field
}

/// Represents a forward reference that needs to be resolved
#[derive(Debug, Clone)]
struct Reference {
    name: String,
    rune: char,
    address: u16,
    line: usize,
    path: String,
    scope: Option<String>, // Add scope context
    token: Option<TokenWithPos>,
}

impl Assembler {
    /// Generate symbol file content in binary format
    /// Format: [address:u16][name:null-terminated string] repeating
    pub fn generate_symbol_file(&self) -> Vec<u8> {
        let mut symbols: Vec<_> = self.symbols.iter().collect();
        symbols.sort_by_key(|(_, symbol)| symbol.address);
        let mut output = Vec::new();
        for (name, symbol) in symbols {
            // Write address as big-endian u16 (C code: hb first, then lb)
            output.push((symbol.address >> 8) as u8); // high byte
            output.push((symbol.address & 0xff) as u8); // low byte
                                                        // Write name as null-terminated string
            output.extend_from_slice(name.as_bytes());
            output.push(0); // null terminator
        }
        output
    }


    /// Generate symbol file content in binary format
    /// Format: [address:u16][name:null-terminated string] repeating
    pub fn generate_symbol_file_binary(&self) -> Vec<u8> {
        let mut symbols: Vec<_> = self.symbols.iter().collect();
        symbols.sort_by_key(|(_, symbol)| symbol.address);

        let mut output = Vec::new();
        for (name, symbol) in symbols {
            // Write address as little-endian u16
            output.extend_from_slice(&symbol.address.to_le_bytes());
            // Write name as null-terminated string
            output.extend_from_slice(name.as_bytes());
            output.push(0); // null terminator
        }
        output
    }

    /// Generate symbol file content in textual format (address and name per line)
    pub fn generate_symbol_file_txt(&self) -> String {
        let mut symbols: Vec<_> = self.symbols.iter().collect();
        symbols.sort_by_key(|(_, symbol)| symbol.address);
        let mut output = String::new();
        for (name, symbol) in symbols {
            output.push_str(&format!("{:04X} {}\n", symbol.address, name));
        }
        output
    }

    /// Create a new assembler instance
    pub fn new() -> Self {
        Self {
            opcodes: Opcodes::new(),
            symbols: HashMap::new(),
            macros: HashMap::new(),
            current_label: None,
            references: Vec::new(),
            device_map: HashMap::new(),
            line_number: 0,
            position_in_line: 0,
            effective_length: 0,
            lambda_counter: 0, // Initialize lambda counter
        }
    }

    /// Update effective length if current position has non-zero content
    fn update_effective_length(&mut self, rom: &Rom) {
        self.effective_length = self.effective_length.max(rom.position().into());
    }

    /// Assemble TAL source code into a ROM
    pub fn assemble(&mut self, source: &str, path: Option<String>) -> Result<Vec<u8>> {
        // Clear previous state
        self.symbols.clear();
        self.current_label = None;
        self.references.clear();
        self.device_map.clear();
        self.line_number = 0;
        self.position_in_line = 0;
        self.effective_length = 0; // Reset effective length
        self.lambda_counter = 0; // Reset lambda counter

        // Tokenize
        let mut lexer = Lexer::new(source.to_string(), path.clone());
        let tokens = lexer.tokenize()?;

        // Parse
        // Use "(input)" as the default path if none is provided
        let mut parser =
            Parser::new_with_source(tokens, path.clone().unwrap_or_default(), source.to_string());
        let ast = parser.parse()?;

        // First pass: collect labels and generate code
        let mut rom = Rom::new();
        rom.set_source(Some(source.to_string()));
        rom.set_path(path.clone());
        self.first_pass(&ast, &mut rom)?;

        // Only print collected labels and references if a label resolution fails (see second_pass)

        // Second pass: resolve references
        self.second_pass(&mut rom)?;
        println!("DEBUG: Resolved {} references", self.references.len());

        // Get the final ROM data using effective length (like uxnasm.c)
        let rom_data = rom.data();

        // Count ALL labels like uxnasm.c does (don't filter out device fields)
        let total_label_count = self.symbols.len();

        // If the ROM has content starting at 0x0100 or later, exclude the first 256 bytes
        // This matches the behavior of ruxnasm and uxnasm.c
        let rom_len = rom.len();
        if rom_len > 0 {
            let has_zero_page_data = rom.has_zero_page_data();
            if !has_zero_page_data {
                println!(
                    "Assembled {} in {} bytes({:.2}% used), {} labels, {} macros.",
                    path.clone().unwrap_or_else(|| "(input)".to_string()),
                    rom_len,
                    rom_len as f64 / 652.80,
                    total_label_count,
                    self.macros.len()
                );
                // Output the ROM data starting at 0x0100, length rom_len
                return Ok(rom.data().to_vec());
            }
        }
        Ok(rom_data[..rom_len.min(rom_data.len())].to_vec())
    }

    fn first_pass(&mut self, ast: &[AstNode], rom: &mut Rom) -> Result<()> {
        let mut i = 0;
        while i < ast.len() {
            // Remove special handling for SublabelDef here
            self.process_node(&ast[i], rom)?;
            i += 1;
        }
        Ok(())
    }

    fn process_node(&mut self, node: &AstNode, rom: &mut Rom) -> Result<()> {
        let path = rom.source_path().cloned().unwrap_or_default();
        let _start_address = rom.position();

        match node {
            AstNode::Ignore => {
                // Do nothing - brackets are completely ignored like in uxnasm.c
            }
            AstNode::ConditionalBlock(nodes) => {
                // --- PATCH: Emit a lambda label and a reference for the conditional block, like uxnasm ---
                // Generate a unique lambda label for this block
                let lambda_label = format!("λ{:02X}", self.lambda_counter);
                self.lambda_counter += 1;

                                // Emit a reference to the lambda label with '?' rune (JCN)
                // Write JCN opcode (0x20), then placeholder for relative address
                rom.write_byte(0x20)?;
                self.update_effective_length(rom);
                // Write placeholder for relative address (2 bytes)
                let position = rom.position();
                rom.write_short(0xffff)?;
                self.update_effective_length(rom);

                // Emit the block contents
                for n in nodes {
                    self.process_node(n, rom)?;
                }
                let len = rom.position() - position;
         

                /*
                No, we do not need to capture the closing '}' or any special token here.
                The block contents have already been processed above, and the lambda label is placed
                immediately after the block, matching uxnasm's behavior.
                */
                // --- FIX: Place the lambda label *after* the block (after the closing }) ---
                // This matches uxnasm: the label is at the instruction after the block.
                // (Nothing needed here; the block contents are already processed above.)
                self.symbols.insert(
                    lambda_label.clone(),
                    Symbol {
                        address: rom.position(),
                        is_sublabel: false,
                        parent_label: None,
                    },
                );

                       // Reference address is the position of the placeholder (JCN's address field)
                let ref_addr = position;
                // rom.write_short_at(ref_addr, position+len as u16)?;
                self.references.push(Reference {
                    name: lambda_label.clone(),
                    rune: '?',
                    address: ref_addr,
                    line: self.line_number,
                    path: rom.source_path().cloned().unwrap_or_default(),
                    scope: self.current_label.clone(),
                    token: None,
                });

                // Do not update current_label here
            }
            AstNode::Padding(pad_addr) => {
                // If this is a |xxxx padding (not $xx skip), reset scope if at or above 0x0100
                if *pad_addr >= 0x0100 {
                    // Reset label scope after device header or |0100, like uxnasm
                    // But also allow sublabels to be defined at the start of a file (before any label)
                    if self.current_label.is_none() {
                        // Allow sublabels at the very start (e.g., device headers)
                        self.current_label = Some(String::new());
                    } else {
                        self.current_label = None;
                    }
                }
                rom.pad_to(*pad_addr)?;
            }
            AstNode::Byte(byte) => {
                rom.write_byte(*byte)?;
                if *byte != 0 {
                    self.update_effective_length(rom);
                }
            }
            AstNode::Short(short) => {
                rom.write_short(*short)?;
                if *short != 0 {
                    self.update_effective_length(rom);
                }
            }
            AstNode::LiteralByte(byte) => {
                // Only emit LIT for explicit byte literals (#xx)
                rom.write_byte(0x80)?; // LIT opcode (always non-zero)
                self.update_effective_length(rom);
                rom.write_byte(*byte)?;
                // Always update effective length for literal bytes, even if zero
                self.update_effective_length(rom);
            }
            AstNode::LiteralShort(short) => {
                // Only emit LIT2 for explicit short literals (#xxxx)
                rom.write_byte(0xa0)?; // LIT2 opcode (always non-zero)
                self.update_effective_length(rom);
                rom.write_short(*short)?;
                // Always update effective length for literal shorts, even if zero
                self.update_effective_length(rom);
            }
            AstNode::Instruction(inst) => {
                eprintln!(
                    "DEBUG: Processing instruction: '{}' at address {:04X}",
                    inst.opcode,
                    rom.position()
                );
                // Remove special-case for <...> labels: treat any unknown instruction as JSR reference
                match self.opcodes.get_opcode(&inst.opcode) {
                    Ok(base_opcode) => {
                        let final_opcode = Opcodes::apply_modes(
                            base_opcode,
                            inst.short_mode,
                            inst.return_mode,
                            inst.keep_mode,
                        );
                        rom.write_byte(final_opcode)?;
                        eprintln!(
                            "DEBUG: Wrote opcode 0x{:02X} ({}) at {:04X}",
                            final_opcode,
                            inst.opcode,
                            rom.position() - 1
                        );
                        if final_opcode != 0 {
                            self.update_effective_length(rom);
                        }
                    }
                    Err(_) => {
                        eprintln!(
                            "DEBUG: Creating JSR reference for unknown opcode: '{}'",
                            inst.opcode
                        );
                        self.references.push(Reference {
                            name: inst.opcode.clone(),
                            rune: ' ',
                            address: rom.position() + 1,
                            line: self.line_number,
                            path: path.clone(),
                            scope: self.current_label.clone(),
                            token: None,
                        });
                        rom.write_byte(0x60)?; // JSR opcode
                        eprintln!(
                            "DEBUG: Wrote JSR opcode 0x60 at {:04X}",
                            rom.position() - 1
                        );
                        self.update_effective_length(rom);
                        rom.write_short(0xffff)?; // Placeholder
                        eprintln!(
                            "DEBUG: Wrote JSR placeholder 0xFFFF at {:04X}-{:04X}",
                            rom.position() - 2,
                            rom.position() - 1
                        );
                        self.update_effective_length(rom);
                    }
                }
                // --- Only expand macro if found and not handled as instruction ---
                if let Some(macro_def) = self.macros.get(&inst.opcode).cloned() {
                    for macro_node in &macro_def.body {
                        self.process_node(macro_node, rom)?;
                    }
                }
            }
            AstNode::LabelDef(label) => {
                // NOTE: The label should be defined at the current ROM position,
                // which should be AFTER all code/data that precedes it.
                // If this is the last label in the file, it should point to the address
                // after the last byte written (i.e., rom.position()).
                // If you emit the label before writing the last data, you may need to add 1.
                if !self.symbols.contains_key(label) {
                    let address = if label == "program" {
                        rom.position() + 1
                    } else {
                        rom.position()
                    };
                    self.symbols.insert(
                        label.clone(),
                        Symbol {
                            address,
                            is_sublabel: label.contains('/'),
                            parent_label: label.rsplitn(2, '/').nth(1).map(|s| s.to_string()),
                        },
                    );
                }
                self.current_label = Some(label.clone());
                eprintln!(
                    "DEBUG: Defined label '{}' at address {:04X}",
                    label,
                    rom.position()
                );
            }
            AstNode::SublabelDef(sublabel) => {
                // --- FIX: Always register sublabel as <main_label>/<sublabel> ---
                let full_name = if let Some(ref parent) = self.current_label {
                    // If current_label is empty or whitespace, treat as global sublabel (for device headers)
                    if parent.trim().is_empty() {
                        sublabel.clone()
                    } else {
                        // Use only the main label (before first '/')
                        let main_label = if let Some(slash) = parent.find('/') {
                            &parent[..slash]
                        } else {
                            parent.as_str()
                        };
                        format!("{}/{}", main_label, sublabel)
                    }
                } else {
                    // If no current_label, treat as global sublabel (for device headers)
                    sublabel.clone()
                };
                if !self.symbols.contains_key(&full_name) {
                    self.symbols.insert(
                        full_name.clone(),
                        Symbol {
                            address: rom.position(),
                            is_sublabel: true,
                            parent_label: if full_name.contains('/') {
                                Some(full_name[..full_name.rfind('/').unwrap()].to_string())
                            } else {
                                None
                            },
                        },
                    );
                }
                // Do NOT update current_label for sublabels (matches uxnasm setscope=0)
                eprintln!(
                    "DEBUG: Defined sublabel '{}' at address {:04X}",
                    full_name,
                    rom.position()
                );
            }
            AstNode::LabelRef(tok) => {
                // Always treat label references as JSR, never as LIT/LIT2
                let label = if let crate::lexer::Token::LabelRef(s) = &tok.token {
                    s.clone()
                } else {
                    println!("DEBUG: Expected LabelRef, found {:?}", tok);
                    if let crate::lexer::Token::Newline = &tok.token {
                        // If it's a newline, just continue (ignore)
                        return Ok(());
                    }
                    return Err(AssemblerError::SyntaxError {
                        path: path.clone(),
                        line: self.line_number,
                        position: self.position_in_line,
                        message: format!("Expected LabelRef, found {:?}", tok.token),
                        source_line: String::new(),
                    });
                };
                self.references.push(Reference {
                    name: label,
                    rune: ' ',
                    address: rom.position(),
                    line: tok.line,
                    path: path.clone(),
                    scope: tok.scope.clone(),
                    token: Some(tok.clone()),
                });
                rom.write_byte(0x60)?; // JSR opcode
                self.update_effective_length(rom);
                rom.write_short(0xffff)?; // Placeholder for relative address
                self.update_effective_length(rom);
            }
            AstNode::SublabelRef(tok) => {
                let sublabel = match &tok.token {
                    crate::lexer::Token::SublabelRef(s) => s.clone(),
                    _ => return Err(AssemblerError::SyntaxError {
                        path: path.clone(),
                        line: self.line_number,
                        position: self.position_in_line,
                        message: "Expected SublabelRef".to_string(),
                        source_line: String::new(),
                    }),
                };
                let full_name = if let Some(ref parent) = self.current_label {
                    format!("{}/{}", parent, sublabel)
                } else {
                    return Err(AssemblerError::SyntaxError {
                        path: path.clone(),
                        line: self.line_number,
                        position: self.position_in_line,
                        message: "Sublabel reference outside of label scope".to_string(),
                        source_line: String::new(),
                    });
                };
                self.references.push(Reference {
                    name: full_name,
                    rune: '_',
                    address: rom.position(),
                    line: tok.line,
                    path: path.clone(),
                    scope: tok.scope.clone(),
                    token: Some(tok.clone()),
                });
                rom.write_byte(0xff)?;
            }
            AstNode::RelativeRef(tok) => {
                let label = match &tok.token {
                    crate::lexer::Token::RelativeRef(s) => s.clone(),
                    _ => return Err(AssemblerError::SyntaxError {
                        path: path.clone(),
                        line: self.line_number,
                        position: self.position_in_line,
                        message: "Expected RelativeRef".to_string(),
                        source_line: String::new(),
                    }),
                };
                self.references.push(Reference {
                    name: label,
                    rune: '/',
                    address: rom.position() + 1,
                    line: tok.line,
                    path: path.clone(),
                    scope: tok.scope.clone(),
                    token: Some(tok.clone()),
                });
                rom.write_byte(0x60)?;
                rom.write_short(0xffff)?;
            }
            AstNode::ConditionalRef(tok) => {
                let label = match &tok.token {
                    crate::lexer::Token::ConditionalRef(s) => s.clone(),
                    _ => return Err(AssemblerError::SyntaxError {
                        path: path.clone(),
                        line: self.line_number,
                        position: self.position_in_line,
                        message: "Expected ConditionalRef".to_string(),
                        source_line: String::new(),
                    }),
                };
                self.references.push(Reference {
                    name: label,
                    rune: '?',
                    address: rom.position() + 1,
                    line: tok.line,
                    path: path.clone(),
                    scope: tok.scope.clone(),
                    token: Some(tok.clone()),
                });
                rom.write_byte(0x20)?;
                rom.write_short(0xffff)?;
            }
            AstNode::DotRef(tok) => {
                let label = match &tok.token {
                    crate::lexer::Token::DotRef(s) => s.clone(),
                    _ => return Err(AssemblerError::SyntaxError {
                        path: path.clone(),
                        line: self.line_number,
                        position: self.position_in_line,
                        message: "Expected DotRef".to_string(),
                        source_line: String::new(),
                    }),
                };
                self.references.push(Reference {
                    name: label,
                    rune: '.',
                    address: rom.position() + 1,
                    line: tok.line,
                    path: path.clone(),
                    scope: tok.scope.clone(),
                    token: Some(tok.clone()),
                });
                rom.write_byte(0x80)?;
                self.update_effective_length(rom);
                rom.write_byte(0xff)?;
                self.update_effective_length(rom);
            }
            AstNode::SemicolonRef(tok) => {
                let label = match &tok.token {
                    crate::lexer::Token::SemicolonRef(s) => s.clone(),
                    _ => return Err(AssemblerError::SyntaxError {
                        path: path.clone(),
                        line: self.line_number,
                        position: self.position_in_line,
                        message: "Expected SemicolonRef".to_string(),
                        source_line: String::new(),
                    }),
                };
                self.references.push(Reference {
                    name: label,
                    rune: ';',
                    address: rom.position() + 1,
                    line: tok.line,
                    path: path.clone(),
                    scope: tok.scope.clone(),
                    token: Some(tok.clone()),
                });
                rom.write_byte(0xa0)?;
                self.update_effective_length(rom);
                rom.write_short(0xffff)?;
                self.update_effective_length(rom);
            }
            AstNode::EqualsRef(tok) => {
                let label = match &tok.token {
                    crate::lexer::Token::EqualsRef(s) => s.clone(),
                    _ => return Err(AssemblerError::SyntaxError {
                        path: path.clone(),
                        line: self.line_number,
                        position: self.position_in_line,
                        message: "Expected EqualsRef".to_string(),
                        source_line: String::new(),
                    }),
                };
                self.references.push(Reference {
                    name: label,
                    rune: '=',
                    address: rom.position(),
                    line: tok.line,
                    path: path.clone(),
                    scope: tok.scope.clone(),
                    token: Some(tok.clone()),
                });
                rom.write_short(0xffff)?;
                self.update_effective_length(rom);
            }
            AstNode::CommaRef(tok) => {
                let label = match &tok.token {
                    crate::lexer::Token::CommaRef(s) => s.clone(),
                    _ => return Err(AssemblerError::SyntaxError {
                        path: path.clone(),
                        line: self.line_number,
                        position: self.position_in_line,
                        message: "Expected CommaRef".to_string(),
                        source_line: String::new(),
                    }),
                };
                self.references.push(Reference {
                    name: label,
                    rune: ',',
                    address: rom.position() + 1,
                    line: tok.line,
                    path: path.clone(),
                    scope: tok.scope.clone(),
                    token: Some(tok.clone()),
                });
                rom.write_byte(0x80)?;
                self.update_effective_length(rom);
                rom.write_byte(0xff)?;
                self.update_effective_length(rom);
            }
            AstNode::UnderscoreRef(tok) => {
                let label = match &tok.token {
                    crate::lexer::Token::UnderscoreRef(s) => s.clone(),
                    _ => return Err(AssemblerError::SyntaxError {
                        path: path.clone(),
                        line: self.line_number,
                        position: self.position_in_line,
                        message: "Expected UnderscoreRef".to_string(),
                        source_line: String::new(),
                    }),
                };
                self.references.push(Reference {
                    name: label,
                    rune: '_',
                    address: rom.position(),
                    line: tok.line,
                    path: path.clone(),
                    scope: tok.scope.clone(),
                    token: Some(tok.clone()),
                });
                rom.write_byte(0xff)?;
                self.update_effective_length(rom);
            }
            AstNode::QuestionRef(tok) => {
                let label = match &tok.token {
                    crate::lexer::Token::QuestionRef(s) => s.clone(),
                    _ => return Err(AssemblerError::SyntaxError {
                        path: path.clone(),
                        line: self.line_number,
                        position: self.position_in_line,
                        message: "Expected QuestionRef".to_string(),
                        source_line: String::new(),
                    }),
                };
                self.references.push(Reference {
                    name: label,
                    rune: '?',
                    address: rom.position() + 1,
                    line: tok.line,
                    path: path.clone(),
                    scope: tok.scope.clone(),
                    token: Some(tok.clone()),
                });
                rom.write_byte(0x20)?;
                self.update_effective_length(rom);
                rom.write_short(0xffff)?;
                self.update_effective_length(rom);
            }
            AstNode::ExclamationRef(tok) => {
                let label = match &tok.token {
                    crate::lexer::Token::ExclamationRef(s) => s.clone(),
                    _ => return Err(AssemblerError::SyntaxError {
                        path: path.clone(),
                        line: self.line_number,
                        position: self.position_in_line,
                        message: "Expected ExclamationRef".to_string(),
                        source_line: String::new(),
                    }),
                };
                let resolved_name = if label.starts_with('/') {
                    let clean_label = &label[1..];
                    if let Some(ref scope) = tok.scope {
                        let main_scope = if let Some(slash_pos) = scope.find('/') {
                            &scope[..slash_pos]
                        } else {
                            scope
                        };
                        format!("{}/{}", main_scope, clean_label)
                    } else {
                        clean_label.to_string()
                    }
                } else {
                    label
                };
                self.references.push(Reference {
                    name: resolved_name,
                    rune: '!',
                    address: rom.position() + 1,
                    line: tok.line,
                    path: path.clone(),
                    scope: tok.scope.clone(),
                    token: Some(tok.clone()),
                });
                rom.write_byte(0x40)?;
                self.update_effective_length(rom);
                rom.write_short(0xffff)?;
                self.update_effective_length(rom);
            }
            AstNode::RawAddressRef(tok) => {
                let label = match &tok.token {
                    crate::lexer::Token::RawAddressRef(s) => s.clone(),
                    _ => return Err(AssemblerError::SyntaxError {
                        path: path.clone(),
                        line: self.line_number,
                        position: self.position_in_line,
                        message: "Expected RawAddressRef".to_string(),
                        source_line: String::new(),
                    }),
                };
                self.references.push(Reference {
                    name: label,
                    rune: '=',
                    address: rom.position(),
                    line: tok.line,
                    path: path.clone(),
                    scope: tok.scope.clone(),
                    token: Some(tok.clone()),
                });
                rom.write_short(0xffff)?;
            }
            AstNode::JSRRef(tok) => {
                let label = match &tok.token {
                    crate::lexer::Token::JSRRef(s) => s.clone(),
                    _ => return Err(AssemblerError::SyntaxError {
                        path: path.clone(),
                        line: self.line_number,
                        position: self.position_in_line,
                        message: "Expected JSRRef".to_string(),
                        source_line: String::new(),
                    }),
                };
                self.references.push(Reference {
                    name: label,
                    rune: '!',
                    address: rom.position() + 1,
                    line: tok.line,
                    path: path.clone(),
                    scope: tok.scope.clone(),
                    token: Some(tok.clone()),
                });
                rom.write_byte(0x60)?;
                rom.write_short(0xffff)?;
            }
            AstNode::HyphenRef(tok) => {
                let label = match &tok.token {
                    crate::lexer::Token::HyphenRef(s) => s.clone(),
                    _ => return Err(AssemblerError::SyntaxError {
                        path: path.clone(),
                        line: self.line_number,
                        position: self.position_in_line,
                        message: "Expected HyphenRef".to_string(),
                        source_line: String::new(),
                    }),
                };
                self.references.push(Reference {
                    name: label,
                    rune: '-',
                    address: rom.position(),
                    line: tok.line,
                    path: path.clone(),
                    scope: tok.scope.clone(),
                    token: Some(tok.clone()),
                });
                rom.write_byte(0xff)?;
            }
            AstNode::PaddingLabel(tok) => {
                let label = match &tok.token {
                    crate::lexer::Token::PaddingLabel(s) => s.clone(),
                    _ => return Err(AssemblerError::SyntaxError {
                        path: path.clone(),
                        line: self.line_number,
                        position: self.position_in_line,
                        message: "Expected PaddingLabel".to_string(),
                        source_line: String::new(),
                    }),
                };
                if let Some(symbol) = self.symbols.get(&label) {
                    rom.pad_to(symbol.address)?;
                } else {
                    return Err(AssemblerError::SyntaxError {
                        path: rom.source_path().cloned().unwrap_or_default(),
                        line: self.line_number,
                        position: self.position_in_line,
                        message: format!("Padding label '{}' not found", label),
                        source_line: rom
                            .source()
                            .map(|s| s.lines().nth(self.line_number).unwrap_or("").to_string())
                            .unwrap_or_default(),
                    });
                }
            }
            AstNode::Skip(count) => {
                for _ in 0..*count {
                    rom.write_byte(0)?;
                    // Don't update effective length for zero bytes
                }
            }
            AstNode::MacroDef(name, body) => {
                // Store macro definition
                self.macros.insert(
                    name.clone(),
                    Macro {
                        name: name.clone(),
                        body: body.clone(),
                    },
                );
            }
            AstNode::MacroCall(name, macro_line, macro_position) => {
                // Expand macro inline
                // If referencing '_', register <current_label>/_ as a sublabel if not already present
                if name == "_" {
                    if let Some(ref parent) = self.current_label {
                        let scoped = format!("{}/_", parent);
                        if !self.symbols.contains_key(&scoped) {
                            self.symbols.insert(
                                scoped.clone(),
                                Symbol {
                                    address: rom.position(),
                                    is_sublabel: true,
                                    parent_label: Some(parent.clone()),
                                },
                            );
                        }
                    }
                }
                if let Some(macro_def) = self.macros.get(name).cloned() {
                    for macro_node in &macro_def.body {
                        self.process_node(macro_node, rom)?;
                    }
                } else {
                    // If macro is not defined, treat as JSR reference (matches uxnasm for <pdec>)
                    self.references.push(Reference {
                        name: name.clone(),
                        rune: ' ',
                        address: rom.position() + 1,
                        line: self.line_number,
                        path: rom.source_path().cloned().unwrap_or_default(),
                        scope: self.current_label.clone(),
                        token: None,
                    });
                    rom.write_byte(0x60)?; // JSR opcode
                    self.update_effective_length(rom);
                    rom.write_short(0xffff)?; // Placeholder
                    self.update_effective_length(rom);
                }
            }
            AstNode::RawString(bytes) => {
                // Write string data byte by byte, updating effective length for each non-zero byte
                for &byte in bytes {
                    rom.write_byte(byte)?;
                    if byte != 0 {
                        self.update_effective_length(rom);
                    }
                }
            }
            AstNode::Include(path) => {
                // Save/restore current_label around includes
                let saved_label = self.current_label.clone();
                self.process_include(path, rom)?;
                self.current_label = saved_label;
            }
        }
        Ok(())
    }

    fn second_pass(&mut self, rom: &mut Rom) -> Result<()> {
        // Debug: print available symbols like WSL does
        if true { // Enable debug output
            println!("DEBUG: Available labels ({}):", self.symbols.len());
            let mut symbols: Vec<_> = self.symbols.iter().collect();
            symbols.sort_by_key(|(_, symbol)| symbol.address);
            for (i, (name, symbol)) in symbols.iter().enumerate() {
                println!("  [{}] '{}' -> 0x{:04X}", i, name, symbol.address);
            }
        }

        for reference in &self.references {
            // Handle '/' rune by resolving scope like uxnasm.c
            let resolved_name = if reference.rune == '/' {
                if let Some(ref scope) = reference.scope {
                    // Extract the main label part (before any '/')
                    let main_scope = if let Some(slash_pos) = scope.find('/') {
                        &scope[..slash_pos]
                    } else {
                        scope
                    };
                    // Preserve angle brackets and add scope - don't strip them
                    format!("{}/{}", main_scope, reference.name)
                } else {
                    reference.name.clone()
                }
            } else {
                reference.name.clone()
            };

            let symbol = self.find_symbol(&resolved_name, reference.scope.as_ref());
            println!("DEBUG: Processing reference: {:?}", reference);
            println!(
                "DEBUG: Resolving reference '{}' -> '{}' at {:04X} (scope: {:?})",
                reference.name, resolved_name, reference.address, reference.scope
            );
            println!("DEBUG: Found symbol: {:?}", symbol);

            // --- PATCH: skip error for instruction-like unresolved references ---
            let is_possible_instruction = {
                let mut base = resolved_name.as_str();
                while let Some(last) = base.chars().last() {
                    if last == 'k' || last == 'r' || last == '2' {
                        base = &base[..base.len() - 1];
                    } else {
                        break;
                    }
                }
                matches!(
                    base,
                    "ADD" | "SUB" | "MUL" | "DIV" | "AND" | "ORA" | "EOR" | "SFT"
                        | "LDZ" | "STZ" | "LDR" | "STR" | "LDA" | "STA" | "DEI" | "DEO"
                        | "INC" | "POP" | "NIP" | "SWP" | "ROT" | "DUP" | "OVR"
                        | "EQU" | "NEQ" | "GTH" | "LTH" | "JMP" | "JCN" | "JSR" | "STH"
                        | "BRK" | "LIT" | "LIT2" | "LITr" | "LIT2r"
                )
            };

            let symbol = if symbol.is_none() {
                // --- PATCH: uxnasm-style scope walk for _ and , runes, even if tokens don't store scope ---
                if reference.rune == '_' || reference.rune == ',' {
                    // Try walking up the scope chain from the enclosing label scope
                    let mut found = None;
                    let mut scope = reference.scope.clone();
                    while let Some(ref s) = scope {
                        let candidate = format!("{}/{}", s, reference.name);
                        if let Some(sym) = self.symbols.get(&candidate) {
                            found = Some(sym);
                            break;
                        }
                        // Walk up to parent scope (remove last / segment)
                        if let Some(last_slash) = s.rfind('/') {
                            scope = Some(s[..last_slash].to_string());
                        } else {
                            scope = None;
                        }
                    }
                    // If not found, try just the name as a global label
                    if found.is_none() {
                        self.symbols.get(&reference.name)
                    } else {
                        found
                    }
                } else {
                    // For all other runes, only try the full name
                    self.symbols.get(&resolved_name)
                }
            } else {
                symbol
            };

            if symbol.is_none() {
                // If this is a reference for an instruction (not a label), skip error
                if reference.rune == ' ' && is_possible_instruction {
                    continue;
                }
                // Debug: print all available symbols when we can't find one
                eprintln!("Available symbols:");
                for (name, sym) in &self.symbols {
                    eprintln!("  {} -> {:04X}", name, sym.address);
                }
                eprintln!(
                    "Looking for: '{}' in scope: {:?}",
                    resolved_name, reference.scope
                );

                let source_line = rom
                    .source()
                    .and_then(|src| {
                        if reference.line > 0 {
                            src.lines().nth(reference.line - 1).map(|s| s.to_string())
                        } else {
                            None
                        }
                    })
                    .unwrap_or_default();

                let message = if is_possible_instruction {
                    format!(
                        "'{}' is not a label, but looks like an instruction. Did you mean to use it as an instruction?",
                        resolved_name
                    )
                } else {
                    format!("Label unknown: {}", resolved_name)
                };

                return Err(AssemblerError::SyntaxError {
                    path: reference.path.clone(),
                    line: reference.line,
                    position: 0,
                    message,
                    source_line,
                });
            }

            let symbol = symbol.unwrap();

            // Apply the reference based on the rune type (following uxnasm.c exactly)
            // From uxnasm.c resolve() function
            match reference.rune {
                '_' | ',' => {
                    // case '_': case ',': *rom = rel = l->addr - r->addr - 2;
                    let rel = (symbol.address as i32 - reference.address as i32 - 2) as i8;
                    rom.write_byte_at(reference.address, rel as u8)?;
                    eprintln!("DEBUG: Resolved reference '{}' at {:04X}: wrote relative offset 0x{:02X} ({})", 
                             resolved_name, reference.address, rel as u8, rel);
                    // Update effective length if resolved value is non-zero
                    if rel as u8 != 0 {
                        self.effective_length =
                            self.effective_length.max(reference.address as usize + 1);
                    }
                    // Range check like uxnasm.c: if((Sint8)data[r->addr] != rel)
                    if rel != (rel as u8 as i8) {
                        return Err(AssemblerError::SyntaxError {
                            path: reference.path.clone(),
                            line: reference.line,
                            position: 0,
                            message: "Reference too far".to_string(),
                            source_line: String::new(),
                        });
                    }
                }
                '-' | '.' => {
                    // case '-': case '.': *rom = l->addr;
                    rom.write_byte_at(reference.address, symbol.address as u8)?;
                    eprintln!(
                        "DEBUG: Resolved reference '{}' at {:04X}: wrote address 0x{:02X}",
                        reference.name, reference.address, symbol.address as u8
                    );
                    // Update effective length if resolved value is non-zero
                    if symbol.address as u8 != 0 {
                        self.effective_length =
                            self.effective_length.max(reference.address as usize + 1);
                    }
                }
                ':' | '=' | ';' => {

                    //                     // Write absolute ROM address (uxnasm.c starts ROM at PAGE)
                    // let absolute_addr = symbol.address + 0x0100;
                    // rom.write_byte_at(reference.address, (absolute_addr >> 8) as u8)?;
                    // rom.write_byte_at(reference.address + 1, (absolute_addr & 0xff) as u8)?;
                    // eprintln!(
                    //     "DEBUG: Resolved reference '{}' at {:04X}: wrote address 0x{:04X} (absolute)",
                    //     reference.name, reference.address, absolute_addr
                    // );
                    // // Update effective length - address references are typically non-zero
                    // if absolute_addr != 0 {
                    //     self.effective_length =
                    //         self.effective_length.max(reference.address as usize + 2);
                    // }

                    // Write raw ROM address (no offset)
                    rom.write_byte_at(reference.address, (symbol.address >> 8) as u8)?;
                    rom.write_byte_at(reference.address + 1, (symbol.address & 0xff) as u8)?;
                    eprintln!(
                        "DEBUG: Resolved reference '{}' at {:04X}: wrote address 0x{:04X} (no offset)",
                        reference.name, reference.address, symbol.address
                    );
                    // Update effective length - address references are typically non-zero
                    if symbol.address != 0 {
                        self.effective_length =
                            self.effective_length.max(reference.address as usize + 2);
                    }
                }
                '!' => {
                    // Fix: match uxnasm.c, use rel = target_addr - ref_addr - 2
                    let rel = (symbol.address as i32 - reference.address as i32-2) as i16;
                    rom.write_byte_at(reference.address, (rel >> 8) as u8)?;
                    rom.write_byte_at(reference.address + 1, (rel & 0xff) as u8)?;
                    eprintln!("DEBUG: Resolved reference '{}' at {:04X}: wrote relative address 0x{:04X} ({})", 
                             reference.name, reference.address, rel as u16, rel);
                    if rel != 0 {
                        self.effective_length =
                            self.effective_length.max(reference.address as usize + 2);
                    }
                }
                '?' | ' ' | '/' => {
                    // For conditional ('?'), space (' '), and slash ('/') runes:
                    // rel = target_addr - ref_addr - 2 (matches uxnasm for relative word references)
                    let rel = (symbol.address as i32 - reference.address as i32 - 2) as i16;
                    rom.write_byte_at(reference.address, (rel >> 8) as u8)?;
                    rom.write_byte_at(reference.address + 1, (rel & 0xff) as u8)?;
                    eprintln!("DEBUG: Resolved reference '{}' at {:04X}: wrote relative address 0x{:04X} ({})", 
                             reference.name, reference.address, rel as u16, rel);
                    if rel != 0 {
                        self.effective_length =
                            self.effective_length.max(reference.address as usize + 2);
                    }
                }
                _ => {
                    return Err(AssemblerError::SyntaxError {
                        path: reference.path.clone(),
                        line: reference.line,
                        position: 0,
                        message: format!("Unknown reference rune: {}", reference.rune),
                        source_line: String::new(),
                    });
                }
            }
        }
        Ok(())
    }

    fn find_symbol(&self, name: &str, reference_scope: Option<&String>) -> Option<&Symbol> {
        eprintln!(
            "DEBUG: find_symbol called with name='{}', reference_scope={:?}",
            name, reference_scope
        );
        eprintln!("DEBUG: current_label={:?}", self.current_label);

        // Handle sublabel references with & prefix
        if name.starts_with('&') {
            let sublabel_name = &name[1..];
            eprintln!("DEBUG: Looking for sublabel '{}'", sublabel_name);

            // First try with the reference's scope context
            if let Some(scope) = reference_scope {
                // Extract the main label part (before any '/')
                let main_scope = if let Some(slash_pos) = scope.find('/') {
                    &scope[..slash_pos]
                } else {
                    scope
                };
                let scoped = format!("{}/{}", main_scope, sublabel_name);
                eprintln!("DEBUG: Trying main scope lookup: '{}'", scoped);
                if let Some(symbol) = self.symbols.get(&scoped) {
                    eprintln!("DEBUG: Found main scope symbol: {:?}", symbol);
                    return Some(symbol);
                }
            }

            // Fallback to current label scope
            if let Some(ref current) = self.current_label {
                // Extract the main label part (before any '/')
                let main_current = if let Some(slash_pos) = current.find('/') {
                    &current[..slash_pos]
                } else {
                    current
                };
                let scoped = format!("{}/{}", main_current, sublabel_name);
                eprintln!("DEBUG: Trying current main scope lookup: '{}'", scoped);
                if let Some(symbol) = self.symbols.get(&scoped) {
                    eprintln!("DEBUG: Found current main scope symbol: {:?}", symbol);
                    return Some(symbol);
                }
            }

            // Try global scope (just the sublabel name without &)
            eprintln!("DEBUG: Trying global lookup: '{}'", sublabel_name);
            if let Some(symbol) = self.symbols.get(sublabel_name) {
                eprintln!("DEBUG: Found global symbol: {:?}", symbol);
                return Some(symbol);
            }
        }

        // --- FIX: Only walk up the scope chain for _ and , runes ---
        // This matches uxnasm's scope resolution for sublabels.
        // We need to know the rune type, so this logic should only be used for those runes.
        // Instead, move the scope-walk logic out of find_symbol and only use it in second_pass for _ and , runes.
        // Here, just do direct lookup.
        if let Some(symbol) = self.symbols.get(name) {
            return Some(symbol);
        }

        // Try with angle brackets for hierarchical lookups
        if !name.starts_with('<') && !name.ends_with('>') {
            let bracketed = format!("<{}>", name);
            if let Some(symbol) = self.symbols.get(&bracketed) {
                return Some(symbol);
            }
        }

        if name.starts_with('<') && name.ends_with('>') && name.len() > 2 {
            let unbracketed = &name[1..name.len() - 1];
            if let Some(symbol) = self.symbols.get(unbracketed) {
                return Some(symbol);
            }
        }

        None
    }

    /// Process an include directive by reading and assembling the included file
    fn process_include(&mut self, path: &str, rom: &mut Rom) -> Result<()> {
        // Read the included file
        let content = match fs::read_to_string(path) {
            Ok(content) => content,
            Err(e) => {
                return Err(AssemblerError::SyntaxError {
                    path: rom.source_path().cloned().unwrap_or_default(),
                    line: self.line_number,
                    position: self.position_in_line,
                    message: format!("Failed to read include file '{}': {}", path, e),
                    source_line: String::new(),
                });
            }
        };

        // Scan the included file for device headers and merge into device_map
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with('|') {
                let mut parts = line.split_whitespace();
                let addr_part = parts.next();
                let label_part = parts.next();
                if let (Some(addr), Some(label)) = (addr_part, label_part) {
                    if label.starts_with('@') {
                        let mut device = label[1..].to_string();
                        if let Some(slash_pos) = device.find('/') {
                            device = device[..slash_pos].to_string();
                        }
                        let base_addr = u16::from_str_radix(&addr[1..], 16).unwrap_or(0);
                        let mut offset = 0u16;
                        let mut iter = parts;
                        // Register the device label itself
                        if !self.symbols.contains_key(&device) {
                            self.symbols.insert(
                                device.clone(),
                                Symbol {
                                    address: base_addr,
                                    is_sublabel: false,
                                    parent_label: None,
                                },
                            );
                        }
                        // Register each field as a sublabel with correct offset
                        while let Some(field_name) = iter.next() {
                            let size_str = iter.next();
                            let size = if let Some(size_str) = size_str {
                                if let Ok(sz) = size_str.parse::<u16>() {
                                    sz
                                } else {
                                    1
                                }
                            } else {
                                1
                            };
                            let sublabel = format!("{}/{}", device, field_name);
                            if !self.symbols.contains_key(&sublabel) {
                                self.symbols.insert(
                                    sublabel.clone(),
                                    Symbol {
                                        address: base_addr + offset,
                                        is_sublabel: true,
                                        parent_label: Some(device.clone()),
                                    },
                                );
                            }
                            offset += size;
                        }
                    }
                }
            }
        }

        // Lex the included file
        let mut lexer = Lexer::new(content.clone(), Some(path.to_string()));
        let tokens = lexer.tokenize()?;

        // Parse the included file
        let mut parser = Parser::new_with_source(tokens, path.to_string(), content);
        let ast = parser.parse()?;

        // Set the ROM path to the included file path for error context
        rom.set_path(Some(path.to_string()));

        // Process the included AST nodes in first pass
        for node in ast {
            self.process_node(&node, rom)?;
        }

        Ok(())
    }
}

impl Default for Assembler {
    fn default() -> Self {
        Self::new()

    }
}