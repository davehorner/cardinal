//! Parser for TAL assembly language

use crate::error::{AssemblerError, Result};
use crate::lexer::Token;

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
    /// Skip N bytes
    Skip(u16),
    /// Device access (e.g., .Screen/width)
    DeviceAccess(String, String), // device, field
    /// Macro definition
    MacroDef(String, Vec<AstNode>), // name, body
    /// Macro call
    MacroCall(String),
    /// Raw string data
    RawString(Vec<u8>),
    /// Inline assembly block
    InlineAssembly(Vec<AstNode>),
    /// Include directive
    Include(String),
}

/// Parser for TAL assembly
pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
    line: usize,
    path: String,
    source: String,
    position_in_line: usize,
}

impl Parser {
    pub fn new_with_source(
        tokens: Vec<Token>,
        path: String,
        source: String,
    ) -> Self {
        Self {
            tokens,
            position: 0,
            line: 1,
            path,
            source,
            position_in_line: 0,
        }
    }

    /// Parse tokens into AST nodes
    pub fn parse(&mut self) -> Result<Vec<AstNode>> {
        let mut nodes = Vec::new();

        while !self.is_at_end() {
            match self.current_token() {
                Token::Newline => {
                    self.advance();
                    continue;
                }
                Token::Comment(_) => {
                    self.advance();
                    continue;
                }
                _ => {
                    nodes.push(self.parse_node()?);
                }
            }
        }

        Ok(nodes)
    }

    fn parse_node(&mut self) -> Result<AstNode> {
        // Robustly skip newlines before parsing a node
        // Robustly skip newlines and comments before parsing a node
        loop {
            match self.current_token() {
                Token::Newline | Token::Comment(_) => self.advance(),
                _ => break,
            }
        }
        match self.current_token().clone() {
            Token::HexLiteral(hex) => {
                let value = self.parse_hex_literal(&hex)?;
                let hex_len = hex.len();
                self.advance();
                if hex_len <= 2 {
                    Ok(AstNode::LiteralByte(value as u8))
                } else {
                    Ok(AstNode::LiteralShort(value))
                }
            }
            Token::RawHex(hex) => {
                let value = self.parse_hex_literal(&hex)?;
                self.advance();
                if value <= 255 {
                    Ok(AstNode::Byte(value as u8))
                } else {
                    Ok(AstNode::Short(value))
                }
            }
            Token::DecLiteral(dec) => {
                let value = dec.parse::<u16>().map_err(|_| {
                    AssemblerError::SyntaxError {
                        path: self.path.clone(),
                        line: self.line,
                        position: self.position_in_line,
                        message: format!("Invalid decimal literal: {}", dec),
                        source_line: self
                            .source
                            .lines()
                            .nth(self.line)
                            .unwrap_or("")
                            .to_string(),
                    }
                })?;
                self.advance();
                if value <= 255 {
                    Ok(AstNode::LiteralByte(value as u8))
                } else {
                    Ok(AstNode::LiteralShort(value))
                }
            }
            Token::BinLiteral(bin) => {
                let value = u16::from_str_radix(&bin, 2).map_err(|_| {
                    AssemblerError::SyntaxError {
                        path: self.path.clone(),
                        line: self.line,
                        position: self.position_in_line,
                        message: format!("Invalid binary literal: {}", bin),
                        source_line: self
                            .source
                            .lines()
                            .nth(self.line)
                            .unwrap_or("")
                            .to_string(),
                    }
                })?;
                self.advance();
                if value <= 255 {
                    Ok(AstNode::LiteralByte(value as u8))
                } else {
                    Ok(AstNode::LiteralShort(value))
                }
            }
            Token::CharLiteral(ch) => {
                let value = ch as u8;
                self.advance();
                Ok(AstNode::Byte(value))
            }
            Token::Instruction(inst) => {
                let instruction = self.parse_instruction(&inst)?;
                self.advance();
                Ok(AstNode::Instruction(instruction))
            }
            Token::LabelDef(label) => {
                self.advance();
                Ok(AstNode::LabelDef(label))
            }
            Token::LabelRef(label) => {
                self.advance();
                Ok(AstNode::LabelRef(label))
            }
            Token::SublabelDef(sublabel) => {
                self.advance();
                Ok(AstNode::SublabelDef(sublabel))
            }
            Token::SublabelRef(sublabel) => {
                self.advance();
                Ok(AstNode::SublabelRef(sublabel))
            }
            Token::RelativeRef(label) => {
                self.advance();
                Ok(AstNode::RelativeRef(label))
            }
            Token::ConditionalRef(label) => {
                self.advance();
                Ok(AstNode::ConditionalRef(label))
            }
            Token::ConditionalOperator => {
                self.advance();
                // Check if next token is BraceOpen to form a conditional block
                if matches!(self.current_token(), Token::BraceOpen) {
                    self.advance(); // consume the '{'
                    let mut block_nodes = Vec::new();

                    // Parse nodes until we find a closing brace
                    while !matches!(
                        self.current_token(),
                        Token::BraceClose | Token::Eof
                    ) {
                        block_nodes.push(self.parse_node()?);
                    }

                    // Expect closing brace
                    if matches!(self.current_token(), Token::BraceClose) {
                        self.advance();
                    } else {
                        return Err(AssemblerError::SyntaxError {
                            path: self.path.clone(),
                            line: self.line,
                            position: self.position_in_line,
                            message: "Expected '}' after conditional block"
                                .to_string(),
                            source_line: self
                                .source
                                .lines()
                                .nth(self.line)
                                .unwrap_or("")
                                .to_string(),
                        });
                    }

                    Ok(AstNode::ConditionalBlock(block_nodes))
                } else {
                    // Standalone conditional operator - this is an error for now
                    Err(AssemblerError::SyntaxError {
                        path: self.path.clone(),
                        line: self.line,
                        position: self.position_in_line,
                        message: "Conditional operator '?' must be followed by a block or label".to_string(),
                        source_line: self.source.lines().nth(self.line).unwrap_or("").to_string(),
                    })
                }
            }
            Token::ConditionalBlockStart => {
                self.advance();
                let mut block_nodes = Vec::new();

                // Parse nodes until we find a closing brace
                while !matches!(
                    self.current_token(),
                    Token::BraceClose | Token::Eof
                ) {
                    block_nodes.push(self.parse_node()?);
                }

                // Expect closing brace
                if matches!(self.current_token(), Token::BraceClose) {
                    self.advance();
                } else {
                    return Err(AssemblerError::SyntaxError {
                        path: self.path.clone(),
                        line: self.line,
                        position: self.position_in_line,
                        message: "Expected '}' after conditional block"
                            .to_string(),
                        source_line: self
                            .source
                            .lines()
                            .nth(self.line)
                            .unwrap_or("")
                            .to_string(),
                    });
                }

                Ok(AstNode::ConditionalBlock(block_nodes))
            }
            Token::RawAddressRef(label) => {
                self.advance();
                Ok(AstNode::RawAddressRef(label))
            }
            Token::JSRRef(label) => {
                self.advance();
                Ok(AstNode::JSRRef(label))
            }
            Token::HyphenRef(identifier) => {
                self.advance();
                Ok(AstNode::HyphenRef(identifier))
            }
            Token::Padding(addr) => {
                self.advance();
                Ok(AstNode::Padding(addr))
            }
            Token::Skip(count) => {
                self.advance();
                Ok(AstNode::Skip(count))
            }
            Token::DeviceAccess(device, field) => {
                self.advance();
                Ok(AstNode::DeviceAccess(device, field))
            }
            Token::BraceOpen => {
                // Raw data block { ... }
                self.advance();
                let mut data = Vec::new();

                while !matches!(
                    self.current_token(),
                    Token::BraceClose | Token::Eof
                ) {
                    match self.current_token() {
                        Token::Comment(_) | Token::Newline => {
                            self.advance();
                            continue;
                        }
                        _ => {
                            let node = self.parse_node()?;
                            match node {
                                AstNode::Byte(b) => data.push(b),
                                AstNode::LiteralByte(b) => data.push(b),
                                AstNode::RawString(bytes) => data.extend(bytes),
                                _ => {
                                    return Err(AssemblerError::SyntaxError {
                                        path: self.path.clone(),
                                        line: self.line,
                                        position: self.position_in_line,
                                        message: "Only raw bytes and strings allowed in data blocks".to_string(),
                                        source_line: self.source.lines().nth(self.line).unwrap_or("").to_string(),
                                    });
                                }
                            }
                        }
                    }
                }

                if matches!(self.current_token(), Token::BraceClose) {
                    self.advance();
                } else {
                    return Err(AssemblerError::SyntaxError {
                        path: self.path.clone(),
                        line: self.line,
                        position: self.position_in_line,
                        message: "Expected '}' after data block".to_string(),
                        source_line: self
                            .source
                            .lines()
                            .nth(self.line)
                            .unwrap_or("")
                            .to_string(),
                    });
                }

                Ok(AstNode::RawString(data))
            }
            Token::MacroDef(name) => {
                self.advance();

                // Skip comments and newlines before opening brace
                while matches!(
                    self.current_token(),
                    Token::Comment(_) | Token::Newline
                ) {
                    self.advance();
                }

                // Expect opening brace
                match self.current_token() {
                    Token::BraceOpen => {
                        self.advance();
                        let mut body = Vec::new();

                        // Parse macro body until closing brace
                        while !matches!(
                            self.current_token(),
                            Token::BraceClose | Token::Eof
                        ) {
                            match self.current_token() {
                                Token::Comment(_) | Token::Newline => {
                                    self.advance();
                                    continue;
                                }
                                _ => {
                                    body.push(self.parse_node()?);
                                }
                            }
                        }

                        // Expect closing brace
                        if matches!(self.current_token(), Token::BraceClose) {
                            self.advance();
                        } else {
                            return Err(AssemblerError::SyntaxError {
                                path: self.path.clone(),
                                line: self.line,
                                position: self.position_in_line,
                                message: "Expected '}' after macro body"
                                    .to_string(),
                                source_line: self
                                    .source
                                    .lines()
                                    .nth(self.line)
                                    .unwrap_or("")
                                    .to_string(),
                            });
                        }

                        Ok(AstNode::MacroDef(name, body))
                    }
                    _ => Err(AssemblerError::SyntaxError {
                        path: self.path.clone(),
                        line: self.line,
                        position: self.position_in_line,
                        message: "Expected '{' after macro name".to_string(),
                        source_line: self
                            .source
                            .lines()
                            .nth(self.line)
                            .unwrap_or("")
                            .to_string(),
                    }),
                }
            }
            Token::RawString(string) => {
                let bytes = string.as_bytes().to_vec();
                self.advance();
                Ok(AstNode::RawString(bytes))
            }
            Token::BracketOpen => {
                self.advance();
                let mut assembly_nodes = Vec::new();

                // Parse inline assembly until closing bracket
                while !matches!(
                    self.current_token(),
                    Token::BracketClose | Token::Eof
                ) {
                    assembly_nodes.push(self.parse_node()?);
                }

                // Expect closing bracket
                if matches!(self.current_token(), Token::BracketClose) {
                    self.advance();
                } else {
                    return Err(AssemblerError::SyntaxError {
                        path: self.path.clone(),
                        line: self.line,
                        position: self.position_in_line,
                        message: "Expected ']' after inline assembly block"
                            .to_string(),
                        source_line: self
                            .source
                            .lines()
                            .nth(self.line)
                            .unwrap_or("")
                            .to_string(),
                    });
                }

                Ok(AstNode::InlineAssembly(assembly_nodes))
            }
            Token::Include(path) => {
                self.advance();
                Ok(AstNode::Include(path))
            }
            Token::MacroCall(name) => {
                // Only treat as macro call if a macro is defined; otherwise treat as label reference (routine call)
                // (In TAL, <word> is a routine call unless %macro is defined)
                if self
                    .tokens
                    .iter()
                    .any(|t| matches!(t, Token::MacroDef(def) if def == &name))
                {
                    self.advance();
                    Ok(AstNode::MacroCall(name))
                } else {
                    self.advance();
                    Ok(AstNode::LabelRef(name))
                }
            }
            _ => Err(AssemblerError::SyntaxError {
                path: self.path.clone(),
                line: self.line,
                position: self.position_in_line,
                message: format!(
                    "Unexpected token: {:?}",
                    self.current_token()
                ),
                source_line: self
                    .source
                    .lines()
                    .nth(self.line)
                    .unwrap_or("")
                    .to_string(),
            }),
        }
    }

    fn parse_instruction(&self, inst: &str) -> Result<Instruction> {
        let mut chars = inst.chars().rev().collect::<Vec<_>>();
        let mut short_mode = false;
        let mut return_mode = false;
        let mut keep_mode = false;

        // Parse mode suffixes
        while let Some(&ch) = chars.first() {
            match ch {
                '2' => {
                    short_mode = true;
                    chars.remove(0);
                }
                'r' => {
                    return_mode = true;
                    chars.remove(0);
                }
                'k' => {
                    keep_mode = true;
                    chars.remove(0);
                }
                _ => break,
            }
        }

        let opcode = chars.into_iter().rev().collect::<String>();

        if opcode.is_empty() {
            return Err(AssemblerError::SyntaxError {
                path: self.path.clone(),
                line: self.line,
                position: self.position_in_line,
                message: "Invalid instruction".to_string(),
                source_line: self
                    .source
                    .lines()
                    .nth(self.line)
                    .unwrap_or("")
                    .to_string(),
            });
        }

        Ok(Instruction {
            opcode,
            short_mode,
            return_mode,
            keep_mode,
        })
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
                .nth(self.line)
                .unwrap_or("")
                .to_string(),
        })
    }

    fn current_token(&self) -> &Token {
        self.tokens.get(self.position).unwrap_or(&Token::Eof)
    }

    fn advance(&mut self) {
        if self.position < self.tokens.len() {
            if let Token::Newline = self.tokens[self.position] {
                self.line += 1;
                self.position_in_line = 0;
            } else {
                self.position_in_line += 1;
            }
            self.position += 1;
        }
    }

    fn is_at_end(&self) -> bool {
        self.position >= self.tokens.len()
    }
}
