/// Trait for resolving a ROM entry and its cache directory.
/// This allows decoupling of entry resolution logic from the application crate.
pub trait RomEntryResolver {
    /// Given a URL, returns (entry_path, cache_dir) or an error string.
    fn resolve_entry_and_cache_dir(
        &self,
        url: &str,
    ) -> Result<(std::path::PathBuf, std::path::PathBuf), String>;
}

/// Default stub implementation that always errors.
pub struct DefaultRomEntryResolver;
impl RomEntryResolver for DefaultRomEntryResolver {
    fn resolve_entry_and_cache_dir(
        &self,
        _url: &str,
    ) -> Result<(std::path::PathBuf, std::path::PathBuf), String> {
        Err("resolve_entry_and_cache_dir is not implemented. Please provide a real implementation in your application crate.".to_string())
    }
}
/// Trait for ROM caching/fetching. Implement this in your application crate.
pub trait RomCache {
    fn get_or_write_cached_rom(&self, url: &str, out_path: &Path) -> Result<PathBuf, String>;
}

/// Default stub implementation that always errors.
pub struct DefaultRomCache;
impl RomCache for DefaultRomCache {
    fn get_or_write_cached_rom(&self, _url: &str, _out_path: &Path) -> Result<PathBuf, String> {
        Err("get_or_write_cached_rom is not implemented. Please provide a real implementation in your application crate.".to_string())
    }
}
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

/// Non-cryptographic, stable hash for cache bucketing.
pub fn hash_url(url: &str) -> u64 {
    let mut h = DefaultHasher::new();
    url.hash(&mut h);
    h.finish()
}

/// Placeholder for AssemblerError type. Replace with real error type in integration.
pub type AssemblerError = String;

/// Stub only: the real implementation must be provided in the application crate (uxn-tal).
pub fn get_or_write_cached_rom(_url: &str, _out_path: &Path) -> Result<PathBuf, AssemblerError> {
    Err("get_or_write_cached_rom is not implemented in uxn-tal-common. Please provide an implementation in your application crate.".to_string())
}
