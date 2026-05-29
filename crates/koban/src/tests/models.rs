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

#[test]
fn resource_accessors_target_expected_resources() {
    let client = ApiClient::new(
        Config::from_values("https://demo.invoiceninja.com", "token").expect("config"),
    );
    assert_eq!(client.clients().resource(), Resource::Clients);
    assert_eq!(client.invoices().resource(), Resource::Invoices);
    assert_eq!(client.payments().resource(), Resource::Payments);
    assert_eq!(client.quotes().resource(), Resource::Quotes);
    assert_eq!(client.credits().resource(), Resource::Credits);
    assert_eq!(client.products().resource(), Resource::Products);
    assert_eq!(client.expenses().resource(), Resource::Expenses);
    assert_eq!(client.vendors().resource(), Resource::Vendors);
    assert_eq!(client.projects().resource(), Resource::Projects);
    assert_eq!(client.tasks().resource(), Resource::Tasks);
    assert_eq!(
        client
            .resource::<serde_json::Value>(Resource::TaxRates)
            .resource(),
        Resource::TaxRates
    );
}

#[test]
fn every_model_round_trips_and_tolerates_string_numbers() {
    use crate::models::{
        Client, Contact, Credit, Expense, Payment, Product, Project, Quote, Task, Vendor,
    };

    let client: Client = serde_json::from_value(serde_json::json!({
        "id": "c1", "display_name": "Acme", "balance": "10.5", "paid_to_date": 4,
        "contacts": [{"email": "a@b.c", "is_primary": true, "zzz": 1}],
        "zzz": "keep"
    }))
    .expect("client");
    assert_eq!(client.balance, 10.5);
    assert_eq!(client.contacts[0].email, "a@b.c");
    assert!(client.contacts[0].extra.contains_key("zzz"));
    assert_eq!(serde_json::to_value(&client).expect("ser")["zzz"], "keep");

    let payment: Payment =
        serde_json::from_value(serde_json::json!({"amount": 5, "refunded": "1.5", "applied": 5}))
            .expect("payment");
    assert_eq!(payment.refunded, 1.5);
    let product: Product =
        serde_json::from_value(serde_json::json!({"price": "9.99", "cost": 1, "quantity": 2}))
            .expect("product");
    assert_eq!(product.price, 9.99);
    let quote: Quote =
        serde_json::from_value(serde_json::json!({"amount": "2", "line_items": [{"cost": 1}]}))
            .expect("quote");
    assert_eq!(quote.amount, 2.0);
    let credit: Credit =
        serde_json::from_value(serde_json::json!({"balance": 3.0})).expect("credit");
    assert_eq!(credit.balance, 3.0);
    let expense: Expense =
        serde_json::from_value(serde_json::json!({"amount": "4", "should_be_invoiced": true}))
            .expect("expense");
    assert!(expense.should_be_invoiced);
    let vendor: Vendor =
        serde_json::from_value(serde_json::json!({"name": "V", "contacts": [{"first_name": "X"}]}))
            .expect("vendor");
    assert_eq!(vendor.contacts[0].first_name, "X");
    let project: Project =
        serde_json::from_value(serde_json::json!({"task_rate": "50", "budgeted_hours": 8}))
            .expect("project");
    assert_eq!(project.task_rate, 50.0);
    let task: Task = serde_json::from_value(serde_json::json!({"rate": 75.0, "is_running": true}))
        .expect("task");
    assert!(task.is_running);
    let contact: Contact =
        serde_json::from_value(serde_json::json!({"first_name": "Y"})).expect("contact");
    assert_eq!(contact.first_name, "Y");

    // Serialize each back to JSON to exercise the derived Serialize impls.
    for value in [
        serde_json::to_value(&payment).expect("payment ser"),
        serde_json::to_value(&product).expect("product ser"),
        serde_json::to_value(&quote).expect("quote ser"),
        serde_json::to_value(&credit).expect("credit ser"),
        serde_json::to_value(&expense).expect("expense ser"),
        serde_json::to_value(&vendor).expect("vendor ser"),
        serde_json::to_value(&project).expect("project ser"),
        serde_json::to_value(&task).expect("task ser"),
        serde_json::to_value(&contact).expect("contact ser"),
    ] {
        assert!(value.is_object());
    }
}

#[test]
fn flexible_deserializers_and_envelopes_cover_all_variants() {
    // created_at as a string and as null exercise both i64_opt_flexible arms.
    let from_string: Invoice =
        serde_json::from_value(serde_json::json!({"created_at": "1716000000"})).expect("string ts");
    assert_eq!(from_string.created_at, Some(1_716_000_000));
    let from_null: Invoice =
        serde_json::from_value(serde_json::json!({"created_at": null})).expect("null ts");
    assert_eq!(from_null.created_at, None);

    // A non-numeric value falls back to 0.0 (the f64_flexible catch-all arm).
    let junk: Invoice =
        serde_json::from_value(serde_json::json!({"amount": {"x": 1}})).expect("junk amount");
    assert_eq!(junk.amount, 0.0);

    // The envelopes serialize as well as deserialize.
    let data = Data {
        data: Invoice::default(),
    };
    assert!(serde_json::to_value(&data).expect("data ser")["data"].is_object());
    let page = Paginated {
        data: vec![Invoice::default()],
        meta: None,
    };
    assert!(serde_json::to_value(&page).expect("page ser")["data"].is_array());
}

#[tokio::test]
async fn list_paginated_exposes_pagination_meta() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(GET).path("/api/v1/clients");
        then.status(200)
            .body(r#"{"data":[{"id":"c1"}],"meta":{"pagination":{"total":1,"per_page":20}}}"#);
    });
    let client = ApiClient::new(Config::from_values(server.base_url(), "token").expect("config"));
    let page = client
        .clients()
        .list_paginated(&[("per_page".to_string(), "20".to_string())])
        .await
        .expect("paginated");
    assert_eq!(page.data.len(), 1);
    let pagination = page
        .meta
        .and_then(|meta| meta.pagination)
        .expect("pagination");
    assert_eq!(pagination.per_page, Some(20));
    mock.assert();
}
