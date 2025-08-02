//! Lexer for TAL assembly language

use crate::error::{AssemblerError, Result};

/// Token types in TAL assembly
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
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
    LabelDef(String),
    /// Label reference (e.g., ;main)
    LabelRef(String),
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
    /// Padding directive (e.g., |0100)
    Padding(u16),
    /// Skip bytes directive (e.g., $2)
    Skip(u16),
    /// Device access (e.g., .Screen/width)
    DeviceAccess(String, String), // device, field
    /// Macro definition (e.g., %MACRO)
    MacroDef(String),
    /// Macro call (e.g., MACRO)
    MacroCall(String),
    /// Brace open
    BraceOpen,
    /// Brace close
    BraceClose,
    /// Bracket open (inline assembly)
    BracketOpen,
    /// Bracket close (inline assembly)
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
}

/// Lexer for TAL assembly language
pub struct Lexer {
    input: String,
    position: usize,
    line: usize,
    path: Option<String>,
    position_on_line: usize,
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
    pub fn tokenize(&mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();

        loop {
            match self.next_token()? {
                Token::Eof => break,
                token => tokens.push(token),
            }
        }

        Ok(tokens)
    }

    /// Get the next token from the input
    pub fn next_token(&mut self) -> Result<Token> {
        // Robustly skip all whitespace except newlines before tokenizing
        while self.position < self.input.len() {
            let ch = self.current_char();
            if ch.is_whitespace() && ch != '\n' {
                self.advance();
            } else {
                break;
            }
        }

        if self.position >= self.input.len() {
            return Ok(Token::Eof);
        }

        let ch = self.current_char();

        match ch {
            '\n' => {
                self.advance();
                self.line += 1;
                Ok(Token::Newline)
            }
            '(' => {
                self.advance();
                let comment = self.read_comment()?;
                Ok(Token::Comment(comment))
            }
            '"' => {
                self.advance();
                // Check if this is a character literal ("a) or a string ("hello)
                let ch = self.current_char();
                if ch != '\0' && !ch.is_whitespace() {
                    // Look ahead to see if there's more content
                    let next_pos = self.position + 1;
                    let next_ch = if next_pos < self.input.len() {
                        self.input.chars().nth(next_pos).unwrap_or('\0')
                    } else {
                        '\0'
                    };

                    // If next character is whitespace or end of input, it's a character literal
                    if next_ch.is_whitespace()
                        || next_ch == '\0'
                        || next_ch == ']'
                        || next_ch == ')'
                    {
                        self.advance(); // consume the character
                        Ok(Token::CharLiteral(ch))
                    } else {
                        // It's a string
                        let string = self.read_string()?;
                        Ok(Token::RawString(string))
                    }
                } else {
                    // Empty string or invalid
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
                        message: "Unexpected end of file in character literal"
                            .to_string(),
                        source_line: self.get_current_line(),
                    });
                }
                self.advance();
                Ok(Token::CharLiteral(ch))
            }
            '#' => {
                self.advance();
                if self.current_char() == 'b' {
                    self.advance();
                    let binary = self.read_binary()?;
                    Ok(Token::BinLiteral(binary))
                } else {
                    let hex = self.read_hex()?;
                    Ok(Token::HexLiteral(hex))
                }
            }
            '@' => {
                self.advance();
                let label = self.read_identifier()?;
                Ok(Token::LabelDef(label))
            }
            ';' | ':' => {
                self.advance();
                if self.current_char() == '&' {
                    self.advance();
                    let sublabel = self.read_identifier()?;
                    Ok(Token::SublabelRef(sublabel))
                } else {
                    let label = self.read_identifier()?;
                    Ok(Token::LabelRef(label))
                }
            }
            '&' => {
                self.advance();
                let sublabel = self.read_identifier()?;
                Ok(Token::SublabelDef(sublabel))
            }
            ',' => {
                self.advance();
                // Check if next char is & (relative sublabel reference)
                if self.current_char() == '&' {
                    self.advance(); // consume &
                    let sublabel = self.read_identifier()?;
                    Ok(Token::RelativeRef(format!("&{}", sublabel)))
                } else {
                    let sublabel = self.read_identifier()?;
                    Ok(Token::SublabelRef(sublabel))
                }
            }
            '/' => {
                self.advance();
                let label = self.read_identifier()?;
                Ok(Token::RelativeRef(label))
            }
            '?' => {
                self.advance();
                // Check for conditional block start ?{
                if self.current_char() == '{' {
                    self.advance(); // consume the '{'
                    Ok(Token::ConditionalBlockStart)
                }
                // Check if next char is & (sublabel reference)
                else if self.current_char() == '&' {
                    self.advance();
                    let sublabel = self.read_identifier()?;
                    Ok(Token::ConditionalRef(format!("&{}", sublabel)))
                }
                // Check if next char starts an identifier
                else if self.current_char().is_ascii_alphabetic()
                    || self.current_char() == '_'
                {
                    let label = self.read_identifier()?;
                    Ok(Token::ConditionalRef(label))
                }
                // Otherwise it's a standalone conditional operator
                else {
                    Ok(Token::ConditionalOperator)
                }
            }
            '!' => {
                self.advance();
                // Check if next char is & (sublabel reference)
                if self.current_char() == '&' {
                    self.advance();
                    let sublabel = self.read_identifier()?;
                    Ok(Token::JSRRef(format!("&{}", sublabel)))
                } else {
                    let label = self.read_identifier()?;
                    Ok(Token::JSRRef(label))
                }
            }
            '-' => {
                self.advance();
                let identifier = self.read_identifier()?;
                Ok(Token::HyphenRef(identifier))
            }
            '|' => {
                self.advance();
                let addr_str = self.read_hex()?;
                let addr =
                    u16::from_str_radix(&addr_str, 16).map_err(|_| {
                        AssemblerError::InvalidNumber {
                            value: addr_str.clone(),
                        }
                    })?;
                Ok(Token::Padding(addr))
            }
            '$' => {
                self.advance();
                let count_str = self.read_hex()?;
                let count =
                    u16::from_str_radix(&count_str, 16).map_err(|_| {
                        AssemblerError::InvalidNumber {
                            value: count_str.clone(),
                        }
                    })?;
                Ok(Token::Skip(count))
            }
            '.' => {
                self.advance();
                let device_and_field = self.read_device_access()?;
                if let Some(slash_pos) = device_and_field.find('/') {
                    let device = device_and_field[..slash_pos].to_string();
                    let field = device_and_field[slash_pos + 1..].to_string();
                    Ok(Token::DeviceAccess(device, field))
                } else {
                    // Variable access without slash - treat as LabelRef
                    Ok(Token::LabelRef(device_and_field))
                }
            }
            '%' => {
                self.advance();
                let macro_name = self.read_identifier()?;
                Ok(Token::MacroDef(macro_name))
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
                Ok(Token::BracketOpen)
            }
            ']' => {
                self.advance();
                Ok(Token::BracketClose)
            }
            '~' => {
                self.advance();
                let filename = self.read_include_path()?;
                Ok(Token::Include(filename))
            }
            '<' => {
                self.advance();
                let macro_name = self.read_macro_call()?;
                Ok(Token::MacroCall(macro_name))
            }
            '=' => {
                self.advance();
                let label = self.read_identifier()?;
                Ok(Token::RawAddressRef(label))
            }
            _ if ch.is_ascii_digit() => {
                let number = self.read_hex_number()?;
                // In TAL, bare hex numbers are treated as raw hex bytes
                Ok(Token::RawHex(number))
            }
            _ if ch.is_ascii_alphabetic() || ch == '_' => {
                // First, peek ahead to see if this looks like hex data
                let start_pos = self.position;
                let identifier = self.read_identifier()?;

                // If it's all hex digits and an even number of chars (2, 4, 6, 8...)
                // and no special characters, treat as hex data
                if identifier.chars().all(|c| c.is_ascii_hexdigit())
                    && identifier.len() % 2 == 0
                    && identifier.len() >= 2
                {
                    // Reset position and read as hex bytes in pairs
                    self.position = start_pos;
                    let hex_byte = self.read_hex_number()?;
                    Ok(Token::RawHex(hex_byte))
                } else {
                    Ok(Token::Instruction(identifier))
                }
            }
            _ => {
                Err(self
                    .syntax_error(format!("Unexpected character: '{}'", ch)))
            }
        }
    }

    /// Get the current character at the position
    fn current_char(&self) -> char {
        self.input.chars().nth(self.position).unwrap_or('\0')
    }

    /// Move to the next character
    fn advance(&mut self) {
        self.position += 1;
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
        while self.position < self.input.len()
            && self.current_char() != delimiter
        {
            result.push(self.current_char());
            self.advance();
        }
        Ok(result)
    }

    /// Read comment with proper nested parentheses handling
    fn read_comment(&mut self) -> Result<String> {
        let mut result = String::new();
        let mut depth = 1; // We've already consumed the opening (

        while self.position < self.input.len() && depth > 0 {
            let ch = self.current_char();

            match ch {
                '(' => {
                    depth += 1;
                    result.push(ch);
                    self.advance();
                }
                ')' => {
                    depth -= 1;
                    if depth > 0 {
                        result.push(ch);
                    }
                    self.advance();
                }
                _ => {
                    result.push(ch);
                    self.advance();
                }
            }
        }

        Ok(result)
    }

    /// Read hexadecimal digits for #hex literals
    fn read_hex(&mut self) -> Result<String> {
        let mut result = String::new();
        while self.position < self.input.len() {
            let ch = self.current_char();
            if ch.is_ascii_hexdigit() {
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
    /// For asset data, splits long hex strings into byte pairs
    fn read_hex_number(&mut self) -> Result<String> {
        let mut result = String::new();
        while self.position < self.input.len() {
            let ch = self.current_char();
            if ch.is_ascii_hexdigit() {
                result.push(ch);
                self.advance();

                // If we have 2 hex digits, that's one byte - stop here
                // This allows c3c7 to be tokenized as c3, then c7
                if result.len() == 2 {
                    break;
                }
            } else {
                break;
            }
        }
        Ok(result)
    }

    /// Read an identifier (labels, instructions, etc.)
    /// Allows alphanumeric characters, underscores, hyphens, and forward slashes
    fn read_identifier(&mut self) -> Result<String> {
        let mut result = String::new();
        while self.position < self.input.len() {
            let ch = self.current_char();
            // Accept alphanumeric, underscore, hyphen, slash, and angle brackets in identifiers
            if ch.is_ascii_alphanumeric()
                || ch == '_'
                || ch == '-'
                || ch == '/'
                || ch == '<'
                || ch == '>'
            {
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
                message: "Expected identifier".to_string(),
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

    /// Read device access syntax like Console/char or Screen/width
    fn read_device_access(&mut self) -> Result<String> {
        let mut result = String::new();
        while self.position < self.input.len() {
            let ch = self.current_char();
            if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' || ch == '/'
            {
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
                message: "Expected device access".to_string(),
                source_line: self.get_current_line(),
            });
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

            // Allow alphanumeric, hyphens and underscores in macro names
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
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
}
