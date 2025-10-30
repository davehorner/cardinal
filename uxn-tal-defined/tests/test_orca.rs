use uxn_tal_defined::v1::ProtocolParser;

#[test]
fn orca_var_set_for_orca_url() {
    let url = "uxntal://https://patchstorage.com/scales.orca";
    let parsed = ProtocolParser::parse(url);
    let orca = parsed.get("orca");
    assert!(orca.is_some(), "orca variable should be present");
    assert_eq!(
        orca.unwrap().as_bool(),
        Some(true),
        "orca variable should be true"
    );
}

#[test]
fn orca_var_set_for_explicit_orca_proto_var() {
    let url = "uxntal:orca://https://patchstorage.com/scales.orca";
    let parsed = ProtocolParser::parse(url);
    let orca = parsed.get("orca");
    assert!(orca.is_some(), "orca variable should be present");
    assert_eq!(
        orca.unwrap().as_bool(),
        Some(true),
        "orca variable should be true"
    );
}

#[test]
fn orca_var_not_set_for_non_orca_url() {
    let url = "uxntal://https://patchstorage.com/scales.tal";
    let parsed = ProtocolParser::parse(url);
    let orca = parsed.get("orca");
    assert!(
        orca.is_none() || orca.unwrap().as_bool() == Some(false),
        "orca variable should not be set or should be false"
    );
}

#[test]
fn orca_var_set_for_explicit_proto_var_only() {
    // .orca is NOT in the URL, but orca is set via protocol
    let url = "uxntal:orca://https://patchstorage.com/scales.tal";
    let parsed = ProtocolParser::parse(url);
    let orca = parsed.get("orca");
    assert!(orca.is_some(), "orca variable should be present");
    assert_eq!(
        orca.unwrap().as_bool(),
        Some(true),
        "orca variable should be true"
    );
}
