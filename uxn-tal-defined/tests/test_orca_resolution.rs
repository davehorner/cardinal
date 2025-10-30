// Test for canonical orca ROM resolution and caching

use std::fs;
use uxn_tal_common::cache::{DefaultRomEntryResolver, RomEntryResolver};
use uxn_tal_defined::consts::CANONICAL_ORCA;
use uxn_tal_defined::emu_uxn::UxnMapper;
use uxn_tal_defined::v1::{ProtocolParseResult, ProtocolVarVar};
use uxn_tal_defined::EmulatorLauncher;
// use std::path::PathBuf;

#[test]
fn test_canonical_orca_rom_resolution() {
    let url = CANONICAL_ORCA;
    // Use the API to get the cache dir and entry path
    // Use the RomEntryResolver trait for decoupled entry/cache dir resolution
    let entry_resolver = DefaultRomEntryResolver;
    let (_tal_path, cache_dir) = entry_resolver
        .resolve_entry_and_cache_dir(url)
        .expect("fetch orca.tal and includes");
    let orca_rom = cache_dir.join("orca.rom");

    // Clean up any existing orca.rom before test
    let _ = fs::remove_file(&orca_rom);

    // Simulate protocol parse result with orca mode
    let mut result = ProtocolParseResult {
        raw: Default::default(),
        proto_vars: Default::default(),
        query_vars: Default::default(),
        bang_vars: Default::default(),
        url: url.to_string(),
        protocol: String::new(),
        query_string: String::new(),
    };
    result
        .proto_vars
        .insert("orca".to_string(), ProtocolVarVar::Bool(true));
    // NOTE: Use DefaultRomCache as a stub. The real implementation must be injected by the application/test harness.
    let rom_cache = uxn_tal_common::cache::DefaultRomCache;
    let mapper = uxn_tal_defined::emu_uxn::UxnMapper {
        rom_cache: &rom_cache,
    };

    // Save current working directory
    let orig_cwd = std::env::current_dir().expect("get cwd");
    std::env::set_current_dir(&cache_dir).expect("set cwd to cache dir");
    // The build_command should resolve and cache the canonical orca ROM
    let _cmd = mapper.build_command(&result, url, std::path::Path::new("uxnemu"));
    // Restore original working directory
    std::env::set_current_dir(orig_cwd).expect("restore cwd");

    // The canonical orca.rom should exist in the cache directory
    assert!(
        orca_rom.exists(),
        "orca.rom should be cached in the cache directory"
    );
    // Optionally, check that the file is non-empty
    let metadata = fs::metadata(&orca_rom).expect("orca.rom metadata");
    assert!(metadata.len() > 0, "orca.rom should not be empty");
}
