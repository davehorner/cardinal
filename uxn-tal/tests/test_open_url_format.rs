use uxn_tal::{parse_uxntal_url, ProtocolParser};

#[test]
fn test_open_url_format_enhanced_parser() {
    // Test uxntal://open?url=ENC format with enhanced parser
    let encoded_url = "https%3A//wiki.xxiivv.com/etc/catclock.tal.txt";
    let url = format!("uxntal://open?url={}", encoded_url);
    let result = parse_uxntal_url(&url);
    assert_eq!(result.url, "https://wiki.xxiivv.com/etc/catclock.tal.txt");

    // Test uxntal://open/?url=ENC format (with trailing slash) with enhanced parser
    let url_with_slash = format!("uxntal://open/?url={}", encoded_url);
    let result_with_slash = parse_uxntal_url(&url_with_slash);
    assert_eq!(
        result_with_slash.url,
        "https://wiki.xxiivv.com/etc/catclock.tal.txt"
    );
}

#[test]
fn test_open_url_format_standard_api() {
    // Test uxntal://open?url=ENC format with standard API (now enhanced)
    let encoded_url = "https%3A//wiki.xxiivv.com/etc/catclock.tal.txt";
    let url = format!("uxntal://open?url={}", encoded_url);
    let result = ProtocolParser::parse(&url);
    assert_eq!(result.url, "https://wiki.xxiivv.com/etc/catclock.tal.txt");

    // Test uxntal://open/?url=ENC format (with trailing slash) with standard API
    let url_with_slash = format!("uxntal://open/?url={}", encoded_url);
    let result_with_slash = ProtocolParser::parse(&url_with_slash);
    assert_eq!(
        result_with_slash.url,
        "https://wiki.xxiivv.com/etc/catclock.tal.txt"
    );
}

#[test]
fn test_open_url_with_protocol_vars() {
    // Test with protocol variables
    let encoded_url = "https%3A//wiki.xxiivv.com/etc/catclock.tal.txt";
    let url = format!("uxntal:widget:debug://open?url={}", encoded_url);

    // Test with enhanced parser
    let result_enhanced = parse_uxntal_url(&url);
    assert_eq!(
        result_enhanced.url,
        "https://wiki.xxiivv.com/etc/catclock.tal.txt"
    );
    assert_eq!(
        result_enhanced.proto_vars.get("widget").unwrap().as_bool(),
        Some(true)
    );
    assert_eq!(
        result_enhanced.proto_vars.get("debug").unwrap().as_bool(),
        Some(true)
    );

    // Test with standard API (now enhanced)
    let result_standard = ProtocolParser::parse(&url);
    assert_eq!(
        result_standard.url,
        "https://wiki.xxiivv.com/etc/catclock.tal.txt"
    );
    assert_eq!(
        result_standard.proto_vars.get("widget").unwrap().as_bool(),
        Some(true)
    );
    assert_eq!(
        result_standard.proto_vars.get("debug").unwrap().as_bool(),
        Some(true)
    );
}
