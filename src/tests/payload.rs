use super::*;

#[test]
fn generic_payload_accepts_json_files_and_guided_fields() {
    let raw = generic_payload(
        GenericPayloadArgs {
            data: Some(r#"{"name":"Acme"}"#.to_string()),
            ..GenericPayloadArgs::default()
        },
        true,
    )
    .expect("raw JSON");
    assert_eq!(raw["name"], "Acme");

    let tempdir = tempfile::tempdir().expect("tempdir");
    let file = tempdir.path().join("payload.json");
    std::fs::write(&file, r#"{"number":"001"}"#).expect("write payload");
    let from_file = generic_payload(
        GenericPayloadArgs {
            data_file: Some(file),
            ..GenericPayloadArgs::default()
        },
        true,
    )
    .expect("file JSON");
    assert_eq!(from_file["number"], "001");

    let guided = generic_payload(
        GenericPayloadArgs {
            fields: vec![
                "client.name=Ada".to_string(),
                "amount=42.5".to_string(),
                "active=true".to_string(),
            ],
            ..GenericPayloadArgs::default()
        },
        true,
    )
    .expect("guided fields");
    assert_eq!(guided["client"]["name"], "Ada");
    assert_eq!(guided["amount"], 42.5);
    assert_eq!(guided["active"], true);

    let forced_string = generic_payload(
        GenericPayloadArgs {
            fields: vec!["number=\"1000\"".to_string()],
            ..GenericPayloadArgs::default()
        },
        true,
    )
    .expect("quoted scalar");
    assert_eq!(forced_string["number"], "1000");
}

#[test]
fn generic_payload_reports_invalid_sources_and_fields() {
    let mixed = generic_payload(
        GenericPayloadArgs {
            data: Some("{}".to_string()),
            fields: vec!["name=Acme".to_string()],
            ..GenericPayloadArgs::default()
        },
        true,
    )
    .expect_err("mixed raw and guided");
    assert!(matches!(mixed, KobanError::InvalidPayload { .. }));

    let missing = generic_payload(GenericPayloadArgs::default(), true).expect_err("missing");
    assert!(missing.to_string().contains("write command requires"));

    let malformed =
        object_from_fields(vec!["missing_separator".to_string()]).expect_err("missing separator");
    assert!(malformed.to_string().contains("key=value"));

    let bad_path =
        object_from_fields(vec!["bad..path=value".to_string()]).expect_err("bad dotted path");
    assert!(bad_path.to_string().contains("must not be empty"));
}

#[test]
fn resource_payload_rejects_raw_json_with_line_items() {
    let mut args = empty_resource_payload_args();
    args.data = Some(r#"{"line_items":[]}"#.to_string());
    args.line_items
        .push("product_key=Consulting,quantity=1,cost=100".to_string());

    let error = resource_payload(args, true).expect_err("raw plus line items");
    assert!(matches!(error, KobanError::InvalidPayload { .. }));
    assert!(error.to_string().contains("--line-item"), "got: {error}");
}
