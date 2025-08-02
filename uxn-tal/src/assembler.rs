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
}

/// Represents a forward reference that needs to be resolved
#[derive(Debug, Clone)]
struct Reference {
    address: u16,
    label: String,
    ref_type: ReferenceType,
    is_short: bool,
    context_label: Option<String>,
}

#[derive(Debug, Clone)]
enum ReferenceType {
    Absolute,
    Relative,
    Sublabel,
    ZeroPage, // Zero-page reference (low byte only)
}

impl Assembler {
    /// Generate symbol file content in binary format
    /// Format: [address:u16][name:null-terminated string] repeating
    pub fn generate_symbol_file(&self) -> Vec<u8> {
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
        }
    }

    /// Assemble TAL source code into a ROM
    pub fn assemble(
        &mut self,
        source: &str,
        path: Option<String>,
    ) -> Result<Vec<u8>> {
        // Clear previous state
        self.symbols.clear();
        self.current_label = None;
        self.references.clear();
        self.device_map.clear();
        self.line_number = 0;
        self.position_in_line = 0;

        // Tokenize
        let mut lexer = Lexer::new(source.to_string(), path.clone());
        let tokens = lexer.tokenize()?;

        // Parse
        // Use "(input)" as the default path if none is provided
        let mut parser = Parser::new_with_source(
            tokens,
            path.clone().unwrap_or_default(),
            source.to_string(),
        );
        let ast = parser.parse()?;

        // First pass: collect labels and generate code
        let mut rom = Rom::new();
        rom.set_source(Some(source.to_string()));
        rom.set_path(path.clone());
        self.first_pass(&ast, &mut rom)?;

        // Only print collected labels and references if a label resolution fails (see second_pass)

        // Second pass: resolve references
        self.second_pass(&mut rom)?;

        // Get the final ROM data, excluding zero page if needed
        let rom_data = rom.data();

        // If the ROM has content starting at 0x0100 or later, exclude the first 256 bytes
        // This matches the behavior of ruxnasm and uxnasm.c
        if rom_data.len() > 256 {
            // Check if there's any non-zero data in the first 256 bytes
            let has_zero_page_data = rom_data[..256].iter().any(|&b| b != 0);

            if !has_zero_page_data {
                // No zero page data, return ROM starting from page 1 (0x0100)
                return Ok(rom_data[256..].to_vec());
            }
        }

        Ok(rom_data.to_vec())
    }

    fn first_pass(&mut self, ast: &[AstNode], rom: &mut Rom) -> Result<()> {
        // Scan the source for device headers and build device_map
        // Also register all device/field addresses as symbols for robust reference resolution
        if let Some(source) = rom.source() {
            let devmaps = parse_device_maps(source);
            for dev in &devmaps {
                // Store DeviceMap by device name
                self.device_map.insert(dev.name.clone(), dev.clone());
                // Register base device address as symbol
                self.symbols.insert(
                    dev.name.clone(),
                    Symbol {
                        address: dev.address,
                        is_sublabel: false,
                        parent_label: None,
                    },
                );
                // Register each field as symbol with correct offset and size
                let mut offset = 0u16;
                for field in &dev.fields {
                    let key = format!("{}/{}", dev.name, field.name);
                    let field_addr = dev.address + offset;
                    self.symbols.insert(
                        key.clone(),
                        Symbol {
                            address: field_addr,
                            is_sublabel: false,
                            parent_label: Some(dev.name.clone()),
                        },
                    );
                    offset += field.size as u16;
                }
            }
        }
        for node in ast {
            self.process_node(node, rom)?;
        }
        Ok(())
    }

    fn process_node(&mut self, node: &AstNode, rom: &mut Rom) -> Result<()> {
        // Add path argument to process_node
        // fn process_node(&mut self, node: &AstNode, rom: &mut Rom, path: Option<String>) -> Result<()> {
        // For now, get path from rom.source_path() if available, else None
        let path = rom.source_path().cloned();
        match node {
            AstNode::Byte(byte) => {
                rom.write_byte(*byte)?;
            }
            AstNode::Short(short) => {
                rom.write_short(*short)?;
            }
            AstNode::LiteralByte(byte) => {
                rom.write_byte(0x80)?; // LIT opcode
                rom.write_byte(*byte)?;
            }
            AstNode::LiteralShort(short) => {
                rom.write_byte(0xa0)?; // LIT2 opcode
                rom.write_short(*short)?;
            }
            AstNode::Instruction(inst) => {
                // Check if this is actually a macro call first
                if let Some(macro_def) = self.macros.get(&inst.opcode).cloned()
                {
                    // Expand macro inline
                    for macro_node in &macro_def.body {
                        self.process_node(macro_node, rom)?;
                    }
                } else {
                    // Handle special LIT instructions that have mode flags built in
                    let opcode = match inst.opcode.as_str() {
                        "LIT" => 0x80,   // LIT (BRK + keep flag)
                        "LIT2" => 0xA0,  // LIT2 (BRK + short + keep flags)
                        "LITr" => 0xC0,  // LITr (BRK + return + keep flags)
                        "LIT2r" => 0xE0, // LIT2r (BRK + short + return + keep flags)
                        _ => {
                            // Try to get the opcode - if it fails, treat as implicit JSR call
                            match self.opcodes.get_opcode(&inst.opcode) {
                                Ok(base_opcode) => {
                                    // It's a regular instruction - apply mode flags
                                    Opcodes::apply_modes(
                                        base_opcode,
                                        inst.short_mode,
                                        inst.return_mode,
                                        inst.keep_mode,
                                    )
                                }
                                Err(_) => {
                                    // Unknown opcode - treat as implicit JSR call
                                    // First emit JSR opcode (0x8d)
                                    rom.write_byte(0x8d)?;
                                    // If referencing '_', register <current_label>/_ as a sublabel if not already present
                                    if inst.opcode == "_" {
                                        if let Some(ref parent) =
                                            self.current_label
                                        {
                                            let scoped =
                                                format!("{}/_", parent);
                                            if !self
                                                .symbols
                                                .contains_key(&scoped)
                                            {
                                                self.symbols.insert(
                                                    scoped.clone(),
                                                    Symbol {
                                                        address: rom.position(),
                                                        is_sublabel: true,
                                                        parent_label: Some(
                                                            parent.clone(),
                                                        ),
                                                    },
                                                );
                                            }
                                        }
                                    }
                                    // Then add reference for the label
                                    let full_label = inst.opcode.clone();
                                    self.references.push(Reference {
                                        address: rom.position(),
                                        label: full_label,
                                        ref_type: ReferenceType::Absolute,
                                        is_short: true,
                                        context_label: self
                                            .current_label
                                            .clone(),
                                    });
                                    rom.write_byte(0)?; // Placeholder high byte
                                    rom.write_byte(0)?; // Placeholder low byte
                                    return Ok(());
                                }
                            }
                        }
                    };
                    rom.write_byte(opcode)?;
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
                // If label is in <name> form, also register plain name
                if label.starts_with('<')
                    && label.ends_with('>')
                    && label.len() > 2
                {
                    let plain = label[1..label.len() - 1].to_string();
                    if !self.symbols.contains_key(&plain) {
                        self.symbols.insert(
                            plain,
                            Symbol {
                                address: rom.position(),
                                is_sublabel: false,
                                parent_label: None,
                            },
                        );
                    }
                }
                self.current_label = Some(label.clone());
            }
            AstNode::LabelRef(label) => {
                // ;label should generate LIT2 + address
                let full_label = label.clone();
                // If referencing '_', register <current_label>/_ as a sublabel if not already present
                if full_label == "_" {
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
                rom.write_byte(0xa0)?; // LIT2 opcode
                self.references.push(Reference {
                    address: rom.position(),
                    label: full_label,
                    ref_type: ReferenceType::Absolute,
                    is_short: true,
                    context_label: self.current_label.clone(),
                });
                rom.write_short(0)?; // Placeholder for address
            }
            AstNode::SublabelDef(sublabel) => {
                let source_line = if let Some(source) = rom.source() {
                    source
                        .lines()
                        .nth(self.line_number)
                        .unwrap_or("")
                        .to_string()
                } else {
                    String::new()
                };
                let full_name = if let Some(ref parent) = self.current_label {
                    format!("{}/{}", parent, sublabel)
                } else {
                    return Err(AssemblerError::SyntaxError {
                        path: path.clone().unwrap_or_default(),
                        line: self.line_number,
                        position: self.position_in_line,
                        message: "Sublabel defined outside of label scope"
                            .to_string(),
                        source_line,
                    });
                };
                //eprintln!("[DEBUG] Registering sublabel: '{}' (len: {})", full_name, full_name.len());
                // Register <scope>/<sublabel>
                if !self.symbols.contains_key(&full_name) {
                    self.symbols.insert(
                        full_name.clone(),
                        Symbol {
                            address: rom.position(),
                            is_sublabel: true,
                            parent_label: self.current_label.clone(),
                        },
                    );
                }
                // Also register global <sublabel>
                if !self.symbols.contains_key(sublabel) {
                    self.symbols.insert(
                        sublabel.clone(),
                        Symbol {
                            address: rom.position(),
                            is_sublabel: true,
                            parent_label: self.current_label.clone(),
                        },
                    );
                }
            }
            AstNode::SublabelRef(sublabel) => {
                let source_line = if let Some(source) = rom.source() {
                    source
                        .lines()
                        .nth(self.line_number)
                        .unwrap_or("")
                        .to_string()
                } else {
                    String::new()
                };
                let full_name = if let Some(ref parent) = self.current_label {
                    format!("{}/{}", parent, sublabel)
                } else {
                    return Err(AssemblerError::SyntaxError {
                        path: path.clone().unwrap_or_default(),
                        line: self.line_number,
                        position: self.position_in_line,
                        message: "Sublabel reference outside of label scope"
                            .to_string(),
                        source_line,
                    });
                };
                // If referencing '_', register <current_label>/_ as a sublabel if not already present
                if sublabel == "_" {
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
                self.references.push(Reference {
                    address: rom.position(),
                    label: full_name.clone(),
                    ref_type: ReferenceType::Sublabel,
                    is_short: false,
                    context_label: self.current_label.clone(),
                });
                rom.write_byte(0)?; // Placeholder
            }
            AstNode::RelativeRef(label) => {
                // Check if this is a sublabel reference (starts with &)
                let source_line = if let Some(source) = rom.source() {
                    source
                        .lines()
                        .nth(self.line_number)
                        .unwrap_or("")
                        .to_string()
                } else {
                    String::new()
                };
                let resolved_label = if label.starts_with('&') {
                    let sublabel_name = &label[1..]; // Remove & prefix
                    if let Some(ref parent) = self.current_label {
                        format!("{}/{}", parent, sublabel_name)
                    } else {
                        return Err(AssemblerError::SyntaxError {
                            path: path.clone().unwrap_or_default(),
                            line: self.line_number,
                            position: self.position_in_line,
                            message:
                                "Sublabel reference outside of label scope"
                                    .to_string(),
                            source_line,
                        });
                    }
                } else {
                    label.clone()
                };

                // If referencing '_', register <current_label>/_ as a sublabel if not already present
                if resolved_label == "_" {
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
                self.references.push(Reference {
                    address: rom.position(),
                    label: resolved_label,
                    ref_type: ReferenceType::Relative,
                    is_short: false,
                    context_label: self.current_label.clone(),
                });
                rom.write_byte(0)?; // Placeholder
            }
            AstNode::ConditionalRef(label) => {
                // Conditional reference generates JCN followed by relative address
                rom.write_byte(0x0d)?; // JCN opcode

                // Check if this is a sublabel reference (starts with &)
                let source_line = if let Some(source) = rom.source() {
                    source
                        .lines()
                        .nth(self.line_number)
                        .unwrap_or("")
                        .to_string()
                } else {
                    String::new()
                };
                let resolved_label = if label.starts_with('&') {
                    let sublabel_name = &label[1..]; // Remove & prefix
                    if let Some(ref parent) = self.current_label {
                        format!("{}/{}", parent, sublabel_name)
                    } else {
                        return Err(AssemblerError::SyntaxError {
                            path: path.clone().unwrap_or_default(),
                            line: self.line_number,
                            position: self.position_in_line,
                            message:
                                "Sublabel reference outside of label scope"
                                    .to_string(),
                            source_line,
                        });
                    }
                } else {
                    label.clone()
                };

                // If referencing '_', register <current_label>/_ as a sublabel if not already present
                if resolved_label == "_" {
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
                self.references.push(Reference {
                    address: rom.position(),
                    label: resolved_label,
                    ref_type: ReferenceType::Relative,
                    is_short: false,
                    context_label: self.current_label.clone(),
                });
                rom.write_byte(0)?; // Placeholder
            }
            AstNode::ConditionalBlock(block_nodes) => {
                // Conditional block ?{ ... } - execute block if top of stack is non-zero
                // Compile as: JCN (skip) ... (block content) ... (skip target)

                // Emit JCN (conditional jump - jumps if top of stack is zero)
                rom.write_byte(0x0d)?; // JCN opcode

                // We need to calculate the offset to skip the block
                // For now, emit a placeholder and resolve later
                let jump_address = rom.position();
                rom.write_byte(0)?; // Placeholder for jump offset

                // Assemble the block content
                for node in block_nodes {
                    self.process_node(node, rom)?;
                }
                let block_end = rom.position();

                // Calculate the offset to jump over the block
                let offset = (block_end - jump_address - 1) as u8;

                // Write the actual offset back to the placeholder
                rom.write_byte_at(jump_address, offset)?;
            }
            AstNode::JSRRef(label) => {
                // JSR call generates JSR followed by absolute address
                rom.write_byte(0x8d)?; // JSR opcode

                // Check if this is a sublabel reference (starts with &)
                let source_line = if let Some(source) = rom.source() {
                    source
                        .lines()
                        .nth(self.line_number)
                        .unwrap_or("")
                        .to_string()
                } else {
                    String::new()
                };
                let resolved_label = if label.starts_with('&') {
                    let sublabel_name = &label[1..]; // Remove & prefix
                    if let Some(ref parent) = self.current_label {
                        format!("{}/{}", parent, sublabel_name)
                    } else {
                        return Err(AssemblerError::SyntaxError {
                            path: path.clone().unwrap_or_default(),
                            line: self.line_number,
                            position: self.position_in_line,
                            message:
                                "Sublabel reference outside of label scope"
                                    .to_string(),
                            source_line,
                        });
                    }
                } else {
                    label.clone()
                };

                // If referencing '_', register <current_label>/_ as a sublabel if not already present
                if resolved_label == "_" {
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
                self.references.push(Reference {
                    address: rom.position(),
                    label: resolved_label,
                    ref_type: ReferenceType::Absolute,
                    is_short: true,
                    context_label: self.current_label.clone(),
                });
                rom.write_byte(0)?; // Placeholder high byte
                rom.write_byte(0)?; // Placeholder low byte
            }
            AstNode::RawAddressRef(label) => {
                // Raw address access - LIT2 + absolute address
                rom.write_byte(0xa0)?; // LIT2 opcode
                self.references.push(Reference {
                    address: rom.position(),
                    label: label.clone(),
                    ref_type: ReferenceType::Absolute,
                    is_short: true,
                    context_label: self.current_label.clone(),
                });
                rom.write_byte(0)?; // Placeholder high byte
                rom.write_byte(0)?; // Placeholder low byte
            }
            AstNode::HyphenRef(identifier) => {
                // Hyphen reference has two forms:
                // 1. -Device/field: device field access (single byte address)
                // 2. -label: zero-page reference (low byte of label address)
                if identifier.contains('/') {
                    // Parse as device/field reference
                    let parts: Vec<&str> = identifier.split('/').collect();
                    if parts.len() == 2 {
                        let device = parts[0];
                        let field = parts[1];
                        let full_label = format!("{}/{}", device, field);
                        self.references.push(Reference {
                            address: rom.position(),
                            label: full_label,
                            ref_type: ReferenceType::ZeroPage, // Use zero-page reference
                            is_short: false,
                            context_label: self.current_label.clone(),
                        });
                        rom.write_byte(0)?; // Placeholder for address
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
                            path: path.clone().unwrap_or_default(),
                            line: self.line_number,
                            position: self.position_in_line,
                            message: format!(
                                "Invalid hyphen reference: -{}",
                                identifier
                            ),
                            source_line,
                        });
                    }
                } else {
                    // Zero-page reference to regular label (low byte only)
                    self.references.push(Reference {
                        address: rom.position(),
                        label: identifier.clone(),
                        ref_type: ReferenceType::ZeroPage,
                        is_short: false,
                        context_label: self.current_label.clone(),
                    });
                    rom.write_byte(0)?; // Placeholder
                }
            }
            AstNode::Padding(addr) => {
                rom.pad_to(*addr)?;
            }
            AstNode::Skip(count) => {
                for _ in 0..*count {
                    rom.write_byte(0)?;
                }
            }
            AstNode::DeviceAccess(device, field) => {
                // Device access like .Screen/width should generate LIT + address
                // This references the sublabel device/field
                let full_label = format!("{}/{}", device, field);
                rom.write_byte(0x80)?; // LIT opcode for byte access
                self.references.push(Reference {
                    address: rom.position(),
                    label: full_label,
                    ref_type: ReferenceType::Absolute,
                    is_short: false,
                    context_label: self.current_label.clone(),
                });
                rom.write_byte(0)?; // Placeholder
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
            AstNode::MacroCall(name) => {
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
                        path: path.clone().unwrap_or_default(),
                        line: self.line_number,
                        position: self.position_in_line,
                        message: format!("Undefined macro: {}", name),
                        source_line,
                    });
                }
            }
            AstNode::RawString(bytes) => {
                rom.write_bytes(bytes)?;
            }
            AstNode::InlineAssembly(nodes) => {
                // Process inline assembly nodes directly
                for inline_node in nodes {
                    self.process_node(inline_node, rom)?;
                }
            }
            AstNode::Include(path) => {
                // Read and process the included file
                self.process_include(path, rom)?;
            }
        }
        Ok(())
    }

    fn second_pass(&mut self, rom: &mut Rom) -> Result<()> {
        for reference in &self.references {
            // Try exact match, then with/without angle brackets
            let mut symbol = {
                // For any relative or sublabel reference, always try <scope>/<label> using context_label first, even if not angle-bracketed
                let mut found = None;
                if matches!(
                    reference.ref_type,
                    ReferenceType::Relative | ReferenceType::Sublabel
                ) {
                    if let Some(ref context) = reference.context_label {
                        // Try context_label/label first
                        let label = if reference.label.starts_with('<')
                            && reference.label.ends_with('>')
                            && reference.label.len() > 2
                        {
                            &reference.label[1..reference.label.len() - 1]
                        } else {
                            reference.label.as_str()
                        };
                        let scoped = format!("{}/{}", context, label);
                        found = self.symbols.get(&scoped);
                        if found.is_none() {
                            // Also try context_label/label without angle brackets
                            found = self.symbols.get(&format!(
                                "{}/{}",
                                context, reference.label
                            ));
                        }
                    }
                }
                if found.is_some() {
                    found
                } else {
                    let direct = self.symbols.get(&reference.label);
                    if direct.is_some() {
                        direct
                    } else if reference.label.starts_with('<')
                        && reference.label.ends_with('>')
                        && reference.label.len() > 2
                    {
                        self.symbols
                            .get(&reference.label[1..reference.label.len() - 1])
                    } else {
                        let bracketed = format!("<{}>", reference.label);
                        let bracketed_found = self.symbols.get(&bracketed);
                        if bracketed_found.is_some() {
                            bracketed_found
                        } else if let Some(pos) = reference.label.rfind('/') {
                            let last = &reference.label[pos + 1..];
                            self.symbols.get(last)
                        } else {
                            None
                        }
                    }
                }
            };

            // Macro resolution: if not found as label, check for macro
            // If symbol is still not found, try to resolve macro reference as a label
            if symbol.is_none() {
                // Try direct macro name as label
                if let Some(sym) = self.symbols.get(&reference.label) {
                    symbol = Some(sym);
                } else {
                    // Try with numeric suffix (e.g., ABS2)
                    for k in self.macros.keys() {
                        if k.starts_with(&reference.label) {
                            if let Some(sym) = self.symbols.get(k) {
                                symbol = Some(sym);
                                break;
                            }
                        }
                    }
                    // Try any label that starts with the reference name (C assembler fallback)
                    if symbol.is_none() {
                        for k in self.symbols.keys() {
                            if k.starts_with(&reference.label) {
                                symbol = self.symbols.get(k);
                                break;
                            }
                        }
                    }
                }
            }

            // Hierarchical sublabel resolution for angle-bracketed references
            if symbol.is_none()
                && reference.label.starts_with('<')
                && reference.label.ends_with('>')
                && reference.label.len() > 2
            {
                let sublabel_name =
                    &reference.label[1..reference.label.len() - 1];
                // Try current scope
                if let Some(ref current_label) = self.current_label {
                    let hierarchical =
                        format!("{}/{}", current_label, sublabel_name);
                    symbol = self.symbols.get(&hierarchical);
                    // Try parent scope (e.g., button/<press>)
                    if symbol.is_none() {
                        if let Some(pos) = current_label.rfind('/') {
                            let parent = &current_label[..pos];
                            let hierarchical =
                                format!("{}/{}", parent, sublabel_name);
                            symbol = self.symbols.get(&hierarchical);
                        }
                    }
                }
            }

            // Special handling for '_' references: search up the scope chain from reference.context_label
            if symbol.is_none() && reference.label == "_" {
                if let Some(ref context) = reference.context_label {
                    let mut scope = context.as_str();
                    loop {
                        let scoped = format!("{}/_", scope);
                        if let Some(sym) = self.symbols.get(&scoped) {
                            symbol = Some(sym);
                            break;
                        }
                        if let Some(pos) = scope.rfind('/') {
                            scope = &scope[..pos];
                        } else {
                            break;
                        }
                    }
                }
            }

            // Device field resolution using device_map
            let mut device_field_addr: Option<u16> = None;
            if symbol.is_none() {
                if let Some(slash_pos) = reference.label.find('/') {
                    let device = &reference.label[..slash_pos];
                    let field = &reference.label[slash_pos + 1..];
                    if let Some(devmap) = self.device_map.get(device) {
                        if let Some(addr) = devmap.get_field_address(field) {
                            device_field_addr = Some(addr);
                        } else {
                            eprintln!(
                                "[DEBUG] device_map for device '{}': {:?}",
                                device, devmap
                            );
                        }
                    }
                    if device_field_addr.is_none() {
                        //
                        if let Some(dev) = crate::devicemap::DEVICES_DEFAULT
                            .iter()
                            .find(|d| d.name == device)
                        {
                            if let Some(addr) = dev.get_field_address(field) {
                                eprintln!("[DEBUG] Found default device '{}' with field '{}': address {}", device, field, addr);
                                device_field_addr = Some(addr);
                                self.device_map
                                    .insert(device.to_string(), dev.clone());
                            } else {
                                eprintln!("[DEBUG] device_map does not contain device '{}'. device_map: {:?}", device, self.device_map);
                            }
                        } else {
                            eprintln!("[DEBUG] Device '{}' not found in default devices", device);
                        }
                    }
                }
            }

            // Robust resolution: opcode, device/field, or subroutine label (JSR)
            if symbol.is_none() && device_field_addr.is_none() {
                // Try opcode resolution (case-insensitive, all mode variants)
                let label = &reference.label;
                let variants = [
                    label.to_string(),
                    label.to_ascii_uppercase(),
                    label.to_ascii_lowercase(),
                    format!("{}2", label),
                    format!("{}r", label),
                    format!("{}k", label),
                    format!("{}2r", label),
                    format!("{}2k", label),
                    format!("{}kr", label),
                    format!("{}2kr", label),
                ];
                let mut found_opcode = None;
                for variant in &variants {
                    if let Ok(opcode) = self.opcodes.get_opcode(variant) {
                        found_opcode = Some(opcode);
                        break;
                    }
                }
                if let Some(opcode) = found_opcode {
                    rom.write_byte_at(reference.address, opcode)?;
                    continue;
                }
                // Try device/field resolution
                if let Some(slash_pos) = reference.label.find('/') {
                    let device = &reference.label[..slash_pos];
                    let field = &reference.label[slash_pos + 1..];
                    if let Some(devmap) = self.device_map.get(device) {
                        if let Some(addr) = devmap.get_field_address(field) {
                            rom.write_byte_at(reference.address, addr as u8)?;
                            continue;
                        }
                    }
                }
                // Treat as subroutine label (JSR): leave placeholder for JSR resolution
                // Instead of erroring, leave the placeholder for JSR resolution (do nothing)
                // This matches the C assembler's fallback behavior
                continue;
            }

            let address = if let Some(symbol) = symbol {
                symbol.address
            } else if let Some(addr) = device_field_addr {
                addr
            } else {
                unreachable!();
            };

            match reference.ref_type {
                ReferenceType::Absolute => {
                    if reference.is_short {
                        rom.write_short_at(reference.address, address)?;
                    } else {
                        rom.write_byte_at(reference.address, address as u8)?;
                    }
                }
                ReferenceType::Relative | ReferenceType::Sublabel => {
                    let current_addr = reference.address.wrapping_add(1);
                    let target_addr = address;
                    let offset = target_addr.wrapping_sub(current_addr) as i16;
                    if offset < -128 || offset > 127 {
                        // Out of range: emit absolute jump (JMP2r)
                        // Overwrite the relative jump opcode and placeholder with JMP2r and absolute address
                        // JMP2r opcode is 0x1d
                        rom.write_byte_at(
                            reference.address.wrapping_sub(1),
                            0x1d,
                        )?;
                        rom.write_short_at(reference.address, address)?;
                    } else {
                        rom.write_byte_at(reference.address, offset as u8)?;
                    }
                }
                ReferenceType::ZeroPage => {
                    // Zero-page reference: emit only the low byte of the address
                    rom.write_byte_at(reference.address, address as u8)?;
                }
            }
        }
        Ok(())
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

    /// Process an include directive by reading and assembling the included file
    fn process_include(&mut self, path: &str, rom: &mut Rom) -> Result<()> {
        // Read the included file
        let content = fs::read_to_string(path)?;

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
                        let base_addr =
                            u16::from_str_radix(&addr[1..], 16).unwrap_or(0);
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
                                    if let Some(ref parent) = self.current_label
                                    {
                                        let scoped = format!(
                                            "{}/{}",
                                            parent, label_name
                                        );
                                        if let Some(symbol) =
                                            self.symbols.get(&scoped)
                                        {
                                            resolved = Some(symbol.address);
                                        }
                                    }
                                    if resolved.is_none() {
                                        if let Some(symbol) =
                                            self.symbols.get(label_name)
                                        {
                                            resolved = Some(symbol.address);
                                        }
                                    }
                                    resolved.unwrap_or_else(|| {
                                        u16::from_str_radix(label_name, 16)
                                            .unwrap_or(1)
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
                            .and_modify(|existing| {
                                existing.extend_fields(devmap.fields.clone())
                            })
                            .or_insert(devmap);
                    }
                }
            }
        }

        // Lex the included file
        let mut lexer = Lexer::new(content.clone(), Some(path.to_string()));
        let tokens = lexer.tokenize()?;

        // Parse the included file
        let mut parser =
            Parser::new_with_source(tokens, path.to_string(), content);
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
