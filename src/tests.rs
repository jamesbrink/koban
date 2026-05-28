use std::path::PathBuf;

use httpmock::{
    Method::{GET, POST, PUT},
    MockServer,
};
use reqwest::StatusCode;
use serde_json::Value;

use crate::{
    api::{ApiClient, api_error},
    cli::*,
    commands::*,
    invoice::*,
    render::*,
    *,
};

#[test]
fn config_defaults_to_invoice_ninja_base_url() {
    let config = Config::from_values(DEFAULT_BASE_URL, "token").expect("config");
    assert_eq!(config.base_url.as_str(), "https://invoicing.co/");
}

#[test]
fn completion_shell_display_uses_documented_names() {
    assert_eq!(CompletionShell::Bash.to_string(), "bash");
    assert_eq!(CompletionShell::Elvish.to_string(), "elvish");
    assert_eq!(CompletionShell::Fish.to_string(), "fish");
    assert_eq!(CompletionShell::Nushell.to_string(), "nushell");
    assert_eq!(CompletionShell::PowerShell.to_string(), "powershell");
    assert_eq!(CompletionShell::Zsh.to_string(), "zsh");
}

#[test]
fn config_preserves_self_hosted_path_prefix_without_trailing_slash() {
    let config = Config::from_values("https://example.com/invoiceninja", "token").expect("config");
    let client = ApiClient::new(config);
    let url = client.endpoint("api/v1/clients", &[]).expect("url");
    assert_eq!(
        url.as_str(),
        "https://example.com/invoiceninja/api/v1/clients"
    );
}

#[test]
fn config_preserves_self_hosted_path_prefix_with_trailing_slash() {
    let config = Config::from_values("https://example.com/invoiceninja/", "token").expect("config");
    let client = ApiClient::new(config);
    let url = client.endpoint("api/v1/clients", &[]).expect("url");
    assert_eq!(
        url.as_str(),
        "https://example.com/invoiceninja/api/v1/clients"
    );
}

#[test]
fn config_rejects_empty_token() {
    let error = Config::from_values(DEFAULT_BASE_URL, "").expect_err("missing token");
    assert!(matches!(error, KobanError::MissingToken));
}

#[test]
fn config_reports_invalid_base_url() {
    let error = Config::from_values("not a url", "token").expect_err("invalid URL");
    assert!(matches!(error, KobanError::InvalidBaseUrl { .. }));
}

#[test]
fn config_rejects_non_local_http() {
    let error = Config::from_values("http://example.com", "token").expect_err("insecure URL");
    assert!(matches!(error, KobanError::InsecureBaseUrl { .. }));
}

#[test]
fn endpoint_builds_pagination_and_include_query() {
    let client =
        ApiClient::new(Config::from_values("http://localhost:1234", "token").expect("config"));
    let url = client
        .endpoint(
            "api/v1/clients",
            &[
                ("page".to_string(), "2".to_string()),
                ("per_page".to_string(), "15".to_string()),
                ("include".to_string(), "activities,ledger".to_string()),
            ],
        )
        .expect("url");
    assert_eq!(
        url.as_str(),
        "http://localhost:1234/api/v1/clients?page=2&per_page=15&include=activities%2Cledger"
    );
}

#[test]
fn endpoint_accepts_leading_slash_paths() {
    let client =
        ApiClient::new(Config::from_values("http://localhost:1234", "token").expect("config"));
    let url = client.endpoint("/api/v1/statics", &[]).expect("url");
    assert_eq!(url.as_str(), "http://localhost:1234/api/v1/statics");
}

#[test]
fn redacts_token_from_text() {
    assert_eq!(
        redact("bad token secret-token failed", "secret-token"),
        "bad token [REDACTED] failed"
    );
}

#[test]
fn redaction_is_noop_without_token() {
    assert_eq!(redact("nothing to hide", ""), "nothing to hide");
}

#[test]
fn json_output_preserves_api_shape() {
    let value = serde_json::json!({"data": [{"id": "abc", "display_name": "Ada"}]});
    let output = render_value(OutputFormat::Json, Some(Resource::Clients), &value).expect("json");
    assert!(output.contains("\"display_name\": \"Ada\""));
}

#[test]
fn table_output_renders_client_fields() {
    let value = serde_json::json!({
        "data": [{
            "id": "abc",
            "display_name": "Ada",
            "balance": 12.5,
            "created_at": 1744305114
        }]
    });
    let output = render_value(OutputFormat::Table, Some(Resource::Clients), &value).expect("table");
    assert!(output.contains("Ada"), "got: {output}");
    assert!(output.contains("12.5"), "got: {output}");
    assert!(output.contains("2025-04-10"), "got: {output}");
}

#[test]
fn table_output_reports_empty_resource_lists() {
    let value = serde_json::json!({"data": []});
    let output = render_value(OutputFormat::Table, Some(Resource::Clients), &value).expect("table");
    assert_eq!(output, "No clients found.");
}

#[test]
fn table_output_renders_statics_summary() {
    let value = serde_json::json!({
        "bulk_updates": {"archive": "Archive", "delete": "Delete"},
        "countries": [{"id": "840", "name": "United States"}],
        "currencies": [{"id": "1", "name": "US Dollar"}],
        "default_size": "A4",
        "invoice_number": 1,
        "enabled": true,
        "nothing": null
    });
    let output = render_value(OutputFormat::Table, None, &value).expect("table");
    assert!(output.contains("bulk_updates"), "got: {output}");
    assert!(output.contains("countries"), "got: {output}");
    assert!(output.contains("currencies"), "got: {output}");
    assert!(output.contains("array"), "got: {output}");
    assert!(output.contains("object"), "got: {output}");
}

#[test]
fn table_output_reports_empty_or_invalid_statics() {
    let empty = render_value(OutputFormat::Table, None, &serde_json::json!({})).expect("table");
    assert_eq!(empty, "No statics found.");

    let scalar = render_value(OutputFormat::Table, None, &serde_json::json!(true)).expect("table");
    assert_eq!(scalar, "No statics found.");
}

#[test]
fn table_output_renders_invoices_and_payments() {
    let invoice = serde_json::json!({
        "data": [{
            "id": "invoice_1",
            "number": "INV-1",
            "client": {"display_name": "Grace Hopper"},
            "status_id": 4,
            "amount": 100,
            "balance": 0,
            "due_date": "2026-06-01"
        }]
    });
    let invoice_output =
        render_value(OutputFormat::Table, Some(Resource::Invoices), &invoice).expect("table");
    assert!(
        invoice_output.contains("Grace Hopper"),
        "got: {invoice_output}"
    );
    assert!(invoice_output.contains("paid"), "got: {invoice_output}");

    let payment = serde_json::json!({
        "data": [{
            "id": "payment_1",
            "number": "PAY-1",
            "client_id": "client_1",
            "status": "completed",
            "amount": 50,
            "date": "2026-06-02"
        }]
    });
    let payment_output =
        render_value(OutputFormat::Table, Some(Resource::Payments), &payment).expect("table");
    assert!(payment_output.contains("PAY-1"), "got: {payment_output}");
    assert!(
        payment_output.contains("completed"),
        "got: {payment_output}"
    );
}

#[test]
fn table_output_renders_new_read_only_resources() {
    let cases = [
        (
            Resource::Quotes,
            serde_json::json!({
                "data": [{
                    "id": "quote_1",
                    "number": "Q-1",
                    "client": {"name": "Quote Client"},
                    "status_id": 3,
                    "amount": 120,
                    "due_date": 1772323200_i64
                }]
            }),
            ["Quote Client", "approved", "2026-03-01"],
        ),
        (
            Resource::Credits,
            serde_json::json!({
                "data": [{
                    "id": "credit_1",
                    "number": "C-1",
                    "client_id": "client_1",
                    "status": "open",
                    "amount": 50,
                    "balance": 10,
                    "date": "2026-03-02"
                }]
            }),
            ["client_1", "open", "2026-03-02"],
        ),
        (
            Resource::Vendors,
            serde_json::json!({
                "data": [{
                    "id": "vendor_1",
                    "vendor_number": "V-1",
                    "contacts": [{"email": "vendor@example.test"}],
                    "balance": 9,
                    "created_at": 1772496000_i64
                }]
            }),
            ["vendor@example.test", "9", "2026-03-03"],
        ),
        (
            Resource::Expenses,
            serde_json::json!({
                "data": [{
                    "id": "expense_1",
                    "transaction_id": "TX-1",
                    "category": {"name": "Travel"},
                    "status_id": "logged",
                    "amount": 33,
                    "date": "2026-03-04"
                }]
            }),
            ["Travel", "logged", "2026-03-04"],
        ),
        (
            Resource::Projects,
            serde_json::json!({
                "data": [{
                    "id": "project_1",
                    "number": "P-1",
                    "name": "Build Koban",
                    "status": "active",
                    "budgeted_hours": 12,
                    "due_date": "2026-03-05"
                }]
            }),
            ["Build Koban", "active", "2026-03-05"],
        ),
        (
            Resource::Tasks,
            serde_json::json!({
                "data": [{
                    "id": "task_1",
                    "number": "T-1",
                    "project": {"name": "Build Koban"},
                    "status": "running",
                    "time": 45,
                    "date": "2026-03-06"
                }]
            }),
            ["Build Koban", "running", "2026-03-06"],
        ),
    ];

    for (resource, value, expected_parts) in cases {
        let output = render_value(OutputFormat::Table, Some(resource), &value).expect("table");
        for expected in expected_parts {
            assert!(output.contains(expected), "missing {expected}: {output}");
        }
    }
}

#[test]
fn quote_status_maps_known_statuses_and_fallbacks() {
    let cases = [(1, "draft"), (2, "sent"), (3, "approved"), (4, "converted")];

    for (status, expected) in cases {
        assert_eq!(
            quote_status(&serde_json::json!({"status_id": status})),
            expected
        );
    }

    assert_eq!(
        quote_status(&serde_json::json!({"status": "custom quote"})),
        "custom quote"
    );
}

#[test]
fn invoice_status_maps_all_known_statuses_and_fallbacks() {
    let cases = [
        (1, "draft"),
        (2, "sent"),
        (3, "partially paid"),
        (4, "paid"),
        (5, "cancelled"),
        (6, "reversed"),
        (-1, "overdue"),
        (-2, "unpaid"),
    ];

    for (status, expected) in cases {
        assert_eq!(
            invoice_status(&serde_json::json!({"status_id": status})),
            expected
        );
    }

    assert_eq!(
        invoice_status(&serde_json::json!({"status": "custom"})),
        "custom"
    );
}

#[test]
fn field_handles_nested_arrays_and_value_kinds() {
    let value = serde_json::json!({
        "contacts": [{"email": "ada@example.test"}],
        "empty": "",
        "nullish": null,
        "flag": true,
        "items": [1, 2],
        "object": {"a": 1}
    });

    assert_eq!(
        field(&value, &["contacts", "0", "email"]),
        "ada@example.test"
    );
    assert_eq!(field(&value, &["contacts", "1", "email"]), "-");
    assert_eq!(field(&value, &["empty"]), "-");
    assert_eq!(field(&value, &["nullish"]), "-");
    assert_eq!(field(&value, &["flag"]), "true");
    assert_eq!(field(&value, &["items"]), "2 items");
    assert_eq!(field(&value, &["object"]), "1 fields");
    assert_eq!(field(&value, &["missing"]), "-");

    assert_eq!(value_kind(&serde_json::json!("x")), "string");
    assert_eq!(value_kind(&serde_json::json!(1)), "number");
    assert_eq!(value_kind(&serde_json::json!(false)), "boolean");
    assert_eq!(value_kind(&Value::Null), "null");
    assert_eq!(value_len(&serde_json::json!("x")), 1);
}

#[test]
fn date_field_formats_unix_timestamps_and_preserves_date_strings() {
    let value = serde_json::json!({
        "created_at": 1744305114,
        "updated_at": "1730754263000",
        "legacy_millis": 946684800000_i64,
        "date": "2026-05-16"
    });

    assert_eq!(date_field(&value, &["created_at"]), "2025-04-10");
    assert_eq!(date_field(&value, &["updated_at"]), "2024-11-04");
    assert_eq!(date_field(&value, &["legacy_millis"]), "2000-01-01");
    assert_eq!(date_field(&value, &["date"]), "2026-05-16");
    assert_eq!(date_field(&value, &["missing"]), "-");
    assert_eq!(
        date_field(&serde_json::json!({"date": true}), &["date"]),
        "true"
    );
}

#[test]
fn response_rows_accepts_common_api_shapes() {
    assert_eq!(
        response_rows(&serde_json::json!({"data": {"id": "one"}})).len(),
        1
    );
    assert_eq!(response_rows(&serde_json::json!([{"id": "one"}])).len(), 1);
    assert_eq!(response_rows(&serde_json::json!({"id": "one"})).len(), 1);
    assert_eq!(response_rows(&serde_json::json!(null)).len(), 0);
}

#[test]
fn apply_limit_truncates_array_responses() {
    let output = apply_limit_to_response(
        serde_json::json!([
            {"id": "one"},
            {"id": "two"}
        ]),
        Some(1),
    );
    assert_eq!(response_rows(&output).len(), 1);
}

#[test]
fn api_errors_redact_tokens() {
    let error = api_error(
        StatusCode::UNAUTHORIZED,
        "/api/v1/clients".to_string(),
        r#"{"message":"token secret-token is bad"}"#.to_string(),
        "secret-token",
    );
    let message = error.to_string();
    assert!(message.contains("[REDACTED]"), "got: {message}");
    assert!(!message.contains("secret-token"), "got: {message}");
}

#[test]
fn api_errors_extract_arrays_objects_and_plain_text() {
    let array_error = api_error(
        StatusCode::UNPROCESSABLE_ENTITY,
        "/api/v1/clients".to_string(),
        r#"{"errors":["name is required",{"email":"invalid"}]}"#.to_string(),
        "token",
    );
    assert!(array_error.to_string().contains("name is required"));

    let object_error = api_error(
        StatusCode::BAD_REQUEST,
        "/api/v1/clients".to_string(),
        r#"{"error":{"message":"bad"}}"#.to_string(),
        "token",
    );
    assert!(object_error.to_string().contains("message"));

    let text_error = api_error(
        StatusCode::INTERNAL_SERVER_ERROR,
        "/api/v1/clients".to_string(),
        "plain failure".to_string(),
        "token",
    );
    assert!(text_error.to_string().contains("plain failure"));

    let numeric_message = api_error(
        StatusCode::BAD_REQUEST,
        "/api/v1/clients".to_string(),
        r#"{"message":123}"#.to_string(),
        "token",
    );
    assert!(numeric_message.to_string().contains("\"message\":123"));
}

#[tokio::test]
async fn get_json_reports_transport_errors() {
    let client =
        ApiClient::new(Config::from_values("http://127.0.0.1:9", "secret-token").expect("config"));
    let error = client
        .get_json("api/v1/statics", &[])
        .await
        .expect_err("transport failure");
    let message = error.to_string();
    assert!(matches!(error, KobanError::Transport { .. }));
    assert!(!message.contains("secret-token"), "got: {message}");
}

#[tokio::test]
async fn json_write_methods_report_decode_api_and_transport_errors() {
    let server = MockServer::start();
    let invalid_json = server.mock(|when, then| {
        when.method(POST).path("/api/v1/invoices");
        then.status(200).body("not json");
    });
    let api_failure = server.mock(|when, then| {
        when.method(PUT).path("/api/v1/invoices/invoice_1");
        then.status(422).body(r#"{"message":"bad invoice"}"#);
    });

    let client = ApiClient::new(Config::from_values(server.base_url(), "token").expect("config"));
    let decode = client
        .post_json("api/v1/invoices", &[], &serde_json::json!({}))
        .await
        .expect_err("invalid JSON");
    assert!(matches!(decode, KobanError::Decode { .. }));
    invalid_json.assert();

    let api = client
        .put_json("api/v1/invoices/invoice_1", &[], &serde_json::json!({}))
        .await
        .expect_err("API failure");
    assert!(matches!(api, KobanError::Api { .. }));
    assert!(api.to_string().contains("bad invoice"));
    api_failure.assert();

    let offline =
        ApiClient::new(Config::from_values("http://127.0.0.1:9", "secret-token").expect("config"));
    let transport = offline
        .delete_json("api/v1/invoices/invoice_1", &[])
        .await
        .expect_err("transport failure");
    assert!(matches!(transport, KobanError::Transport { .. }));
    assert!(!transport.to_string().contains("secret-token"));
}

#[tokio::test]
async fn json_write_methods_redact_transport_errors() {
    let client =
        ApiClient::new(Config::from_values("http://127.0.0.1:9", "secret-token").expect("config"));

    let post = client
        .post_json("api/v1/invoices", &[], &serde_json::json!({}))
        .await
        .expect_err("post transport");
    let put = client
        .put_json("api/v1/invoices/invoice_1", &[], &serde_json::json!({}))
        .await
        .expect_err("put transport");

    for error in [post, put] {
        assert!(matches!(error, KobanError::Transport { .. }));
        assert!(!error.to_string().contains("secret-token"));
    }
}

#[tokio::test]
async fn multipart_upload_reports_api_failure_after_reading_files() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let upload = tempdir.path().join("upload.txt");
    std::fs::write(&upload, b"document").expect("upload file");

    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(PUT).path("/api/v1/invoices/invoice_1/upload");
        then.status(400).body(r#"{"message":"upload rejected"}"#);
    });
    let client = ApiClient::new(Config::from_values(server.base_url(), "token").expect("config"));
    let error = client
        .put_multipart("api/v1/invoices/invoice_1/upload", &[], &[upload])
        .await
        .expect_err("upload failure");
    assert!(matches!(error, KobanError::Api { .. }));
    assert!(error.to_string().contains("upload rejected"));
    mock.assert();
}

#[tokio::test]
async fn multipart_upload_redacts_transport_errors_after_reading_files() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let upload = tempdir.path().join("upload.txt");
    std::fs::write(&upload, b"document").expect("upload file");
    let client =
        ApiClient::new(Config::from_values("http://127.0.0.1:9", "secret-token").expect("config"));
    let error = client
        .put_multipart("api/v1/invoices/invoice_1/upload", &[], &[upload])
        .await
        .expect_err("transport failure");
    assert!(matches!(error, KobanError::Transport { .. }));
    assert!(!error.to_string().contains("secret-token"));
}

#[tokio::test]
async fn get_bytes_reports_api_and_transport_errors() {
    let server = MockServer::start();
    let failing_download = server.mock(|when, then| {
        when.method(GET).path("/api/v1/invoice/invitation/download");
        then.status(404)
            .body(r#"{"message":"missing secret-token"}"#);
    });

    let client =
        ApiClient::new(Config::from_values(server.base_url(), "secret-token").expect("config"));
    let error = client
        .get_bytes("api/v1/invoice/invitation/download", &[])
        .await
        .expect_err("api failure");
    let message = error.to_string();
    assert!(matches!(error, KobanError::Api { .. }));
    assert!(message.contains("[REDACTED]"), "got: {message}");
    assert!(!message.contains("secret-token"), "got: {message}");
    failing_download.assert();

    let client =
        ApiClient::new(Config::from_values("http://127.0.0.1:9", "secret-token").expect("config"));
    let error = client
        .get_bytes("api/v1/invoice/invitation/download", &[])
        .await
        .expect_err("transport failure");
    assert!(matches!(error, KobanError::Transport { .. }));
}

#[tokio::test]
async fn get_bytes_returns_success_bytes() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(GET).path("/api/v1/invoice/invitation/download");
        then.status(200).body("pdf bytes");
    });

    let client = ApiClient::new(Config::from_values(server.base_url(), "token").expect("config"));
    let bytes = client
        .get_bytes("api/v1/invoice/invitation/download", &[])
        .await
        .expect("bytes");
    assert_eq!(bytes, b"pdf bytes");
    mock.assert();
}

#[test]
fn download_path_requires_existing_parent_and_force_for_overwrite() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let existing = tempdir.path().join("invoice.pdf");
    std::fs::write(&existing, b"old").expect("seed file");

    let error = ensure_download_path(&existing, false).expect_err("refuse overwrite");
    assert!(matches!(error, KobanError::File { .. }));

    ensure_download_path(&existing, true).expect("force overwrite allowed");

    let missing_parent = tempdir.path().join("missing").join("invoice.pdf");
    let error = ensure_download_path(&missing_parent, true).expect_err("missing parent");
    assert!(matches!(error, KobanError::File { .. }));
}

#[test]
fn write_download_file_writes_bytes_and_reports_write_errors() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let output = tempdir.path().join("invoice.pdf");
    write_download_file(&output, b"pdf".to_vec(), false).expect("write PDF");
    assert_eq!(std::fs::read(&output).expect("read PDF"), b"pdf");

    let error = write_download_file(tempdir.path(), b"pdf".to_vec(), true).expect_err("directory");
    assert!(matches!(error, KobanError::File { .. }));
}

#[test]
fn push_filters_rejects_empty_keys() {
    let error = push_filters(&mut Vec::new(), vec![" =value".to_string()]).expect_err("empty key");
    assert!(matches!(error, KobanError::InvalidFilter { .. }));
}

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

fn empty_payload_args() -> InvoicePayloadArgs {
    InvoicePayloadArgs {
        data: None,
        data_file: None,
        stdin: false,
        client_id: None,
        date: None,
        due_date: None,
        number: None,
        po_number: None,
        public_notes: None,
        private_notes: None,
        terms: None,
        footer: None,
        project_id: None,
        line_items: Vec::new(),
    }
}

fn empty_trigger_args() -> InvoiceTriggerArgs {
    InvoiceTriggerArgs {
        send_email: false,
        mark_sent: false,
        paid: false,
        amount_paid: None,
        cancel: false,
        save_default_footer: false,
        save_default_terms: false,
        retry_e_send: false,
    }
}

#[tokio::test]
async fn execute_handles_non_network_commands_without_configured_endpoint() {
    let config = Config::from_values("http://localhost:1234", "token").expect("config");
    let output = execute_with_config(
        Cli {
            output: OutputFormat::Table,
            command: Some(Commands::Completions {
                shell: CompletionShell::Bash,
            }),
        },
        config.clone(),
    )
    .await
    .expect("execute completions");
    assert!(output.is_empty());

    let output = execute_with_config(
        Cli {
            output: OutputFormat::Table,
            command: None,
        },
        config.clone(),
    )
    .await
    .expect("execute none");
    assert!(output.is_empty());

    let output = execute(Cli {
        output: OutputFormat::Table,
        command: Some(Commands::Update {
            check: true,
            force: false,
            tag: None,
            nightly: true,
        }),
    })
    .await
    .expect("execute nightly check");
    assert!(output.contains("Nightly build available"), "got: {output}");

    let output = execute_with_config(
        Cli {
            output: OutputFormat::Table,
            command: Some(Commands::Update {
                check: true,
                force: false,
                tag: None,
                nightly: true,
            }),
        },
        config,
    )
    .await
    .expect("execute nightly check with config");
    assert!(output.contains("Nightly build available"), "got: {output}");
}

#[tokio::test]
async fn execute_invoice_dry_runs_cover_write_commands_without_network() {
    let config = Config::from_values("http://localhost:1234", "token").expect("config");

    let update = execute_with_config(
        Cli {
            output: OutputFormat::Json,
            command: Some(Commands::Invoices(Box::new(InvoiceCommand::Update(
                UpdateInvoiceArgs {
                    id: "invoice_1".to_string(),
                    payload: {
                        let mut args = empty_payload_args();
                        args.public_notes = Some("updated".to_string());
                        args
                    },
                    triggers: InvoiceTriggerArgs {
                        mark_sent: true,
                        ..empty_trigger_args()
                    },
                    safety: WriteSafetyArgs {
                        dry_run: true,
                        yes: false,
                    },
                    include: vec!["client".to_string()],
                },
            )))),
        },
        config.clone(),
    )
    .await
    .expect("update dry run");
    assert!(update.contains("\"method\": \"PUT\""), "got: {update}");
    assert!(update.contains("mark_sent"), "got: {update}");

    let bulk = execute_with_config(
        Cli {
            output: OutputFormat::Json,
            command: Some(Commands::Invoices(Box::new(InvoiceCommand::Bulk(
                BulkArgs {
                    action: "archive".to_string(),
                    ids: vec!["one".to_string(), "two".to_string()],
                    email_type: Some("invoice".to_string()),
                    safety: WriteSafetyArgs {
                        dry_run: true,
                        yes: false,
                    },
                    include: Vec::new(),
                },
            )))),
        },
        config.clone(),
    )
    .await
    .expect("bulk dry run");
    assert!(bulk.contains("\"action\": \"archive\""), "got: {bulk}");

    let tempdir = tempfile::tempdir().expect("tempdir");
    let upload = tempdir.path().join("upload.txt");
    std::fs::write(&upload, b"document").expect("upload fixture");
    let upload_output = execute_with_config(
        Cli {
            output: OutputFormat::Json,
            command: Some(Commands::Invoices(Box::new(InvoiceCommand::Upload(
                UploadArgs {
                    id: "invoice_1".to_string(),
                    files: vec![upload],
                    safety: WriteSafetyArgs {
                        dry_run: true,
                        yes: false,
                    },
                    include: Vec::new(),
                },
            )))),
        },
        config.clone(),
    )
    .await
    .expect("upload dry run");
    assert!(
        upload_output.contains("\"method\": \"PUT\""),
        "got: {upload_output}"
    );

    let action = execute_with_config(
        Cli {
            output: OutputFormat::Json,
            command: Some(Commands::Invoices(Box::new(InvoiceCommand::Action(
                InvoiceActionArgs {
                    id: "invoice_1".to_string(),
                    action: "mark_paid".to_string(),
                    safety: WriteSafetyArgs {
                        dry_run: true,
                        yes: false,
                    },
                    include: Vec::new(),
                },
            )))),
        },
        config,
    )
    .await
    .expect("action dry run");
    assert!(
        action.contains("api/v1/invoices/invoice_1/mark_paid"),
        "got: {action}"
    );
}

#[tokio::test]
async fn multipart_upload_reports_missing_file_before_network() {
    let client = ApiClient::new(
        Config::from_values("http://localhost:1234", "secret-token").expect("config"),
    );
    let error = client
        .put_multipart(
            "api/v1/invoices/invoice_1/upload",
            &[],
            &[PathBuf::from("/tmp/koban-missing-upload.txt")],
        )
        .await
        .expect_err("missing upload");
    assert!(matches!(error, KobanError::File { .. }));
}

#[tokio::test]
async fn execute_resource_commands_against_mock_api() {
    let server = MockServer::start();
    let config = Config::from_values(server.base_url(), "token").expect("config");

    let clients = server.mock(|when, then| {
        when.method(GET)
            .path("/api/v1/clients")
            .query_param("page", "1")
            .query_param("per_page", "20");
        then.status(200).json_body(serde_json::json!({
            "data": [{"id": "client_1", "display_name": "Ada"}]
        }));
    });
    let invoices = server.mock(|when, then| {
        when.method(GET).path("/api/v1/invoices/invoice_1");
        then.status(200).json_body(serde_json::json!({
            "data": {"id": "invoice_1", "number": "INV-1", "status_id": 2}
        }));
    });
    let invoice_list = server.mock(|when, then| {
        when.method(GET)
            .path("/api/v1/invoices")
            .query_param("page", "1")
            .query_param("per_page", "10");
        then.status(200).json_body(serde_json::json!({
            "data": [{"id": "invoice_2", "number": "INV-2", "status_id": 1}]
        }));
    });
    let invoice_template = server.mock(|when, then| {
        when.method(GET).path("/api/v1/invoices/create");
        then.status(200).json_body(serde_json::json!({
            "data": {"id": "", "number": "", "line_items": []}
        }));
    });
    let payments = server.mock(|when, then| {
        when.method(GET)
            .path("/api/v1/payments")
            .query_param("include", "client");
        then.status(200).json_body(serde_json::json!({
            "data": [{"id": "payment_1", "number": "PAY-1"}]
        }));
    });
    let client_template = server.mock(|when, then| {
        when.method(GET)
            .path("/api/v1/clients/create")
            .query_param("include", "contacts");
        then.status(200).json_body(serde_json::json!({
            "data": {"id": "", "display_name": "", "contacts": []}
        }));
    });
    let invoice_edit_template = server.mock(|when, then| {
        when.method(GET)
            .path("/api/v1/invoices/invoice_1/edit")
            .query_param("include", "client");
        then.status(200).json_body(serde_json::json!({
            "data": {"id": "invoice_1", "number": "INV-1", "client": {"display_name": "Ada"}}
        }));
    });

    let output = execute_with_config(
        Cli {
            output: OutputFormat::Table,
            command: Some(Commands::Clients(ResourceCommand::List(ListArgs {
                page: 1,
                per_page: 20,
                include: Vec::new(),
                filters: Vec::new(),
                sort: None,
                all: false,
                limit: None,
            }))),
        },
        config.clone(),
    )
    .await
    .expect("clients list");
    assert!(output.contains("Ada"), "got: {output}");

    let output = execute_with_config(
        Cli {
            output: OutputFormat::Table,
            command: Some(Commands::Invoices(Box::new(InvoiceCommand::Show(
                ShowArgs {
                    id: "invoice_1".to_string(),
                    include: Vec::new(),
                },
            )))),
        },
        config.clone(),
    )
    .await
    .expect("invoice show");
    assert!(output.contains("sent"), "got: {output}");

    let output = execute_with_config(
        Cli {
            output: OutputFormat::Table,
            command: Some(Commands::Invoices(Box::new(InvoiceCommand::List(
                ListArgs {
                    page: 1,
                    per_page: 10,
                    include: Vec::new(),
                    filters: Vec::new(),
                    sort: None,
                    all: false,
                    limit: None,
                },
            )))),
        },
        config.clone(),
    )
    .await
    .expect("invoice list");
    assert!(output.contains("INV-2"), "got: {output}");

    let output = execute_with_config(
        Cli {
            output: OutputFormat::Json,
            command: Some(Commands::Invoices(Box::new(InvoiceCommand::Template(
                TemplateArgs {
                    include: Vec::new(),
                },
            )))),
        },
        config.clone(),
    )
    .await
    .expect("invoice template");
    assert!(output.contains("line_items"), "got: {output}");

    let output = execute_with_config(
        Cli {
            output: OutputFormat::Json,
            command: Some(Commands::Payments(ResourceCommand::List(ListArgs {
                page: 1,
                per_page: 20,
                include: vec!["client".to_string(), " ".to_string()],
                filters: Vec::new(),
                sort: None,
                all: false,
                limit: None,
            }))),
        },
        config.clone(),
    )
    .await
    .expect("payments list");
    assert!(output.contains("payment_1"), "got: {output}");

    let output = execute_with_config(
        Cli {
            output: OutputFormat::Json,
            command: Some(Commands::Clients(ResourceCommand::Template(TemplateArgs {
                include: vec!["contacts".to_string()],
            }))),
        },
        config.clone(),
    )
    .await
    .expect("client template");
    assert!(output.contains("contacts"), "got: {output}");

    let output = execute_with_config(
        Cli {
            output: OutputFormat::Json,
            command: Some(Commands::Invoices(Box::new(InvoiceCommand::EditTemplate(
                ShowArgs {
                    id: "invoice_1".to_string(),
                    include: vec!["client".to_string()],
                },
            )))),
        },
        config,
    )
    .await
    .expect("invoice edit template");
    assert!(output.contains("invoice_1"), "got: {output}");

    clients.assert();
    invoices.assert();
    invoice_list.assert();
    invoice_template.assert();
    payments.assert();
    client_template.assert();
    invoice_edit_template.assert();
}

#[tokio::test]
async fn get_json_reports_decode_errors() {
    let server = MockServer::start();
    let invalid_json = server.mock(|when, then| {
        when.method(GET).path("/api/v1/statics");
        then.status(200).body("not json");
    });

    let client = ApiClient::new(Config::from_values(server.base_url(), "token").expect("config"));
    let error = client
        .get_json("api/v1/statics", &[])
        .await
        .expect_err("decode failure");
    assert!(matches!(error, KobanError::Decode { .. }));
    invalid_json.assert();
}
