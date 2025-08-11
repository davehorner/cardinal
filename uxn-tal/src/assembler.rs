//! Main assembler implementation

use crate::devicemap::{parse_device_maps, Device, DeviceField};
use crate::devicemap::DEVICES_DEFAULT; // NEW: bring in default devices
use crate::error::{AssemblerError, Result};
use crate::lexer::{Lexer, TokenWithPos};
use crate::opcodes::Opcodes;
use crate::parser::{AstNode, Parser};
use crate::rom::Rom;
use crate::runes::Rune;
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
    symbol_order: Vec<String>, // preserve insertion order like uxnasm
    macros: HashMap<String, Macro>,
    current_label: Option<String>,
    references: Vec<Reference>,
    device_map: HashMap<String, Device>, // device name -> Device
    line_number: usize,
    position_in_line: usize,
    effective_length: usize, // Track effective length like uxnasm.c
    //lambda_counter: u16, // Add lambda counter as a field
    lambda_counter: usize,
    lambda_stack: Vec<usize>,
    last_top_label: Option<String>, // remember last top-level label to scope stray sublabels
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
        // Match uxnasm: emit in insertion order, not sorted.
        let mut out = Vec::new();
        for name in &self.symbol_order {
            if let Some(sym) = self.symbols.get(name) {
                out.push((sym.address >> 8) as u8);
                out.push((sym.address & 0xff) as u8);
                out.extend_from_slice(name.as_bytes());
                out.push(0);
            }
        }
        out
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
            symbol_order: Vec::new(),
            macros: HashMap::new(),
            current_label: None,
            references: Vec::new(),
            device_map: HashMap::new(),
            line_number: 0,
            position_in_line: 0,
            effective_length: 0,
lambda_counter: 0,
lambda_stack: Vec::new(),
last_top_label: None,
        }
    }

    /// Insert symbol preserving first-seen address and append to ordered list (no overwrite).
    fn insert_symbol_if_new(&mut self, name: &str, sym: Symbol) {
        if !self.symbols.contains_key(name) {
            self.symbols.insert(name.to_string(), sym);
            self.symbol_order.push(name.to_string());
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
        self.symbol_order.clear();
        self.current_label = None;
        self.references.clear();
        self.device_map.clear();
        self.line_number = 0;
        self.position_in_line = 0;
        self.effective_length = 0; // Reset effective length
        self.lambda_counter = 0; // Reset lambda counter (start at 1 to avoid λ0)
        self.last_top_label = None;

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

        // --- Ensure ROM pointer starts at 0x0100 (Varvara/uxn convention) ---
        rom.pad_to(0x0100)?;

        self.first_pass(&ast, &mut rom)?;

        // Second pass: resolve references and emit metadata header if needed
        self.second_pass(&mut rom)?;
        println!("DEBUG: Resolved {} references", self.references.len());
self.prune_lambda_aliases();
        // --- FIX: robust program extraction (supports two Rom storage strategies) ---
        let page_start = 0x0100usize;
        let end = self.effective_length;
        if end <= page_start {
            println!("DEBUG: No non-zero bytes beyond PAGE (effective_length=0x{:04X})", end);
            return Ok(Vec::new());
        }
        let mut rom_data = rom.data().to_vec();
        // Ensure backing buffer can be sliced up to `end` like uxnasm's 64K `data[]`.
        if rom_data.len() < end {
            rom_data.resize(end, 0);
        }
        let result = &rom_data[page_start..end];

       let mut prog = rom.data().to_vec();
       // Use assembler’s absolute end to allow trailing zeros:
       let end_rel = end - page_start;
       if prog.len() < end_rel {
           prog.resize(end_rel, 0);
       }
       let result = &prog[..end_rel];
        println!(
            "Assembled {} in {} bytes({:.2}% used), {} labels, {} macros. (effective_length=0x{:04X})",
            path.clone().unwrap_or_else(|| "(input)".to_string()),
            result.len(),
            result.len() as f64 / 652.80,
            self.symbols.len(),
            self.macros.len(),
            end
        );
        Ok(result.to_vec())
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

        // --- Rune table for reference ---
        // rune '?' : conditional branch (0x20 + rel word)
        // rune '!' : exclamation branch (0x40 + rel word)
        // rune ' ' : JSR (unknown token, 0x60 + rel word)
        // rune '='/':'/';' : absolute word
        // rune '-' / '.' : absolute byte
        // rune '_' / ',' : relative byte (+ int8 range check)
        // (see uxnasm.c resolve() switch)
        match node {
            AstNode::ConditionalBlockStart(tok) => {
                // 1) new lambda id
                let id = self.lambda_counter;
                self.lambda_counter += 1;
                self.lambda_stack.push(id);

                // 2) its label name
                let name = format_lambda_label(id);

                // 3) record a reference at the first byte of the word (after opcode), rune '?'
                let ref_addr = rom.position() + 1; // <-- FIX: was rom.position()
                self.references.push(Reference {
                    name: name.clone(),
                    rune: '?',
                    address: ref_addr as u16,
                    line: tok.line,
                    path: String::new(),
                    scope: tok.scope.clone(),
                    token: Some(tok.clone()),
                });
                // 4) emit JCN and 0xFFFF placeholder
                rom.write_byte(0x20)?;        // JCN
                rom.write_short(0xFFFF)?;     // placeholder for relative word
                self.update_effective_length(rom);
            }
            AstNode::ConditionalBlockEnd(tok) => {
                let id = match self.lambda_stack.pop() {
                    Some(id) => id,
                    None => {
                        return Err(AssemblerError::SyntaxError {
                            path: String::new(),
                            line: tok.line,
                            position: 0,
                            message: "Unmatched '}'".to_string(),
                            source_line: String::new(),
                        });
                    }
                };
                let name = format_lambda_label(id);

                // Only insert the lambda label if no non-lambda label exists at this address
                let addr = rom.position() as u16;
                let has_named_here = self.symbol_order.iter().any(|n| {
                    if let Some(s) = self.symbols.get(n) {
                        s.address == addr && !n.starts_with('λ')
                    } else {
                        false
                    }
                });
                if !has_named_here && !self.symbols.contains_key(&name) {
                    self.insert_symbol_if_new(&name, Symbol {
                        address: addr,
                        is_sublabel: false,
                        parent_label: None
                    });
                }
            }
            AstNode::Padding(pad_addr) => {
                // Only clear scope on the very first |0100 (match drifblim keeping scope afterwards)
                if *pad_addr == 0x0100 && self.last_top_label.is_none() {
                    self.current_label = None;
                }
                rom.pad_to(*pad_addr)?;
                // self.update_effective_length(rom);
            }
            AstNode::Byte(byte) => {
                rom.write_byte(*byte)?;
                //self.update_effective_length(rom);
                if *byte != 0 {
                    self.update_effective_length(rom);
                }

            }
            AstNode::Short(short) => {
//                 rom.write_short(*short)?;
//    if *short != 0 {
//        self.update_effective_length(rom);
//    } 
let hi = (*short >> 8) as u8;
let lo = (*short & 0xFF) as u8;

rom.write_byte(hi)?;
if hi != 0 { self.update_effective_length(rom); }

rom.write_byte(lo)?;
if lo != 0 { self.update_effective_length(rom); }
                    //self.update_effective_length(rom);
                
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
                // Special-case BRK: always emit 0x00, matching uxnasm.c
                if inst.opcode.eq_ignore_ascii_case("BRK") {
                    rom.write_byte(0x00)?;
                    eprintln!(
                        "DEBUG: Wrote opcode 0x00 (BRK) at {:04X}",
                        rom.position() - 1
                    );
                    // Do not update effective_length for BRK (matches C)
                    return Ok(());
                }
                // Always emit a JSR reference for unknown instructions (not in opcode table)
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
                            //self.update_effective_length(rom);
                        // Only expand macro if found and handled as instruction
                        if let Some(macro_def) = self.macros.get(&inst.opcode).cloned() {
                            for macro_node in &macro_def.body {
                                self.process_node(macro_node, rom)?;
                            }
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
            }
            AstNode::LabelRef { label,rune,token} => {
                // DEBUG: Log when a bare label reference is encountered
                println!(
                    "DEBUG: AstNode::LabelRef encountered at line {}, emitting JSR to label {:?} at address {:04X}",
                    token.line,
                    token,
                    rom.position()
                );
                // // Extract the bare word
                // let label = if let crate::lexer::Token::LabelRef(s,r) = &token.token {
                //     s.clone()
                // } else {
                //     println!("DEBUG: Expected LabelRef, found {:?}", token);
                //     if let crate::lexer::Token::Newline = &token.token {
                //         // If it's a newline, just continue (ignore)
                //         return Ok(());
                //     }
                //     return Err(AssemblerError::SyntaxError {
                //         path: path.clone(),
                //         line: self.line_number,
                //         position: self.position_in_line,
                //         message: format!("Expected LabelRef, found {:?}", token.token),
                //         source_line: String::new(),
                //     });
                // };

                // NEW: If it’s a macro name, expand inline like uxnasm’s findmacro+walkmacro
                if let Some(m) = self.macros.get(label.as_str()).cloned() {
                    println!("DEBUG: Expanding macro '{}' at {:04X}", label, rom.position());
                    for macro_node in &m.body {
                        self.process_node(macro_node, rom)?;
                    }
                    return Ok(());
                }
      match rune {
    // '=': direct absolute word (big-endian), resolved in second_pass
    Rune::RawAbsolute => {
        self.references.push(Reference {
            name: label.clone(),
            rune: '=',                                  // mark as absolute
            address: rom.position() as u16,             // where the 16-bit will live
            line: self.line_number,
                    path: path.clone(),
            scope: self.current_label.clone(),
            token: None, // or Some(tok) if you have one here
        });
        rom.write_short(0xFFFF)?;                       // reserve space
        self.update_effective_length(rom);
    }

    // Everything else: treat as a call (JSR + rel16 placeholder)
    _ => {
                self.references.push(Reference {
            name: label.clone(),
            rune: ' ',                                  // mark as relative
            address: rom.position() + 1 as u16,             // start of the rel16 operand
            line: self.line_number,
                    path: path.clone(),
            scope: self.current_label.clone(),
            token: None, // or Some(tok)
        });
        rom.write_byte(0x60)?;                          // JSR
        self.update_effective_length(rom);



        rom.write_short(0xFFFF)?;                       // reserve space
        self.update_effective_length(rom);
    }
}

            }
            AstNode::LabelDef(label) => {
                if !self.symbols.contains_key(label) {
                    let address = rom.position();
                    let label_clone = label.clone();
                    self.insert_symbol_if_new(&label_clone, Symbol {
                        address,
                        is_sublabel: label_clone.contains('/'),
                        parent_label: label_clone.rsplitn(2, '/').nth(1).map(|s| s.to_string()),
                    });
                }
                // Track full current label; also store its top-level base for later stray sublabels
                let label_for_tracking = label.clone();
                self.current_label = Some(label_for_tracking.clone());
                let top = label_for_tracking.split('/').next().unwrap_or("").to_string();
                self.last_top_label = Some(top);
                eprintln!(
                    "DEBUG: Defined label '{}' at address {:04X}",
                    label_for_tracking,
                    rom.position()
                );
            }
            AstNode::SublabelDef(sublabel) => {
                // If current_label lost due to padding but we have a remembered last_top_label, use it.
                let parent_scope = if let Some(ref cur) = self.current_label {
                    if cur.trim().is_empty() {
                        None
                    } else {
                        let main = cur.split('/').next().unwrap_or(cur);
                        Some(main.to_string())
                    }
                } else {
                    self.last_top_label.clone()
                };
                let full_name = if let Some(parent) = parent_scope {
                    format!("{}/{}", parent, sublabel)
                } else {
                    sublabel.clone()
                };
                if !self.symbols.contains_key(&full_name) {
                    let parent = if full_name.contains('/') {
                        Some(full_name[..full_name.rfind('/').unwrap()].to_string())
                    } else { None };
                    self.insert_symbol_if_new(&full_name, Symbol {
                        address: rom.position(),
                        is_sublabel: true,
                        parent_label: parent.clone(),
                    });
                    // Prefer parent label for display when it shares the same address with a sublabel
                    if let Some(parent_name) = parent {
                        if let (Some(parent_sym), Some(subl_sym)) =
                            (self.symbols.get(&parent_name), self.symbols.get(&full_name))
                        {
                            if parent_sym.address == subl_sym.address {
                                // remove sublabel then push to end (parent now precedes)
                                if let Some(idx) = self.symbol_order.iter().position(|n| n == &full_name) {
                                    let sub_entry = self.symbol_order.remove(idx);
                                    self.symbol_order.push(sub_entry);
                                }
                            }
                        }
                    }
                }
                eprintln!(
                    "DEBUG: Defined sublabel '{}' at address {:04X}",
                    full_name,
                    rom.position()
                );
            }
            AstNode::ExclamationRef(tok) => {
                // Special-case !{  (lambda call + definition)
                if let crate::lexer::Token::ExclamationRef(s) = &tok.token {
                    if s == "{" {
                        // allocate lambda id
                        let id = self.lambda_counter;
                        self.lambda_counter += 1;
                        self.lambda_stack.push(id);
                        let name = format_lambda_label(id);
                        // reference (rune '!')
                        self.references.push(Reference {
                            name: name,
                            rune: '!',
                            address: rom.position() + 1,
                            line: tok.line,
                            path: rom.source_path().cloned().unwrap_or_default(),
                            scope: tok.scope.clone(),
                            token: Some(tok.clone()),
                        });
                        // emit opcode + placeholder (same as normal !label)
                        rom.write_byte(0x40)?;
                        self.update_effective_length(rom);
                        rom.write_short(0xffff)?;
                        self.update_effective_length(rom);
                        return Ok(());
                    }
                }
                // Normal handling for !label
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
                        let main_scope = if let Some(slash_pos) = scope.find('/') { &scope[..slash_pos] } else { scope };
                        format!("{}/{}", main_scope, clean_label)
                    } else {
                        clean_label.to_string()
                    }
                } else { label };

                rom.write_byte(0x40)?;
                self.update_effective_length(rom);
                self.references.push(Reference {
                    name: resolved_name,
                    rune: '!',
                    address: rom.position(),
                    line: tok.line,
                    path: path.clone(),
                    scope: tok.scope.clone(),
                    token: Some(tok.clone()),
                });
                rom.write_short(0xffff)?;
                self.update_effective_length(rom);
            }
            AstNode::PaddingLabel(tok) => {
                // existing absolute padding by label (add support for leading '/' relative)
                let raw = match &tok.token {
                    crate::lexer::Token::PaddingLabel(s) => s.clone(),
                    _ => return Err(AssemblerError::SyntaxError {
                        path: path.clone(),
                        line: self.line_number,
                        position: self.position_in_line,
                        message: "Expected PaddingLabel".to_string(),
                        source_line: String::new(),
                    }),
                };
                // --- PATCH: resolve &name as sublabel of current label, like uxnasm ---
                let label = if raw.starts_with('/') {
                    // Resolve /name relative to main scope
                    self.resolve_relative_label(&raw, tok.scope.as_ref().or(self.current_label.as_ref()))
                } else if raw.starts_with('&') {
                    // Resolve as sublabel of current label's main scope
                    if let Some(ref parent) = self.current_label {
                        let sublabel = &raw[1..];
                        let main_label = if let Some(slash) = parent.find('/') {
                            &parent[..slash]
                        } else {
                            parent.as_str()
                        };
                        format!("{}/{}", main_label, sublabel)
                    } else {
                        raw.clone()
                    }
                } else {
                    raw
                };
                if let Some(symbol) = self.symbols.get(&label) {
                    rom.pad_to(symbol.address)?;
                } else {
                    return Err(AssemblerError::SyntaxError {
                        path: rom.source_path().cloned().unwrap_or_default(),
                        line: self.line_number,
                        position: self.position_in_line,
                        message: format!("Padding label '{}' not found", label),
                        source_line: String::new(),
                    });
                }
            }
            AstNode::RelativePadding(count) => {
                // $HHHH : advance pointer by hex bytes (relative)
                let new_addr = rom.position() as u16 + count;
                rom.pad_to(new_addr)?;
            }
            AstNode::RelativePaddingLabel(tok) => {
                // $label : ptr = current + label.addr (label must exist already)
                let raw = match &tok.token {
                    crate::lexer::Token::RelativePaddingLabel(s) => s.clone(),
                    _ => unreachable!("Token/Ast mismatch for RelativePaddingLabel"),
                };
                let label_name = if raw.starts_with('/') {
                    self.resolve_relative_label(&raw, tok.scope.as_ref().or(self.current_label.as_ref()))
                } else {
                    raw
                };
                let cur = rom.position() as u16;
                if let Some(sym) = self.symbols.get(&label_name) {
                    let new_addr = cur.wrapping_add(sym.address);
                    rom.pad_to(new_addr)?;
                } else {
                    return Err(AssemblerError::SyntaxError {
                        path: rom.source_path().cloned().unwrap_or_default(),
                        line: tok.line,
                        position: tok.start_pos,
                        message: format!("Relative padding label '{}' not found", label_name),
                        source_line: String::new(),
                    });
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
                // Debug: log macro expansion
                static mut MACRO_EXPAND_DEPTH: usize = 0;
                unsafe {
                    MACRO_EXPAND_DEPTH += 1;
                    println!("DEBUG: Expanding macro '{}' at depth {} (line {}, pos {})", name, MACRO_EXPAND_DEPTH, macro_line, macro_position);
                }
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
                unsafe {
                    MACRO_EXPAND_DEPTH -= 1;
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
            AstNode::Include(tok) => {
                // Save/restore current_label around includes
                let saved_label = self.current_label.clone();
                if let crate::lexer::Token::Include(ref path) = tok.token {
                    self.process_include_with_token(path, tok, rom)?;
                }
                self.current_label = saved_label;
            }
            AstNode::LambdaStart(tok) => {
                // Standalone '{' lambda:
                // 1) allocate id
                let id = self.lambda_counter;
                self.lambda_counter += 1;
                self.lambda_stack.push(id);
                let name = format_lambda_label(id);
                // 2) create JSR reference (space rune) at ptr+1
                self.references.push(Reference {
                    name: name.clone(),
                    rune: ' ',               // same rune as unknown token (JSR)
                    address: (rom.position() + 1) as u16,
                    line: tok.line,
                    path: rom.source_path().cloned().unwrap_or_default(),
                    scope: tok.scope.clone(),
                    token: Some(tok.clone()),
                });
                rom.write_byte(0x60)?;       // JSR opcode
                self.update_effective_length(rom);
                rom.write_short(0xffff)?;    // placeholder relative word
                self.update_effective_length(rom);
                // Code of lambda body now follows; label defined at LambdaEnd
            }
            AstNode::LambdaEnd(tok) => {
                // Define lambda label at current position.
                let id = match self.lambda_stack.pop() {
                    Some(id) => id,
                    None => {
                        return Err(AssemblerError::SyntaxError {
                            path: rom.source_path().cloned().unwrap_or_default(),
                            line: tok.line,
                            position: 0,
                            message: "Unmatched '}' (lambda)".to_string(),
                            source_line: String::new(),
                        });
                    }
                };

                let addr = rom.position() as u16;
                // Only insert the lambda label if no non-lambda label exists at this address
                let has_named_here = self.symbol_order.iter().any(|n| {
                    if let Some(s) = self.symbols.get(n) {
                        s.address == addr && !n.starts_with('λ')
                    } else {
                        false
                    }
                });
                if !has_named_here {
                                    let name = format_lambda_label(id);
                    self.insert_symbol_if_new(&name, Symbol {
                        address: addr,
                        is_sublabel: false,
                        parent_label: None
                    });
                }
            }
            AstNode::SublabelRef(tok) => {
                let sublabel = match &tok.token {
                    crate::lexer::Token::SublabelRef(s) => s.clone(),
                    _ => return Err(AssemblerError::SyntaxError {
                        path: rom.source_path().cloned().unwrap_or_default(),
                        line: tok.line,
                        position: tok.start_pos,
                        message: "Expected SublabelRef".to_string(),
                        source_line: String::new(),
                    }),
                };
                let full_name = if let Some(ref parent) = self.current_label {
                    format!("{}/{}", parent, sublabel)
                } else {
                    return Err(AssemblerError::SyntaxError {
                        path: rom.source_path().cloned().unwrap_or_default(),
                        line: tok.line,
                        position: tok.start_pos,
                        message: "Sublabel reference outside of label scope".to_string(),
                        source_line: String::new(),
                    });
                };
                self.references.push(Reference {
                    name: full_name,
                    rune: '_',
                    address: rom.position(),
                    line: tok.line,
                    path: rom.source_path().cloned().unwrap_or_default(),
                    scope: tok.scope.clone(),
                    token: Some(tok.clone()),
                });
                rom.write_byte(0xff)?;
            }
            AstNode::RelativeRef(tok) => {
                let label = match &tok.token {
                    crate::lexer::Token::RelativeRef(s) => s.clone(),
                    _ => unreachable!("RelativeRef token mismatch"),
                };
                self.references.push(Reference {
                    name: label,
                    rune: '/',
                    address: rom.position() + 1,
                    line: tok.line,
                    path: rom.source_path().cloned().unwrap_or_default(),
                    scope: tok.scope.clone(),
                    token: Some(tok.clone()),
                });
                rom.write_byte(0x60)?;
                rom.write_short(0xffff)?;
            }
            AstNode::ConditionalRef(tok) => {
                let label = match &tok.token {
                    crate::lexer::Token::ConditionalRef(s) => s.clone(),
                    _ => unreachable!("ConditionalRef token mismatch"),
                };
                self.references.push(Reference {
                    name: label,
                    rune: '?',
                    address: rom.position() + 1,
                    line: tok.line,
                    path: rom.source_path().cloned().unwrap_or_default(),
                    scope: tok.scope.clone(),
                    token: Some(tok.clone()),
                });
                rom.write_byte(0x20)?;
                rom.write_short(0xffff)?;
            }
            AstNode::RawAddressRef(tok) => {
                let label = match &tok.token {
                    crate::lexer::Token::RawAddressRef(s) => s.clone(),
                    _ => unreachable!("RawAddressRef token mismatch"),
                };
                self.references.push(Reference {
                    name: label,
                    rune: '=',
                    address: rom.position(),
                    line: tok.line,
                    path: rom.source_path().cloned().unwrap_or_default(),
                    scope: tok.scope.clone(),
                    token: Some(tok.clone()),
                });
                rom.write_short(0xffff)?;
            }
            AstNode::JSRRef(tok) => {
                let label = match &tok.token {
                    crate::lexer::Token::JSRRef(s) => s.clone(),
                    _ => unreachable!("JSRRef token mismatch"),
                };
                self.references.push(Reference {
                    name: label,
                    rune: '!',
                    address: rom.position() + 1,
                    line: tok.line,
                    path: rom.source_path().cloned().unwrap_or_default(),
                    scope: tok.scope.clone(),
                    token: Some(tok.clone()),
                });
                rom.write_byte(0x60)?;
                rom.write_short(0xffff)?;
            }
            AstNode::HyphenRef(tok) => {
                let label = match &tok.token {
                    crate::lexer::Token::HyphenRef(s) => s.clone(),
                    _ => unreachable!("HyphenRef token mismatch"),
                };
                self.references.push(Reference {
                    name: label,
                    rune: '-',
                    address: rom.position(),
                    line: tok.line,
                    path: rom.source_path().cloned().unwrap_or_default(),
                    scope: tok.scope.clone(),
                    token: Some(tok.clone()),
                });
                rom.write_byte(0xff)?;
            }
            AstNode::DotRef(tok) => {
                let label = match &tok.token {
                    crate::lexer::Token::DotRef(s) => s.clone(),
                    _ => unreachable!("DotRef token mismatch"),
                };
                self.references.push(Reference {
                    name: label,
                    rune: '.',
                    address: rom.position() + 1,
                    line: tok.line,
                    path: rom.source_path().cloned().unwrap_or_default(),
                    scope: tok.scope.clone(),
                    token: Some(tok.clone()),
                });
                rom.write_byte(0x80)?;
                self.update_effective_length(rom);
                rom.write_byte(0xff)?;
                self.update_effective_length(rom);
            }
            AstNode::SemicolonRef(tok) => {
                // Special-case ;{ (lambda via LIT2 form)
                if let crate::lexer::Token::SemicolonRef(s) = &tok.token {
                    if s == "{" {
                        let id = self.lambda_counter;
                        self.lambda_counter += 1;
                        self.lambda_stack.push(id);
                        let name = format_lambda_label(id);
                        self.references.push(Reference {
                            name,
                            rune: ';',
                            address: rom.position() + 1,
                            line: tok.line,
                            path: rom.source_path().cloned().unwrap_or_default(),
                            scope: tok.scope.clone(),
                            token: Some(tok.clone()),
                        });
                        rom.write_byte(0xa0)?; // LIT2
                        self.update_effective_length(rom);
                        rom.write_short(0xffff)?; // placeholder
                        self.update_effective_length(rom);
                        return Ok(());
                    }
                }
                // Normal handling for ;label
                let label = match &tok.token {
                    crate::lexer::Token::SemicolonRef(s) => s.clone(),
                    _ => unreachable!("SemicolonRef token mismatch"),
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
                rom.write_byte(0xa0)?; // LIT2 opcode
                self.update_effective_length(rom);
                rom.write_short(0xffff)?; // Placeholder
                self.update_effective_length(rom);
            }
            AstNode::EqualsRef(tok) => {
                // SPECIAL-CASE: "={" start of a lambda producing a raw 16-bit address(=rune)
                if let crate::lexer::Token::EqualsRef(s) = &tok.token {
                    if s == "{" {
                        let id = self.lambda_counter;
                        self.lambda_counter += 1;
                        self.lambda_stack.push(id);
                        let name = format_lambda_label(id);
                        self.references.push(Reference {
                            name,
                            rune: '=',
                            address: rom.position(),
                            line: tok.line,
                            path: rom.source_path().cloned().unwrap_or_default(),
                            scope: tok.scope.clone(),
                            token: Some(tok.clone()),
                        });
                        // emit placeholder 16-bit (raw address form for '=' rune)
                        rom.write_short(0xffff)?;
                        self.update_effective_length(rom);
                        return Ok(());
                    }
                }
                let label = match &tok.token {
                    crate::lexer::Token::EqualsRef(s) => s.clone(),
                    _ => unreachable!("EqualsRef token mismatch"),
                };
                self.references.push(Reference {
                    name: label,
                    rune: '=',
                    address: rom.position(),
                    line: tok.line,
                    path: rom.source_path().cloned().unwrap_or_default(),
                    scope: tok.scope.clone(),
                    token: Some(tok.clone()),
                });
                rom.write_short(0xffff)?;
                self.update_effective_length(rom);
            }
            AstNode::CommaRef(tok) => {
                // Special-case ,{ (lambda via LIT + relative byte)
                if let crate::lexer::Token::CommaRef(s) = &tok.token {
                    if s == "{" {
                        let id = self.lambda_counter;
                        self.lambda_counter += 1;
                        self.lambda_stack.push(id);
                        let name = format_lambda_label(id);
                        self.references.push(Reference {
                            name,
                            rune: ',',
                            address: rom.position() + 1,
                            line: tok.line,
                            path: rom.source_path().cloned().unwrap_or_default(),
                            scope: tok.scope.clone(),
                            token: Some(tok.clone()),
                        });
                        rom.write_byte(0x80)?; // LIT
                        self.update_effective_length(rom);
                        rom.write_byte(0xff)?; // placeholder byte
                        self.update_effective_length(rom);
                        return Ok(());
                    }
                }
                // Normal handling for ,label
                let label = match &tok.token {
                    crate::lexer::Token::CommaRef(s) => s.clone(),
                    _ => unreachable!("CommaRef token mismatch"),
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
                rom.write_byte(0x80)?; // LIT opcode
                self.update_effective_length(rom);
                rom.write_byte(0xff)?; // Placeholder byte
                self.update_effective_length(rom);
            }
            AstNode::UnderscoreRef(tok) => {
                // Special-case _{ (lambda via relative byte)
                if let crate::lexer::Token::UnderscoreRef(s) = &tok.token {
                    if s == "{" {
                        let id = self.lambda_counter;
                        self.lambda_counter += 1;
                        self.lambda_stack.push(id);
                        let name = format_lambda_label(id);
                        self.references.push(Reference {
                            name,
                            rune: '_',
                            address: rom.position(),
                            line: tok.line,
                            path: rom.source_path().cloned().unwrap_or_default(),
                            scope: tok.scope.clone(),
                            token: Some(tok.clone()),
                        });
                        rom.write_byte(0xff)?; // placeholder byte
                        self.update_effective_length(rom);
                        return Ok(());
                    }
                }
                // Normal handling for _label
                let label = match &tok.token {
                    crate::lexer::Token::UnderscoreRef(s) => s.clone(),
                    _ => unreachable!("UnderscoreRef token mismatch"),
                };
                self.references.push(Reference {
                    name: label,
                    rune: '_',
                    address: rom.position(),
                            line: tok.line,
                            path: rom.source_path().cloned().unwrap_or_default(),
                            scope: tok.scope.clone(),
                            token: Some(tok.clone()),
                        });
                        rom.write_byte(0xff)?; // placeholder byte
                        self.update_effective_length(rom);
            }
            AstNode::QuestionRef(tok) => {
                let label = match &tok.token {
                    crate::lexer::Token::QuestionRef(s) => s.clone(),
                    _ => unreachable!("QuestionRef token mismatch"),
                };
                self.references.push(Reference {
                    name: label,
                    rune: '?',
                    address: rom.position() + 1,
                    line: tok.line,
                    path: rom.source_path().cloned().unwrap_or_default(),
                    scope: tok.scope.clone(),
                    token: Some(tok.clone()),
                });
                rom.write_byte(0x20)?;
                self.update_effective_length(rom);
                rom.write_short(0xffff)?;
                self.update_effective_length(rom);
            }
            // AstNode::RawString(bytes) => {
            //     // Write string data byte by byte, updating effective length for each non-zero byte
            //     for &byte in bytes {
            //         rom.write_byte(byte)?;
            //         if byte != 0 {
            //             self.update_effective_length(rom);
            //         }
            //     }
            // }
            // AstNode::Include(tok) => {
            //     // Save/restore current_label around includes
            //     let saved_label = self.current_label.clone();
            //     if let crate::lexer::Token::Include(ref path) = tok.token {
            //         self.process_include_with_token(path, tok, rom)?;
            //     }
            //     self.current_label = saved_label;
            // }
            // AstNode::LambdaStart(tok) => {
            //     // Standalone '{' lambda:
            //     // 1) allocate id
            //     let id = self.lambda_counter;
            //     self.lambda_counter += 1;
            //     self.lambda_stack.push(id);
            //     let name = format_lambda_label(id);
            //     // 2) create JSR reference (space rune) at ptr+1
            //     self.references.push(Reference {
            //         name: name.clone(),
            //         rune: ' ',               // same rune as unknown token (JSR)
            //         address: (rom.position() + 1) as u16,
            //         line: tok.line,
            //         path: rom.source_path().cloned().unwrap_or_default(),
            //         scope: tok.scope.clone(),
            //         token: Some(tok.clone()),
            //     });
            //     rom.write_byte(0x60)?;       // JSR opcode
            //     self.update_effective_length(rom);
            //     rom.write_short(0xffff)?;    // placeholder relative word
            //     self.update_effective_length(rom);
            //     // Code of lambda body now follows; label defined at LambdaEnd
            // }
            // AstNode::LambdaEnd(tok) => {
            //     // Define lambda label at current position.
            //     let id = match self.lambda_stack.pop() {
            //         Some(id) => id,
            //         None => {
            //             return Err(AssemblerError::SyntaxError {
            //                 path: rom.source_path().cloned().unwrap_or_default(),
            //                 line: tok.line,
            //                 position: 0,
            //                 message: "Unmatched '}' (lambda)".to_string(),
            //                 source_line: String::new(),
            //             });
            //         }
            //     };
            //     let name = format_lambda_label(id);
            //     if self.symbols.contains_key(&name) {
            //         return Err(AssemblerError::SyntaxError {
            //             path: rom.source_path().cloned().unwrap_or_default(),
            //             line: tok.line,
            //             position: 0,
            //             message: format!("Duplicate lambda label {}", name),
            //             source_line: String::new(),
            //         });
            //     }
            //     self.insert_symbol_if_new(&name, Symbol {
            //         address: rom.position() as u16,
            //         is_sublabel: false,
            //         parent_label: None
            //     });
            // }
            // // duplicate LambdaEnd & ConditionalBlockEnd patterns later in file:
            // // (second occurrence near end)
            // AstNode::LambdaEnd(tok) => {
            //     // Define lambda label at current position.
            //     let id = match self.lambda_stack.pop() {
            //         Some(id) => id,
            //         None => {
            //             return Err(AssemblerError::SyntaxError {
            //                 path: rom.source_path().cloned().unwrap_or_default(),
            //                 line: tok.line,
            //                 position: 0,
            //                 message: "Unmatched '}' (lambda)".to_string(),
            //                 source_line: String::new(),
            //             });
            //         }
            //     };
            //     let name = format_lambda_label(id);
            //     if self.symbols.contains_key(&name) {
            //         return Err(AssemblerError::SyntaxError {
            //             path: rom.source_path().cloned().unwrap_or_default(),
            //             line: tok.line,
            //             position: 0,
            //             message: format!("Duplicate lambda label {}", name),
            //             source_line: String::new(),
            //         });
            //     }
            //     self.insert_symbol_if_new(&name, Symbol {
            //         address: rom.position() as u16,
            //         is_sublabel: false,
            //         parent_label: None
            //     });
            // }
        }
        Ok(())
    }

    /// Generate a Rust module exposing all labels as pub const u16 plus *_SIZE constants.
    /// Size rule:
    ///   size(label) = next_greater_label_address - label.address
    ///   (last label uses program_end = effective_length)
    ///   labels sharing an address get size 0.
    /// NOTE: Addresses are absolute (include zero-page); consumer usually subtracts 0x0100 for ROM offset.
    pub fn generate_rust_interface_module(&self, module_name: &str) -> String {
        fn norm(raw: &str) -> String {
            let mut s: String =
                raw.chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '_' }).collect();
            if s.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                s.insert(0, '_');
            }
            s.to_ascii_uppercase()
        }
        // Gather (name, address) sorted by address to derive sizes.
        let mut by_addr: Vec<(&str, u16)> =
            self.symbols.iter().map(|(n, s)| (n.as_str(), s.address)).collect();
        by_addr.sort_by_key(|(_, a)| *a);
        // Precompute next greater address map.
        use std::collections::HashMap;
        let mut next_map: HashMap<u16, u16> = HashMap::new();
        for (idx, &(_, addr)) in by_addr.iter().enumerate() {
            if next_map.contains_key(&addr) {
                continue; // already set (we only care about first greater)
            }
            let next_greater = by_addr
                .iter()
                .skip(idx + 1)
                .find(|(_, a2)| *a2 > addr)
                .map(|(_, a2)| *a2)
                .unwrap_or(self.effective_length as u16);
            next_map.insert(addr, next_greater);
        }

        let mut used = std::collections::HashSet::new();
        let mut lines = Vec::new();
        lines.push(format!("pub mod {} {{", module_name));
        lines.push("    #![allow(non_upper_case_globals)]".into());
        lines.push("    // Auto-generated: label address & size constants".into());
        for name in &self.symbol_order {
            if let Some(sym) = self.symbols.get(name) {
                let mut id = norm(name);
                if id.is_empty() {
                    continue;
                }
                if used.contains(&id) {
                    let base = id.clone();
                    let mut n = 2;
                    while used.contains(&format!("{}_{}", base, n)) {
                        n += 1;
                    }
                    id = format!("{}_{}", base, n);
                }
                used.insert(id.clone());
                // Address constant
                lines.push(format!("    pub const {id}: u16 = 0x{:04X};", sym.address));
                // Size constant (avoid collision)
                let mut size_ident = format!("{}_SIZE", id);
                if used.contains(&size_ident) {
                    let mut n = 2;
                    while used.contains(&format!("{}_SIZE_{n}", id)) {
                        n += 1;
                    }
                    size_ident = format!("{}_SIZE_{n}", id);
                }
                let next = *next_map.get(&sym.address).unwrap_or(&(self.effective_length as u16));
                let size = if next > sym.address { next - sym.address } else { 0 };
                used.insert(size_ident.clone());
                lines.push(format!("    pub const {size_ident}: u16 = 0x{:04X};", size));
            }
        }
        lines.push("}".into());
        lines.join("\n")
    }

    // NEW: inject default device + its fields if referenced but not declared
    fn try_inject_device_symbols(&mut self, full_name: &str) {
        // Extract device part before slash (or whole name if no slash)
        if full_name.starts_with('<') { return; } // ignore lambda / macro-style names
        let dev_name = full_name.split('/').next().unwrap_or(full_name);
        if self.symbols.contains_key(dev_name) {
            // Already injected (device label present)
            return;
        }
        if let Some(dev) = DEVICES_DEFAULT.iter().find(|d| d.name == dev_name) {
            // Insert device root label
            self.insert_symbol_if_new(
                dev_name,
                Symbol {
                    address: dev.address,
                    is_sublabel: false,
                    parent_label: None,
                },
            );
            // Insert each field as sublabel with accumulated offset
            let mut offset: u16 = 0;
            for field in &dev.fields {
                let sub = format!("{}/{}", dev.name, field.name);
                if !self.symbols.contains_key(&sub) {
                    self.insert_symbol_if_new(
                        &sub,
                        Symbol {
                            address: dev.address + offset,
                            is_sublabel: true,
                            parent_label: Some(dev.name.clone()),
                        },
                    );
                }
                offset += field.size as u16;
            }
            eprintln!(
                "DEBUG: Injected default device '{}' with {} fields (base=0x{:02X})",
                dev.name,
                dev.fields.len(),
                dev.address
            );
        }
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

        // --- REMOVE: do not patch metadata header at |0100 ---
        // let meta_addr = self.symbols.get("meta").map(|m| m.address).unwrap_or(0x019f);
        // println!(
        //     "DEBUG: About to patch |0100 with metadata header (a0 {:02x} {:02x} 80 06 37)",
        //     (meta_addr >> 8) & 0xff,
        //     meta_addr & 0xff
        // );
        // let page = 0x0100;
        // rom.write_byte_at(page, 0xa0)?;
        // rom.write_byte_at(page + 1, ((meta_addr >> 8) & 0xff) as u8)?;
        // rom.write_byte_at(page + 2, (meta_addr & 0xff) as u8)?;
        // rom.write_byte_at(page + 3, 0x80)?;
        // rom.write_byte_at(page + 4, 0x06)?;
        // rom.write_byte_at(page + 5, 0x37)?;
        // println!(
        //     "DEBUG: Patched |0100 with metadata header (a0 {:02x} {:02x} 80 06 37) for Varvara/uxn compatibility",
        //     (meta_addr >> 8) & 0xff,
        //     meta_addr & 0xff
        // );

        // Collect references into a temporary vector to avoid borrowing self mutably and immutably at the same time
        let references: Vec<_> = self.references.iter().cloned().collect();

        for reference in &references {
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

            if resolved_name.is_empty() || resolved_name == " " {
                eprintln!(
                    "DEBUG: resolved_name is empty for reference: {:?} (name='{}', rune='{}', scope={:?})",
                    reference, reference.name, reference.rune, reference.scope
                );
            }
            let symbol = self.find_symbol(&resolved_name, reference.scope.as_ref());
            // println!("DEBUG: Processing reference: {:?}", reference);
            // println!(
            //     "DEBUG: Resolving reference '{}' -> '{}' at {:04X} (scope: {:?})",
            //     reference.name, resolved_name, reference.address, reference.scope
            // );
            // println!("DEBUG: Found symbol: {:?}", symbol);

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

            let mut symbol = if symbol.is_none() {
                if reference.rune == '_' || reference.rune == ',' {
                    // --- PATCH: uxnasm-style scope walk for _ and , runes, even if tokens don't store scope ---
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

            // NEW: attempt device injection before failing
            if symbol.is_none() {
                let resolved_name_clone = resolved_name.clone();
                self.try_inject_device_symbols(&resolved_name_clone);
                // retry lookup after injection
                symbol = self.symbols.get(&resolved_name_clone);
            }

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
                    format!("Label unknown: \"{}\" DEBUG: resolved_name is empty for reference: {:?} (name='{}', rune='{}', scope={:?})",
                        resolved_name, reference, reference.name, reference.rune, reference.scope)
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

            // PATCH: uxnasm's relative word calculation for '?' rune is: rel = l->addr - r->addr - 2
            // But the bug is here: for the '?' rune, uxnasm.c uses rel = l->addr - r->addr - 2,
            // but writes it as a signed 16-bit value, not as an unsigned.
            // The difference in your output is that you write rel as (symbol.address as i32 - reference.address as i32 - 2) as i16,
            // but uxnasm.c writes it as (symbol.address - reference.address - 2) as Sint16, then stores it as a little-endian word.

            match reference.rune {
                '_' | ',' => {
                    // case '_': case ',': *rom = rel = l->addr - r->addr - 2;
                    let rel = (symbol.address as i32 - reference.address as i32 - 2) as i8;
                    rom.write_byte_at(reference.address, rel as u8)?;
                    // Update effective length if resolved value is non-zero
                    let end = reference.address as usize + 1;
                    if rel as u8 != 0 {
                        self.effective_length = self.effective_length.max(end);
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
                    let end = reference.address as usize + 1;
                    if symbol.address as u8 != 0 {
                        self.effective_length = self.effective_length.max(end);
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
                    let end = reference.address as usize + 2;
                    if symbol.address != 0 {
                        self.effective_length = self.effective_length.max(end);
                    }
                }
                '!' | '?' | ' ' | '/' => {
                    // For conditional ('?'), space (' '), and slash ('/') runes:
                    // rel = target_addr - ref_addr - 2 (matches uxnasm for relative word references)
                    let rel = (symbol.address as i32 - reference.address as i32 - 2) as i16;
                    // --- DEBUG PRINTS ---
                    println!(
                        "DEBUG: [second_pass] '{}': symbol.address=0x{:04X}, reference.address=0x{:04X}, rel={}(0x{:04X})",
                        reference.rune, symbol.address, reference.address, rel, rel as u16
                    );
                    // Write as little-endian (low byte first)
                                      rom.write_short_at(reference.address, rel as u16)?;
                    // rom.write_byte_at(reference.address, (rel & 0xff) as u8)?;
                    // rom.write_byte_at(reference.address + 1, ((rel >> 8) & 0xff) as u8)?;
                    eprintln!("DEBUG: Resolved reference '{}' at {:04X}: wrote relative address 0x{:04X} ({})", 
                             reference.name, reference.address, rel as u16, rel);
                    if rel != 0 {
                        self.effective_length =
                            self.effective_length.max(reference.address as usize + 2);
                    }
                }
                _ => {
                //     return Err(AssemblerError::SyntaxError {
                //         path: reference.path.clone(),
                //         line: reference.line,
                //         position: 0,
                //         message: format!("Unknown reference rune: {}", reference.rune),
                //         source_line: String::new(),
                //     });
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

        // Try direct match
        if let Some(symbol) = self.symbols.get(name) {
            return Some(symbol);
        }

        // For /down, try scope + "/" + name if not already present
        if name.starts_with('/') {
            if let Some(scope) = reference_scope {
                let main_scope = if let Some(pos) = scope.find('/') { &scope[..pos] } else { scope };
                let candidate = format!("{}/{}", main_scope, &name[1..]);
                if let Some(symbol) = self.symbols.get(&candidate) {
                    return Some(symbol);
                }
            }
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

    /// Process an include directive by reading and assembling the included file, using token for error context
    fn process_include_with_token(&mut self, path: &str, tok: &TokenWithPos, rom: &mut Rom) -> Result<()> {
        println!("DEBUG: Current working directory: {:?}", std::env::current_dir());
        println!("DEBUG: Including file at path: {}", path);
        let content = match fs::read_to_string(path) {
            Ok(content) => content,
            Err(e) => {
            // Try path without its parent directory if it has one
            if let Some(filename) = std::path::Path::new(path).file_name() {
                let filename_str = filename.to_string_lossy();
                if let Ok(content2) = fs::read_to_string(filename_str.as_ref()) {
                println!("DEBUG: Fallback include succeeded with filename '{}'", filename_str);
                // Use the fallback content
                content2
                } else {
                return Err(AssemblerError::SyntaxError {
                    path: rom.source_path().cloned().unwrap_or_default(),
                    line: tok.line,
                    position: tok.start_pos,
                    message: format!("Failed to read include file '{}' '{}': {}", filename_str.as_ref(), path, e),
                    source_line: String::new(),
                });
                }
            } else {
                return Err(AssemblerError::SyntaxError {
                path: rom.source_path().cloned().unwrap_or_default(),
                line: tok.line,
                position: tok.start_pos,
                message: format!("Failed to read include file '{}': {}", path, e),
                source_line: String::new(),
                });
            }
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
                                self.insert_symbol_if_new(&sublabel, Symbol {
                                    address: base_addr + offset,
                                    is_sublabel: true,
                                    parent_label: Some(device.clone()),
                                });
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

    /// Helper: resolve leading-slash label relative to main scope.
    /// raw: original token (may start with '/')
    /// scope_opt: optional current scope (e.g., from token.scope or current_label)
    fn resolve_relative_label(&self, raw: &str, scope_opt: Option<&String>) -> String {
        if !raw.starts_with('/') {
            return raw.to_string();
        }
        let name = &raw[1..];
        if let Some(scope) = scope_opt {
            // Main scope is the part before the first '/'
            let main_scope = if let Some(pos) = scope.find('/') { &scope[..pos] } else { scope };
            format!("{}/{}", main_scope, name)
        } else {
            name.to_string()
        }
    }

        fn prune_lambda_aliases(&mut self) {
        // addresses that have at least one non-λ (i.e., real) symbol
        let named_addrs: std::collections::HashSet<u16> = self
            .symbols
            .iter()
            .filter(|(n, _)| !n.starts_with('λ'))
            .map(|(_, s)| s.address)
            .collect();

        // collect λ-names that live at those addresses
        let to_remove: Vec<String> = self
            .symbols
            .iter()
            .filter(|(n, _)| n.starts_with('λ'))
            .filter(|(_, s)| named_addrs.contains(&s.address))
            .map(|(n, _)| n.clone())
            .collect();

        for name in to_remove {
            self.symbols.remove(&name);
            if let Some(i) = self.symbol_order.iter().position(|x| *x == name) {
                self.symbol_order.remove(i);
            }
            eprintln!("DEBUG: pruned λ alias '{}' (address already named)", name);
        }
    }
}

// Helper to format lambda label (e.g., λ1, λ2, ... single hex, no leading zero)
fn format_lambda_label(lambda_id: usize) -> String {
    format!("λ{:02x}", lambda_id)
}

impl Default for Assembler {
    fn default() -> Self {
        Self::new()
    }
}