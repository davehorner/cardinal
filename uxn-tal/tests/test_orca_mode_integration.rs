use std::fs;
use std::io::Write;
use uxn_tal::mode_orca;
use uxn_tal_defined::consts::CANONICAL_ORCA;
use uxn_tal_defined::v1::{ProtocolParseResult, ProtocolVarVar};

#[test]
#[ignore = "requires network access to git.sr.ht for canonical orca download, not available on GitHub CI"]
fn test_orca_mode_with_real_rom_cache() {
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

    // This test actually resolves and caches the canonical orca ROM
    let (orca_rom_path, _cache_dir) = mode_orca::resolve_canonical_orca_rom()
        .expect("Should resolve and cache canonical orca ROM");

    // Verify the orca ROM was cached and is not empty
    assert!(orca_rom_path.exists(), "orca.rom should be cached");
    let metadata = fs::metadata(&orca_rom_path).expect("orca.rom metadata");
    assert!(metadata.len() > 0, "orca.rom should not be empty");

    // Test the orca mode handling
    let mapper = uxn_tal_defined::emu_uxn::UxnMapper;
    let cmd = mode_orca::handle_orca_mode(
        &result,
        &orca_path.display().to_string(),
        &mapper,
        std::path::Path::new("uxnemu"),
        Some(temp_dir.path()),
    )
    .expect("Should handle orca mode");

    // Check that the command args include both the canonical orca ROM and the user .orca file
    let args: Vec<String> = cmd
        .get_args()
        .map(|a| a.to_string_lossy().to_string())
        .collect();

    // The command should have the canonical orca ROM first, then the user .orca file
    assert!(
        args.len() >= 2,
        "Should have at least 2 arguments: canonical orca ROM and .orca file"
    );

    // Check that the canonical orca ROM is in the arguments
    let orca_rom_name = orca_rom_path.file_name().unwrap().to_string_lossy();
    assert!(
        args.iter().any(|arg| arg.contains(&*orca_rom_name)),
        "emulator args should include canonical orca ROM, got: {:?}",
        args
    );

    // Check that the user .orca file is in the arguments
    let orca_file_name = orca_path.file_name().unwrap().to_string_lossy();
    assert!(
        args.iter().any(|arg| arg.contains(&*orca_file_name)),
        "emulator args should include user .orca file, got: {:?}",
        args
    );
}

#[test]
fn test_orca_mode_with_workspace_rom() {
    // Test using the workspace orca.rom if available (faster test that doesn't need network)
    if let Ok((orca_rom_path, _roms_dir)) = mode_orca::get_workspace_canonical_orca_rom() {
        // Create a temp .orca file
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

        // Verify the workspace orca ROM exists and is not empty
        assert!(orca_rom_path.exists(), "workspace orca.rom should exist");
        let metadata = fs::metadata(&orca_rom_path).expect("orca.rom metadata");
        assert!(metadata.len() > 0, "workspace orca.rom should not be empty");

        // Test the orca mode handling
        let mapper = uxn_tal_defined::emu_uxn::UxnMapper;
        let cmd = mode_orca::handle_orca_mode(
            &result,
            &orca_path.display().to_string(),
            &mapper,
            std::path::Path::new("uxnemu"),
            Some(temp_dir.path()),
        )
        .expect("Should handle orca mode");

        // Check that the command args include both the canonical orca ROM and the user .orca file
        let args: Vec<String> = cmd
            .get_args()
            .map(|a| a.to_string_lossy().to_string())
            .collect();

        assert!(
            args.len() >= 2,
            "Should have at least 2 arguments: canonical orca ROM and .orca file"
        );

        // Check that the canonical orca ROM is in the arguments
        let orca_rom_name = orca_rom_path.file_name().unwrap().to_string_lossy();
        assert!(
            args.iter().any(|arg| arg.contains(&*orca_rom_name)),
            "emulator args should include canonical orca ROM, got: {:?}",
            args
        );

        // Check that the user .orca file is in the arguments
        let orca_file_name = orca_path.file_name().unwrap().to_string_lossy();
        assert!(
            args.iter().any(|arg| arg.contains(&*orca_file_name)),
            "emulator args should include user .orca file, got: {:?}",
            args
        );
    } else {
        // Skip this test if no workspace orca.rom is available
        eprintln!("Skipping test: no workspace orca.rom found");
    }
}
