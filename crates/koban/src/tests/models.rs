use super::*;
use crate::models::{Data, Invoice, Paginated};

#[test]
fn invoice_deserializes_common_fields_and_preserves_unmodelled_extras() {
    let value = serde_json::json!({
        "id": "inv_1",
        "number": "INV-0001",
        "client_id": "client_1",
        "amount": 150.5,
        "balance": "50.25",
        "status_id": "2",
        "line_items": [
            {"product_key": "Consulting", "cost": 99.5, "quantity": 1, "line_total": 99.5}
        ],
        "created_at": 1_716_000_000,
        "custom_value1": "keep-me"
    });

    let invoice: Invoice = serde_json::from_value(value).expect("invoice");
    assert_eq!(invoice.number, "INV-0001");
    assert_eq!(invoice.amount, 150.5);
    // `balance` arrives as a string but the flexible deserializer parses it.
    assert_eq!(invoice.balance, 50.25);
    assert_eq!(invoice.created_at, Some(1_716_000_000));
    assert_eq!(invoice.line_items.len(), 1);
    assert_eq!(invoice.line_items[0].product_key, "Consulting");
    // Unmodelled fields are preserved verbatim.
    assert_eq!(invoice.extra.get("custom_value1").unwrap(), "keep-me");
}

#[test]
fn models_default_when_fields_are_absent() {
    let invoice: Invoice = serde_json::from_value(serde_json::json!({})).expect("empty invoice");
    assert_eq!(invoice.number, "");
    assert_eq!(invoice.amount, 0.0);
    assert_eq!(invoice.created_at, None);
    assert!(invoice.line_items.is_empty());
    assert!(invoice.extra.is_empty());
}

#[test]
fn data_and_paginated_envelopes_unwrap_records() {
    let single: Data<Invoice> =
        serde_json::from_value(serde_json::json!({"data": {"number": "INV-1"}})).expect("data");
    assert_eq!(single.data.number, "INV-1");

    let page: Paginated<Invoice> = serde_json::from_value(serde_json::json!({
        "data": [{"number": "INV-1"}, {"number": "INV-2"}],
        "meta": {"pagination": {"total": 2, "count": 2, "current_page": 1, "total_pages": 1}}
    }))
    .expect("paginated");
    assert_eq!(page.data.len(), 2);
    let pagination = page
        .meta
        .and_then(|meta| meta.pagination)
        .expect("pagination");
    assert_eq!(pagination.total, Some(2));
    assert_eq!(pagination.current_page, Some(1));
}

#[test]
fn paginated_tolerates_missing_meta() {
    let page: Paginated<Invoice> =
        serde_json::from_value(serde_json::json!({"data": []})).expect("paginated");
    assert!(page.data.is_empty());
    assert!(page.meta.is_none());
}

#[tokio::test]
async fn typed_list_and_get_resource_hit_expected_routes() {
    let server = MockServer::start();
    let list = server.mock(|when, then| {
        when.method(GET).path("/api/v1/invoices");
        then.status(200).body(
            r#"{"data":[{"id":"inv_1","number":"INV-1","balance":10.0}],"meta":{"pagination":{"total":1}}}"#,
        );
    });
    let show = server.mock(|when, then| {
        when.method(GET).path("/api/v1/invoices/inv_1");
        then.status(200)
            .body(r#"{"data":{"id":"inv_1","number":"INV-1","amount":42.0}}"#);
    });

    let client = ApiClient::new(Config::from_values(server.base_url(), "token").expect("config"));

    let invoices = client.invoices().list().await.expect("list");
    assert_eq!(invoices.len(), 1);
    assert_eq!(invoices[0].number, "INV-1");
    list.assert();

    let invoice = client.invoices().get("inv_1").await.expect("get");
    assert_eq!(invoice.amount, 42.0);
    show.assert();
}

#[tokio::test]
async fn typed_create_update_delete_round_trip_through_data_envelope() {
    let server = MockServer::start();
    let create = server.mock(|when, then| {
        when.method(POST).path("/api/v1/clients");
        then.status(200)
            .body(r#"{"data":{"id":"client_1","name":"Acme"}}"#);
    });
    let update = server.mock(|when, then| {
        when.method(PUT).path("/api/v1/clients/client_1");
        then.status(200)
            .body(r#"{"data":{"id":"client_1","name":"Acme Inc"}}"#);
    });
    let delete = server.mock(|when, then| {
        when.method(httpmock::Method::DELETE)
            .path("/api/v1/clients/client_1");
        then.status(200)
            .body(r#"{"data":{"id":"client_1","is_deleted":true}}"#);
    });

    let client = ApiClient::new(Config::from_values(server.base_url(), "token").expect("config"));

    let created = client
        .clients()
        .create(&serde_json::json!({"name": "Acme"}))
        .await
        .expect("create");
    assert_eq!(created.name, "Acme");
    create.assert();

    let updated = client
        .clients()
        .update("client_1", &serde_json::json!({"name": "Acme Inc"}))
        .await
        .expect("update");
    assert_eq!(updated.name, "Acme Inc");
    update.assert();

    let deleted = client.clients().delete("client_1").await.expect("delete");
    assert!(deleted.is_deleted);
    delete.assert();
}

#[tokio::test]
async fn generic_resource_handle_supports_caller_chosen_types() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(GET).path("/api/v1/tax_rates");
        then.status(200)
            .body(r#"{"data":[{"name":"GST","rate":10.0}]}"#);
    });

    let client = ApiClient::new(Config::from_values(server.base_url(), "token").expect("config"));
    let rows = client
        .resource::<serde_json::Value>(Resource::TaxRates)
        .list()
        .await
        .expect("list tax rates");
    assert_eq!(rows[0]["name"], "GST");
    mock.assert();
}
