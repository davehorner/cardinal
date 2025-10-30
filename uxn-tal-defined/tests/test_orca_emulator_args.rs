use std::fs;
use std::io::Write;
use std::path::PathBuf;
use uxn_tal_defined::consts::CANONICAL_ORCA;
use uxn_tal_defined::emu_uxn::UxnMapper;
use uxn_tal_defined::v1::{ProtocolParseResult, ProtocolVarVar};
use uxn_tal_defined::EmulatorLauncher;

#[test]
fn test_orca_url_runs_with_cached_canonical_rom() {
    // Create a temp .orca file in a temp dir
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let orca_path = temp_dir.path().join("test.orca");
    let mut f = fs::File::create(&orca_path).expect("create .orca file");
    writeln!(f, "; test orca file").unwrap();

    // Simulate protocol parse result with orca mode
    let mut result = ProtocolParseResult {
        raw: Default::default(),
        proto_vars: Default::default(),
        query_vars: Default::default(),
        bang_vars: Default::default(),
        url: CANONICAL_ORCA.to_string(),
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
    std::env::set_current_dir(&temp_dir).expect("set cwd to temp dir");
    // The build_command should resolve and cache the canonical orca ROM and build the correct command
    let cmd = mapper.build_command(
        &result,
        orca_path.to_str().unwrap(),
        std::path::Path::new("uxnemu"),
    );
    // Restore original working directory
    std::env::set_current_dir(orig_cwd).expect("restore cwd");

    // The canonical orca.rom should exist in the temp dir
    let orca_rom = temp_dir.path().join("orca.rom");
    assert!(
        orca_rom.exists(),
        "orca.rom should be cached in the temp dir"
    );
    // Check that the command args are [orca.rom, test.orca]
    let args: Vec<String> = cmd
        .get_args()
        .map(|a| a.to_string_lossy().to_string())
        .collect();
    assert_eq!(
        args,
        vec![
            orca_rom.to_string_lossy().to_string(),
            orca_path.to_string_lossy().to_string()
        ],
        "emulator args should be [orca.rom, .orca file]"
    );
}
