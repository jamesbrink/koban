use assert_cmd::Command;
use httpmock::{Method::GET, MockServer};
use predicates::prelude::*;
use tempfile::tempdir;

/// A koban command isolated from any ambient credentials in the dev shell.
fn koban() -> Command {
    let mut cmd = Command::cargo_bin("koban").expect("koban binary");
    cmd.env_remove("INVOICE_NINJA_API_TOKEN")
        .env_remove("INVOICE_NINJA_BASE_URL");
    cmd
}

#[test]
fn login_no_verify_writes_config_and_status_reports_it() {
    let dir = tempdir().expect("tempdir");

    koban()
        .env("KOBAN_CONFIG_DIR", dir.path())
        .args([
            "auth",
            "login",
            "--token",
            "tok-file-123",
            "--base-url",
            "https://demo.invoiceninja.com",
            "--no-verify",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("config file"));

    let config = dir.path().join("config.json");
    let contents = std::fs::read_to_string(&config).expect("config written");
    assert!(contents.contains("tok-file-123"));
    assert!(contents.contains("https://demo.invoiceninja.com"));

    koban()
        .env("KOBAN_CONFIG_DIR", dir.path())
        .args(["auth", "status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("config file"));
}

#[cfg(unix)]
#[test]
fn login_writes_token_file_with_owner_only_permissions() {
    use std::os::unix::fs::PermissionsExt;

    let dir = tempdir().expect("tempdir");
    koban()
        .env("KOBAN_CONFIG_DIR", dir.path())
        .args(["auth", "login", "--token", "tok-secret", "--no-verify"])
        .assert()
        .success();

    let metadata = std::fs::metadata(dir.path().join("config.json")).expect("config metadata");
    assert_eq!(metadata.permissions().mode() & 0o777, 0o600);
}

#[test]
fn login_verifies_against_api_then_saves() {
    let server = MockServer::start();
    let statics = server.mock(|when, then| {
        when.method(GET)
            .path("/api/v1/statics")
            .header("X-API-TOKEN", "tok-verify");
        then.status(200).json_body(json_empty());
    });
    let dir = tempdir().expect("tempdir");

    koban()
        .env("KOBAN_CONFIG_DIR", dir.path())
        .args([
            "auth",
            "login",
            "--token",
            "tok-verify",
            "--base-url",
            &server.base_url(),
        ])
        .assert()
        .success();

    statics.assert();
    assert!(dir.path().join("config.json").exists());
}

#[test]
fn login_failed_verification_does_not_save() {
    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(GET).path("/api/v1/statics");
        then.status(401).json_body(json_message("invalid token"));
    });
    let dir = tempdir().expect("tempdir");

    koban()
        .env("KOBAN_CONFIG_DIR", dir.path())
        .args([
            "auth",
            "login",
            "--token",
            "tok-bad",
            "--base-url",
            &server.base_url(),
        ])
        .assert()
        .failure();

    assert!(
        !dir.path().join("config.json").exists(),
        "a failed verification must not persist the token"
    );
}

#[test]
fn status_json_does_not_leak_the_token() {
    let dir = tempdir().expect("tempdir");
    koban()
        .env("KOBAN_CONFIG_DIR", dir.path())
        .args([
            "auth",
            "login",
            "--token",
            "super-secret-token",
            "--no-verify",
        ])
        .assert()
        .success();

    koban()
        .env("KOBAN_CONFIG_DIR", dir.path())
        .args(["--output", "json", "auth", "status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"authenticated\": true"))
        .stdout(predicate::str::contains("super-secret-token").not());
}

#[test]
fn logout_removes_stored_credentials() {
    let dir = tempdir().expect("tempdir");
    koban()
        .env("KOBAN_CONFIG_DIR", dir.path())
        .args(["auth", "login", "--token", "tok-remove", "--no-verify"])
        .assert()
        .success();

    koban()
        .env("KOBAN_CONFIG_DIR", dir.path())
        .args(["auth", "logout"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Removed"));

    koban()
        .env("KOBAN_CONFIG_DIR", dir.path())
        .args(["auth", "status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Not authenticated"));
}

#[test]
fn stored_credential_resolves_for_normal_commands() {
    let server = MockServer::start();
    let clients = server.mock(|when, then| {
        when.method(GET)
            .path("/api/v1/clients")
            .header("X-API-TOKEN", "tok-stored");
        then.status(200).json_body(json_client());
    });
    let dir = tempdir().expect("tempdir");

    // No environment token: the command must resolve the stored credential.
    koban()
        .env("KOBAN_CONFIG_DIR", dir.path())
        .args([
            "auth",
            "login",
            "--token",
            "tok-stored",
            "--base-url",
            &server.base_url(),
            "--no-verify",
        ])
        .assert()
        .success();

    koban()
        .env("KOBAN_CONFIG_DIR", dir.path())
        .args(["clients", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Ada Lovelace"));

    clients.assert();
}

#[test]
fn environment_token_takes_precedence_over_stored_file() {
    let server = MockServer::start();
    let clients = server.mock(|when, then| {
        when.method(GET)
            .path("/api/v1/clients")
            .header("X-API-TOKEN", "env-wins");
        then.status(200).json_body(json_client());
    });
    let dir = tempdir().expect("tempdir");

    koban()
        .env("KOBAN_CONFIG_DIR", dir.path())
        .args(["auth", "login", "--token", "file-loses", "--no-verify"])
        .assert()
        .success();

    koban()
        .env("KOBAN_CONFIG_DIR", dir.path())
        .env("INVOICE_NINJA_API_TOKEN", "env-wins")
        .env("INVOICE_NINJA_BASE_URL", server.base_url())
        .args(["clients", "list"])
        .assert()
        .success();

    clients.assert();
}

fn json_empty() -> serde_json::Value {
    serde_json::json!({ "data": [] })
}

fn json_message(message: &str) -> serde_json::Value {
    serde_json::json!({ "message": message })
}

fn json_client() -> serde_json::Value {
    serde_json::json!({
        "data": [{ "id": "client_1", "display_name": "Ada Lovelace", "balance": 0 }]
    })
}
