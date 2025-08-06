//! Main assembler implementation

use crate::devicemap::{parse_device_maps, Device, DeviceField};
use crate::error::{AssemblerError, Result};
use crate::lexer::Lexer;
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
        if self.effective_length > 256 {
            // Check if there's any non-zero data in the first 256 bytes
            let has_zero_page_data = rom_data[..256].iter().any(|&b| b != 0);

            if !has_zero_page_data {
                println!(
                    "Assembled {} in {} bytes({:.2}% used), {} labels, {} macros.",
                    path.clone().unwrap_or_else(|| "(input)".to_string()),
                    self.effective_length - 256,
                    (self.effective_length - 256) as f64 / 652.80,
                    total_label_count,
                    self.macros.len()
                );
                return Ok(rom_data[256..self.effective_length].to_vec());
            }
        }
        Ok(rom_data[..self.effective_length].to_vec())
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
                rom.write_byte(0x80)?; // LIT opcode (always non-zero)
                self.update_effective_length(rom);
                rom.write_byte(*byte)?;
                // Always update effective length for literal bytes, even if zero
                self.update_effective_length(rom);
            }
            AstNode::LiteralShort(short) => {
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
                // Check if this is actually a macro call first
                if let Some(macro_def) = self.macros.get(&inst.opcode).cloned() {
                    // Expand macro inline
                    for macro_node in &macro_def.body {
                        self.process_node(macro_node, rom)?;
                    }
                } else {
                    // Handle special LIT instructions that have mode flags built in
                    let opcode = match inst.opcode.as_str() {
                        "LIT" => {
                            // Handle LIT with mode flags - this is for cases where parser 
                            // breaks down LIT2r into LIT + flags
                            let mut base_opcode = 0x80; // Base LIT opcode (0x80 = BRK + keep)
                            if inst.short_mode { base_opcode |= 0x20; } // Add short flag
                            if inst.return_mode { base_opcode |= 0x40; } // Add return flag
                            // keep_mode is already included in base LIT opcode
                            base_opcode
                        }
                        "LIT2" => 0xA0,  // LIT2 (BRK + short + keep flags)
                        "LITr" => 0xC0,  // LITr (BRK + return + keep flags)
                        "LIT2r" => 0xE0, // LIT2r (BRK + short + return + keep flags)
                        _ => {
                            // Try to get the opcode - if it fails, treat as implicit JSR call
                            match self.opcodes.get_opcode(&inst.opcode) {
                                Ok(base_opcode) => {
                                    // It's a regular instruction - apply mode flags
                                    let final_opcode = Opcodes::apply_modes(
                                        base_opcode,
                                        inst.short_mode,
                                        inst.return_mode,
                                        inst.keep_mode,
                                    );

                                    // Check if this is a STR instruction that should create storage for pending comma references
                                    if inst.opcode.starts_with("STR") {
                                        // Create storage for any pending comma reference that would target this position
                                        // This mimics uxnasm.c behavior where STR creates the storage location
                                        eprintln!("DEBUG: STR instruction at {:04X} - this creates storage", rom.position());
                                    }

                                    final_opcode
                                }
                                Err(_) => {
                                    // Unknown opcode - treat as implicit JSR call
                                    eprintln!(
                                        "DEBUG: Creating JSR reference for unknown opcode: '{}'",
                                        inst.opcode
                                    );
                                    // From uxnasm.c: return makeref(w, ' ', ptr + 1, ctx) && writebyte(0x60, ctx) && writeshort(0xffff);
                                    self.references.push(Reference {
                                        name: inst.opcode.clone(),
                                        rune: ' ',
                                        address: rom.position() + 1,
                                        line: self.line_number,
                                        path: path.clone(),
                                        scope: self.current_label.clone(), // Save current scope
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
                                    return Ok(());
                                }
                            }
                        }
                    };
                    rom.write_byte(opcode)?;
                    eprintln!(
                        "DEBUG: Wrote opcode 0x{:02X} ({}) at {:04X}",
                        opcode,
                        inst.opcode,
                        rom.position() - 1
                    );
                    // Only update effective length if opcode is non-zero (following uxnasm.c exactly)
                    if opcode != 0 {
                        self.update_effective_length(rom);
                    }
                }
            }
            AstNode::LabelDef(label) => {
                // Register label as-is, including '_' as a valid label
                if !self.symbols.contains_key(label) {
                    self.symbols.insert(
                        label.clone(),
                        Symbol {
                            address: rom.position(),
                            is_sublabel: false,
                            parent_label: self.current_label.clone(),
                        },
                    );
                }
                
                self.current_label = Some(label.clone());
                eprintln!(
                    "DEBUG: Defined label '{}' at address {:04X}",
                    label,
                    rom.position()
                );
                // NOTE: Labels don't advance the ROM position - the next instruction writes at the same address
            }
            AstNode::LabelRef(label) => {
                // Label reference without prefix - treat as implicit JSR (space rune)
                self.references.push(Reference {
                    name: label.clone(),
                    rune: ' ',
                    address: rom.position() + 1,
                    line: self.line_number,
                    path: path.clone(),
                    scope: self.current_label.clone(),
                });
                rom.write_byte(0x60)?; // JSR opcode
                rom.write_short(0xffff)?; // Placeholder
            }
            AstNode::SublabelDef(sublabel) => {
                // Register sublabel at the current ROM position (after any data written before this node)
                let full_name = if let Some(ref parent) = self.current_label {
                    format!("{}/{}", parent, sublabel)
                } else {
                    return Err(AssemblerError::SyntaxError {
                        path: path.clone(),
                        line: self.line_number,
                        position: self.position_in_line,
                        message: "Sublabel defined outside of label scope".to_string(),
                        source_line: rom
                            .source()
                            .map(|s| s.lines().nth(self.line_number).unwrap_or("").to_string())
                            .unwrap_or_default(),
                    });
                };
                let address = rom.position();
                if !self.symbols.contains_key(&full_name) {
                    self.symbols.insert(
                        full_name.clone(),
                        Symbol {
                            address,
                            is_sublabel: true,
                            parent_label: self.current_label.clone(),
                        },
                    );
                }
                eprintln!(
                    "DEBUG: Defined sublabel '{}' at address {:04X}",
                    full_name,
                    address
                );
                // Do NOT advance ROM position here!
            }
            AstNode::SublabelRef(sublabel) => {
                let full_name = if let Some(ref parent) = self.current_label {
                    format!("{}/{}", parent, sublabel)
                } else {
                    return Err(AssemblerError::SyntaxError {
                        path: path.clone(),
                        line: self.line_number,
                        position: self.position_in_line,
                        message: "Sublabel reference outside of label scope".to_string(),
                        source_line: rom
                            .source()
                            .map(|s| s.lines().nth(self.line_number).unwrap_or("").to_string())
                            .unwrap_or_default(),
                    });
                };
                self.references.push(Reference {
                    name: full_name,
                    rune: '_', // Sublabel references use underscore rune for relative addressing
                    address: rom.position(),
                    line: self.line_number,
                    path: path.clone(),
                    scope: self.current_label.clone(),
                });
                rom.write_byte(0xff)?; // Placeholder
            }
            AstNode::RelativeRef(label) => {
                // Store the reference with the '/' rune and let find_symbol handle resolution
                // This matches uxnasm.c behavior where '/' rune triggers special scope handling
                self.references.push(Reference {
                    name: label.clone(), // Store original label (without leading slash if present)
                    rune: '/', // Use '/' rune to trigger special handling in find_symbol
                    address: rom.position() + 1,
                    line: self.line_number,
                    path: path.clone(),
                    scope: self.current_label.clone(),
                });
                rom.write_byte(0x60)?; // JSR opcode
                rom.write_short(0xffff)?; // 16-bit placeholder
            }
            AstNode::ConditionalRef(label) => {
                // Conditional reference generates JCN followed by relative address
                self.references.push(Reference {
                    name: label.clone(),
                    rune: '?',
                    address: rom.position() + 1,
                    line: self.line_number,
                    path: path.clone(),
                    scope: self.current_label.clone(),
                });
                rom.write_byte(0x20)?; // JCN opcode
                rom.write_short(0xffff)?; // Placeholder
            }
            AstNode::ConditionalBlock(block_nodes) => {
                // WSL creates lambda labels for conditional blocks like uxnasm.c
                // Generate a unique lambda label name
                let lambda_name = format!("λ{:02}", self.lambda_counter);
                self.lambda_counter += 1;
                
                // Emit JCN (conditional jump - jumps if top of stack is zero)
                rom.write_byte(0x20)?; // JCN opcode
                
                // Create a reference to the lambda label for the jump target
                self.references.push(Reference {
                    name: lambda_name.clone(),
                    rune: '?',
                    address: rom.position(),
                    line: self.line_number,
                    path: path.clone(),
                    scope: self.current_label.clone(),
                });
                rom.write_short(0xffff)?; // Placeholder

                // Assemble the block content
                for node in block_nodes {
                    self.process_node(node, rom)?;
                }
                
                // Define the lambda label at the end of the block (like WSL does)
                if !self.symbols.contains_key(&lambda_name) {
                    self.symbols.insert(
                        lambda_name.clone(),
                        Symbol {
                            address: rom.position(),
                            is_sublabel: false,
                            parent_label: self.current_label.clone(),
                        },
                    );
                }
                
                eprintln!("DEBUG: Created lambda label '{}' at address {:04X}", lambda_name, rom.position());
            }
            AstNode::JSRRef(label) => {
                // JSR call generates JSR followed by relative address
                self.references.push(Reference {
                    name: label.clone(),
                    rune: '!',
                    address: rom.position() + 1,
                    line: self.line_number,
                    path: path.clone(),
                    scope: self.current_label.clone(),
                });
                rom.write_byte(0x60)?; // JSR opcode (changed from 0x8d to 0x60)
                rom.write_short(0xffff)?; // Placeholder
            }
            AstNode::RawAddressRef(label) => {
                // Raw address access - use equals rune for 16-bit address
                self.references.push(Reference {
                    name: label.clone(),
                    rune: '=',
                    address: rom.position(),
                    line: self.line_number,
                    path: path.clone(),
                    scope: self.current_label.clone(),
                });
                rom.write_short(0xffff)?; // Placeholder
            }
            AstNode::HyphenRef(identifier) => {
                // Hyphen reference uses the '-' rune for direct byte addressing
                self.references.push(Reference {
                    name: identifier.clone(),
                    rune: '-',
                    address: rom.position(),
                    line: self.line_number,
                    path: path.clone(),
                    scope: self.current_label.clone(),
                });
                rom.write_byte(0xff)?; // Placeholder
            }
            AstNode::Padding(addr) => {
                rom.pad_to(*addr)?;
            }
            AstNode::PaddingLabel(ref label) => {
                if let Some(symbol) = self.symbols.get(label) {
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
            AstNode::DeviceAccess(device, field) => {
                // Device access like .Screen/width should generate LIT + address
                let full_label = format!("{}/{}", device, field);
                self.references.push(Reference {
                    name: full_label,
                    rune: '.',
                    address: rom.position() + 1,
                    line: self.line_number,
                    path: path.clone(),
                    scope: self.current_label.clone(),
                });
                rom.write_byte(0x80)?; // LIT opcode (always non-zero)
                self.update_effective_length(rom);
                rom.write_byte(0xff)?; // Placeholder (non-zero)
                self.update_effective_length(rom);
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
                    let source_line = if let Some(source) = rom.source() {
                        source
                            .lines()
                            .nth(self.line_number)
                            .unwrap_or("")
                            .to_string()
                    } else {
                        String::new()
                    };
                    return Err(AssemblerError::SyntaxError {
                        path: path.clone(),
                        line: self.line_number,
                        position: self.position_in_line,
                        message: format!(
                            "Undefined macro: {} {}:{}",
                            name, macro_line, macro_position
                        ),
                        source_line,
                    });
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
                // Read and process the included file
                self.process_include(path, rom)?;
            }
            AstNode::DotRef(label) => {
                // Generate LIT + 8-bit address (like uxnasm's '.' rune)
                self.references.push(Reference {
                    name: label.clone(),
                    rune: '.',
                    address: rom.position() + 1,
                    line: self.line_number,
                    path: path.clone(),
                    scope: self.current_label.clone(),
                });
                rom.write_byte(0x80)?; // LIT opcode (always non-zero)
                self.update_effective_length(rom);
                rom.write_byte(0xff)?; // Placeholder (non-zero)
                self.update_effective_length(rom);
            }
            AstNode::SemicolonRef(label) => {
                // Generate LIT2 + 16-bit address (like uxnasm's ';' rune)
                self.references.push(Reference {
                    name: label.clone(),
                    rune: ';',
                    address: rom.position() + 1,
                    line: self.line_number,
                    path: path.clone(),
                    scope: self.current_label.clone(),
                });
                rom.write_byte(0xa0)?; // LIT2 opcode (always non-zero)
                self.update_effective_length(rom);
                rom.write_short(0xffff)?; // Placeholder (non-zero)
                self.update_effective_length(rom);
            }
            AstNode::EqualsRef(label) => {
                // Generate 16-bit address directly (like uxnasm's '=' rune)
                self.references.push(Reference {
                    name: label.clone(),
                    rune: '=',
                    address: rom.position(),
                    line: self.line_number,
                    path: path.clone(),
                    scope: self.current_label.clone(),
                });
                rom.write_short(0xffff)?; // Placeholder (non-zero)
                self.update_effective_length(rom);
            }
            AstNode::CommaRef(label) => {
                // Generate LIT + relative 8-bit address (like uxnasm's ',' rune)
                // In uxnasm.c: return makeref(w + 1, w[0], ptr + 1, ctx) && writebyte(0x80, ctx) && writebyte(0xff, ctx);
                // The reference address should point to the second byte (after LIT opcode)
                self.references.push(Reference {
                    name: label.clone(),
                    rune: ',',
                    address: rom.position() + 1, // Point to the byte after LIT opcode
                    line: self.line_number,
                    path: path.clone(),
                    scope: self.current_label.clone(),
                });
                rom.write_byte(0x80)?; // LIT opcode (always non-zero)
                self.update_effective_length(rom);
                rom.write_byte(0xff)?; // Placeholder (non-zero)
                self.update_effective_length(rom);
            }
            AstNode::UnderscoreRef(label) => {
                // Generate relative 8-bit address (like uxnasm's '_' rune)
                self.references.push(Reference {
                    name: label.clone(),
                    rune: '_',
                    address: rom.position(),
                    line: self.line_number,
                    path: path.clone(),
                    scope: self.current_label.clone(),
                });
                rom.write_byte(0xff)?; // Placeholder (non-zero)
                self.update_effective_length(rom);
            }
            AstNode::QuestionRef(label) => {
                // Generate conditional jump (like uxnasm's '?' rune)
                self.references.push(Reference {
                    name: label.clone(),
                    rune: '?',
                    address: rom.position() + 1,
                    line: self.line_number,
                    path: path.clone(),
                    scope: self.current_label.clone(),
                });
                rom.write_byte(0x20)?; // JCN opcode (always non-zero)
                self.update_effective_length(rom);
                rom.write_short(0xffff)?; // Placeholder (non-zero)
                self.update_effective_length(rom);
            }
            AstNode::ExclamationRef(label) => {
                // Generate JSR call (like uxnasm's '!' rune)
                // If the label starts with '/', treat it as a relative reference and apply scope resolution
                let resolved_name = if label.starts_with('/') {
                    // Handle relative reference within JSR - remove leading '/' and apply scope resolution
                    let clean_label = &label[1..];
                    if let Some(ref scope) = self.current_label {
                        // Extract the main label part (before any '/')
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
                    label.clone()
                };
                
                self.references.push(Reference {
                    name: resolved_name,
                    rune: '!',
                    address: rom.position() + 1,
                    line: self.line_number,
                    path: path.clone(),
                    scope: self.current_label.clone(),
                });
                rom.write_byte(0x40)?; // JSR opcode - uxnasm.c uses 0x40, not 0x60
                self.update_effective_length(rom);
                rom.write_short(0xffff)?; // Placeholder (non-zero)
                self.update_effective_length(rom);
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

            if symbol.is_none() {
                // Debug: print all available symbols when we can't find one
                eprintln!("Available symbols:");
                for (name, sym) in &self.symbols {
                    eprintln!("  {} -> {:04X}", name, sym.address);
                }
                eprintln!(
                    "Looking for: '{}' in scope: {:?}",
                    resolved_name, reference.scope
                );

                return Err(AssemblerError::SyntaxError {
                    path: reference.path.clone(),
                    line: reference.line,
                    position: 0,
                    message: format!("Label unknown: {}", resolved_name),
                    source_line: String::new(),
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

        // Direct lookup for non-sublabel references
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

        // Try without angle brackets
        if name.starts_with('<') && name.ends_with('>') && name.len() > 2 {
            let unbracketed = &name[1..name.len() - 1];
            if let Some(symbol) = self.symbols.get(unbracketed) {
                return Some(symbol);
            }
        }

        // Check for references with / rune (special handling)
        if name.contains('/') {
            let parts: Vec<_> = name.split('/').collect();
            if parts.len() == 2 {
                let main_label = parts[0];
                let sub_label = parts[1];

                // First, try to find the main label
                if let Some(main_symbol) = self.symbols.get(main_label) {
                    // If the main label is found, check for the sublabel within the same scope
                    let scoped_sublabel = format!("{}/{}", main_label, sub_label);
                    if let Some(sublabel_symbol) = self.symbols.get(&scoped_sublabel) {
                        return Some(sublabel_symbol);
                    }
                }
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
                            // Device name is before '/', first field is after
                            device = device[..slash_pos].to_string();
                        }
                        let base_addr = u16::from_str_radix(&addr[1..], 16).unwrap_or(0);
                        let mut fields = Vec::new();
                        let mut _offset = 0u16;
                        let mut iter = parts;
                        while let Some(field_part) = iter.next() {
                            let field_name = if field_part.starts_with('&') {
                                field_part[1..].to_string()
                            } else {
                                field_part.to_string()
                            };
                            let size_part = iter.next();
                            let size = if let Some(size_str) = size_part {
                                if size_str.starts_with('$') {
                                    let label_name = &size_str[1..];
                                    let mut resolved = None;
                                    if let Some(ref parent) = self.current_label {
                                        let scoped = format!("{}/{}", parent, label_name);
                                        if let Some(symbol) = self.symbols.get(&scoped) {
                                            resolved = Some(symbol.address);
                                        }
                                    }
                                    if resolved.is_none() {
                                        if let Some(symbol) = self.symbols.get(label_name) {
                                            resolved = Some(symbol.address);
                                        }
                                    }
                                    resolved.unwrap_or_else(|| {
                                        u16::from_str_radix(label_name, 16).unwrap_or(1)
                                    }) as u8
                                } else {
                                    1
                                }
                            } else {
                                1
                            };
                            fields.push(DeviceField {
                                name: field_name,
                                size,
                            });
                            _offset += size as u16;
                        }
                        let devmap = Device {
                            address: base_addr,
                            name: device.clone(),
                            fields,
                        };
                        self.device_map
                            .entry(device)
                            .and_modify(|existing| existing.extend_fields(devmap.fields.clone()))
                            .or_insert(devmap);
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