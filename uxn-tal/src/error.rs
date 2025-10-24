use std::fmt;
impl fmt::Display for LabelReferenceErrorData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Label reference error at {}:{}:{} for label '{}'\nToken: {:?}\nSource: {}",
            self.path, self.line, self.position, self.label, self.token, self.source_line
        )
    }
}

use thiserror::Error;

/// Errors that can occur during TAL assembly
#[derive(Error, Debug)]
pub struct LabelReferenceErrorData {
    pub path: String,
    pub line: usize,
    pub position: usize,
    pub label: String,
    pub token: crate::lexer::TokenWithPos,
    pub source_line: String,
}

#[derive(Error, Debug)]
pub enum AssemblerError {
    #[error("Label reference error at {0:?}")]
    LabelReferenceError(Box<LabelReferenceErrorData>),
    #[error("Expected identifier at {path}:{line}:{position} after token: {after_token} (found '{found}')\nSource: {source_line}")]
    ExpectedIdentifierError {
        path: String,
        line: usize,
        position: usize,
        after_token: String,
        found: char,
        source_line: String,
    },
    #[error("File read error at {path}: {message}")]
    FileReadError { path: String, message: String },
    #[error("Syntax error at {path}:{line}:{position} {message}\nSource: {source_line}")]
    SyntaxError {
        path: String,
        line: usize,
        position: usize,
        message: String,
        source_line: String,
    },

    #[error("Unknown opcode: {opcode}")]
    UnknownOpcode { opcode: String },

    #[error("Invalid number format: {value}")]
    InvalidNumber { value: String },

    #[error("UTF-8 error at {path}:{line}:{position} {message}\nSource: {source_line}")]
    Utf8Error {
        path: String,
        line: usize,
        position: usize,
        message: String,
        source_line: String,
    },

    #[error("Undefined label: {label}")]
    UndefinedLabel { label: String },

    #[error("Label already defined: {label}")]
    DuplicateLabel { label: String },

    #[error("Invalid addressing mode")]
    InvalidAddressing,

    #[error("ROM too large: {size} bytes (maximum 65536)")]
    RomTooLarge { size: usize },

    #[error("Invalid padding address: {address:04x}")]
    InvalidPadding { address: u16 },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Internal error: {message}")]
    Internal { message: String },

    #[error("Backend error: {message}")]
    Backend { message: String },

    #[error("Disassembly error: {message}")]
    Disassembly { message: String },
}

/// Result type for assembler operations
pub type Result<T> = std::result::Result<T, AssemblerError>;
