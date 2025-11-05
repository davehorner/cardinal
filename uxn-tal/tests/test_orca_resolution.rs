// Test for canonical orca ROM resolution and caching

use std::fs;
use uxn_tal::mode_orca;
use uxn_tal_defined::consts::CANONICAL_ORCA;
use uxn_tal_defined::v1::{ProtocolParseResult, ProtocolVarVar};

#[test]
#[ignore = "requires network access to git.sr.ht for canonical orca download, not available on GitHub CI"]
fn test_canonical_orca_rom_resolution() {
    let url = CANONICAL_ORCA;
    // Use the real uxn-tal resolver to get the cache dir and entry path
    let (_tal_path, cache_dir) =
        uxn_tal::resolve_entry_from_url(url).expect("fetch orca.tal and includes");
    let orca_rom = cache_dir.join("orca.rom");

    // Clean up any existing orca.rom before test
    let _ = fs::remove_file(&orca_rom);

    // Simulate protocol parse result with orca mode
    let mut result = ProtocolParseResult {
        url_raw: url.to_string(),
        raw: Default::default(),
        proto_vars: Default::default(),
        query_vars: Default::default(),
        bang_vars: Default::default(),
        url: url.to_string(),
        protocol: String::new(),
        query_string: String::new(),
        repo_ref: None,
    };
    result
        .proto_vars
        .insert("orca".to_string(), ProtocolVarVar::Bool(true));

    // Use the real RomCache implementation from uxn-tal
    let mapper = uxn_tal_defined::emu_uxn::UxnMapper;

    // Actually use the orca mode functionality to resolve and cache the canonical ROM
    let (canonical_orca_rom, canonical_cache_dir) = mode_orca::resolve_canonical_orca_rom()
        .expect("Should resolve and cache canonical orca ROM");

    // Copy to our test cache dir if different
    if canonical_cache_dir != cache_dir {
        std::fs::copy(&canonical_orca_rom, &orca_rom)
            .expect("Should copy canonical orca ROM to test cache dir");
    }

    // Save current working directory
    let orig_cwd = std::env::current_dir().expect("get cwd");
    std::env::set_current_dir(&cache_dir).expect("set cwd to cache dir");

    // Use the orca mode functionality to build the proper command
    let _cmd = mode_orca::handle_orca_mode(
        &result,
        "test.orca", // dummy orca file name
        &mapper,
        std::path::Path::new("uxnemu"),
        Some(&cache_dir),
    )
    .expect("Should handle orca mode");

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
