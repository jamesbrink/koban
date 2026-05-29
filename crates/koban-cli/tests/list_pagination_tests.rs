use assert_cmd::Command;
use httpmock::{Method::GET, MockServer};
use predicates::prelude::*;
use serde_json::json;

fn koban() -> Command {
    Command::cargo_bin("koban").expect("koban binary")
}

#[test]
fn list_all_stops_at_page_cap_without_fetching_forever() {
    let server = MockServer::start();
    let mut page_mocks = Vec::new();
    for page in 1..=100 {
        page_mocks.push(server.mock(|when, then| {
            when.method(GET)
                .path("/api/v1/payments")
                .query_param("page", page.to_string())
                .query_param("per_page", "1");
            then.status(200).json_body(json!({
                "data": [{"id": format!("payment_{page}")}]
            }));
        }));
    }
    let overflow = server.mock(|when, then| {
        when.method(GET)
            .path("/api/v1/payments")
            .query_param("page", "101")
            .query_param("per_page", "1");
        then.status(500)
            .json_body(json!({"message": "page cap missed"}));
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
            "1",
            "--all",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"payment_100\""))
        .stdout(predicate::str::contains("\"pages_fetched\": 100"))
        .stdout(predicate::str::contains("\"page_cap\": 100"))
        .stdout(predicate::str::contains("\"page_cap_reached\": true"));

    for page_mock in page_mocks {
        page_mock.assert();
    }
    overflow.assert_calls(0);
}
