//! Error types for the UXN TAL assembler

use thiserror::Error;

/// Errors that can occur during TAL assembly
#[derive(Error, Debug)]
pub enum AssemblerError {
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
}

/// Result type for assembler operations
pub type Result<T> = std::result::Result<T, AssemblerError>;
