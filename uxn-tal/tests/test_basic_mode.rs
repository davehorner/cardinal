use std::fs;
use std::io::Write;
use uxn_tal_defined::v1::{ProtocolParseResult, ProtocolVarVar};
use uxn_tal_defined::EmulatorLauncher;

#[test]
fn test_basic_mode_with_uxn_mapper() {
    // Create a temp directory
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let bas_path = temp_dir.path().join("test.bas");
    let mut f = fs::File::create(&bas_path).expect("create .bas file");
    writeln!(f, "10 PRINT \"HELLO WORLD\"").unwrap();

    // Simulate protocol parse result with basic mode
    let mut result = ProtocolParseResult {
        url_raw:
            "uxntal://https://git.sr.ht/~rabbits/tiny-basic/tree/main/item/examples/sierpinski.bas"
                .to_string(),
        raw: Default::default(),
        proto_vars: Default::default(),
        query_vars: Default::default(),
        bang_vars: Default::default(),
        url: "https://git.sr.ht/~rabbits/tiny-basic/tree/main/item/examples/sierpinski.bas"
            .to_string(),
        protocol: "uxntal".to_string(),
        query_string: "".to_string(),
        repo_ref: None,
    };
    result
        .proto_vars
        .insert("basic".to_string(), ProtocolVarVar::Bool(true));

    // Use the real RomCache implementation from uxn-tal
    let mapper = uxn_tal_defined::emu_uxn::UxnMapper;

    // Build the command
    let cmd = mapper.build_command(
        &result,
        "test.bas",
        std::path::Path::new("uxnemu"),
        Some(temp_dir.path()),
    );

    // Check that the command args include just the .bas file (basic ROM handling is done by mode_basic)
    let args: Vec<String> = cmd
        .get_args()
        .map(|a| a.to_string_lossy().to_string())
        .collect();

    // Should be: [test.bas] - the basic ROM would be handled by mode_basic module
    assert_eq!(args.len(), 1, "Should have 1 argument: the .bas file");
    assert_eq!(args[0], "test.bas", "First arg should be the .bas file");
}

#[test]
fn test_auto_detection_bas_file() {
    // Test that URLs ending with .bas automatically enable basic mode
    let url =
        "uxntal://https://git.sr.ht/~rabbits/tiny-basic/tree/main/item/examples/sierpinski.bas";
    let parsed = uxn_tal::parse_uxntal_url(url);

    // Check that basic=true was automatically set
    assert_eq!(
        parsed.proto_vars.get("basic"),
        Some(&uxn_tal_defined::v1::ProtocolVarVar::Bool(true)),
        "basic mode should be automatically enabled for .bas files"
    );
}

#[test]
fn test_bas_and_orca_modes_dont_conflict() {
    // Test that .bas files don't trigger orca mode and vice versa
    let bas_url = "uxntal://github.com/example/repo/program.bas";
    let bas_parsed = uxn_tal::parse_uxntal_url(bas_url);

    let orca_url = "uxntal://github.com/example/repo/song.orca";
    let orca_parsed = uxn_tal::parse_uxntal_url(orca_url);

    // .bas file should enable basic mode but not orca mode
    assert_eq!(
        bas_parsed.proto_vars.get("basic"),
        Some(&uxn_tal_defined::v1::ProtocolVarVar::Bool(true)),
        ".bas file should enable basic mode"
    );
    assert_eq!(
        bas_parsed.proto_vars.get("orca"),
        None,
        ".bas file should not enable orca mode"
    );

    // .orca file should enable orca mode but not basic mode
    assert_eq!(
        orca_parsed.proto_vars.get("orca"),
        Some(&uxn_tal_defined::v1::ProtocolVarVar::Bool(true)),
        ".orca file should enable orca mode"
    );
    assert_eq!(
        orca_parsed.proto_vars.get("basic"),
        None,
        ".orca file should not enable basic mode"
    );
}
