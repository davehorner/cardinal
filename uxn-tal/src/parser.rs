//! Parser for TAL assembly language

use crate::error::{AssemblerError, Result};
use crate::lexer::{Token, TokenWithPos};
use crate::opcode_table::UXN_OPCODE_TABLE;
use crate::opcodes::Opcodes;
use std::collections::HashMap;

/// Represents a parsed instruction with modes
#[derive(Debug, Clone)]
pub struct Instruction {
    pub opcode: String,
    pub short_mode: bool,
    pub return_mode: bool,
    pub keep_mode: bool,
}

/// AST node types
#[derive(Debug, Clone)]
pub enum AstNode {
    Ignore, // Used for empty nodes or ignored sections
    /// Raw byte value
    Byte(u8),
    /// 16-bit short value
    Short(u16),
    /// Literal byte value (prefixed with #)
    LiteralByte(u8),
    /// Literal short value (prefixed with #)
    LiteralShort(u16),
    /// Instruction with mode flags
    Instruction(Instruction),
    /// Label definition
    LabelDef(String),
    /// Label reference
    LabelRef(String),
    /// Sublabel definition
    SublabelDef(String),
    /// Sublabel reference
    SublabelRef(String),
    /// Relative address reference
    RelativeRef(String),
    /// Conditional jump reference  
    ConditionalRef(String),
    /// Conditional block (e.g., ?{ ... })
    ConditionalBlock(Vec<AstNode>),
    /// Raw address reference
    RawAddressRef(String),
    /// JSR call reference
    JSRRef(String),
    /// Hyphen address reference
    HyphenRef(String),
    /// Padding to specific address
    Padding(u16),
    PaddingLabel(String),
    /// Skip N bytes
    Skip(u16),
    /// Device access (e.g., .Screen/width)
    DeviceAccess(String, String), // device, field
    /// Macro definition
    MacroDef(String, Vec<AstNode>), // name, body
    /// Macro call (name, line, position)
    MacroCall(String, usize, usize),
    /// Raw string data
    RawString(Vec<u8>),
    /// Include directive
    Include(String),
    /// Dot reference - generates LIT + 8-bit address (like uxnasm's '.' rune)
    DotRef(String),
    /// Semicolon reference - generates LIT2 + 16-bit address (like uxnasm's ';' rune)  
    SemicolonRef(String),
    /// Equals reference - generates 16-bit address directly (like uxnasm's '=' rune)
    EqualsRef(String),
    /// Comma reference - generates LIT + relative 8-bit address (like uxnasm's ',' rune)
    CommaRef(String),
    /// Underscore reference - generates relative 8-bit address (like uxnasm's '_' rune)
    UnderscoreRef(String),
    /// Question reference - generates conditional jump (like uxnasm's '?' rune)
    QuestionRef(String),
    /// Exclamation reference - generates JSR call (like uxnasm's '!' rune)
    ExclamationRef(String),
}

/// Parser for TAL assembly
pub struct Parser {
    tokens: Vec<TokenWithPos>,
    position: usize,
    line: usize,
    position_in_line: usize,
    path: String,
    source: String,
    current_scope: Option<String>,
    labels: HashMap<String, u16>,
    current_address: u16,
}

impl Parser {
    pub fn new_with_source(tokens: Vec<TokenWithPos>, path: String, source: String) -> Self {
        let line = tokens.get(0).map(|t| t.line).unwrap_or(1);
        let position_in_line = tokens.get(0).map(|t| t.start_pos).unwrap_or(1);
        Self {
            tokens,
            position: 0,
            line,
            position_in_line,
            path,
            source,
            current_scope: None,
            labels: HashMap::new(),
            current_address: 0x100,
        }
    }

    /// Parse tokens into AST nodes
    pub fn parse(&mut self) -> Result<Vec<AstNode>> {
        let mut nodes = Vec::new();

        while !self.is_at_end() {
            let tok_with_pos = self.current_token();
            // Print path:line:start_pos-end_pos for every token
            println!(
                "{}:{}:{}-{} {:?}",
                self.path,
                tok_with_pos.line,
                tok_with_pos.start_pos,
                tok_with_pos.end_pos,
                tok_with_pos.token
            );
            let token = &tok_with_pos.token;
            match token {
                Token::Newline => {
                    self.advance();
                    continue;
                }
                Token::Comment(comment) => {
                    println!("Comment({:?})", comment);
                    self.advance();
                    continue;
                }
                Token::BracketOpen | Token::BracketClose => {
                    // Completely ignore brackets - they're just ignored in uxnasm.c
                    self.advance();
                    continue;
                }
                _ => {
                    let node = self.parse_node()?;
                    println!("{:?}", node);
                    nodes.push(node);
                }
            }
        }

        Ok(nodes)
    }

    fn parse_node(&mut self) -> Result<AstNode> {
        // Robustly skip newlines before parsing a node
        // Robustly skip newlines and comments before parsing a node
        loop {
            let token = &self.current_token().token;
            match token {
                Token::Newline | Token::Comment(_) => self.advance(),
                Token::BraceClose => {
                    self.advance();
                    continue;
                }
                _ => break,
            }
        }
        let token = &self.current_token().token;
        match token {
            Token::HexLiteral(hex) => {
                let value = self.parse_hex_literal(hex)?;
                let hex_len = hex.len();
                self.advance();
                if hex_len <= 2 {
                    Ok(AstNode::LiteralByte(value as u8))
                } else {
                    Ok(AstNode::LiteralShort(value))
                }
            }
            Token::RawHex(hex) => {
                let value = self.parse_hex_literal(hex)?;
                self.advance();
                if value <= 255 {
                    Ok(AstNode::Byte(value as u8))
                } else {
                    Ok(AstNode::Short(value))
                }
            }
            Token::DecLiteral(dec) => {
                let line = self.current_token().line;
                let position = self.current_token().start_pos;
                let value = dec
                    .parse::<u16>()
                    .map_err(|_| AssemblerError::SyntaxError {
                        path: self.path.clone(),
                        line,
                        position,
                        message: format!("Invalid decimal literal: {}", dec),
                        source_line: self.get_source_line(line),
                    })?;
                self.advance();
                if value <= 255 {
                    Ok(AstNode::LiteralByte(value as u8))
                } else {
                    Ok(AstNode::LiteralShort(value))
                }
            }
            Token::BinLiteral(bin) => {
                let line = self.current_token().line;
                let position = self.current_token().start_pos;
                let value =
                    u16::from_str_radix(bin, 2).map_err(|_| AssemblerError::SyntaxError {
                        path: self.path.clone(),
                        line,
                        position,
                        message: format!("Invalid binary literal: {}", bin),
                        source_line: self.get_source_line(line),
                    })?;
                self.advance();
                if value <= 255 {
                    Ok(AstNode::LiteralByte(value as u8))
                } else {
                    Ok(AstNode::LiteralShort(value))
                }
            }
            Token::CharLiteral(ch) => {
                let value = *ch as u8;
                self.advance();
                Ok(AstNode::Byte(value))
            }
            Token::Instruction(inst) => {
                let inst = inst.clone();
                self.advance();
                let ast_node = self.parse_instruction(inst)?;
                if let AstNode::Instruction(instr) = ast_node {
                    Ok(AstNode::Instruction(instr))
                } else {
                    Err(AssemblerError::SyntaxError {
                        path: self.path.clone(),
                        line: self.line,
                        position: self.position_in_line,
                        message: "Expected instruction node".to_string(),
                        source_line: self
                            .source
                            .lines()
                            .nth(self.line.saturating_sub(1))
                            .unwrap_or("")
                            .to_string(),
                    })
                }
            }
            Token::LabelDef(label) => {
                let label = label.clone();
                self.advance();
                Ok(AstNode::LabelDef(label))
            }
            Token::LabelRef(label) => {
                let label = label.clone();
                self.advance();
                Ok(AstNode::LabelRef(label))
            }
            Token::SublabelDef(sublabel) => {
                let sublabel = sublabel.clone();
                self.advance();
                Ok(AstNode::SublabelDef(sublabel))
            }
            Token::SublabelRef(sublabel) => {
                let sublabel = sublabel.clone();
                self.advance();
                Ok(AstNode::SublabelRef(sublabel))
            }
            Token::RelativeRef(label) => {
                let label = label.clone();
                self.advance();
                Ok(AstNode::RelativeRef(label))
            }
            Token::ConditionalRef(label) => {
                let label = label.clone();
                self.advance();
                Ok(AstNode::ConditionalRef(label))
            }
            Token::DotRef(label) => {
                let label = label.clone();
                self.advance();
                Ok(AstNode::DotRef(label))
            }
            Token::SemicolonRef(label) => {
                let label = label.clone();
                self.advance();
                Ok(AstNode::SemicolonRef(label))
            }
            Token::EqualsRef(label) => {
                let label = label.clone();
                self.advance();
                Ok(AstNode::EqualsRef(label))
            }
            Token::CommaRef(label) => {
                let label = label.clone();
                self.advance();
                Ok(AstNode::CommaRef(label))
            }
            Token::UnderscoreRef(label) => {
                let label = label.clone();
                self.advance();
                Ok(AstNode::UnderscoreRef(label))
            }
            Token::QuestionRef(label) => {
                let label = label.clone();
                self.advance();
                Ok(AstNode::QuestionRef(label))
            }
            Token::ExclamationRef(label) => {
                let label = label.clone();
                self.advance();
                Ok(AstNode::ExclamationRef(label))
            }
            Token::ConditionalOperator => {
                self.advance();
                // Skip any newlines or comments after '?'
                while matches!(
                    &self.current_token().token,
                    Token::Newline | Token::Comment(_)
                ) {
                    self.advance();
                }
                let next_token = &self.current_token().token;
                match next_token {
                    Token::BraceOpen => {
                        self.advance();
                        let mut block_nodes = Vec::new();
                        while !matches!(&self.current_token().token, Token::BraceClose | Token::Eof)
                        {
                            let token = &self.current_token().token;
                            match token {
                                Token::BracketOpen | Token::BracketClose => {
                                    // Ignore brackets in conditional blocks too
                                    self.advance();
                                    continue;
                                }
                                Token::Comment(_) | Token::Newline => {
                                    self.advance();
                                    continue;
                                }
                                _ => {
                                    block_nodes.push(self.parse_node()?);
                                }
                            }
                        }
                        if matches!(&self.current_token().token, Token::BraceClose) {
                            self.advance();
                        } else {
                            let line = self.current_token().line;
                            let position = self.current_token().start_pos;
                            return Err(AssemblerError::SyntaxError {
                                path: self.path.clone(),
                                line,
                                position,
                                message: "Expected '}' after conditional block".to_string(),
                                source_line: self.get_source_line(line),
                            });
                        }
                        Ok(AstNode::ConditionalBlock(block_nodes))
                    }
                    _ => {
                        let line = self.current_token().line;
                        let position = self.current_token().start_pos;
                        Err(AssemblerError::SyntaxError {
                            path: self.path.clone(),
                            line,
                            position,
                            message: "Conditional operator '?' must be followed by a block"
                                .to_string(),
                            source_line: self.get_source_line(line),
                        })
                    }
                }
            }
            Token::RawAddressRef(label) => {
                let label = label.clone();
                self.advance();
                Ok(AstNode::RawAddressRef(label))
            }
            Token::JSRRef(label) => {
                let label = label.clone();
                self.advance();
                Ok(AstNode::JSRRef(label))
            }
            Token::HyphenRef(identifier) => {
                let identifier = identifier.clone();
                self.advance();
                Ok(AstNode::HyphenRef(identifier))
            }
            Token::Padding(addr) => {
                let addr = *addr;
                self.advance();
                Ok(AstNode::Padding(addr))
            }
            Token::PaddingLabel(label) => {
                let label = label.clone();
                self.advance();
                Ok(AstNode::PaddingLabel(label))
            }
            Token::Skip(count) => {
                let count = *count;
                self.advance();
                if let Token::LabelDef(label) = &self.current_token().token {
                    let label_owned = label.clone();
                    self.advance();
                    Ok(AstNode::LabelDef(label_owned))
                } else {
                    Ok(AstNode::Skip(count))
                }
            }
            Token::DeviceAccess(device, field) => {
                let device = device.clone();
                let field = field.clone();
                self.advance();
                Ok(AstNode::DeviceAccess(device, field))
            }
            Token::MacroDef(name) => {
                let name = name.clone();
                self.advance();

                while matches!(
                    &self.current_token().token,
                    Token::Comment(_) | Token::Newline
                ) {
                    self.advance();
                }

                match &self.current_token().token {
                    Token::BraceOpen => {
                        self.advance();
                        let mut body = Vec::new();

                        while !matches!(&self.current_token().token, Token::BraceClose | Token::Eof)
                        {
                            let token = &self.current_token().token;
                            match token {
                                Token::Comment(_) | Token::Newline => {
                                    self.advance();
                                    continue;
                                }
                                _ => {
                                    body.push(self.parse_node()?);
                                }
                            }
                        }

                        if matches!(&self.current_token().token, Token::BraceClose) {
                            self.advance();
                        } else {
                            let line = self.current_token().line;
                            let position = self.current_token().start_pos;
                            return Err(AssemblerError::SyntaxError {
                                path: self.path.clone(),
                                line,
                                position,
                                message: "Expected '}' after macro body".to_string(),
                                source_line: self.get_source_line(line),
                            });
                        }

                        Ok(AstNode::MacroDef(name, body))
                    }
                    _ => {
                        let line = self.current_token().line;
                        let position = self.current_token().start_pos;
                        Err(AssemblerError::SyntaxError {
                            path: self.path.clone(),
                            line,
                            position,
                            message: "Expected '{' after macro name".to_string(),
                            source_line: self.get_source_line(line),
                        })
                    }
                }
            }
            Token::RawString(string) => {
                let bytes = string.as_bytes().to_vec();
                self.advance();
                Ok(AstNode::RawString(bytes))
            }
            Token::Include(path) => {
                let path = path.clone();
                self.advance();
                Ok(AstNode::Include(path))
            }
            Token::MacroCall(name) => {
                let name = name.clone();
                let macro_line = self.current_token().line;
                let macro_pos = self.current_token().start_pos;
                self.advance();
                
                // In uxnasm.c, <name> syntax is always treated as a label reference, not a macro call
                // Macros are defined with % and called by their bare name
                // Preserve the angle brackets in the label name to match uxnasm.c behavior
                let label_name = format!("<{}>", name);
                Ok(AstNode::LabelRef(label_name))
            }
            _ => {
                let line = self.current_token().line;
                let position = self.current_token().start_pos;
                Err(AssemblerError::SyntaxError {
                    path: self.path.clone(),
                    line,
                    position,
                    message: format!("Unexpected token: {:?}", self.current_token().token),
                    source_line: self.get_source_line(line),
                })
            }
        }
    }

    fn parse_instruction(&mut self, name: String) -> Result<AstNode> {
        let opcodes = Opcodes::new();
        let mut opcode = name.clone();
        let mut short_mode = false;
        let mut return_mode = false;
        let mut keep_mode = false;

        // Debug output for tracing
        eprintln!("DEBUG: parse_instruction input name: '{}'", name);

        // Always parse mode flags for all instructions, including LIT/LIT2/LIT2r/LITr
        let mut base = name.as_str();
        let mut mode_chars = String::new();
        while let Some(last) = base.chars().last() {
            if last == 'k' || last == 'r' || last == '2' {
                mode_chars.insert(0, last);
                base = &base[..base.len() - 1];
            } else {
                break;
            }
        }
        eprintln!("DEBUG: base after stripping flags: '{}', mode_chars: '{}'", base, mode_chars);

        if opcodes.get_opcode(base).is_ok() {
            opcode = base.to_string();
            for c in mode_chars.chars() {
                match c {
                    'k' => {
                        keep_mode = true;
                        eprintln!("DEBUG: found 'k' flag, keep_mode = true");
                    }
                    'r' => {
                        return_mode = true;
                        eprintln!("DEBUG: found 'r' flag, return_mode = true");
                    }
                    '2' => {
                        short_mode = true;
                        eprintln!("DEBUG: found '2' flag, short_mode = true");
                    }
                    _ => {}
                }
            }
        } else {
            opcode = name.clone();
            eprintln!("DEBUG: base '{}' not found in opcode table, using original name '{}'", base, name);
        }

        // For LIT/LIT2/LITr/LIT2r, always use base "LIT" and set flags accordingly
        if opcode == "LIT" || name.starts_with("LIT") {
            eprintln!("DEBUG: opcode is 'LIT' or starts with 'LIT', checking for '2' and 'r' in name '{}'", name);
            if name.contains('2') {
                short_mode = true;
                eprintln!("DEBUG: name contains '2', short_mode = true");
            }
            if name.contains('r') {
                return_mode = true;
                eprintln!("DEBUG: name contains 'r', return_mode = true");
            }
            keep_mode = true;
            eprintln!("DEBUG: LIT always sets keep_mode = true");
            opcode = "LIT".to_string();
        }

        eprintln!(
            "DEBUG: parse_instruction result: opcode='{}', short_mode={}, return_mode={}, keep_mode={}",
            opcode, short_mode, return_mode, keep_mode
        );

        Ok(AstNode::Instruction(Instruction {
            opcode,
            short_mode,
            return_mode,
            keep_mode,
        }))
    }

    fn parse_hex_literal(&self, hex: &str) -> Result<u16> {
        u16::from_str_radix(hex, 16).map_err(|_| AssemblerError::SyntaxError {
            path: self.path.clone(),
            line: self.line,
            position: self.position_in_line,
            message: format!("Invalid hexadecimal literal: {}", hex),
            source_line: self
                .source
                .lines()
                .nth(self.line - 1)
                .unwrap_or("")
                .to_string(),
        })
    }

    fn current_token(&self) -> &TokenWithPos {
        self.tokens.get(self.position).unwrap_or(&TokenWithPos {
            token: Token::Eof,
            line: 0,
            start_pos: 0,
            end_pos: 0,
        })
    }

    fn advance(&mut self) {
        self.position += 1;
        if let Some(tok) = self.tokens.get(self.position) {
            self.line = tok.line;
            self.position_in_line = tok.start_pos;
        }
    }

    fn is_at_end(&self) -> bool {
        self.position >= self.tokens.len()
    }

    fn get_source_line(&self, line: usize) -> String {
        self.source
            .lines()
            .nth(line.saturating_sub(1))
            .unwrap_or("")
            .to_string()
    }

    fn parse_label_definition(&mut self) -> Result<AstNode> {
        if let Token::LabelDef(label) = &self.current_token().token {
            let label = label.clone();
            self.advance();
            Ok(AstNode::LabelDef(label))
        } else {
            Err(AssemblerError::SyntaxError {
                path: self.path.clone(),
                line: self.current_token().line,
                position: self.current_token().start_pos,
                message: "Expected label definition".to_string(),
                source_line: self.get_source_line(self.current_token().line),
            })
        }
    }

    fn parse_sublabel_definition(&mut self) -> Result<AstNode> {
        if let Token::SublabelDef(sublabel) = &self.current_token().token {
            let sublabel = sublabel.clone();
            self.advance();
            Ok(AstNode::SublabelDef(sublabel))
        } else {
            Err(AssemblerError::SyntaxError {
                path: self.path.clone(),
                line: self.current_token().line,
                position: self.current_token().start_pos,
                message: "Expected sublabel definition".to_string(),
                source_line: self.get_source_line(self.current_token().line),
            })
        }
    }

    fn parse_padding(&mut self) -> Result<AstNode> {
        match &self.current_token().token {
            Token::Padding(addr) => {
                let addr = *addr;
                self.advance();
                Ok(AstNode::Padding(addr))
            }
            Token::PaddingLabel(label) => {
                let label = label.clone();
                self.advance();
                Ok(AstNode::PaddingLabel(label))
            }
            _ => Err(AssemblerError::SyntaxError {
                path: self.path.clone(),
                line: self.current_token().line,
                position: self.current_token().start_pos,
                message: "Expected padding directive".to_string(),
                source_line: self.get_source_line(self.current_token().line),
            }),
        }
    }

    fn parse_expression(&mut self) -> Result<AstNode> {
        match &self.current_token().token {
            Token::Padding(_) | Token::PaddingLabel(_) => self.parse_padding(),
            Token::LabelDef(_) => self.parse_label_definition(),
            Token::SublabelDef(_) => self.parse_sublabel_definition(),
            Token::Skip(count) => {
                let count = *count;
                self.advance();
                Ok(AstNode::Skip(count))
            }
            // ...existing code for other tokens...
            _ => Err(AssemblerError::SyntaxError {
                path: self.path.clone(),
                line: self.current_token().line,
                position: self.current_token().start_pos,
                message: format!("Unexpected token: {:?}", self.current_token().token),
                source_line: self.get_source_line(self.current_token().line),
            }),
        }
    }

    fn resolve_label_reference(&self, name: &str) -> Result<u16> {
        // Handle sublabel references that start with &
        if name.starts_with('&') {
            let sublabel_name = &name[1..]; // Remove the & prefix
            if let Some(current_scope) = &self.current_scope {
                let full_name = format!("{}/{}", current_scope, sublabel_name);
                if let Some(address) = self.labels.get(&full_name) {
                    return Ok(*address);
                }
            }
            // If not found in current scope, try global scope
            if let Some(address) = self.labels.get(sublabel_name) {
                return Ok(*address);
            }
        }

        // Handle regular label references
        if let Some(address) = self.labels.get(name) {
            return Ok(*address);
        }

        // Handle scoped references (label/sublabel format)
        if name.contains('/') {
            if let Some(address) = self.labels.get(name) {
                return Ok(*address);
            }
        }

        Err(AssemblerError::SyntaxError {
            path: self.path.clone(),
            line: 0,
            position: 0,
            message: format!("Label unknown: {}", name),
            source_line: self.get_source_line(0),
        })
    }

    fn define_label(&mut self, name: String) -> Result<()> {
        let address = self.current_address;
        self.labels.insert(name.clone(), address);

        // Update current scope for regular labels
        if !name.starts_with('&') {
            self.current_scope = Some(name);
        }

        Ok(())
    }

    fn define_sublabel(&mut self, name: String) -> Result<()> {
        let address = self.current_address;

        // Store sublabel with current scope prefix
        if let Some(scope) = &self.current_scope {
            let full_name = format!("{}/{}", scope, name);
            self.labels.insert(full_name, address);
        }

        // Also store without scope for backward compatibility
        self.labels.insert(name, address);

        Ok(())
    }
}

// Add this helper function at the bottom of the file (or near is_instruction_name in lexer.rs)
fn is_known_instruction(opcode: &str) -> bool {
    // Use the UXN_OPCODE_TABLE for dynamic lookup
    UXN_OPCODE_TABLE.iter().any(|&(_, name)| name == opcode)
}