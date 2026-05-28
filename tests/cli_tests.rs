use assert_cmd::Command;
use httpmock::{Method::GET, MockServer};
use predicates::prelude::*;
use serde_json::json;

fn koban() -> Command {
    Command::cargo_bin("koban").expect("koban binary")
}

#[test]
fn help_mentions_invoice_ninja_resources_and_completions() {
    koban()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Invoice Ninja"))
        .stdout(predicate::str::contains("clients"))
        .stdout(predicate::str::contains("invoices"))
        .stdout(predicate::str::contains("payments"))
        .stdout(predicate::str::contains("completions"));
}

#[test]
fn resource_help_includes_examples_and_usage() {
    koban()
        .args(["clients", "list", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage:"))
        .stdout(predicate::str::contains(
            "koban clients list --page 1 --per-page 20",
        ))
        .stdout(predicate::str::contains("--include"));
}

#[test]
fn resource_help_includes_read_only_template_commands() {
    koban()
        .args(["invoices", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\n  template\n"))
        .stdout(predicate::str::contains("edit-template"));
}

#[test]
fn version_reports_package_version() {
    koban()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn no_args_prints_help() {
    koban()
        .assert()
        .failure()
        .stderr(predicate::str::contains("Usage:"));
}

#[test]
fn missing_token_is_actionable() {
    koban()
        .env_remove("INVOICE_NINJA_API_TOKEN")
        .env_remove("INVOICE_NINJA_BASE_URL")
        .arg("statics")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Invoice Ninja API token is not configured",
        ))
        .stderr(predicate::str::contains("INVOICE_NINJA_API_TOKEN"));
}

#[test]
fn statics_outputs_json_from_mock_api() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/api/v1/statics")
            .header("X-API-TOKEN", "test-token")
            .header("X-Requested-With", "XMLHttpRequest");
        then.status(200)
            .json_body(json!({"data": [{"id": "1", "name": "United States"}]}));
    });

    koban()
        .env("INVOICE_NINJA_API_TOKEN", "test-token")
        .env("INVOICE_NINJA_BASE_URL", server.base_url())
        .args(["--output", "json", "statics"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"United States\""));

    mock.assert();
}

#[test]
fn clients_list_sends_pagination_include_and_renders_table() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/api/v1/clients")
            .query_param("page", "2")
            .query_param("per_page", "15")
            .query_param("include", "activities,ledger")
            .header("X-API-TOKEN", "test-token");
        then.status(200).json_body(json!({
            "data": [{
                "id": "client_1",
                "display_name": "Ada Lovelace",
                "balance": 42.5
            }]
        }));
    });

    koban()
        .env("INVOICE_NINJA_API_TOKEN", "test-token")
        .env("INVOICE_NINJA_BASE_URL", server.base_url())
        .args([
            "clients",
            "list",
            "--page",
            "2",
            "--per-page",
            "15",
            "--include",
            "activities,ledger",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Ada Lovelace"))
        .stdout(predicate::str::contains("42.5"));

    mock.assert();
}

#[test]
fn invoices_show_preserves_json_shape() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/api/v1/invoices/invoice_1")
            .query_param("include", "client")
            .header("X-Requested-With", "XMLHttpRequest");
        then.status(200).json_body(json!({
            "data": {
                "id": "invoice_1",
                "number": "INV-100",
                "status_id": 2,
                "custom_field": "kept"
            }
        }));
    });

    koban()
        .env("INVOICE_NINJA_API_TOKEN", "test-token")
        .env("INVOICE_NINJA_BASE_URL", server.base_url())
        .args([
            "--output",
            "json",
            "invoices",
            "show",
            "invoice_1",
            "--include",
            "client",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"custom_field\": \"kept\""));

    mock.assert();
}

#[test]
fn clients_template_uses_read_only_create_route() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/api/v1/clients/create")
            .query_param("include", "contacts");
        then.status(200).json_body(json!({
            "data": {
                "id": "",
                "display_name": "",
                "contacts": []
            }
        }));
    });

    koban()
        .env("INVOICE_NINJA_API_TOKEN", "test-token")
        .env("INVOICE_NINJA_BASE_URL", server.base_url())
        .args([
            "--output",
            "json",
            "clients",
            "template",
            "--include",
            "contacts",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"contacts\": []"));

    mock.assert();
}

#[test]
fn invoices_edit_template_uses_read_only_edit_route() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/api/v1/invoices/invoice_1/edit")
            .query_param("include", "client");
        then.status(200).json_body(json!({
            "data": {
                "id": "invoice_1",
                "number": "INV-100",
                "client": {"display_name": "Ada Lovelace"}
            }
        }));
    });

    koban()
        .env("INVOICE_NINJA_API_TOKEN", "test-token")
        .env("INVOICE_NINJA_BASE_URL", server.base_url())
        .args([
            "--output",
            "json",
            "invoices",
            "edit-template",
            "invoice_1",
            "--include",
            "client",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"invoice_1\""))
        .stdout(predicate::str::contains("\"Ada Lovelace\""));

    mock.assert();
}

#[test]
fn payments_show_maps_server_error_without_leaking_token() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(GET).path("/api/v1/payments/missing");
        then.status(404)
            .json_body(json!({"message": "payment missing for token secret-token"}));
    });

    koban()
        .env("INVOICE_NINJA_API_TOKEN", "secret-token")
        .env("INVOICE_NINJA_BASE_URL", server.base_url())
        .args(["payments", "show", "missing"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("HTTP 404"))
        .stderr(predicate::str::contains("[REDACTED]"))
        .stderr(predicate::str::contains("secret-token").not());

    mock.assert();
}
