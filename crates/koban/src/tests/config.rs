use super::*;

#[test]
fn config_defaults_to_invoice_ninja_base_url() {
    let config = Config::from_values(DEFAULT_BASE_URL, "token").expect("config");
    assert_eq!(config.base_url.as_str(), "https://invoicing.co/");
}

#[test]
fn expanded_resource_labels_and_upload_methods_are_stable() {
    let labels = [
        (Resource::PurchaseOrders, "purchase orders"),
        (Resource::RecurringExpenses, "recurring expenses"),
        (Resource::BankTransactions, "bank transactions"),
        (Resource::BankIntegrations, "bank integrations"),
        (Resource::BankTransactionRules, "bank transaction rules"),
        (Resource::PaymentTerms, "payment terms"),
        (Resource::TaskStatuses, "task statuses"),
        (Resource::SystemLogs, "system logs"),
        (Resource::CompanyGateways, "company gateways"),
        (Resource::CompanyLedger, "company ledger"),
        (Resource::CompanyUsers, "company users"),
        (Resource::ClientGatewayTokens, "client gateway tokens"),
    ];

    for (resource, label) in labels {
        assert_eq!(resource.label(), label);
    }
    assert_eq!(Resource::Invoices.upload_method(), "PUT");
    assert_eq!(Resource::Products.upload_method(), "POST");
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
