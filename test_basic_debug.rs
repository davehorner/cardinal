use uxn_tal::util::RealRomCache;
use uxn_tal_defined::v1::{ProtocolParseResult, ProtocolVarVar};
use uxn_tal_defined::EmulatorLauncher;
use std::collections::HashMap;

fn main() {
    // Simulate the debug protocol parse result
    let mut result = ProtocolParseResult {
        url_raw: "uxntal:debug://https://git.sr.ht/~rabbits/tiny-basic/tree/main/item/examples/sierpinski.bas".to_string(),
        raw: {
            let mut map = HashMap::new();
            map.insert("debug".to_string(), "true".to_string());
            map
        },
        proto_vars: {
            let mut map = HashMap::new();
            map.insert("debug".to_string(), ProtocolVarVar::Bool(true));
            map.insert("basic".to_string(), ProtocolVarVar::Bool(true));
            map
        },
        query_vars: Default::default(),
        bang_vars: Default::default(),
        url: "https://git.sr.ht/~rabbits/tiny-basic/tree/main/item/examples/sierpinski.bas".to_string(),
        protocol: "uxntal:debug://".to_string(),
        query_string: "".to_string(),
        repo_ref: None,
    };

    let rom_cache = RealRomCache;
    let mapper = uxn_tal_defined::emu_uxn::UxnMapper {
        rom_cache: &rom_cache,
    };

    // Build the command
    let cmd = mapper.build_command(
        &result,
        "sierpinski.bas",
        std::path::Path::new("cardinal-gui.exe"),
        None,
    );

    // Print the command args
    let args: Vec<String> = cmd
        .get_args()
        .map(|a| a.to_string_lossy().to_string())
        .collect();
    
    println!("Command: {:?}", cmd.get_program());
    println!("Args: {:?}", args);
}