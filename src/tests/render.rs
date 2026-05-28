use super::*;

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
