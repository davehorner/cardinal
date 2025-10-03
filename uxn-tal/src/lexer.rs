//! Lexer for TAL assembly language

use crate::{
    error::{AssemblerError, Result},
    runes::Rune,
};

/// Token types in TAL assembly
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Word(String), // could be a macro name before syntax error.
    /// Hexadecimal literal (e.g., #1234, #ab)
    HexLiteral(String),
    /// Raw hexadecimal byte (e.g., 20, ff)
    RawHex(String),
    /// Decimal literal (e.g., 42)
    DecLiteral(String),
    /// Binary literal (e.g., #b10101010)
    BinLiteral(String),
    /// Character literal (e.g., 'A')
    CharLiteral(char),
    /// Instruction/opcode (e.g., ADD, LDA2k)
    Instruction(String),
    /// Label definition (e.g., @main)
    LabelDef(Rune, String),
    /// Label reference (e.g., ;main)
    LabelRef(Rune, String),
    /// Sublabel definition (e.g., &loop)
    SublabelDef(String),
    /// Sublabel reference (e.g., ,loop)
    SublabelRef(String),
    /// Relative address reference (e.g., /loop)
    RelativeRef(String),
    /// Conditional jump reference (e.g., ?loop)
    ConditionalRef(String),
    /// Conditional operator (e.g., ?)
    ConditionalOperator,
    /// Conditional block start (e.g., ?{)
    ConditionalBlockStart,
    /// Raw address reference (e.g., =label)
    RawAddressRef(String),
    /// JSR call reference (e.g., !label)
    JSRRef(String),
    /// Hyphen address reference (e.g., -Screen/auto)
    HyphenRef(String),
    /// Padding directive (absolute) |ADDR
    Padding(u16),
    PaddingLabel(String),
    /// Relative padding (was Skip): $HEX means advance current address by HEX bytes
    RelativePadding(u16),
    /// Relative padding to label: $label sets ptr = current + label.addr
    RelativePaddingLabel(String),
    /// Device access (e.g., .Screen/width)
    DeviceAccess(String, String), // device, field

    /// Macro definition (e.g., MACRO)
    MacroDef(String),

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
    /// Brace open
    BraceOpen,
    /// Brace close
    BraceClose,
    /// Bracket open
    BracketOpen,
    /// Bracket close
    BracketClose,
    /// Include directive (e.g., ~filename.tal)
    Include(String),
    /// Raw string literal
    RawString(String),
    /// Comment
    Comment(String),
    /// Newline
    Newline,
    /// End of file
    Eof,
    Ignored,
}

/// Token with position information and optional scope
#[derive(Debug, Clone, PartialEq)]
pub struct TokenWithPos {
    pub token: Token,
    pub line: usize,
    pub start_pos: usize,
    pub end_pos: usize,
    pub scope: Option<String>, // <-- Add scope field
}

/// Lexer for TAL assembly language
pub struct Lexer {
    input: String,
    position: usize,
    line: usize,
    path: Option<String>,
    position_on_line: usize,
    next_conditional_sublabel: Option<String>,
    current_scope: Option<String>, // <-- Track current scope
}

impl Lexer {
    // get_column removed; use position_on_line directly
    pub fn new(input: String, path: Option<String>) -> Self {
        Self {
            input,
            position: 0,
            line: 1,
            path,
            position_on_line: 1,
            next_conditional_sublabel: None,
            current_scope: None, // <-- Initialize
        }
    }

    /// Get the current line content for error reporting
    fn get_current_line(&self) -> String {
        let lines: Vec<&str> = self.input.lines().collect();
        if self.line > 0 && self.line <= lines.len() {
            lines[self.line - 1].to_string()
        } else {
            String::new()
        }
    }

    /// Create a syntax error with current line information
    fn syntax_error(&self, message: String) -> AssemblerError {
        AssemblerError::SyntaxError {
            path: self.path.clone().unwrap_or_default(),
            line: self.line,
            position: self.position_on_line,
            message,
            source_line: self.get_current_line(),
        }
    }

    pub fn line(&self) -> usize {
        self.line
    }

    /// Tokenize the entire input
    pub fn tokenize(&mut self) -> Result<Vec<TokenWithPos>> {
        let mut tokens = Vec::new();

        while self.position < self.input.len() {
            let start_line = self.line;
            let start_pos = self.position_on_line;
            let token_start_position = self.position;

            // Handle newlines and whitespace
            let ch = self.current_char();
            if ch == '\n' {
                tokens.push(TokenWithPos {
                    token: Token::Newline,
                    line: start_line,
                    start_pos,
                    end_pos: start_pos,
                    scope: self.current_scope.clone(),
                });
                self.advance();
                continue;
            }
            if ch.is_whitespace() && ch != '\n' {
                self.advance();
                continue;
            }

            // Handle comments
            if ch == '(' {
                let comment_start_line = self.line;
                let comment_start_pos = self.position_on_line;
                self.advance();
                let comment = self.read_comment()?;
                let comment_end_pos = self.position_on_line;
                tokens.push(TokenWithPos {
                    token: Token::Comment(comment),
                    line: comment_start_line,
                    start_pos: comment_start_pos,
                    end_pos: comment_end_pos,
                    scope: self.current_scope.clone(),
                });
                continue;
            }

            // Get next token
            match self.next_token()? {
                Token::Eof => break,
                token => {
                    let token_end_position = self.position;
                    let token_end_pos =
                        start_pos + (token_end_position - token_start_position).max(1) - 1;

                    // --- SCOPE TRACKING ---
                    // Update current_scope for label/sublabel definitions
                    match &token {
                        Token::LabelDef(_rune, label) => {
                            self.current_scope = Some(label.clone());
                        }
                        // Token::SublabelDef(sublabel) => {
                        //     if let Some(ref parent) = self.current_scope {
                        //         self.current_scope = Some(format!("{}/{}", parent, sublabel));
                        //     } else {
                        //         self.current_scope = Some(sublabel.clone());
                        //     }
                        // }
                        _ => {}
                    }

                    // For sublabel definition, set scope to parent label (not full sublabel path)
                    let token_scope = match &token {
                        Token::RelativeRef(_) => {
                            // For /down, use parent label scope (not full sublabel scope)
                            if let Some(ref scope) = self.current_scope {
                                if let Some(pos) = scope.find('/') {
                                    Some(scope[..pos].to_string())
                                } else {
                                    Some(scope.clone())
                                }
                            } else {
                                None
                            }
                        }
                        Token::SublabelDef(_)
                        | Token::SublabelRef(_)
                        | Token::CommaRef(_)
                        | Token::UnderscoreRef(_) => {
                            // Use parent label as scope
                            if let Some(ref scope) = self.current_scope {
                                if let Some(pos) = scope.rfind('/') {
                                    Some(scope[..pos].to_string())
                                } else {
                                    Some(scope.clone())
                                }
                            } else {
                                None
                            }
                        }
                        _ => self.current_scope.clone(),
                    };

                    // Debug print for position tracking
                    // println!(
                    //     "DEBUG: token={:?} line={} start_pos={} end_pos={} position={} position_on_line={}",
                    //     token, start_line, start_pos, token_end_pos, self.position, self.position_on_line
                    // );
                    tokens.push(TokenWithPos {
                        token,
                        line: start_line,
                        start_pos,
                        end_pos: token_end_pos,
                        scope: token_scope,
                    });
                }
            }
        }

        Ok(tokens)
    }

    /// Move to the next character
    fn advance(&mut self) {
        if self.position < self.input.len() {
            if self.input.chars().nth(self.position) == Some('\n') {
                self.line += 1;
                self.position_on_line = 1;
            } else {
                self.position_on_line += 1;
            }
            self.position += 1;
        }
    }

    /// Skip whitespace characters except newlines
    fn skip_whitespace(&mut self) {
        while self.position < self.input.len() {
            let ch = self.current_char();
            if ch.is_whitespace() && ch != '\n' {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// Read characters until the specified delimiter is found
    #[allow(dead_code)]
    fn read_until(&mut self, delimiter: char) -> Result<String> {
        let mut result = String::new();
        while self.position < self.input.len() && self.current_char() != delimiter {
            result.push(self.current_char());
            self.advance();
        }
        Ok(result)
    }

    /// Read comment with proper nested parentheses handling
    fn read_comment(&mut self) -> Result<String> {
        // Match uxnasm's walkcomment: only consider '(' and ')' that begin a token
        // (i.e., occur right after whitespace) for nesting and termination.
        let mut result = String::new();
        let mut depth = 1;
        let mut last: char = '\0'; // last token-head character or 0 if in whitespace

        while self.position < self.input.len() && depth > 0 {
            let ch = self.current_char();

            if ch.is_ascii() && ch <= ' ' {
                // Whitespace: process the last token-head seen, then reset
                result.push(ch);
                self.advance();
                if last == '(' {
                    depth += 1;
                } else if last == ')' {
                    depth -= 1;
                    if depth < 1 {
                        break; // end of comment
                    }
                }
                last = '\0';
            } else if last <= ' ' {
                // Start of a new token: remember its first character
                last = ch;
                result.push(ch);
                self.advance();
            } else {
                // Inside a token: ignore nested parens here
                last = '~';
                result.push(ch);
                self.advance();
            }
        }

        Ok(result)
    }

    /// Read hexadecimal digits for #hex literals
    fn read_hex(&mut self) -> Result<String> {
        let mut result = String::new();
        while self.position < self.input.len() {
            let ch = self.current_char();
            if ch.is_ascii_hexdigit() && (ch.is_ascii_lowercase() || ch.is_ascii_digit()){
                result.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        if result.is_empty() {
            return Err(AssemblerError::SyntaxError {
                path: self.path.clone().unwrap_or_default(),
                line: self.line,
                position: self.position_on_line,
                message: "Expected hexadecimal digits".to_string(),
                source_line: self.get_current_line(),
            });
        }
        Ok(result)
    }

    /// Read binary digits for #b binary literals
    fn read_binary(&mut self) -> Result<String> {
        let mut result = String::new();
        while self.position < self.input.len() {
            let ch = self.current_char();
            if ch == '0' || ch == '1' {
                result.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        if result.is_empty() {
            return Err(AssemblerError::SyntaxError {
                path: self.path.clone().unwrap_or_default(),
                line: self.line,
                position: self.position_on_line,
                message: "Expected binary digits".to_string(),
                source_line: self.get_current_line(),
            });
        }
        Ok(result)
    }

    /// Read a decimal number (digits only)
    #[allow(dead_code)]
    fn read_number(&mut self) -> Result<String> {
        let mut result = String::new();
        while self.position < self.input.len() {
            let ch = self.current_char();
            if ch.is_ascii_digit() {
                result.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        Ok(result)
    }

    /// Read a hexadecimal number (hex digits only)
    /// For asset data, reads up to 4 hex digits for RawHex tokens
    fn read_hex_number(&mut self) -> Result<String> {
        let mut result = String::new();
        let mut count = 0;
        while self.position < self.input.len() && count < 4 {
            let ch = self.current_char();
            if ch.is_ascii_hexdigit() {
                result.push(ch);
                self.advance();
                count += 1;
            } else {
                break;
            }
        }
        // Fix position_on_line drift: always update position_on_line based on chars consumed
        // This is already handled by advance(), so do NOT manually update position_on_line here.
        Ok(result)
    }

    /// Read an identifier (labels, instructions, etc.)
    /// Allows alphanumeric characters, underscores, hyphens, forward slashes, asterisks, etc.
    fn read_identifier(&mut self) -> Result<String> {
        let mut result = String::new();
        let mut first = true;
        // Special handling: if the previous character was '%', read until whitespace or '{' or '}'
        // This ensures macro names like '\;#' are read as a single name, and allows macros with multiple blocks.
        let mut macro_mode = false;
        if self.position > 0 && self.input.chars().nth(self.position - 1) == Some('%') {
            macro_mode = true;
        }
        while self.position < self.input.len() {
            let ch = self.current_char();
            // Skip tab characters entirely (do not include in identifiers)
            // while ch == '\t' {
            //     self.advance();
            //     ch = self.current_char();
            // }
            if macro_mode {
                // Accept any non-whitespace, non-'{' and non-'}' character as part of macro name
                if ch.is_whitespace() {
                    break;
                }
            } else {
                // Accept any non-whitespace, non-parenthesis character as part of identifier
                if ch.is_whitespace() || ch == '(' || ch == ')' {
                    break;
                }
            }
            // Special case: treat "" as a CharLiteral('"')
            if first && ch == '"' && self.input.chars().nth(self.position + 1) == Some('"') {
                self.advance();
                self.advance();
                return Ok(String::from("\""));
            }
            result.push(ch);
            self.advance();
            first = false;
        }
        // If the identifier is just "_", skip it and try to read the next identifier
        // if result == "_" {
        //     self.advance();
        //     self.advance();
        //     return self.read_identifier();
        // }
        // Debug: print what we read to see if it's being truncated
        // eprintln!("DEBUG: read_identifier result: '{}'", result);
        // If we returned "" above, result will be empty, so skip error
        if result.is_empty() {
            // If we just parsed "", treat as CharLiteral('"')
            if self.position > 1
                && self.input.chars().nth(self.position - 2) == Some('"')
                && self.input.chars().nth(self.position - 1) == Some('"')
            {
                return Ok(String::from("\""));
            }
            if self.position >= self.input.len() - 1 && self.current_char() == '\n' {
                return Ok(String::new());
            }
            return Err(AssemblerError::SyntaxError {
                path: self.path.clone().unwrap_or_default(),
                line: self.line,
                position: self.position_on_line,
                message: format!("Expected identifier @ position {}", self.position_on_line),
                source_line: self.get_current_line(),
            });
        }
        Ok(result)
    }

    /// Read a string literal, handling both quoted and unquoted TAL strings
    /// Quoted strings end at the closing quote, unquoted strings end at whitespace
    fn read_string(&mut self) -> Result<String> {
        let mut result = String::new();

        while self.position < self.input.len() {
            let ch = self.current_char();
            if ch == '"' {
                self.advance(); // Skip closing quote
                break;
            } else if ch.is_whitespace() {
                // In TAL, unclosed strings end at whitespace
                break;
            } else {
                result.push(ch);
                self.advance();
            }
        }

        Ok(result)
    }

    /// Read include path after ~ token
    fn read_include_path(&mut self) -> Result<String> {
        let mut path = String::new();

        // Skip any whitespace after ~
        self.skip_whitespace();

        while self.position < self.input.len() {
            let ch = self.current_char();

            // Include path ends at whitespace or newline
            if ch.is_whitespace() {
                break;
            }

            path.push(ch);
            self.advance();
        }

        if path.is_empty() {
            return Err(AssemblerError::SyntaxError {
                path: self.path.clone().unwrap_or_default(),
                line: self.line,
                position: self.position_on_line,
                message: "Empty include path".to_string(),
                source_line: self.get_current_line(),
            });
        }

        Ok(path)
    }

    /// Read macro call name between < and >
    fn read_macro_call(&mut self) -> Result<String> {
        let mut name = String::new();

        while self.position < self.input.len() {
            let ch = self.current_char();

            if ch == '>' {
                self.advance(); // consume closing >
                break;
            }

            // Allow alphanumeric, hyphens, underscores, slashes, and '?' in macro names
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == '/' || ch == '?' {
                name.push(ch);
                self.advance();
            } else {
                return Err(AssemblerError::SyntaxError {
                    path: self.path.clone().unwrap_or_default(),
                    line: self.line,
                    position: self.position_on_line,
                    message: format!("Invalid character in macro call: {}", ch),
                    source_line: self.get_current_line(),
                });
            }
        }

        if name.is_empty() {
            return Err(AssemblerError::SyntaxError {
                path: self.path.clone().unwrap_or_default(),
                line: self.line,
                position: self.position_on_line,
                message: "Empty macro call".to_string(),
                source_line: self.get_current_line(),
            });
        }

        Ok(name)
    }

    /// Get the current character at the position
    fn current_char(&self) -> char {
        self.input.chars().nth(self.position).unwrap_or('\0')
    }

    /// Get the next token from the input
    pub fn next_token(&mut self) -> Result<Token> {
        // Check for pending sublabel/label from conditional operator
        if let Some(label) = self.next_conditional_sublabel.take() {
            if label.starts_with('&') {
                return Ok(Token::SublabelRef(
                    label.trim_start_matches('&').to_string(),
                ));
            }
            if label.starts_with('<') {
                let rune = Rune::from('<');
                return Ok(Token::LabelRef(
                    rune,
                    label.trim_start_matches('<').to_string(),
                ));
            }
            // Use Rune based on the first character of the label
            // let rune = Rune::from(label.chars().next().unwrap_or('\0'));
            return Ok(Token::LabelRef(
                Rune::from(label.chars().next().unwrap_or('\0')),
                label,
            ));
        }

        // Robustly skip all whitespace except newlines before tokenizing
        while self.position < self.input.len() {
            let ch = self.current_char();
            if ch.is_whitespace() && ch != '\n' {
                self.advance();
            } else {
                break;
            }
        }

        // Skip comments before tokenizing
        while self.position < self.input.len() {
            let ch = self.current_char();
            if ch.is_whitespace() && ch != '\n' {
                self.advance();
            } else if ch == '(' {
                self.advance();
                let mut depth = 1;
                while self.position < self.input.len() && depth > 0 {
                    let ch = self.current_char();
                    if ch == '(' {
                        depth += 1;
                    } else if ch == ')' {
                        depth -= 1;
                    }
                    self.advance();
                }
                continue;
            } else {
                break;
            }
        }

        // if self.position >= self.input.len() {
        //     return Ok(Token::Eof);
        // }

        let ch = self.current_char();

        match ch {
            '\n' => {
                self.advance();
                Ok(Token::Newline)
            }
            '(' => {
                self.advance();
                let comment = self.read_comment()?;
                Ok(Token::Comment(comment))
            }
            '"' => {
                self.advance();
                if self.current_char() == '"' {
                    self.advance();
                    return Ok(Token::RawString(String::new()));
                }
                let ch = self.current_char();
                if ch != '\0' && !ch.is_whitespace() {
                    let next_pos = self.position + 1;
                    let next_ch = if next_pos < self.input.len() {
                        self.input.chars().nth(next_pos).unwrap_or('\0')
                    } else {
                        '\0'
                    };
                    if next_ch.is_whitespace()
                        || next_ch == '\0'
                        || next_ch == ']'
                        || next_ch == ')'
                    {
                        self.advance();
                        Ok(Token::CharLiteral(ch))
                    } else {
                        let string = self.read_string()?;
                        Ok(Token::RawString(string))
                    }
                } else {
                    let string = self.read_string()?;
                    Ok(Token::RawString(string))
                }
            }
            '\'' => {
                self.advance();
                let ch = self.current_char();
                if ch == '\0' {
                    return Err(AssemblerError::SyntaxError {
                        path: self.path.clone().unwrap_or_default(),
                        line: self.line,
                        position: self.position_on_line,
                        message: "Unexpected end of file in character literal".to_string(),
                        source_line: self.get_current_line(),
                    });
                }
                self.advance();
                Ok(Token::CharLiteral(ch))
            }
            '#' => {
                self.advance();
                loop {
                    let ch = self.current_char();
                    if ch.is_whitespace() && ch != '\n' {
                        self.advance();
                    } else if ch == '(' {
                        self.advance();
                        let comment = self.read_comment()?;
                        let comment_newlines = comment.chars().filter(|c| *c == '\n').count();
                        self.line += comment_newlines;
                    } else {
                        break;
                    }
                }
                if self.current_char() == '"' {
                    self.advance();
                    let ch = self.current_char();
                    if ch != '\0' {
                        self.advance();
                        return Ok(Token::CharLiteral(ch));
                    }
                }
                if self.current_char() == '\'' {
                    self.advance();
                    let ch = self.current_char();
                    if ch != '\0' {
                        self.advance();
                        return Ok(Token::CharLiteral(ch));
                    }
                }
                if self.current_char() == '[' {
                    self.advance();
                    let ch = self.current_char();
                    if ch != '\0' && self.input.chars().nth(self.position + 1) == Some(']') {
                        self.advance();
                        self.advance();
                        return Ok(Token::CharLiteral(ch));
                    }
                }
                if self.current_char().is_ascii_alphabetic()
                    && self
                        .input
                        .chars()
                        .nth(self.position + 1)
                        .map(|c| c.is_whitespace() || c == ']')
                        == Some(true)
                {
                    let ch = self.current_char();
                    self.advance();
                    return Ok(Token::CharLiteral(ch));
                }
                if !self.current_char().is_ascii_hexdigit() {
                    return Err(AssemblerError::SyntaxError {
                        path: self.path.clone().unwrap_or_default(),
                        line: self.line,
                        position: self.position_on_line,
                        message: "Expected hexadecimal digits after '#'".to_string(),
                        source_line: self.get_current_line(),
                    });
                }
                let hex = self.read_hex()?;
                Ok(Token::HexLiteral(hex))
            }
            '@' => {
                self.advance();
                let label = self.read_identifier()?;
                println!("LEXER DEBUG: Parsed label definition: @{}", label);
                Ok(Token::LabelDef('@'.into(), label))
            }
            ';' => {
                self.advance();
                let label = self.read_identifier()?;
                Ok(Token::SemicolonRef(label))
            }
            '.' => {
                self.advance();
                let ident = self.read_identifier()?;
                Ok(Token::DotRef(ident))
            }
            '=' => {
                self.advance();
                let label = self.read_identifier()?;
                Ok(Token::EqualsRef(label))
            }
            ',' => {
                self.advance();
                if self.current_char() == '&' {
                    self.advance();
                }
                let label = self.read_identifier()?;
                Ok(Token::CommaRef(label))
            }
            '_' => {
                self.advance();
                if self.current_char() == '&' {
                    self.advance();
                }
                let label = self.read_identifier()?;
                // println!("DEBUG: Read underscore label: '{}'", label);
                // if label == "_" {
                //     // If the label is just "_", skip and try to read the next identifier
                //     let label = self.read_identifier()?;
                //     // println!("DEBUG: Read underscore label (after skip): '{}'", label);
                //     return Ok(Token::UnderscoreRef(label));
                // }
                Ok(Token::UnderscoreRef(label))
            }
            '-' => {
                // // Lookahead: treat 8+ consecutive '-' followed by whitespace/EOF as a separator comment.
                // let mut look = self.position;
                // let mut count = 0;
                // while look < self.input.len()
                //     && self.input.chars().nth(look) == Some('-')
                // {
                //     count += 1;
                //     look += 1;
                // }
                // let next_ch = self.input.chars().nth(look).unwrap_or('\0');
                // if count >= 8 && (next_ch == '\0' || next_ch.is_whitespace()) {
                //     // Consume all the dashes.
                //     for _ in 0..count {
                //         self.advance();
                //     }
                //     return Ok(Token::Comment("-".repeat(count)));
                // }
                // Normal hyphen reference
                self.advance();
                let identifier = self.read_identifier()?;
                Ok(Token::HyphenRef(identifier))
            }
            '/' => {
                self.advance();
                let label = self.read_identifier()?;
                // Don't include the leading slash in the label name - it's just syntax
                Ok(Token::RelativeRef(label))
            }
            '?' => {
                self.advance();
                if self.current_char() == '{' {
                    self.advance(); // consume '{'
                    Ok(Token::ConditionalBlockStart)
                } else {
                    let name = self.read_identifier()?;
                    Ok(Token::ConditionalRef(name))
                }
            }
            '!' => {
                self.advance();
                let label = self.read_identifier()?;
                Ok(Token::ExclamationRef(label))
            }
            '{' => {
                self.advance();
                Ok(Token::BraceOpen)
            }
            '}' => {
                self.advance();
                Ok(Token::BraceClose)
            }
            '[' => {
                self.advance();
                if self.current_char() == '"'
                    && self.input.chars().nth(self.position + 1) == Some('"')
                {
                    self.advance();
                    self.advance();
                    return Ok(Token::CharLiteral('"'));
                }
                Ok(Token::BracketOpen)
            }
            ']' => {
                self.advance();
                Ok(Token::BracketClose)
            }
            '~' => {
                self.advance();
                let filename = self.read_include_path()?;
                println!("LEXER DEBUG: Include path: {:?}", self.path);
                println!("LEXER DEBUG: Parsed include filename: ~{}", filename);
                Ok(Token::Include(filename))
            }
            '<' => {
                self.advance();
                let macro_name = self.read_macro_call()?;
                // NEW: allow immediate "/sublabel" after closing '>' (e.g., <phex>/b)
                let mut full = format!("<{}>", macro_name);
                if self.current_char() == '/' {
                    self.advance();
                    // read sublabel segment (stop at whitespace or rune delimiters)
                    let mut sub = String::new();
                    while self.position < self.input.len() {
                        let ch = self.current_char();
                        if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
                            sub.push(ch);
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    if !sub.is_empty() {
                        full.push('/');
                        full.push_str(&sub);
                    }
                }
                let rune = Rune::from(' ');
                return Ok(Token::LabelRef(rune, full));
                // Ok(Token::LabelRef(full)) // Always treat <name> (and optional /sub) as LabelRef
            }
            _ if ch.is_ascii_digit() => {
                let number = self.read_hex_number()?;
                // PATCH: treat 4 hex digits as a short, 2 as a byte
                if number.len() == 4 && number.chars().all(|c| c.is_ascii_hexdigit()) {
                    // This will be handled as AstNode::Short in parser
                    Ok(Token::RawHex(number))
                } else if number.len() == 2 && number.chars().all(|c| c.is_ascii_hexdigit()) {
                    Ok(Token::RawHex(number))
                } else {
                    Ok(Token::LabelRef(Rune::from(' '), number)) // Use Rune::from(' ') for numeric labels
                }
            }
            _ if ch.is_ascii_alphabetic() || ch == '_' => {
                let identifier = self.read_identifier()?;
                // Only treat as hex if it's a simple pattern like "ff", "1234", etc.
                // Don't treat words with hyphens or complex patterns as hex
                // Also don't treat known instruction names as hex
                // Don't treat instruction names as hex
                if identifier.len() >= 2
                    && identifier.len() <= 4
                    && identifier.len() % 2 == 0
                    && identifier.chars().all(|c| c.is_ascii_hexdigit() && (c.is_ascii_lowercase() || c.is_ascii_digit()))
                    && !identifier.contains('-')  // No hyphens in hex values
                    && !identifier.contains('/')  // No slashes in hex values
                    && !identifier.contains('_')  // No underscores in hex values
                    && !is_instruction_name(&identifier)
                {
                    Ok(Token::RawHex(identifier))
                } else if is_instruction_name(&identifier) {
                    Ok(Token::Instruction(identifier))
                } else {
                    // If not a known instruction, treat as a label reference (bare word)
                    Ok(Token::LabelRef(Rune::from(' '), identifier))
                }
            }
            '|' => {
                self.advance();
                let padding_value = self.read_identifier()?;

                // Check if it's a hex value or a label
                if padding_value.chars().all(|c| c.is_ascii_hexdigit()) {
                    // Parse as hex address
                    let addr = u16::from_str_radix(&padding_value, 16)
                        .map_err(|_| self.syntax_error("Invalid padding address".to_string()))?;
                    Ok(Token::Padding(addr))
                } else {
                    // It's a label reference for padding
                    Ok(Token::PaddingLabel(padding_value))
                }
            }
            '$' => {
                self.advance();
                if self.current_char() == '&' {
                    self.advance();
                }
                let val = self.read_identifier()?;
                if val.chars().all(|c| c.is_ascii_hexdigit()) {
                    // relative hex padding
                    let count = u16::from_str_radix(&val, 16).map_err(|_| {
                        self.syntax_error("Invalid relative padding value".to_string())
                    })?;
                    Ok(Token::RelativePadding(count))
                } else {
                    // relative padding to label
                    Ok(Token::RelativePaddingLabel(val))
                }
            }
            '%' => {
                self.advance();
                let macro_name = self.read_identifier()?;
                println!("Macro name: {}", macro_name);
                Ok(Token::MacroDef(macro_name))
            }
            '&' => {
                self.advance();
                // NEW: Support bare '&' (no identifier) to define an empty sublabel (parent/)
                let next = self.current_char();
                if next == '\n'
                    || next == '\0'
                    || next.is_whitespace()
                    || next == ')'
                    || next == ']'
                    || next == '}'
                {
                    // Treat as empty sublabel definition
                    return Ok(Token::SublabelDef(String::new()));
                }
                let sublabel = self.read_identifier()?;
                // In uxnasm.c, '&' always creates a sublabel definition (makelabel)
                Ok(Token::SublabelDef(sublabel))
            }
            // Try to parse as macro call if it looks like a macro invocation (e.g., MACRO_NAME!)
            _ => {
                // If at EOF or at a newline, return Eof (don't error on trailing empty lines)
                if self.position >= self.input.len() || ch == '\0' || ch == '\n' {
                    return Ok(Token::Eof);
                }
                let name = self.read_identifier().ok();
                if let Some(name) = name {
                    if name.is_empty()
                        && (self.position >= self.input.len() || self.current_char() == '\0')
                    {
                        return Ok(Token::Eof);
                    }
                    return Ok(Token::Word(name));
                }
                Err(self.syntax_error(format!("Unexpected character: '{}'", ch)))
            }
        }
    }
}

/// Check if an identifier is a known instruction name
fn is_instruction_name(identifier: &str) -> bool {
    // List of UXN instruction names that could be confused with hex
    const INSTRUCTIONS: &[&str] = &[
        "ADD", "ADD2", "SUB", "SUB2", "MUL", "MUL2", "DIV", "DIV2", "AND", "AND2", "ORA", "ORA2",
        "EOR", "EOR2", "SFT", "SFT2", "LDZ", "LDZ2", "STZ", "STZ2", "LDR", "LDR2", "STR", "STR2",
        "LDA", "LDA2", "STA", "STA2", "DEI", "DEI2", "DEO", "DEO2", "INC", "INC2", "POP", "POP2",
        "NIP", "NIP2", "SWP", "SWP2", "ROT", "ROT2", "DUP", "DUP2", "OVR", "OVR2", "EQU", "EQU2",
        "NEQ", "NEQ2", "GTH", "GTH2", "LTH", "LTH2", "JMP", "JMP2", "JCN", "JCN2", "JSR", "JSR2",
        "STH", "STH2", "BRK", "LIT", "LIT2",
    ];

    // Check the base instruction name (without mode flags)
    let base_name = if identifier.len() > 3 {
        let mut base = identifier.to_string();
        // Remove mode flags (k, r, 2)
        while base.ends_with('k') || base.ends_with('r') || base.ends_with('2') {
            base.pop();
        }
        base
    } else {
        identifier.to_string()
    };

    INSTRUCTIONS.iter().any(|&inst| {
        inst == identifier
            || inst == base_name
            || (inst.ends_with('2') && inst[..inst.len() - 1] == base_name)
    })
}
