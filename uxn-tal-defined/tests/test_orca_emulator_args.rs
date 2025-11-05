use uxn_tal_defined::v1::ProtocolParseResult;
use uxn_tal_defined::EmulatorLauncher;

#[test]
fn test_uxn_mapper_basic_functionality() {
    // Test that the UxnMapper can build basic commands without orca mode
    let temp_dir = tempfile::tempdir().expect("create temp dir");

    let result = ProtocolParseResult {
        url_raw: "test".to_string(),
        raw: Default::default(),
        proto_vars: Default::default(),
        query_vars: Default::default(),
        bang_vars: Default::default(),
        url: "test".to_string(),
        protocol: String::new(),
        query_string: String::new(),
        repo_ref: None,
    };

    let mapper = uxn_tal_defined::emu_uxn::UxnMapper;
    let cmd = mapper.build_command(
        &result,
        "test.rom",
        std::path::Path::new("uxnemu"),
        Some(temp_dir.path()),
    );

    let args: Vec<String> = cmd
        .get_args()
        .map(|a| a.to_string_lossy().to_string())
        .collect();

    assert_eq!(args, vec!["test.rom"], "Should have ROM as argument");
}
