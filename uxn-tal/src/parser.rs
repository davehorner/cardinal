//! Parser for TAL assembly language

use crate::error::{AssemblerError, Result};
use crate::lexer::{Token, TokenWithPos};
use crate::opcodes::Opcodes;
use crate::runes::Rune;

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
    LabelDef(Rune, String),
    /// Label reference
    LabelRef {
        label: String,
        rune: Rune,
        token: TokenWithPos,
    },
    /// Sublabel definition
    SublabelDef(TokenWithPos),
    /// Sublabel reference
    SublabelRef(TokenWithPos),
    /// Relative address reference
    RelativeRef(TokenWithPos),
    /// Conditional jump reference  
    ConditionalRef(TokenWithPos),
    /// Raw address reference
    RawAddressRef(TokenWithPos),
    /// JSR call reference
    JSRRef(TokenWithPos),
    /// Hyphen address reference
    HyphenRef(TokenWithPos),
    /// Padding to specific address
    Padding(u16),
    PaddingLabel(TokenWithPos),
    /// Relative padding by hex count ($1234)
    RelativePadding(u16),
    /// Relative padding to label ($label)
    RelativePaddingLabel(TokenWithPos),
    /// Macro definition
    MacroDef(String, Vec<AstNode>), // name, body
    /// Macro call (name, line, position)
    MacroCall(String, usize, usize),
    /// Raw string data
    RawString(Vec<u8>),
    /// Include directive
    Include(TokenWithPos),
    /// Dot reference - generates LIT + 8-bit address (like uxnasm's '.' rune)
    DotRef(TokenWithPos),
    /// Semicolon reference - generates LIT2 + 16-bit address (like uxnasm's ';' rune)  
    SemicolonRef(TokenWithPos),
    /// Equals reference - generates 16-bit address directly (like uxnasm's '=' rune)
    EqualsRef(TokenWithPos),
    /// Comma reference - generates LIT + relative 8-bit address (like uxnasm's ',' rune)
    CommaRef(TokenWithPos),
    /// Underscore reference - generates relative 8-bit address (like uxnasm's '_' rune)
    UnderscoreRef(TokenWithPos),
    /// Question reference - generates conditional jump (like uxnasm's '?' rune)
    QuestionRef(TokenWithPos),
    /// Exclamation reference - generates JSR call (like uxnasm's '!' rune)
    ExclamationRef(TokenWithPos),
    /// Conditional block start (e.g., ?{) - lambda id for auto label
    ConditionalBlockStart(TokenWithPos),
    /// Conditional block end (e.g., }) - lambda id for auto label
    ConditionalBlockEnd(TokenWithPos),
    /// Lambda block start '{' (standalone, not '?{')
    LambdaStart(TokenWithPos),
    /// Lambda block end '}' corresponding to LambdaStart
    LambdaEnd(TokenWithPos),
    Eof,
    Ignored, // Used for stray '}' after macro body or block
}

/// Parser for TAL assembly
pub struct Parser {
    tokens: Vec<TokenWithPos>,
    position: usize,
    line: usize,
    position_in_line: usize,
    path: String,
    source: String,
    brace_stack: Vec<BraceKind>, // track lambda vs conditional braces
    macro_table: std::collections::HashSet<String>, // <-- Add macro table
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BraceKind {
    Conditional,
    Lambda,
}

impl Parser {
    pub fn new_with_source(tokens: Vec<TokenWithPos>, path: String, source: String) -> Self {
        let line = tokens.first().map(|t| t.line).unwrap_or(1);
        let position_in_line = tokens.first().map(|t| t.start_pos).unwrap_or(1);
        // Build macro table from tokens
        let mut macro_table = std::collections::HashSet::new();
        for t in &tokens {
            if let Token::MacroDef(ref name) = t.token {
                macro_table.insert(name.clone());
            }
        }
        Self {
            tokens,
            position: 0,
            line,
            position_in_line,
            path,
            source,
            brace_stack: Vec::new(),
            macro_table,
        }
    }

    /// Parse tokens into AST nodes
    pub fn parse(&mut self) -> Result<Vec<AstNode>> {
        let mut nodes = Vec::new();

        while !self.is_at_end() {
            let tok_with_pos = self.current_token();
            // Print path:line:start_pos-end_pos for every token
            // println!(
            //     "{}:{}:{}-{} {:?}",
            //     self.path,
            //     tok_with_pos.line,
            //     tok_with_pos.start_pos,
            //     tok_with_pos.end_pos,
            //     tok_with_pos.token
            // );
            let token = &tok_with_pos.token;
            match token {
                Token::Newline => {
                    self.advance();
                    continue;
                }
                Token::Comment(_comment) => {
                    // println!("Comment({:?})", comment);
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
                    // println!("{:?}", node);
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
                Token::Newline | Token::Comment(_) | Token::BracketOpen | Token::BracketClose => {
                    self.advance();
                    continue;
                }
                _ => break,
            }
        }
        let token = self.current_token().token.clone();
        let path = if self.path.is_empty() {
            "(input)".to_string()
        } else {
            self.path.clone()
        };
        let line = self.current_token().line;
        let position = self.current_token().start_pos;
        match token {
            Token::Comment(_) => {
                // Comments are already skipped in the main loop
                self.advance();
                Ok(AstNode::Ignored)
            }
            Token::Word(name) => {
                // If the word is a macro name, treat it as a macro call

                if self.is_macro_defined(&name) {
                    self.advance();
                    return Ok(AstNode::MacroCall(name.clone(), line, position));
                }
                // Otherwise, treat as a label reference (bare word)
                let tok = self.current_token().clone();
                self.advance();
                // Check if label is defined in tokens
                let label_defined = self.tokens.iter().any(|t| match &t.token {
                    Token::LabelDef(_, def_name) => def_name == &name,
                    _ => false,
                });
                if label_defined {
                    Ok(AstNode::LabelRef {
                        label: name.clone(),
                        rune: Rune::from(' '),
                        token: tok,
                    })
                } else {
                    Err(AssemblerError::SyntaxError {
                        path: path.clone(),
                        line,
                        position,
                        message: format!("Label reference '{}' is not defined", name),
                        source_line: self.get_source_line(line),
                    })
                }
            }
            Token::HexLiteral(hex) => {
                let hex = hex.clone();
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
                // Match uxnasm: 1–2 hex digits => byte, 3–4 hex digits => short.
                let value = self.parse_hex_literal(&hex)?;
                let hex_len = hex.len();
                self.advance();
                if hex_len <= 2 {
                    Ok(AstNode::Byte(value as u8))
                } else {
                    // PATCH: always mask to 16 bits for >2 digits
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
                    u16::from_str_radix(&bin, 2).map_err(|_| AssemblerError::SyntaxError {
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
                let value = ch as u8;
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
            Token::LabelDef(rune, label) => {
                // Avoid holding a reference to self across self.advance()
                let label = label.clone();
                self.advance();
                // println!("DEBUG: Parsed label definition: @{}", label);
                Ok(AstNode::LabelDef(rune, label))
            }
            Token::LabelRef(rune, label) => {
                // Clone the values before advancing to avoid borrow checker issues
                let label = label.to_string();
                let tok = self.current_token().clone();
                self.advance();
                // println!(
                //     "LabelRef: label='{}', rune={:?}, token=({}:{}:{})",
                //     label, rune, self.path, tok.line, tok.start_pos
                // );
                Ok(AstNode::LabelRef {
                    label,
                    rune,
                    token: tok,
                })
            }
            Token::SublabelDef(_) => {
                let tok = self.current_token().clone();
                self.advance();
                Ok(AstNode::SublabelDef(tok))
            }
            Token::SublabelRef(_) => {
                let tok = self.current_token().clone();
                self.advance();
                Ok(AstNode::SublabelRef(tok))
            }
            Token::RelativeRef(_) => {
                let tok = self.current_token().clone();
                self.advance();
                Ok(AstNode::RelativeRef(tok))
            }
            Token::ConditionalRef(_) => {
                let tok = self.current_token().clone();
                self.advance();
                Ok(AstNode::ConditionalRef(tok))
            }

            Token::DotRef(_) => {
                let tok = self.current_token().clone();
                self.advance();
                Ok(AstNode::DotRef(tok))
            }
            Token::EqualsRef(label) => {
                let tok = self.current_token().clone();
                if label == "{" {
                    // Anonymous lambda: ={ ... }
                    self.advance();
                    self.brace_stack.push(BraceKind::Lambda);
                    Ok(AstNode::EqualsRef(tok))
                } else {
                    self.advance();
                    Ok(AstNode::EqualsRef(tok))
                }
            }
            Token::SemicolonRef(label) => {
                let tok = self.current_token().clone();
                if label == "{" {
                    // Anonymous lambda: ;{ ... }
                    self.advance();
                    self.brace_stack.push(BraceKind::Lambda);
                    Ok(AstNode::SemicolonRef(tok))
                } else {
                    self.advance();
                    Ok(AstNode::SemicolonRef(tok))
                }
            }
            Token::CommaRef(_) => {
                let tok = self.current_token().clone();
                self.advance();
                Ok(AstNode::CommaRef(tok))
            }
            Token::UnderscoreRef(label) => {
                let tok = self.current_token().clone();
                if label == "{" {
                    // Anonymous lambda: _{ ... }
                    self.advance();
                    self.brace_stack.push(BraceKind::Lambda);
                    Ok(AstNode::UnderscoreRef(tok))
                } else {
                    self.advance();
                    Ok(AstNode::UnderscoreRef(tok))
                }
            }
            Token::QuestionRef(_) => {
                let tok = self.current_token().clone();
                self.advance();
                Ok(AstNode::QuestionRef(tok))
            }
            Token::ExclamationRef(_) => {
                let tok = self.current_token().clone();
                self.advance();
                Ok(AstNode::ExclamationRef(tok))
            }
            // Token::ConditionalOperator => {
            //     self.advance();
            //     // Skip any newlines or comments after '?'
            //     while matches!(
            //         &self.current_token().token,
            //         Token::Newline | Token::Comment(_)
            //     ) {
            //         self.advance();
            //     }
            //     &self.current_token().token
            //     // let next_token = &self.current_token().token;
            //     // match next_token {
            //     //     _ => {
            //     //         let line = self.current_token().line;
            //     //         let position = self.current_token().start_pos;
            //     //         Err(AssemblerError::SyntaxError {
            //     //             path: self.path.clone(),
            //     //             line,
            //     //             position,
            //     //             message: "Conditional operator '?' must be followed by a block"
            //     //                 .to_string(),
            //     //             source_line: self.get_source_line(line),
            //     //         })
            //     //     }
            //     // }
            // }
            Token::RawAddressRef(_) => {
                let tok = self.current_token().clone();
                self.advance();
                Ok(AstNode::RawAddressRef(tok))
            }
            Token::JSRRef(_) => {
                let tok = self.current_token().clone();
                self.advance();
                Ok(AstNode::JSRRef(tok))
            }
            Token::HyphenRef(_) => {
                let tok = self.current_token().clone();
                self.advance();
                Ok(AstNode::HyphenRef(tok))
            }
            Token::Padding(addr) => {
                self.advance();
                Ok(AstNode::Padding(addr))
            }
            Token::PaddingLabel(_) => {
                let tok = self.current_token().clone();
                // Check for empty or invalid label
                if let Token::PaddingLabel(ref label) = tok.token {
                    if label.trim().is_empty() {
                        let line = tok.line;
                        let position = tok.start_pos;
                        return Err(AssemblerError::SyntaxError {
                            path: self.path.clone(),
                            line,
                            position,
                            message: "Expected identifier after '|' for padding label".to_string(),
                            source_line: self.get_source_line(line),
                        });
                    }
                }
                self.advance();
                Ok(AstNode::PaddingLabel(tok))
            }
            Token::RelativePadding(count) => {
                let c = count;
                self.advance();
                Ok(AstNode::RelativePadding(c))
            }
            Token::RelativePaddingLabel(_) => {
                let tok = self.current_token().clone();
                self.advance();
                Ok(AstNode::RelativePaddingLabel(tok))
            }
            Token::MacroDef(name) => {
                // Accept macro names like '=' for ={ ... } blocks
                let name = name.clone();
                // println!(
                //     "MacroDef: {} {}:{}:{}:{} {:?}",
                //     name,
                //     self.path,
                //     self.current_token().line,
                //     self.current_token().start_pos,
                //     self.current_token().end_pos,
                //     self.current_token().token
                // );

                self.advance();

                while matches!(
                    &self.current_token().token,
                    Token::Comment(_) | Token::Newline
                ) {
                    self.advance();
                }
                // println!(
                //     "MacroDef: {} {}:{}:{}:{} {:?}",
                //     name,
                //     self.path,
                //     self.current_token().line,
                //     self.current_token().start_pos,
                //     self.current_token().end_pos,
                //     self.current_token().token
                // );

                let mut body = Vec::new();
                let mut depth = 1;
                match &self.current_token().token {
                    Token::BraceOpen | Token::ConditionalBlockStart => {
                        // Accept either '{' or '?{' as block openers
                        self.advance();
                        while matches!(
                            &self.current_token().token,
                            Token::Comment(_) | Token::Newline
                        ) {
                            self.advance();
                        }
                        // println!(
                        //     "DEBUG: Macro body starts at token: {:?}",
                        //     self.current_token()
                        // );
                        while !self.is_at_end() && depth > 0 {
                            match &self.current_token().token {
                                Token::Comment(_)
                                | Token::Newline
                                | Token::BracketClose
                                | Token::BracketOpen => {
                                    self.advance();
                                }
                                Token::BraceOpen | Token::ConditionalBlockStart => {
                                    depth += 1;
                                    // println!(
                                    //     "BraceOpen Inside macro '{}', depth = {}, token = {:?}",
                                    //     name,
                                    //     depth,
                                    //     self.current_token()
                                    // );
                                    body.push(self.parse_node()?);
                                    // println!(
                                    //     "BraceOpen Inside macro '{}', depth = {}, token = {:?}",
                                    //     name,
                                    //     depth,
                                    //     self.current_token()
                                    // );
                                }
                                Token::BraceClose => {
                                    depth -= 1;
                                    if depth == 0 {
                                        // println!("BraceClose Inside macro '{}', depth = {}, token = {:?}", name, depth, self.current_token());
                                        self.advance();
                                        break;
                                    } else {
                                        // println!("BraceClose Inside macro '{}', depth = {}, token = {:?}", name, depth, self.current_token());
                                        let n = self.parse_node()?;
                                        // println!("BraceClose Inside macro '{}', depth = {}, token = {:?}", name, depth, self.current_token());
                                        body.push(n);
                                    }
                                }
                                Token::Eof => {
                                    // println!(
                                    //     "Eof Inside macro '{}', depth = {}, token = {:?}",
                                    //     name,
                                    //     depth,
                                    //     self.current_token()
                                    // );
                                    break;
                                }
                                _ => {
                                    // if matches!(self.current_token().token, Token::Eof) {
                                    //     break;
                                    // }
                                    // println!("Unexpected token inside macro '{}', depth = {}, token = {:?}", name, depth, self.current_token());
                                    // Only parse and push if not a macro name
                                    if let Token::Word(ref w) = self.current_token().token {
                                        if self.is_macro_defined(w) {
                                            // Don't expand macro calls inside macro definitions
                                            self.advance();
                                            continue;
                                        }
                                    }
                                    body.push(self.parse_node()?);
                                }
                            }
                        }
                        if depth != 0 && self.current_token().token != Token::Eof {
                            let line = self.current_token().line;
                            let position = self.current_token().start_pos;
                            return Err(AssemblerError::SyntaxError {
                                path: self.path.clone(),
                                line,
                                position,
                                message: format!("Expected '}}' after macro body for macro '{}', depth={} token={:?}", name, depth, self.current_token()),
                                source_line: self.get_source_line(line),
                            });
                        }
                        Ok(AstNode::MacroDef(name, body))
                    }
                    Token::Eof => Ok(AstNode::Ignored),
                    _ => {
                        // println!(
                        //     "DEBUG: Unexpected token after macro name: {:?}",
                        //     self.current_token().token
                        // );
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
            Token::Include(_) => {
                println!(
                    "DEBUG: Include directive found: {:?}",
                    self.current_token().token
                );
                let tok = self.current_token().clone();
                self.advance();
                Ok(AstNode::Include(tok))
            }
            Token::ConditionalBlockStart => {
                let tok = self.current_token().clone();
                self.advance();
                self.brace_stack.push(BraceKind::Conditional);
                Ok(AstNode::ConditionalBlockStart(tok))
            }
            Token::BraceOpen => {
                let tok = self.current_token().clone();
                self.advance();
                self.brace_stack.push(BraceKind::Lambda);
                Ok(AstNode::LambdaStart(tok))
            }
            Token::BraceClose => {
                let tok = self.current_token().clone();
                self.advance();

                // if self.brace_stack.is_empty() {
                //     // Ignore stray '}' after macro body or after macro block just closed
                //     return Ok(AstNode::Ignored);
                // }
                // if self.brace_stack.is_empty() {

                //         // Ignore stray '}' after macro body or after macro block just closed
                //         return Ok(AstNode::Ignored);

                //     eprintln!("Unmatched '}}' at line {}. Current macro table:", tok.line);
                //     for name in &self.macro_table {
                //         eprintln!("Macro '{}'", name);
                //     }
                //     eprintln!("Brace stack: {:?}", self.brace_stack);
                //     return Err(AssemblerError::SyntaxError {
                //         path: self.path.clone(),
                //         line: tok.line,
                //         position: tok.start_pos,
                //         message: "Unmatched '}' (no open '{' or '?{')".to_string(),
                //         source_line: self.get_source_line(tok.line),
                //     });
                // }
                Ok(match self.brace_stack.pop().unwrap_or(BraceKind::Lambda) {
                    BraceKind::Conditional => AstNode::ConditionalBlockEnd(tok),
                    BraceKind::Lambda => AstNode::LambdaEnd(tok),
                })
            }
            Token::Eof => Ok(AstNode::Eof),
            _ => {
                let line = self.current_token().line;
                let position = self.current_token().start_pos;
                Err(AssemblerError::SyntaxError {
                    path: path.clone(),
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
        let mut opcode;
        let mut short_mode = false;
        let mut return_mode = false;
        let mut keep_mode = false;

        // Debug output for tracing
        // eprintln!("DEBUG: parse_instruction input name: '{}'", name);

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
        // eprintln!("DEBUG: base after stripping flags: '{}', mode_chars: '{}'", base, mode_chars);

        if opcodes.get_opcode(base).is_ok() {
            opcode = base.to_string();
            for c in mode_chars.chars() {
                match c {
                    'k' => {
                        keep_mode = true;
                        // eprintln!("DEBUG: found 'k' flag, keep_mode = true");
                    }
                    'r' => {
                        return_mode = true;
                        // eprintln!("DEBUG: found 'r' flag, return_mode = true");
                    }
                    '2' => {
                        short_mode = true;
                        // eprintln!("DEBUG: found '2' flag, short_mode = true");
                    }
                    _ => {}
                }
            }
        } else {
            opcode = name.clone();
            // eprintln!("DEBUG: base '{}' not found in opcode table, using original name '{}'", base, name);
        }

        // For LIT/LIT2/LITr/LIT2r, always use base "LIT" and set flags accordingly
        if opcode == "LIT" || name.starts_with("LIT") {
            // eprintln!("DEBUG: opcode is 'LIT' or starts with 'LIT', checking for '2' and 'r' in name '{}'", name);
            if name.contains('2') {
                short_mode = true;
                // eprintln!("DEBUG: name contains '2', short_mode = true");
            }
            if name.contains('r') {
                return_mode = true;
                // eprintln!("DEBUG: name contains 'r', return_mode = true");
            }
            keep_mode = true;
            // eprintln!("DEBUG: LIT always sets keep_mode = true");
            opcode = "LIT".to_string();
        }

        // eprintln!(
        // "DEBUG: parse_instruction result: opcode='{}', short_mode={}, return_mode={}, keep_mode={}",
        // opcode, short_mode, return_mode, keep_mode
        // );

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

    fn current_token(&self) -> TokenWithPos {
        self.tokens
            .get(self.position)
            .cloned()
            .unwrap_or(TokenWithPos {
                token: Token::Eof,
                line: self.line,
                start_pos: self.position_in_line,
                end_pos: self.position_in_line,
                scope: None, // <-- Add default scope
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

    /// Returns true if the macro name is defined in the macro table
    fn is_macro_defined(&self, name: &str) -> bool {
        self.macro_table.contains(name)
    }
}
