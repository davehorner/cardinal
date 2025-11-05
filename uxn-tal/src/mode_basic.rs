use std::path::Path;
use std::process::Command;
use uxn_tal_defined::v1::{EmulatorLauncher, ProtocolParseResult};

/// Returns the canonical basic ROM path and its cache dir for the canonical basic URL, without fetching or assembling.
/// Returns (basic_rom_path, cache_dir) if present, or an error if not found.
pub fn get_cached_canonical_basic_rom() -> Result<(std::path::PathBuf, std::path::PathBuf), String>
{
    use uxn_tal_common::hash_url;
    use uxn_tal_defined::consts::CANONICAL_BASIC;
    // Get the canonical basic URL and hash it for the cache dir
    let url = CANONICAL_BASIC;
    let roms_dir =
        crate::paths::uxntal_roms_get_path().ok_or("Failed to get uxntal roms directory")?;
    let cache_dir = roms_dir.join(format!("{}", hash_url(url)));
    let basic_rom = cache_dir.join("basic.rom");
    if !basic_rom.exists() {
        return Err(format!(
            "basic.rom not found in cache dir: {}",
            basic_rom.display()
        ));
    }
    let metadata =
        std::fs::metadata(&basic_rom).map_err(|e| format!("basic.rom metadata error: {e}"))?;
    if metadata.len() == 0 {
        return Err(format!(
            "basic.rom is empty in cache dir: {}",
            basic_rom.display()
        ));
    }
    Ok((basic_rom, cache_dir))
}

/// Resolves and ensures the canonical basic ROM is present in the cache directory, returning its path.
/// Returns (basic_rom_path, cache_dir) on success.
pub fn resolve_canonical_basic_rom() -> Result<(std::path::PathBuf, std::path::PathBuf), String> {
    // Try to get the cached canonical basic ROM without fetching
    if let Ok(pair) = get_cached_canonical_basic_rom() {
        return Ok(pair);
    }
    // If not present, fall back to fetching/assembling
    use uxn_tal_common::cache::RomEntryResolver;
    use uxn_tal_defined::consts::CANONICAL_BASIC;
    let entry_resolver = crate::util::RealRomEntryResolver;
    let (tal_path, cache_dir) = entry_resolver
        .resolve_entry_and_cache_dir(CANONICAL_BASIC)
        .map_err(|e| format!("Failed to resolve canonical basic: {e}"))?;
    let basic_rom = cache_dir.join("basic.rom");
    // If basic.rom is missing, but we have the TAL, assemble and cache it
    if !basic_rom.exists() {
        // Assemble the canonical basic TAL to basic.rom, setting CWD to cache_dir for include resolution
        let prev_dir =
            std::env::current_dir().map_err(|e| format!("Failed to get current dir: {e}"))?;
        let set_dir = std::env::set_current_dir(&cache_dir);
        let rom_bytes = match set_dir {
            Ok(_) => {
                let result = crate::assemble_file(&tal_path)
                    .map_err(|e| format!("Failed to assemble canonical basic.tal: {e}"));
                // Restore previous dir
                let _ = std::env::set_current_dir(&prev_dir);
                result?
            }
            Err(e) => {
                return Err(format!("Failed to set current dir to cache dir: {e}"));
            }
        };
        std::fs::write(&basic_rom, &rom_bytes)
            .map_err(|e| format!("Failed to write canonical basic.rom: {e}"))?;
    }
    let metadata =
        std::fs::metadata(&basic_rom).map_err(|e| format!("basic.rom metadata error: {e}"))?;
    if metadata.len() == 0 {
        return Err(format!(
            "basic.rom is empty in cache dir: {}",
            basic_rom.display()
        ));
    }
    Ok((basic_rom, cache_dir))
}

/// Handle basic mode execution: resolve canonical basic ROM and launch emulator with it
pub fn handle_basic_mode(
    result: &ProtocolParseResult,
    user_rom_path: &str,
    mapper: &dyn EmulatorLauncher,
    emulator_path: &Path,
    working_dir: Option<&Path>,
) -> Result<Command, Box<dyn std::error::Error>> {
    // Resolve canonical basic ROM
    let (canonical_basic_rom, _canonical_cache_dir) = resolve_canonical_basic_rom()?;

    // Build command with canonical ROM first, user ROM second
    let mut cmd = mapper.build_command(
        result,
        &canonical_basic_rom.display().to_string(),
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

/// Handle .bas file processing with protocol result
pub fn handle_basic_file_with_protocol(
    result: &ProtocolParseResult,
    canon_input_p: &Path,
    verbose: bool,
) -> Result<(), String> {
    use uxn_tal_defined::get_emulator_launcher;
    use uxn_tal_defined::v1::ProtocolVarVar;

    // Get emulator launcher from parsed result
    let (mapper, emulator_path) =
        get_emulator_launcher(result).ok_or("Failed to get emulator launcher")?;

    let basic_dir = canon_input_p.parent().unwrap_or_else(|| Path::new("."));
    let rel_basic = match canon_input_p.strip_prefix(basic_dir) {
        Ok(rel) => rel.display().to_string(),
        Err(_) => canon_input_p.display().to_string(),
    };

    // Check if basic mode is explicitly requested (uses canonical ROM)
    if let Some(ProtocolVarVar::Bool(true)) = result.proto_vars.get("basic") {
        let cmd = handle_basic_mode(
            result,
            &rel_basic,
            mapper.as_ref(),
            &emulator_path,
            Some(basic_dir),
        )
        .map_err(|e| format!("Failed to setup basic mode: {}", e))?;
        crate::emulator_utils::spawn_emulator_with_timeout(
            mapper.as_ref(),
            result,
            cmd,
            &emulator_path,
            verbose,
        )?;
    } else {
        // Normal .bas file handling (no canonical ROM needed)
        let mut cmd = mapper.build_command(result, &rel_basic, &emulator_path, Some(basic_dir));
        cmd.current_dir(basic_dir);
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
