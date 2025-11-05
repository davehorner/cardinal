// Test for basic emulator mapper functionality (not actual ROM resolution)

use uxn_tal_defined::v1::ProtocolParseResult;
use uxn_tal_defined::EmulatorLauncher;

#[test]
fn test_emulator_mapper_basic_command_building() {
    // This test verifies that the emulator mappers can build basic commands
    // without requiring network access or actual ROM caching

    let temp_dir = tempfile::tempdir().expect("create temp dir");

    // Simulate a basic protocol parse result
    let result = ProtocolParseResult {
        url_raw: "test://example.com/test.tal".to_string(),
        raw: Default::default(),
        proto_vars: Default::default(),
        query_vars: Default::default(),
        bang_vars: Default::default(),
        url: "test://example.com/test.tal".to_string(),
        protocol: "test".to_string(),
        query_string: String::new(),
        repo_ref: None,
    };

    let mapper = uxn_tal_defined::emu_uxn::UxnMapper;

    // Test that the mapper can build a command
    let cmd = mapper.build_command(
        &result,
        "test.rom",
        std::path::Path::new("uxnemu"),
        Some(temp_dir.path()),
    );

    // The command should be properly configured
    assert_eq!(cmd.get_program(), "uxnemu");

    let args: Vec<String> = cmd
        .get_args()
        .map(|a| a.to_string_lossy().to_string())
        .collect();
    assert_eq!(args, vec!["test.rom"], "Should have ROM as argument");
}
