use super::*;

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
