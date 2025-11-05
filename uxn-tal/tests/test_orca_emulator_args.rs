use std::fs;
use std::io::Write;
use uxn_tal::mode_orca;
use uxn_tal_defined::consts::CANONICAL_ORCA;
use uxn_tal_defined::v1::{ProtocolParseResult, ProtocolVarVar};

#[test]
#[ignore = "requires network access to git.sr.ht for canonical orca download, not available on GitHub CI"]
fn test_orca_url_runs_with_cached_canonical_rom() {
    // Create a temp .orca file in a temp dir
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let orca_path = temp_dir.path().join("test.orca");
    let mut f = fs::File::create(&orca_path).expect("create .orca file");
    writeln!(f, "; test orca file").unwrap();

    // Simulate protocol parse result with orca mode
    let mut result = ProtocolParseResult {
        url_raw: CANONICAL_ORCA.to_string(),
        raw: Default::default(),
        proto_vars: Default::default(),
        query_vars: Default::default(),
        bang_vars: Default::default(),
        url: CANONICAL_ORCA.to_string(),
        protocol: String::new(),
        query_string: String::new(),
        repo_ref: None,
    };
    result
        .proto_vars
        .insert("orca".to_string(), ProtocolVarVar::Bool(true));

    // Use the real RomCache implementation from uxn-tal
    let mapper = uxn_tal_defined::emu_uxn::UxnMapper;

    // Actually resolve and cache the canonical orca ROM using the orca mode functionality
    let (canonical_orca_rom, _canonical_cache_dir) = mode_orca::resolve_canonical_orca_rom()
        .expect("Should resolve and cache canonical orca ROM");

    // Copy the canonical orca ROM to our test temp dir so the mapper can find it
    let orca_rom_in_temp = temp_dir.path().join("orca.rom");
    std::fs::copy(&canonical_orca_rom, &orca_rom_in_temp)
        .expect("Should copy canonical orca ROM to temp dir");

    // Save current working directory
    let orig_cwd = std::env::current_dir().expect("get cwd");
    std::env::set_current_dir(&temp_dir).expect("set cwd to temp dir");

    // Use the orca mode functionality to build the proper command
    let cmd = mode_orca::handle_orca_mode(
        &result,
        "file.orca",
        &mapper,
        std::path::Path::new("uxnemu"),
        Some(temp_dir.path()),
    )
    .expect("Should handle orca mode");

    // Restore original working directory
    std::env::set_current_dir(orig_cwd).expect("restore cwd");

    // The canonical orca.rom should exist in the temp dir
    let orca_rom = temp_dir.path().join("orca.rom");
    assert!(
        orca_rom.exists(),
        "orca.rom should be cached in the temp dir"
    );

    // Check that the command args include both the canonical orca ROM and the .orca file
    let args: Vec<String> = cmd
        .get_args()
        .map(|a| a.to_string_lossy().to_string())
        .collect();

    // The command should include both ROMs - exact paths may vary due to orca mode handling
    assert!(
        args.len() >= 2,
        "Should have at least 2 arguments: orca ROM and .orca file"
    );
    assert!(
        args.iter().any(|arg| arg.contains("orca.rom")),
        "emulator args should include orca.rom, got: {:?}",
        args
    );
    assert!(
        args.iter().any(|arg| arg.contains("file.orca")),
        "emulator args should include .orca file, got: {:?}",
        args
    );
}
