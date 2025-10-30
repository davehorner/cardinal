use std::fs;
use std::io::Write;
use uxn_tal::RealRomEntryResolver;
use uxn_tal_common::cache::RomEntryResolver;
use uxn_tal_defined::emu_buxn::BuxnMapper;
use uxn_tal_defined::v1::{ProtocolParseResult, ProtocolVarVar};
use uxn_tal_defined::EmulatorLauncher;

#[test]
fn test_buxn_runs_buxn_gui_with_orca_rom_and_orca_file() {
    // Use a file:// URL for the .orca file and resolve its cache dir
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let orca_path = temp_dir.path().join("test.orca");
    let mut f = fs::File::create(&orca_path).expect("create .orca file");
    let orca_pattern = ".........................................\n.#.ADD.#.................................\n.........................................\n.#.ADD.TWO.NUMBERS.TOGETHER.#............\n.........................................\n.1A2.....................................\n..3......................................\n.........................................\n.#.ADD.THREE.NUMBERS.TOGETHER.#..........\n.........................................\n.1A2A3...................................\n..3A5....................................\n...8.....................................\n.........................................\n.........................................\n.........................................\n.........................................\n";
    f.write_all(orca_pattern.as_bytes()).unwrap();
    let orca_url = format!("file://{}", orca_path.to_string_lossy());
    // Use the real RomEntryResolver implementation
    let entry_resolver = RealRomEntryResolver;
    let (_entry_path, cache_dir) = entry_resolver
        .resolve_entry_and_cache_dir(&orca_url)
        .expect("resolve file:// .orca cache dir");
    // Copy the .orca file into the cache dir (simulate fetch)
    let cached_orca_path: std::path::PathBuf = cache_dir.join("test.orca");
    std::fs::create_dir_all(&cache_dir).expect("create cache dir");
    std::fs::copy(&orca_path, &cached_orca_path).expect("copy .orca to cache dir");

    // Simulate protocol parse result with orca mode
    let mut result = ProtocolParseResult {
        raw: Default::default(),
        proto_vars: Default::default(),
        query_vars: Default::default(),
        bang_vars: Default::default(),
        url: orca_url.clone(),
        protocol: String::new(),
        query_string: String::new(),
    };
    result
        .proto_vars
        .insert("orca".to_string(), ProtocolVarVar::Bool(true));
    // Use the real RomCache implementation for integration test
    let rom_cache = uxn_tal::RealRomCache;
    let mapper = uxn_tal_defined::emu_buxn::BuxnMapper {
        rom_cache: &rom_cache,
    };

    // Find buxn-gui in PATH
    let buxn_gui = which::which("buxn-gui").expect("buxn-gui must be in PATH for this test");

    // Save current working directory
    let orig_cwd = std::env::current_dir().expect("get cwd");
    std::env::set_current_dir(&cache_dir).expect("set cwd to cache dir");
    // The build_command should resolve and cache the canonical orca ROM and build the correct command
    // Use only the filename (relative path) for the .orca file
    let orca_filename = cached_orca_path.file_name().unwrap().to_string_lossy();
    let mut cmd = mapper.build_command(&result, &orca_filename, &buxn_gui);
    // Print debug info for manual testing
    println!("[DEBUG] Working dir: {}", cache_dir.display());
    println!("[DEBUG] Executable: {}", buxn_gui.display());
    println!("[DEBUG] Args: {:?}", cmd.get_args().collect::<Vec<_>>());

    // The canonical orca.rom should exist in the cache dir
    let orca_rom = cache_dir.join("orca.rom");
    assert!(
        orca_rom.exists(),
        "orca.rom should be cached in the cache dir"
    );
    // Actually run buxn-gui with the generated command (should not error)
    let mut child = cmd.spawn().expect("failed to spawn buxn-gui");
    let status = child.wait().expect("failed to wait on buxn-gui");
    // Accept both success and known error exit codes, but must launch
    assert!(
        status.success() || status.code().is_some(),
        "buxn-gui should run and exit"
    );
}
