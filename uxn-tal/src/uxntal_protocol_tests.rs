// uxntal_protocol_tests.rs
// Inline tests for the uxntal protocol parser and related logic

#[cfg(test)]
mod tests {
    #[test]
    fn timeout_protocol_var_maps_to_arg() {
        use crate::uxntal_protocol::{ProtocolParser, get_emulator_mapper};
        // t^2 should map to --timeout=2
        let url = "uxntal:t^2//https://example.com/rom.tal";
        let result = ProtocolParser::parse(url);
        let (mapper, _path) = get_emulator_mapper(&result).expect("should get emulator mapper");
        let args = mapper.map_args(&result);
        assert!(args.iter().any(|a| a == "--timeout=2"), "args: {:?}", args);

        // timeout^2 should map to --timeout=2
        let url2 = "uxntal:timeout^2//https://example.com/rom.tal";
        let result2 = ProtocolParser::parse(url2);
        let (mapper2, _path2) = get_emulator_mapper(&result2).expect("should get emulator mapper");
        let args2 = mapper2.map_args(&result2);
        assert!(args2.iter().any(|a| a == "--timeout=2"), "args: {:?}", args2);
    }

    #[test]
    fn normalizes_double_protocol_prefix() {
    let url = "uxntal:widget:debug//https://https://wiki.xxiivv.com/etc/catclock.tal.txt";
    let expected = "https://wiki.xxiivv.com/etc/catclock.tal.txt";
    let result = ProtocolParser::parse(url);
    assert_eq!(result.url, expected);
    }

    #[test]
    fn normalizes_colon_before_url() {
    let url_missing_colon = "uxntal:widget:debug//https://wiki.xxiivv.com/etc/catclock.tal.txt";
    let url_with_colon = "uxntal:widget:debug://https://wiki.xxiivv.com/etc/catclock.tal.txt";
    let result_missing = ProtocolParser::parse(url_missing_colon);
    let result_with = ProtocolParser::parse(url_with_colon);
    // The protocol portion should be normalized (for reconstructing the URL)
    assert_eq!(result_missing.protocol, result_with.protocol);
    // The extracted URL should be the same
    assert_eq!(result_missing.url, result_with.url);
    }
    use super::*;
    use crate::uxntal_protocol::{ProtocolParser, ProtocolVarVar};

    #[test]
    fn parses_with_and_without_trailing_colon() {
        let url1 = "uxntal:widget:debug://https://wiki.xxiivv.com/etc/catclock.tal.txt";
        let url2 = "uxntal:widget:debug//https://wiki.xxiivv.com/etc/catclock.tal.txt";
        let url3 = "uxntal:widget:debug://file:///tal/clock.tal";
        let url4 = "uxntal:widget:debug//file:///tal/clock.tal";
        for url in [url1, url2, url3, url4] {
            let result = ProtocolParser::parse(url);
            assert_eq!(result.proto_vars.get("widget"), Some(&ProtocolVarVar::Bool(true)));
            assert_eq!(result.proto_vars.get("debug"), Some(&ProtocolVarVar::Bool(true)));
            assert!(result.url.starts_with("https://wiki.xxiivv.com/") || result.url.starts_with("file:///tal/clock.tal"));
        }
    }

    #[test]
    fn parses_bang_vars() {
        let url = "uxntal:widget:!x=100:!y=200:!w=640:!h=480://file:///tal/clock.tal";
        let result = ProtocolParser::parse(url);
        assert_eq!(result.proto_vars.get("widget"), Some(&ProtocolVarVar::Bool(true)));
        assert_eq!(result.query_vars.get("x"), Some(&ProtocolVarVar::String("100".to_string())));
        assert_eq!(result.query_vars.get("y"), Some(&ProtocolVarVar::String("200".to_string())));
        assert_eq!(result.query_vars.get("w"), Some(&ProtocolVarVar::String("640".to_string())));
        assert_eq!(result.query_vars.get("h"), Some(&ProtocolVarVar::String("480".to_string())));
    }

    #[test]
    fn parses_bool_and_enum_vars() {
        let url = "uxntal:widget:ontop^false:emu^^uxn://https://wiki.xxiivv.com/etc/catclock.tal.txt";
        let result = ProtocolParser::parse(url);
        assert_eq!(result.proto_vars.get("widget"), Some(&ProtocolVarVar::Bool(true)));
        assert_eq!(result.proto_vars.get("ontop"), Some(&ProtocolVarVar::Bool(false)));
        assert_eq!(result.proto_vars.get("emu"), Some(&ProtocolVarVar::Enum("uxn")));
    }
}
