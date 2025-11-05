use std::path::Path;
use std::process::Command;
use uxn_tal_defined::v1::{EmulatorLauncher, ProtocolParseResult};

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
    let (tal_path, cache_dir) = entry_resolver
        .resolve_entry_and_cache_dir(CANONICAL_ORCA)
        .map_err(|e| format!("Failed to resolve canonical orca: {e}"))?;
    let orca_rom = cache_dir.join("orca.rom");
    // If orca.rom is missing, but we have the TAL, assemble and cache it
    if !orca_rom.exists() {
        // Assemble the canonical orca TAL to orca.rom, setting CWD to cache_dir for include resolution
        let prev_dir =
            std::env::current_dir().map_err(|e| format!("Failed to get current dir: {e}"))?;
        let set_dir = std::env::set_current_dir(&cache_dir);
        let rom_bytes = match set_dir {
            Ok(_) => {
                let result = crate::assemble_file(&tal_path)
                    .map_err(|e| format!("Failed to assemble canonical orca.tal: {e}"));
                // Restore previous dir
                let _ = std::env::set_current_dir(&prev_dir);
                result?
            }
            Err(e) => {
                return Err(format!("Failed to set current dir to cache dir: {e}"));
            }
        };
        std::fs::write(&orca_rom, &rom_bytes)
            .map_err(|e| format!("Failed to write canonical orca.rom: {e}"))?;
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

/// Handle orca mode execution: resolve canonical orca ROM and launch emulator with it
pub fn handle_orca_mode(
    result: &ProtocolParseResult,
    user_rom_path: &str,
    mapper: &dyn EmulatorLauncher,
    emulator_path: &Path,
    working_dir: Option<&Path>,
) -> Result<Command, Box<dyn std::error::Error>> {
    // Resolve canonical orca ROM
    let (canonical_orca_rom, _canonical_cache_dir) = resolve_canonical_orca_rom()?;

    // Build command with canonical ROM first, user ROM second
    let mut cmd = mapper.build_command(
        result,
        &canonical_orca_rom.display().to_string(),
        emulator_path,
        working_dir,
    );

    // Add user ROM as second argument
    cmd.arg(user_rom_path);

    // Set working directory if provided
    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }

    Ok(cmd)
}

/// Handle .orca file processing with protocol result
pub fn handle_orca_file_with_protocol(
    result: &ProtocolParseResult,
    canon_input_p: &Path,
    verbose: bool,
) -> Result<(), String> {
    use uxn_tal_defined::get_emulator_launcher;
    use uxn_tal_defined::v1::ProtocolVarVar;

    // Get emulator launcher from parsed result
    let (mapper, emulator_path) =
        get_emulator_launcher(result).ok_or("Failed to get emulator launcher")?;

    let orca_dir = canon_input_p.parent().unwrap_or_else(|| Path::new("."));
    let rel_orca = match canon_input_p.strip_prefix(orca_dir) {
        Ok(rel) => rel.display().to_string(),
        Err(_) => canon_input_p.display().to_string(),
    };

    // Use orca mode if protocol var is set, or if filename ends with .orca and PatchStorage
    //let is_patchstorage = result.url_raw.contains("patchstorage.com") || result.url.contains("patchstorage.com");
    let is_orca_file = canon_input_p
        .extension()
        .map(|e| e == "orca")
        .unwrap_or(false);
    let orca_mode = matches!(
        result.proto_vars.get("orca"),
        Some(ProtocolVarVar::Bool(true))
    ) || is_orca_file;
    if orca_mode {
        let cmd = handle_orca_mode(
            result,
            &rel_orca,
            mapper.as_ref(),
            &emulator_path,
            Some(orca_dir),
        )
        .map_err(|e| format!("Failed to setup orca mode: {}", e))?;
        crate::emulator_utils::spawn_emulator_with_timeout(
            mapper.as_ref(),
            result,
            cmd,
            &emulator_path,
            verbose,
        )?;
    } else {
        // Normal .orca file handling (no canonical ROM needed)
        let mut cmd = mapper.build_command(result, &rel_orca, &emulator_path, Some(orca_dir));
        cmd.current_dir(orca_dir);
        crate::emulator_utils::spawn_emulator_with_timeout(
            mapper.as_ref(),
            result,
            cmd,
            &emulator_path,
            verbose,
        )?;
    }

    Ok(())
}
