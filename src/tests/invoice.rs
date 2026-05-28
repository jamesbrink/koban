use super::*;

#[test]
fn upload_file_requires_existing_regular_file() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let existing = tempdir.path().join("document.txt");
    std::fs::write(&existing, b"upload").expect("seed file");
    ensure_upload_file(&existing).expect("regular files are uploadable");

    let missing = tempdir.path().join("missing.txt");
    let error = ensure_upload_file(&missing).expect_err("missing file");
    assert!(matches!(error, KobanError::File { .. }));

    let error = ensure_upload_file(tempdir.path()).expect_err("directory");
    assert!(matches!(error, KobanError::File { .. }));
}

#[test]
fn invoice_payload_reports_missing_and_malformed_sources() {
    let create_error =
        invoice_payload(empty_payload_args(), true, false).expect_err("create requires payload");
    assert!(matches!(create_error, KobanError::InvalidPayload { .. }));
    assert!(
        create_error
            .to_string()
            .contains("create requires JSON input")
    );

    let update_error =
        invoice_payload(empty_payload_args(), false, false).expect_err("update requires payload");
    assert!(
        update_error
            .to_string()
            .contains("update requires JSON input")
    );

    let trigger_only = invoice_payload(empty_payload_args(), false, true).expect("empty body");
    assert_eq!(trigger_only, serde_json::json!({}));

    let mut invalid_json = empty_payload_args();
    invalid_json.data = Some("{not json".to_string());
    let error = invoice_payload(invalid_json, true, false).expect_err("invalid JSON");
    assert!(error.to_string().contains("JSON could not be parsed"));

    let mut missing_file = empty_payload_args();
    missing_file.data_file = Some(PathBuf::from("/tmp/koban-missing-payload.json"));
    let error = invoice_payload(missing_file, true, false).expect_err("missing file");
    assert!(error.to_string().contains("could not read"));
}

#[test]
fn guided_invoice_payload_handles_all_common_fields_and_line_item_scalars() {
    let mut args = empty_payload_args();
    args.client_id = Some("client_1".to_string());
    args.date = Some("2026-05-28".to_string());
    args.due_date = Some("2026-06-28".to_string());
    args.number = Some("INV-1".to_string());
    args.po_number = Some("PO-1".to_string());
    args.public_notes = Some("public".to_string());
    args.private_notes = Some("private".to_string());
    args.terms = Some("Net 30".to_string());
    args.footer = Some("footer".to_string());
    args.project_id = Some("project_1".to_string());
    args.line_items = vec![
        "product_key=Consulting,quantity=1,cost=99.5,is_amount_discount=false,optional=null"
            .to_string(),
    ];

    let payload = invoice_payload(args, true, false).expect("payload");
    assert_eq!(payload["client_id"], "client_1");
    assert_eq!(payload["due_date"], "2026-06-28");
    assert_eq!(payload["line_items"][0]["quantity"], 1);
    assert_eq!(payload["line_items"][0]["cost"], 99.5);
    assert_eq!(payload["line_items"][0]["is_amount_discount"], false);
    assert!(payload["line_items"][0]["optional"].is_null());
}

#[test]
fn line_item_parser_reports_bad_parts() {
    let error = parse_line_item("not-a-pair").expect_err("missing equals");
    assert!(error.to_string().contains("must use key=value"));

    let error = parse_line_item("=value").expect_err("empty key");
    assert!(error.to_string().contains("empty key"));
}

#[test]
fn invoice_trigger_helpers_build_query_and_confirm_risky_actions() {
    let triggers = InvoiceTriggerArgs {
        send_email: true,
        mark_sent: true,
        paid: true,
        amount_paid: Some("12.50".to_string()),
        cancel: true,
        save_default_footer: true,
        save_default_terms: true,
        retry_e_send: true,
    };
    assert!(triggers.has_any());
    assert!(triggers.requires_confirmation());

    let mut query = Vec::new();
    push_invoice_triggers(&mut query, &triggers);
    assert!(query.contains(&("send_email".to_string(), "true".to_string())));
    assert!(query.contains(&("amount_paid".to_string(), "12.50".to_string())));
    assert!(query.contains(&("retry_e_send".to_string(), "true".to_string())));

    let safety = WriteSafetyArgs {
        dry_run: false,
        yes: false,
    };
    let error = require_confirmation("invoice action", &safety).expect_err("confirmation");
    assert!(matches!(error, KobanError::ConfirmationRequired { .. }));

    require_confirmation(
        "invoice action",
        &WriteSafetyArgs {
            dry_run: true,
            yes: false,
        },
    )
    .expect("dry run allowed");

    let invalid = InvoiceTriggerArgs {
        amount_paid: Some("12.50".to_string()),
        ..empty_trigger_args()
    };
    let error = validate_invoice_triggers(&invalid).expect_err("amount requires paid");
    assert!(error.to_string().contains("--amount-paid requires --paid"));
}

#[test]
fn dry_run_output_includes_body_query_and_files() {
    let files = vec![PathBuf::from("/tmp/document.pdf")];
    let output = render_dry_run(
        "PUT",
        "api/v1/invoices/invoice_1/upload",
        &[("include".to_string(), "documents".to_string())],
        Some(&serde_json::json!({"client_id": "client_1"})),
        Some(&files),
    )
    .expect("dry run");
    assert!(output.contains("\"method\": \"PUT\""), "got: {output}");
    assert!(
        output.contains("\"client_id\": \"client_1\""),
        "got: {output}"
    );
    assert!(output.contains("/tmp/document.pdf"), "got: {output}");
}

#[test]
fn path_segment_validation_rejects_route_changing_actions() {
    validate_path_segment("invoice action", "mark_paid").expect("known safe action");
    validate_path_segment("invoice action", "clone-to-quote").expect("hyphens allowed");

    for bad in [
        "",
        ".",
        "..",
        "../clients",
        "mark/paid",
        "email?include=client",
    ] {
        let error = validate_path_segment("invoice action", bad).expect_err("unsafe action");
        assert!(
            error.to_string().contains("safe single path segment"),
            "got: {error}"
        );
    }
}
