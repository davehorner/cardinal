// Test that verifies protocol handler (uxntal://...) fetches and caches an .orca file
use std::fs;
use uxn_tal::util::RealRomEntryResolver;
use uxn_tal_common::cache::RomEntryResolver;
use uxn_tal_defined::ProtocolParser;

#[test]
fn test_uxntal_protocol_jumper_orca_content() {
    // Protocol URL for the .orca file
    let proto_url =
        "uxntal://https://git.sr.ht/~rabbits/orca-examples/tree/master/item/basics/j.orca";
    // Parse protocol
    let parsed = ProtocolParser::parse(proto_url);
    // Use the real resolver to fetch and cache the file
    let entry_resolver = RealRomEntryResolver;
    let (entry_path, _cache_dir) = entry_resolver
        .resolve_entry_and_cache_dir(&parsed.url)
        .expect("fetch j.orca and includes via protocol");
    // Read the file content
    let content = fs::read_to_string(&entry_path).expect("read j.orca");
    // The expected content (trimmed for whitespace)
    let expected = ".........................................\n.#.JUMPER.#..............................\n.........................................\n.2bO2bO2bO2bO2bO2bO2bO2bO2bO2bO2bO..D....\n............*.......................*....\n...J..J..J..J..J..J..J..J..J..J..J..J....\n............*.......................*....\n...J..J..J..J..J..J..J..J..J..J..J..J....\n............*.......................*....\n...J..J..J..J..J..J..J..J..J..J..J..J....\n............*.......................*....\n...J..J..J..J..J..J..J..J..J..J..J..J....\n............*.......................*....\n...J..J..J..J..J..J..J..J..J..J..J..J....\n............*.......................*....\n.........................................\n.........................................\n";
    let actual_replaced = content.replace("\r\n", "\n");
    let actual = actual_replaced.trim_end();
    let expected_trimmed = expected.trim_end();
    assert_eq!(actual, expected_trimmed);
}
