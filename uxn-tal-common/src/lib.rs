// Shared utilities and mappers for uxn-tal and uxn-tal-defined

pub mod cache;
pub use cache::{get_or_write_cached_rom, hash_url, AssemblerError};

use std::path::PathBuf;

pub struct UxnMapper;

impl UxnMapper {
    pub fn is_available_in_path() -> Option<PathBuf> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            which::which("uxnemu").ok()
        }
        #[cfg(target_arch = "wasm32")]
        {
            None
        }
    }
    // Add feature support table and mapping logic here
}
