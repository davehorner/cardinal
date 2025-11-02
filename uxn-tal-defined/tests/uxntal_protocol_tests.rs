#[cfg(test)]
mod tests {
    use uxn_tal_defined::v1::{ProtocolParser, ProtocolVarVar};

    #[test]
    fn timeout_protocol_var_maps_to_arg() {
        use uxn_tal_common::cache::DefaultRomCache;
        use uxn_tal_defined::v1::{get_emulator_mapper, ProtocolParser};
        let rom_cache = DefaultRomCache;
        // t^2 should map to --timeout=2
        let url = "uxntal:t^2//https://example.com/rom.tal";
        let result = ProtocolParser::parse(url);
        let (mapper, _path) =
            get_emulator_mapper(&result, &rom_cache).expect("should get emulator mapper");
        let args = mapper.map_args(&result);
        assert!(args.iter().any(|a| a == "--timeout=2"), "args: {:?}", args);

        // timeout^2 should map to --timeout=2
        let url2 = "uxntal:timeout^2//https://example.com/rom.tal";
        let result2 = ProtocolParser::parse(url2);
        let (mapper2, _path2) =
            get_emulator_mapper(&result2, &rom_cache).expect("should get emulator mapper");
        let args2 = mapper2.map_args(&result2);
        assert!(
            args2.iter().any(|a| a == "--timeout=2"),
            "args: {:?}",
            args2
        );
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

    #[test]
    fn parses_with_and_without_trailing_colon() {
        let url1 = "uxntal:widget:debug://https://wiki.xxiivv.com/etc/catclock.tal.txt";
        let url2 = "uxntal:widget:debug//https://wiki.xxiivv.com/etc/catclock.tal.txt";
        let url3 = "uxntal:widget:debug://file:///tal/clock.tal";
        let url4 = "uxntal:widget:debug//file:///tal/clock.tal";
        for url in [url1, url2, url3, url4] {
            let result = ProtocolParser::parse(url);
            assert_eq!(
                result.proto_vars.get("widget"),
                Some(&ProtocolVarVar::Bool(true))
            );
            assert_eq!(
                result.proto_vars.get("debug"),
                Some(&ProtocolVarVar::Bool(true))
            );
            assert!(
                result.url.starts_with("https://wiki.xxiivv.com/")
                    || result.url.starts_with("file:///tal/clock.tal")
            );
        }
    }

    #[test]
    fn parses_bang_vars() {
        let url = "uxntal:widget:!x=100:!y=200:!w=640:!h=480://file:///tal/clock.tal";
        let result = ProtocolParser::parse(url);
        assert_eq!(
            result.proto_vars.get("widget"),
            Some(&ProtocolVarVar::Bool(true))
        );
        assert_eq!(
            result.proto_vars.get("!x"),
            Some(&ProtocolVarVar::String("100".to_string()))
        );
        assert_eq!(
            result.proto_vars.get("!y"),
            Some(&ProtocolVarVar::String("200".to_string()))
        );
        assert_eq!(
            result.proto_vars.get("!w"),
            Some(&ProtocolVarVar::String("640".to_string()))
        );
        assert_eq!(
            result.proto_vars.get("!h"),
            Some(&ProtocolVarVar::String("480".to_string()))
        );
    }

    #[test]
    fn parses_bool_and_enum_vars() {
        let url =
            "uxntal:widget:ontop^false:emu^^uxn://https://wiki.xxiivv.com/etc/catclock.tal.txt";
        let result = ProtocolParser::parse(url);
        assert_eq!(
            result.proto_vars.get("widget"),
            Some(&ProtocolVarVar::Bool(true))
        );
        assert_eq!(
            result.proto_vars.get("ontop"),
            Some(&ProtocolVarVar::Bool(false))
        );
        assert_eq!(
            result.proto_vars.get("emu"),
            Some(&ProtocolVarVar::Enum("uxn"))
        );
    }

    #[test]
    fn parses_open_url_format() {
        // Test uxntal://open?url=ENC format
        let encoded_url = "https%3A//wiki.xxiivv.com/etc/catclock.tal.txt";
        let url = format!("uxntal://open?url={}", encoded_url);
        let result = ProtocolParser::parse(&url);
        assert_eq!(result.url, "https://wiki.xxiivv.com/etc/catclock.tal.txt");

        // Test uxntal://open/?url=ENC format (with trailing slash)
        let url_with_slash = format!("uxntal://open/?url={}", encoded_url);
        let result_with_slash = ProtocolParser::parse(&url_with_slash);
        assert_eq!(result_with_slash.url, "https://wiki.xxiivv.com/etc/catclock.tal.txt");

        // Test with protocol variables
        let url_with_vars = format!("uxntal:widget:debug://open?url={}", encoded_url);
        let result_with_vars = ProtocolParser::parse(&url_with_vars);
        assert_eq!(result_with_vars.url, "https://wiki.xxiivv.com/etc/catclock.tal.txt");
        assert_eq!(
            result_with_vars.proto_vars.get("widget"),
            Some(&ProtocolVarVar::Bool(true))
        );
        assert_eq!(
            result_with_vars.proto_vars.get("debug"),
            Some(&ProtocolVarVar::Bool(true))
        );
    }
}
