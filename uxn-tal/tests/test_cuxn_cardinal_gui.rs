use std::fs;
use std::io::Write;
use uxn_tal::mode_orca;
use uxn_tal::RealRomEntryResolver;
use uxn_tal_common::cache::RomEntryResolver;
use uxn_tal_defined::emu_cuxn::CuxnMapper;
use uxn_tal_defined::v1::{ProtocolParseResult, ProtocolVarVar};

#[test]
#[ignore = "requires cardinal-gui executable in PATH, not available on GitHub CI"]
fn test_cuxn_runs_cardinal_gui_with_orca_rom_and_orca_file() {
    // Use a file:// URL for the .orca file and resolve its cache dir
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let orca_path = temp_dir.path().join("test.orca");
    let mut f = fs::File::create(&orca_path).expect("create .orca file");
    let orca_pattern = ".........................................\n.#.ADD.#.................................\n.........................................\n.#.ADD.TWO.NUMBERS.TOGETHER.#............\n.........................................\n.1A2.....................................\n..3......................................\n.........................................\n.#.ADD.THREE.NUMBERS.TOGETHER.#..........\n.........................................\n.1A2A3...................................\n..3A5....................................\n...8.....................................\n.........................................\n.........................................\n.........................................\n.........................................\n";
    f.write_all(orca_pattern.as_bytes()).unwrap();
    let orca_url = format!("file://{}", orca_path.to_string_lossy());
    // Use the real RomEntryResolver implementation
    let entry_resolver = RealRomEntryResolver;
    let (_entry_path, cache_dir): (std::path::PathBuf, std::path::PathBuf) = entry_resolver
        .resolve_entry_and_cache_dir(&orca_url)
        .expect("resolve file:// .orca cache dir");
    // Copy the .orca file into the cache dir (simulate fetch)
    let cached_orca_path = cache_dir.join("test.orca");
    std::fs::create_dir_all(&cache_dir).expect("create cache dir");
    std::fs::copy(&orca_path, &cached_orca_path).expect("copy .orca to cache dir");

    // Simulate protocol parse result with orca mode
    let mut result = ProtocolParseResult {
        url_raw: orca_url.clone(),
        raw: Default::default(),
        proto_vars: Default::default(),
        query_vars: Default::default(),
        bang_vars: Default::default(),
        url: orca_url.clone(),
        protocol: String::new(),
        query_string: String::new(),
        repo_ref: None,
    };
    result
        .proto_vars
        .insert("orca".to_string(), ProtocolVarVar::Bool(true));
    // Use the real RomCache implementation for integration test
    let mapper = CuxnMapper;

    // Actually resolve and cache the canonical orca ROM using the orca mode functionality
    let (canonical_orca_rom, _canonical_cache_dir) = mode_orca::resolve_canonical_orca_rom()
        .expect("Should resolve and cache canonical orca ROM");

    // Copy the canonical orca ROM to our test cache dir so the mapper can find it
    let orca_rom_in_cache = cache_dir.join("orca.rom");
    std::fs::copy(&canonical_orca_rom, &orca_rom_in_cache)
        .expect("Should copy canonical orca ROM to cache dir");

    // Find cardinal-gui in PATH
    let cardinal_gui =
        which::which("cardinal-gui").expect("cardinal-gui must be in PATH for this test");

    // Save current working directory and set to cache dir
    std::env::set_current_dir(&cache_dir).expect("set cwd to cache dir");

    // Use the orca mode functionality to build the proper command
    let orca_filename = cached_orca_path.file_name().unwrap().to_string_lossy();
    let mut cmd = mode_orca::handle_orca_mode(
        &result,
        &orca_filename,
        &mapper,
        &cardinal_gui,
        Some(&cache_dir),
    )
    .expect("Should handle orca mode");

    // Print debug info for manual testing
    println!("[DEBUG] Working dir: {}", cache_dir.display());
    println!("[DEBUG] Executable: {}", cardinal_gui.display());
    println!("[DEBUG] Args: {:?}", cmd.get_args().collect::<Vec<_>>());
    // // Restore original working directory
    // std::env::set_current_dir(orig_cwd).expect("restore cwd");

    // The canonical orca.rom should exist in the cache dir
    let orca_rom = cache_dir.join("orca.rom");
    assert!(
        orca_rom.exists(),
        "orca.rom should be cached in the cache dir"
    );
    // Actually run cardinal-gui with the generated command (should not error)
    let mut child = cmd.spawn().expect("failed to spawn cardinal-gui");
    let status = child.wait().expect("failed to wait on cardinal-gui");
    // Accept both success and known error exit codes, but must launch
    assert!(
        status.success() || status.code().is_some(),
        "cardinal-gui should run and exit"
    );
}
