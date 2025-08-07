//! ROM file generation and management

use crate::error::{AssemblerError, Result};
use std::io::Write;

/// Represents a UXN ROM file
pub struct Rom {
    data: Vec<u8>,
    position: u16,
    size: usize,
    source: Option<String>,
    path: Option<String>,
}

impl Rom {
    /// Create a new ROM instance
    pub fn new() -> Self {
        Self {
            data: vec![0; 0x10000],
            position: 0x0100, // Start at 0x0100 like uxnasm does
            size: 0,     
            source: None,
            path: None,
        }
    }

    pub fn set_source(&mut self, source: Option<String>) {
        self.source = source;
    }

    pub fn source(&self) -> Option<&String> {
        self.source.as_ref()
    }
    /// Returns the source path if available
    pub fn source_path(&self) -> Option<&String> {
        self.path.as_ref()
    }
    pub fn set_path(&mut self, path: Option<String>) {
        self.path = path;
    }
    /// Write a byte to the ROM at the current position
    pub fn write_byte(&mut self, byte: u8) -> Result<()> {
        if self.position as usize >= 65536 {
            return Err(AssemblerError::RomTooLarge {
                size: self.position as usize + 1,
            });
        }
        self.data[self.position as usize] = byte;
        self.position += 1;
        self.size = self.size.max(self.position as usize);
        Ok(())
    }

    /// Write a 16-bit value at a specific position without changing current position (big-endian)
    pub fn write_word_at(&mut self, position: u16, value: u16) -> Result<()> {
        self.write_byte_at(position, (value >> 8) as u8)?;
        self.write_byte_at(position + 1, value as u8)?;
        Ok(())
    }

    /// Write a 16-bit value to the ROM at the current position (big-endian)
    pub fn write_short(&mut self, value: u16) -> Result<()> {
        self.write_byte((value >> 8) as u8)?;
        self.write_byte(value as u8)?;
        Ok(())
    }

    /// Write multiple bytes to the ROM
    pub fn write_bytes(&mut self, bytes: &[u8]) -> Result<()> {
        for &byte in bytes {
            self.write_byte(byte)?;
        }
        Ok(())
    }

    /// Pad the ROM to a specific address
    pub fn pad_to(&mut self, address: u16) -> Result<()> {
        if address as usize > 65536 {
            return Err(AssemblerError::InvalidPadding { address });
        }
        self.position = address;
        self.size = self.size.max(address as usize);
        Ok(())
    }

    /// Get the current position in the ROM
    pub fn position(&self) -> u16 {
        self.position
    }

    /// Set the current position in the ROM
    pub fn set_position(&mut self, position: u16) -> Result<()> {
        if position as usize > 65536 {
            return Err(AssemblerError::InvalidPadding { address: position });
        }
        self.position = position;
        self.size = self.size.max(position as usize);
        Ok(())
    }

    /// Get the ROM data as a slice, trimmed to actual size
    pub fn data(&self) -> &[u8] {
        let start = 0x0100;
        let end = self.size.clamp(start, self.data.len());
        if end > self.data.len() || end <= start {
            &[]
        } else {
            // Find the first nonzero byte after 0x0100
            let data = &self.data[start..end];
            let first_nonzero = data.iter().position(|&b| b != 0).unwrap_or(0);
            &data[first_nonzero..]
        }
    }

        /// Returns true if any byte in the zero page (0x0000..0x0100) is nonzero
    pub fn has_zero_page_data(&self) -> bool {
        if self.data.len() < 0x0100 {
            return false;
        }
        self.data[..0x0100].iter().any(|&b| b != 0)
    }
    /// Get the size of the ROM
    pub fn len(&self) -> usize {
        // Compute the length of the ROM data as returned by data()
        let start = 0x0100;
        let end = self.size.clamp(start, self.data.len());
        if end > self.data.len() || end <= start {
            0
        } else {
            let data = &self.data[start..end];
            let first_nonzero = data.iter().position(|&b| b != 0).unwrap_or(0);
            data.len().saturating_sub(first_nonzero)
        }
    }

    /// Check if the ROM is empty
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    /// Save the ROM to a file
    pub fn save_to_file<P: AsRef<std::path::Path>>(&self, path: P) -> Result<()> {
        let mut file = std::fs::File::create(path)?;
        file.write_all(self.data())?;
        Ok(())
    }

    /// Write a byte at a specific position without changing current position
    pub fn write_byte_at(&mut self, position: u16, byte: u8) -> Result<()> {
        self.data[position as usize] = byte;
        if position as usize >= self.size {
            self.size = position as usize + 1;
        }
        Ok(())
    }

    /// Write a short at a specific position without changing current position
    pub fn write_short_at(&mut self, position: u16, value: u16) -> Result<()> {
        self.write_byte_at(position, (value >> 8) as u8)?;
        self.write_byte_at(position + 1, value as u8)?;
        Ok(())
    }

    /// Advance the ROM position by a specified number of bytes without writing data
    pub fn advance_position(&mut self, bytes: usize) {
        self.position = self.position.saturating_add(bytes as u16);
        self.size = self.size.max(self.position as usize);
    }
}

impl Default for Rom {
    fn default() -> Self {
        Self::new()
    }
}
