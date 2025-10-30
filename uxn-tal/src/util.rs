/// Returns the canonical orca ROM path and its cache dir for the canonical orca URL, without fetching or assembling.
/// Returns (orca_rom_path, cache_dir) if present, or an error if not found.
pub fn get_cached_canonical_orca_rom() -> Result<(std::path::PathBuf, std::path::PathBuf), String> {
    use uxn_tal_common::hash_url;
    use uxn_tal_defined::consts::CANONICAL_ORCA;
    // Get the canonical orca URL and hash it for the cache dir
    let url = CANONICAL_ORCA;
    let roms_dir =
        crate::paths::uxntal_roms_get_path().ok_or("Failed to get uxntal roms directory")?;
    let cache_dir = roms_dir.join(format!("{}", hash_url(url)));
    let orca_rom = cache_dir.join("orca.rom");
    if !orca_rom.exists() {
        return Err(format!(
            "orca.rom not found in cache dir: {}",
            orca_rom.display()
        ));
    }
    let metadata =
        std::fs::metadata(&orca_rom).map_err(|e| format!("orca.rom metadata error: {e}"))?;
    if metadata.len() == 0 {
        return Err(format!(
            "orca.rom is empty in cache dir: {}",
            orca_rom.display()
        ));
    }
    Ok((orca_rom, cache_dir))
}
/// Returns the path to the canonical orca ROM in the workspace, if it exists, without fetching or parsing includes.
/// Returns (orca_rom_path, roms_dir) on success.
pub fn get_workspace_canonical_orca_rom() -> Result<(std::path::PathBuf, std::path::PathBuf), String>
{
    let roms_dir = std::path::PathBuf::from("roms");
    let orca_rom = roms_dir.join("orca.rom");
    if !orca_rom.exists() {
        return Err(format!(
            "orca.rom not found in workspace roms dir: {}",
            orca_rom.display()
        ));
    }
    let metadata =
        std::fs::metadata(&orca_rom).map_err(|e| format!("orca.rom metadata error: {e}"))?;
    if metadata.len() == 0 {
        return Err(format!(
            "orca.rom is empty in workspace roms dir: {}",
            orca_rom.display()
        ));
    }
    Ok((orca_rom, roms_dir))
}
/// Resolves and ensures the canonical orca ROM is present in the cache directory, returning its path.
/// Returns (orca_rom_path, cache_dir) on success.
pub fn resolve_canonical_orca_rom() -> Result<(std::path::PathBuf, std::path::PathBuf), String> {
    // Try to get the cached canonical orca ROM without fetching
    if let Ok(pair) = get_cached_canonical_orca_rom() {
        return Ok(pair);
    }
    // If not present, fall back to fetching/assembling
    use uxn_tal_common::cache::RomEntryResolver;
    use uxn_tal_defined::consts::CANONICAL_ORCA;
    let entry_resolver = crate::util::RealRomEntryResolver;
    let (_tal_path, cache_dir) = entry_resolver
        .resolve_entry_and_cache_dir(CANONICAL_ORCA)
        .map_err(|e| format!("Failed to resolve canonical orca: {e}"))?;
    let orca_rom = cache_dir.join("orca.rom");
    if !orca_rom.exists() {
        return Err(format!(
            "orca.rom not found in cache dir: {}",
            orca_rom.display()
        ));
    }
    let metadata =
        std::fs::metadata(&orca_rom).map_err(|e| format!("orca.rom metadata error: {e}"))?;
    if metadata.len() == 0 {
        return Err(format!(
            "orca.rom is empty in cache dir: {}",
            orca_rom.display()
        ));
    }
    Ok((orca_rom, cache_dir))
}
use uxn_tal_common::cache::RomEntryResolver;

/// Real implementation of RomEntryResolver for use in uxn-tal and integration tests.
pub struct RealRomEntryResolver;
impl RomEntryResolver for RealRomEntryResolver {
    fn resolve_entry_and_cache_dir(
        &self,
        url: &str,
    ) -> Result<(std::path::PathBuf, std::path::PathBuf), String> {
        crate::fetch::downloader::resolve_and_fetch_entry(url).map_err(|e| format!("{e}"))
    }
}
use crate::assemble_file;
// src/util.rs
use std::io::IsTerminal;

// Helper: pause for 15 seconds on error
pub fn pause_on_error() {
    if !std::io::stderr().is_terminal() && !std::io::stdout().is_terminal() {
        // no console attached — don't pause
        return;
    }
    use std::{thread, time};
    eprintln!("\n---\nKeeping window open for 15 seconds so you can read the above. Press Enter to continue...");
    use std::sync::mpsc;

    let (tx, rx) = mpsc::channel();
    #[cfg(not(target_arch = "wasm32"))]
    thread::spawn(move || {
        let mut _buf = String::new();
        let _ = std::io::stdin().read_line(&mut _buf);
        let _ = tx.send(());
    });
    let _ = rx.recv_timeout(time::Duration::from_secs(15));
}
// Helper: pause for Windows console
pub fn pause_for_windows() {
    #[cfg(target_os = "windows")]
    {
        if std::io::stdout().is_terminal() || std::io::stderr().is_terminal() {
            use std::io::Write;

            print!("Press Enter to continue...");
            let _ = std::io::stdout().flush();
            let mut _buf = String::new();
            let _ = std::io::stdin().read_line(&mut _buf);
        }
    }
}

use std::path::{Path, PathBuf};
/// Real implementation of get_or_write_cached_rom for RomCache trait
pub struct RealRomCache;
impl uxn_tal_common::cache::RomCache for RealRomCache {
    fn get_or_write_cached_rom(&self, url: &str, out_path: &Path) -> Result<PathBuf, String> {
        // Try to resolve and fetch the entry (tal or orca file) and get the cache dir
        let (entry_path, cache_dir) = crate::fetch::downloader::resolve_and_fetch_entry(url)
            .map_err(|e| format!("resolve_and_fetch_entry failed: {e}"))?;
        let rom_path = cache_dir.join(
            out_path
                .file_name()
                .unwrap_or_else(|| std::ffi::OsStr::new("out.rom")),
        );
        // If ROM already exists, return it
        if rom_path.exists() {
            return Ok(rom_path);
        }
        // If entry is a .rom, just copy it
        if let Some(ext) = entry_path.extension() {
            if ext == "rom" {
                std::fs::copy(&entry_path, &rom_path)
                    .map_err(|e| format!("Failed to copy ROM: {e}"))?;
                return Ok(rom_path);
            }
        }
        // Otherwise, assemble .tal to .rom
        let tal_path = entry_path;
        let rom_bytes = assemble_file(&tal_path).map_err(|e| format!("Assembler error: {e}"))?;
        std::fs::write(&rom_path, &rom_bytes).map_err(|e| format!("Failed to write ROM: {e}"))?;
        Ok(rom_path)
    }
}
