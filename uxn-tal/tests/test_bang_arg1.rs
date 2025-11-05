use std::borrow::Cow;
use std::fs;
use std::io::Write;
use uxn_tal_defined::v1::{
    ProtocolParseResult, ProtocolQueryType, ProtocolQueryVar, ProtocolQueryVarVar,
};
use uxn_tal_defined::EmulatorLauncher;

#[test]
fn test_bang_arg1_with_cuxn_mapper() {
    // Create a temp directory
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let tal_path = temp_dir.path().join("test.tal");
    let mut f = fs::File::create(&tal_path).expect("create .tal file");
    writeln!(f, "; test tal file").unwrap();

    // Simulate protocol parse result with !arg1 bang variable
    let mut result = ProtocolParseResult {
        url_raw: "uxntal://github.com/example/repo/test.tal?!arg1=--debug".to_string(),
        raw: Default::default(),
        proto_vars: Default::default(),
        query_vars: Default::default(),
        bang_vars: Default::default(),
        url: "https://github.com/example/repo/test.tal".to_string(),
        protocol: "uxntal".to_string(),
        query_string: "!arg1=--debug".to_string(),
        repo_ref: None,
    };
    result.bang_vars.insert(
        "arg1".to_string(),
        ProtocolQueryVar {
            name: Cow::Borrowed("arg1"),
            description: Cow::Borrowed("First argument to emulator"),
            example: Cow::Borrowed("--debug"),
            var_type: ProtocolQueryType::String,
            value: ProtocolQueryVarVar::String("--debug".to_string()),
        },
    );

    // Use the real RomCache implementation from uxn-tal
    let mapper = uxn_tal_defined::emu_cuxn::CuxnMapper;

    // Build the command
    let cmd = mapper.build_command(
        &result,
        "test.tal",
        std::path::Path::new("cardinal-gui"),
        Some(temp_dir.path()),
    );

    // Check that the command args include the ROM and then "-- --debug"
    let args: Vec<String> = cmd
        .get_args()
        .map(|a| a.to_string_lossy().to_string())
        .collect();

    // Should be: [test.tal, --, --debug]
    assert_eq!(
        args.len(),
        3,
        "Should have 3 arguments: TAL file, --, and bang arg"
    );
    assert_eq!(args[0], "test.tal", "First arg should be TAL file");
    assert_eq!(
        args[1], "--",
        "Second arg should be -- separator for cardinal-gui"
    );
    assert_eq!(args[2], "--debug", "Third arg should be the !arg1 value");
}

#[test]
fn test_bang_arg1_with_buxn_mapper() {
    // Create a temp directory
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let tal_path = temp_dir.path().join("test.tal");
    let mut f = fs::File::create(&tal_path).expect("create .tal file");
    writeln!(f, "; test tal file").unwrap();

    // Simulate protocol parse result with !arg1 bang variable
    let mut result = ProtocolParseResult {
        url_raw: "uxntal://github.com/example/repo/test.tal?!arg1=--verbose".to_string(),
        raw: Default::default(),
        proto_vars: Default::default(),
        query_vars: Default::default(),
        bang_vars: Default::default(),
        url: "https://github.com/example/repo/test.tal".to_string(),
        protocol: "uxntal".to_string(),
        query_string: "!arg1=--verbose".to_string(),
        repo_ref: None,
    };
    result.bang_vars.insert(
        "arg1".to_string(),
        ProtocolQueryVar {
            name: Cow::Borrowed("arg1"),
            description: Cow::Borrowed("First argument to emulator"),
            example: Cow::Borrowed("--verbose"),
            var_type: ProtocolQueryType::String,
            value: ProtocolQueryVarVar::String("--verbose".to_string()),
        },
    );

    // Use the real RomCache implementation from uxn-tal
    let mapper = uxn_tal_defined::emu_buxn::BuxnMapper;

    // Build the command
    let cmd = mapper.build_command(
        &result,
        "test.tal",
        std::path::Path::new("buxn"),
        Some(temp_dir.path()),
    );

    // Check that the command args include the ROM and then the bang arg (no -- separator for buxn)
    let args: Vec<String> = cmd
        .get_args()
        .map(|a| a.to_string_lossy().to_string())
        .collect();

    // Should be: [test.tal, --verbose] (no -- separator for buxn)
    assert_eq!(
        args.len(),
        2,
        "Should have 2 arguments: TAL file and bang arg"
    );
    assert_eq!(args[0], "test.tal", "First arg should be TAL file");
    assert_eq!(args[1], "--verbose", "Second arg should be the !arg1 value");
}

#[test]
fn test_bang_arg1_with_uxn_mapper() {
    // Create a temp directory
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let tal_path = temp_dir.path().join("test.tal");
    let mut f = fs::File::create(&tal_path).expect("create .tal file");
    writeln!(f, "; test tal file").unwrap();

    // Simulate protocol parse result with !arg1 bang variable
    let mut result = ProtocolParseResult {
        url_raw: "uxntal://github.com/example/repo/test.tal?!arg1=--help".to_string(),
        raw: Default::default(),
        proto_vars: Default::default(),
        query_vars: Default::default(),
        bang_vars: Default::default(),
        url: "https://github.com/example/repo/test.tal".to_string(),
        protocol: "uxntal".to_string(),
        query_string: "!arg1=--help".to_string(),
        repo_ref: None,
    };
    result.bang_vars.insert(
        "arg1".to_string(),
        ProtocolQueryVar {
            name: Cow::Borrowed("arg1"),
            description: Cow::Borrowed("First argument to emulator"),
            example: Cow::Borrowed("--help"),
            var_type: ProtocolQueryType::String,
            value: ProtocolQueryVarVar::String("--help".to_string()),
        },
    );

    // Use the real RomCache implementation from uxn-tal
    let mapper = uxn_tal_defined::emu_uxn::UxnMapper;

    // Build the command
    let cmd = mapper.build_command(
        &result,
        "test.tal",
        std::path::Path::new("uxnemu"),
        Some(temp_dir.path()),
    );

    // Check that the command args include the ROM and then the bang arg (no -- separator for uxn)
    let args: Vec<String> = cmd
        .get_args()
        .map(|a| a.to_string_lossy().to_string())
        .collect();

    // Should be: [test.tal, --help] (no -- separator for uxn)
    assert_eq!(
        args.len(),
        2,
        "Should have 2 arguments: TAL file and bang arg"
    );
    assert_eq!(args[0], "test.tal", "First arg should be TAL file");
    assert_eq!(args[1], "--help", "Second arg should be the !arg1 value");
}

#[test]
fn test_no_bang_arg1_still_works() {
    // Create a temp directory
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let tal_path = temp_dir.path().join("test.tal");
    let mut f = fs::File::create(&tal_path).expect("create .tal file");
    writeln!(f, "; test tal file").unwrap();

    // Simulate protocol parse result without !arg1 bang variable
    let result = ProtocolParseResult {
        url_raw: "uxntal://github.com/example/repo/test.tal".to_string(),
        raw: Default::default(),
        proto_vars: Default::default(),
        query_vars: Default::default(),
        bang_vars: Default::default(),
        url: "https://github.com/example/repo/test.tal".to_string(),
        protocol: "uxntal".to_string(),
        query_string: "".to_string(),
        repo_ref: None,
    };

    // Use the real RomCache implementation from uxn-tal
    let mapper = uxn_tal_defined::emu_uxn::UxnMapper;

    // Build the command
    let cmd = mapper.build_command(
        &result,
        "test.tal",
        std::path::Path::new("uxnemu"),
        Some(temp_dir.path()),
    );

    // Check that the command args only include the ROM (no bang args)
    let args: Vec<String> = cmd
        .get_args()
        .map(|a| a.to_string_lossy().to_string())
        .collect();

    // Should be: [test.tal] (no bang args)
    assert_eq!(args.len(), 1, "Should have 1 argument: just the TAL file");
    assert_eq!(args[0], "test.tal", "First arg should be TAL file");
}
