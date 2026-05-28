use assert_cmd::Command;
use httpmock::{Method::GET, MockServer};
use predicates::prelude::*;
use serde_json::json;
use tempfile::tempdir;

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
        .stdout(predicate::str::contains(
            "clients      List, show, and inspect clients",
        ))
        .stdout(predicate::str::contains(
            "invoices     List, show, inspect, and download invoices",
        ))
        .stdout(predicate::str::contains("clients"))
        .stdout(predicate::str::contains("invoices"))
        .stdout(predicate::str::contains("payments"))
        .stdout(predicate::str::contains("quotes"))
        .stdout(predicate::str::contains("credits"))
        .stdout(predicate::str::contains("vendors"))
        .stdout(predicate::str::contains("expenses"))
        .stdout(predicate::str::contains("projects"))
        .stdout(predicate::str::contains("tasks"))
        .stdout(predicate::str::contains("update"))
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
        .stdout(predicate::str::contains("--filter"))
        .stdout(predicate::str::contains("--sort"))
        .stdout(predicate::str::contains("--all"))
        .stdout(predicate::str::contains("--limit"))
        .stdout(predicate::str::contains("--include"));
}

#[test]
fn resource_help_includes_read_only_template_commands() {
    koban()
        .args(["invoices", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "template       Show the default invoice template",
        ))
        .stdout(predicate::str::contains("edit-template"))
        .stdout(predicate::str::contains("download"))
        .stdout(predicate::str::contains("delivery-note"));
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
fn update_help_mentions_supported_install_sources() {
    koban()
        .args(["update", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Nix"))
        .stdout(predicate::str::contains("cargo install"))
        .stdout(predicate::str::contains("Homebrew"))
        .stdout(predicate::str::contains("--nightly"))
        .stdout(predicate::str::contains("api.github.com"));
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
fn list_sends_raw_filters_sort_and_limit() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/api/v1/clients")
            .query_param("page", "1")
            .query_param("per_page", "20")
            .query_param("balance", "gt:1000")
            .query_param("name", "Ada")
            .query_param("sort", "name|desc");
        then.status(200).json_body(json!({
            "data": [
                {"id": "client_1", "display_name": "Ada Lovelace"},
                {"id": "client_2", "display_name": "Hidden By Limit"}
            ]
        }));
    });

    koban()
        .env("INVOICE_NINJA_API_TOKEN", "test-token")
        .env("INVOICE_NINJA_BASE_URL", server.base_url())
        .args([
            "clients",
            "list",
            "--filter",
            "balance=gt:1000",
            "--filter",
            "name=Ada",
            "--sort",
            "name|desc",
            "--limit",
            "1",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Ada Lovelace"))
        .stdout(predicate::str::contains("Hidden By Limit").not());

    mock.assert();
}

#[test]
fn malformed_filter_is_actionable_without_network_call() {
    koban()
        .env("INVOICE_NINJA_API_TOKEN", "test-token")
        .env("INVOICE_NINJA_BASE_URL", "http://127.0.0.1:9")
        .args(["clients", "list", "--filter", "not-a-filter"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("list filter is not valid"))
        .stderr(predicate::str::contains("key=value"));
}

#[test]
fn list_all_fetches_pages_and_emits_json_envelope() {
    let server = MockServer::start();
    let first_page = server.mock(|when, then| {
        when.method(GET)
            .path("/api/v1/payments")
            .query_param("page", "1")
            .query_param("per_page", "2")
            .query_param("include", "client");
        then.status(200).json_body(json!({
            "data": [
                {"id": "payment_1", "number": "001"},
                {"id": "payment_2", "number": "002"}
            ]
        }));
    });
    let second_page = server.mock(|when, then| {
        when.method(GET)
            .path("/api/v1/payments")
            .query_param("page", "2")
            .query_param("per_page", "2")
            .query_param("include", "client");
        then.status(200).json_body(json!({
            "data": [{"id": "payment_3", "number": "003"}]
        }));
    });

    koban()
        .env("INVOICE_NINJA_API_TOKEN", "test-token")
        .env("INVOICE_NINJA_BASE_URL", server.base_url())
        .args([
            "--output",
            "json",
            "payments",
            "list",
            "--per-page",
            "2",
            "--include",
            "client",
            "--all",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"payment_3\""))
        .stdout(predicate::str::contains("\"pages_fetched\": 2"));

    first_page.assert();
    second_page.assert();
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
fn invoice_download_saves_pdf_and_refuses_overwrite_without_force() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/api/v1/invoice/invitation_key/download")
            .query_param("include", "client")
            .header("X-API-TOKEN", "test-token")
            .header("X-Requested-With", "XMLHttpRequest");
        then.status(200)
            .header("content-type", "application/pdf")
            .body("%PDF-1.7");
    });
    let dir = tempdir().expect("tempdir");
    let output_file = dir.path().join("invoice.pdf");

    koban()
        .env("INVOICE_NINJA_API_TOKEN", "test-token")
        .env("INVOICE_NINJA_BASE_URL", server.base_url())
        .args([
            "invoices",
            "download",
            "invitation_key",
            "--include",
            "client",
            "--output-file",
        ])
        .arg(&output_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Wrote"));

    assert_eq!(
        std::fs::read_to_string(&output_file).expect("downloaded pdf"),
        "%PDF-1.7"
    );

    koban()
        .env("INVOICE_NINJA_API_TOKEN", "test-token")
        .env("INVOICE_NINJA_BASE_URL", server.base_url())
        .args(["invoices", "download", "invitation_key", "--output-file"])
        .arg(&output_file)
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));

    mock.assert_calls(1);
}

#[test]
fn invoice_delivery_note_saves_pdf_with_force() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/api/v1/invoices/invoice_1/delivery_note")
            .header("X-API-TOKEN", "test-token");
        then.status(200).body("%PDF-delivery");
    });
    let dir = tempdir().expect("tempdir");
    let output_file = dir.path().join("delivery-note.pdf");
    std::fs::write(&output_file, "old").expect("seed file");

    koban()
        .env("INVOICE_NINJA_API_TOKEN", "test-token")
        .env("INVOICE_NINJA_BASE_URL", server.base_url())
        .args(["invoices", "delivery-note", "invoice_1", "--output-file"])
        .arg(&output_file)
        .arg("--force")
        .assert()
        .success();

    assert_eq!(
        std::fs::read_to_string(&output_file).expect("downloaded pdf"),
        "%PDF-delivery"
    );

    mock.assert();
}

#[test]
fn core_ops_resources_use_read_only_routes() {
    let server = MockServer::start();
    let quote = server.mock(|when, then| {
        when.method(GET)
            .path("/api/v1/quotes")
            .query_param("page", "1")
            .query_param("per_page", "20");
        then.status(200).json_body(json!({
            "data": [{"id": "quote_1", "number": "Q-1", "status_id": 3}]
        }));
    });
    let vendor = server.mock(|when, then| {
        when.method(GET).path("/api/v1/vendors/vendor_1");
        then.status(200).json_body(json!({
            "data": {"id": "vendor_1", "display_name": "Paper Co"}
        }));
    });
    let credit = server.mock(|when, then| {
        when.method(GET)
            .path("/api/v1/credits/credit_1")
            .query_param("include", "client");
        then.status(200).json_body(json!({
            "data": {"id": "credit_1", "number": "CR-1", "client_id": "client_1"}
        }));
    });
    let expense_template = server.mock(|when, then| {
        when.method(GET).path("/api/v1/expenses/create");
        then.status(200)
            .json_body(json!({"data": {"id": "", "amount": 0}}));
    });
    let project = server.mock(|when, then| {
        when.method(GET)
            .path("/api/v1/projects")
            .query_param("page", "1")
            .query_param("per_page", "20")
            .query_param("client_id", "client_1");
        then.status(200).json_body(json!({
            "data": [{"id": "project_1", "name": "Website"}]
        }));
    });
    let task_edit = server.mock(|when, then| {
        when.method(GET).path("/api/v1/tasks/task_1/edit");
        then.status(200)
            .json_body(json!({"data": {"id": "task_1", "description": "Review"}}));
    });

    koban()
        .env("INVOICE_NINJA_API_TOKEN", "test-token")
        .env("INVOICE_NINJA_BASE_URL", server.base_url())
        .args(["quotes", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("approved"));

    koban()
        .env("INVOICE_NINJA_API_TOKEN", "test-token")
        .env("INVOICE_NINJA_BASE_URL", server.base_url())
        .args(["vendors", "show", "vendor_1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Paper Co"));

    koban()
        .env("INVOICE_NINJA_API_TOKEN", "test-token")
        .env("INVOICE_NINJA_BASE_URL", server.base_url())
        .args([
            "--output",
            "json",
            "credits",
            "show",
            "credit_1",
            "--include",
            "client",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"CR-1\""));

    koban()
        .env("INVOICE_NINJA_API_TOKEN", "test-token")
        .env("INVOICE_NINJA_BASE_URL", server.base_url())
        .args(["--output", "json", "expenses", "template"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"amount\": 0"));

    koban()
        .env("INVOICE_NINJA_API_TOKEN", "test-token")
        .env("INVOICE_NINJA_BASE_URL", server.base_url())
        .args(["projects", "list", "--filter", "client_id=client_1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Website"));

    koban()
        .env("INVOICE_NINJA_API_TOKEN", "test-token")
        .env("INVOICE_NINJA_BASE_URL", server.base_url())
        .args(["tasks", "edit-template", "task_1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Review"));

    quote.assert();
    vendor.assert();
    credit.assert();
    expense_template.assert();
    project.assert();
    task_edit.assert();
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
