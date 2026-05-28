use super::*;

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
async fn expanded_resource_and_endpoint_dry_runs_do_not_touch_network() {
    let config = Config::from_values("http://localhost:1234", "token").expect("config");

    let create = execute_with_config(
        Cli {
            output: OutputFormat::Json,
            command: Some(Commands::Products(ResourceCommand::Create(
                ResourceWriteArgs {
                    payload: {
                        let mut args = empty_resource_payload_args();
                        args.name = Some("Consulting".to_string());
                        args.price = Some("100".to_string());
                        args.fields.push("tax_name1=GST".to_string());
                        args
                    },
                    safety: WriteSafetyArgs {
                        dry_run: true,
                        yes: false,
                    },
                    include: vec!["documents".to_string()],
                },
            ))),
        },
        config.clone(),
    )
    .await
    .expect("product create dry run");
    assert!(
        create.contains("\"path\": \"api/v1/products\""),
        "got: {create}"
    );
    assert!(create.contains("\"price\": 100"), "got: {create}");

    let update = execute_with_config(
        Cli {
            output: OutputFormat::Json,
            command: Some(Commands::PurchaseOrders(ResourceCommand::Update(
                UpdateResourceArgs {
                    id: "po_1".to_string(),
                    payload: {
                        let mut args = empty_resource_payload_args();
                        args.vendor_id = Some("vendor_1".to_string());
                        args.line_items
                            .push("product_key=Paper,quantity=2,cost=4".to_string());
                        args
                    },
                    safety: WriteSafetyArgs {
                        dry_run: true,
                        yes: false,
                    },
                    include: Vec::new(),
                },
            ))),
        },
        config.clone(),
    )
    .await
    .expect("purchase order update dry run");
    assert!(
        update.contains("api/v1/purchase_orders/po_1"),
        "got: {update}"
    );
    assert!(update.contains("\"line_items\""), "got: {update}");

    let search = execute_with_config(
        Cli {
            output: OutputFormat::Json,
            command: Some(Commands::Search(EndpointCommand::Run(EndpointArgs {
                endpoint: None,
                method: Some(HttpMethod::Post),
                payload: {
                    let mut args = empty_resource_payload_args();
                    args.fields.push("query=acme".to_string());
                    args
                },
                safety: WriteSafetyArgs {
                    dry_run: true,
                    yes: false,
                },
                include: Vec::new(),
            }))),
        },
        config,
    )
    .await
    .expect("search dry run");
    assert!(
        search.contains("\"path\": \"api/v1/search\""),
        "got: {search}"
    );
    assert!(search.contains("\"query\": \"acme\""), "got: {search}");
}

#[tokio::test]
async fn every_expanded_resource_has_reachable_dry_run_writes() {
    let config = Config::from_values("http://localhost:1234", "token").expect("config");

    for (label, command) in expanded_resource_create_commands() {
        let output = execute_with_config(
            Cli {
                output: OutputFormat::Json,
                command: Some(command),
            },
            config.clone(),
        )
        .await
        .unwrap_or_else(|error| panic!("{label} create dry run failed: {error}"));
        assert!(output.contains("\"dry_run\": true"), "{label}: {output}");
        assert!(output.contains("\"method\": \"POST\""), "{label}: {output}");
    }

    let delete = execute_with_config(
        Cli {
            output: OutputFormat::Json,
            command: Some(Commands::Webhooks(ResourceCommand::Delete(
                ConfirmableIdArgs {
                    id: "webhook_1".to_string(),
                    safety: WriteSafetyArgs {
                        dry_run: true,
                        yes: false,
                    },
                    include: Vec::new(),
                },
            ))),
        },
        config.clone(),
    )
    .await
    .expect("webhook delete dry run");
    assert!(delete.contains("\"method\": \"DELETE\""), "got: {delete}");

    let bulk = execute_with_config(
        Cli {
            output: OutputFormat::Json,
            command: Some(Commands::TaxRates(ResourceCommand::Bulk(BulkArgs {
                action: "archive".to_string(),
                ids: vec!["one".to_string()],
                email_type: None,
                safety: WriteSafetyArgs {
                    dry_run: true,
                    yes: false,
                },
                include: Vec::new(),
            }))),
        },
        config.clone(),
    )
    .await
    .expect("tax rates bulk dry run");
    assert!(bulk.contains("api/v1/tax_rates/bulk"), "got: {bulk}");

    let tempdir = tempfile::tempdir().expect("tempdir");
    let upload_file = tempdir.path().join("doc.txt");
    std::fs::write(&upload_file, b"document").expect("upload fixture");
    let upload = execute_with_config(
        Cli {
            output: OutputFormat::Json,
            command: Some(Commands::Documents(ResourceCommand::Upload(UploadArgs {
                id: "document_1".to_string(),
                files: vec![upload_file],
                safety: WriteSafetyArgs {
                    dry_run: true,
                    yes: false,
                },
                include: Vec::new(),
            }))),
        },
        config.clone(),
    )
    .await
    .expect("document upload dry run");
    assert!(upload.contains("\"method\": \"POST\""), "got: {upload}");

    let get_action = execute_with_config(
        Cli {
            output: OutputFormat::Json,
            command: Some(Commands::RecurringInvoices(ResourceCommand::Action(
                ResourceActionArgs {
                    id: "recurring_1".to_string(),
                    action: "start".to_string(),
                    payload: empty_resource_payload_args(),
                    safety: WriteSafetyArgs {
                        dry_run: true,
                        yes: false,
                    },
                    include: Vec::new(),
                },
            ))),
        },
        config.clone(),
    )
    .await
    .expect("recurring invoice action dry run");
    assert!(
        get_action.contains("\"method\": \"POST\""),
        "got: {get_action}"
    );
    assert!(
        get_action.contains("api/v1/recurring_invoices/bulk"),
        "got: {get_action}"
    );
    assert!(
        get_action.contains("\"ids\": [\n      \"recurring_1\"\n    ]"),
        "got: {get_action}"
    );

    let post_action = execute_with_config(
        Cli {
            output: OutputFormat::Json,
            command: Some(Commands::Clients(ResourceCommand::Action(
                ResourceActionArgs {
                    id: "client_1".to_string(),
                    action: "updateTaxData".to_string(),
                    payload: {
                        let mut args = empty_resource_payload_args();
                        args.fields.push("country_id=840".to_string());
                        args
                    },
                    safety: WriteSafetyArgs {
                        dry_run: true,
                        yes: false,
                    },
                    include: Vec::new(),
                },
            ))),
        },
        config,
    )
    .await
    .expect("client post action dry run");
    assert!(
        post_action.contains("\"method\": \"POST\""),
        "got: {post_action}"
    );
}

#[tokio::test]
async fn generic_resource_and_endpoint_writes_require_confirmation_without_dry_run() {
    let config = Config::from_values("http://localhost:1234", "token").expect("config");

    let create_error = execute_with_config(
        Cli {
            output: OutputFormat::Json,
            command: Some(Commands::Tokens(ResourceCommand::Create(
                ResourceWriteArgs {
                    payload: {
                        let mut args = empty_resource_payload_args();
                        args.name = Some("automation".to_string());
                        args
                    },
                    safety: WriteSafetyArgs {
                        dry_run: false,
                        yes: false,
                    },
                    include: Vec::new(),
                },
            ))),
        },
        config.clone(),
    )
    .await
    .expect_err("token create should require confirmation");
    assert!(matches!(
        create_error,
        KobanError::ConfirmationRequired { .. }
    ));

    let update_error = execute_with_config(
        Cli {
            output: OutputFormat::Json,
            command: Some(Commands::Webhooks(ResourceCommand::Update(
                UpdateResourceArgs {
                    id: "webhook_1".to_string(),
                    payload: {
                        let mut args = empty_resource_payload_args();
                        args.name = Some("webhook".to_string());
                        args
                    },
                    safety: WriteSafetyArgs {
                        dry_run: false,
                        yes: false,
                    },
                    include: Vec::new(),
                },
            ))),
        },
        config.clone(),
    )
    .await
    .expect_err("webhook update should require confirmation");
    assert!(matches!(
        update_error,
        KobanError::ConfirmationRequired { .. }
    ));

    let endpoint_error = execute_with_config(
        Cli {
            output: OutputFormat::Json,
            command: Some(Commands::Charts(EndpointCommand::Run(EndpointArgs {
                endpoint: None,
                method: Some(HttpMethod::Put),
                payload: {
                    let mut args = empty_resource_payload_args();
                    args.fields.push("name=revenue".to_string());
                    args
                },
                safety: WriteSafetyArgs {
                    dry_run: false,
                    yes: false,
                },
                include: Vec::new(),
            }))),
        },
        config,
    )
    .await
    .expect_err("endpoint put should require confirmation");
    assert!(matches!(
        endpoint_error,
        KobanError::ConfirmationRequired { .. }
    ));
}

#[tokio::test]
async fn utility_defaults_to_safe_ping_get() {
    let config = Config::from_values("http://localhost:1234", "token").expect("config");
    let output = execute_with_config(
        Cli {
            output: OutputFormat::Json,
            command: Some(Commands::Utility(EndpointCommand::Run(EndpointArgs {
                endpoint: None,
                method: None,
                payload: empty_resource_payload_args(),
                safety: WriteSafetyArgs {
                    dry_run: true,
                    yes: false,
                },
                include: Vec::new(),
            }))),
        },
        config,
    )
    .await
    .expect("utility ping dry run");
    assert!(output.contains("\"method\": \"GET\""), "got: {output}");
    assert!(
        output.contains("\"path\": \"api/v1/ping\""),
        "got: {output}"
    );
}

#[tokio::test]
async fn generic_resource_and_endpoint_commands_hit_expected_routes() {
    let server = MockServer::start();
    let config = Config::from_values(server.base_url(), "token").expect("config");

    let product_create = server.mock(|when, then| {
        when.method(POST).path("/api/v1/products");
        then.status(200)
            .json_body(serde_json::json!({"data": {"id": "product_1"}}));
    });
    let product_update = server.mock(|when, then| {
        when.method(PUT).path("/api/v1/products/product_1");
        then.status(200)
            .json_body(serde_json::json!({"data": {"id": "product_1"}}));
    });
    let webhook_delete = server.mock(|when, then| {
        when.method(httpmock::Method::DELETE)
            .path("/api/v1/webhooks/webhook_1");
        then.status(200)
            .json_body(serde_json::json!({"data": {"id": "webhook_1"}}));
    });
    let report_get = server.mock(|when, then| {
        when.method(GET).path("/api/v1/reports");
        then.status(200)
            .json_body(serde_json::json!({"data": [{"name": "sales"}]}));
    });
    let chart_put = server.mock(|when, then| {
        when.method(PUT).path("/api/v1/charts");
        then.status(200)
            .json_body(serde_json::json!({"data": {"name": "chart"}}));
    });
    let utility_delete = server.mock(|when, then| {
        when.method(httpmock::Method::DELETE)
            .path("/api/v1/support/ticket_1");
        then.status(200)
            .json_body(serde_json::json!({"data": {"id": "ticket_1"}}));
    });

    execute_with_config(
        Cli {
            output: OutputFormat::Json,
            command: Some(Commands::Products(ResourceCommand::Create(
                ResourceWriteArgs {
                    payload: {
                        let mut args = empty_resource_payload_args();
                        args.name = Some("Consulting".to_string());
                        args
                    },
                    safety: WriteSafetyArgs {
                        dry_run: false,
                        yes: true,
                    },
                    include: Vec::new(),
                },
            ))),
        },
        config.clone(),
    )
    .await
    .expect("product create");

    execute_with_config(
        Cli {
            output: OutputFormat::Json,
            command: Some(Commands::Products(ResourceCommand::Update(
                UpdateResourceArgs {
                    id: "product_1".to_string(),
                    payload: {
                        let mut args = empty_resource_payload_args();
                        args.price = Some("150".to_string());
                        args
                    },
                    safety: WriteSafetyArgs {
                        dry_run: false,
                        yes: true,
                    },
                    include: Vec::new(),
                },
            ))),
        },
        config.clone(),
    )
    .await
    .expect("product update");

    execute_with_config(
        Cli {
            output: OutputFormat::Json,
            command: Some(Commands::Webhooks(ResourceCommand::Delete(
                ConfirmableIdArgs {
                    id: "webhook_1".to_string(),
                    safety: WriteSafetyArgs {
                        dry_run: false,
                        yes: true,
                    },
                    include: Vec::new(),
                },
            ))),
        },
        config.clone(),
    )
    .await
    .expect("webhook delete");

    execute_with_config(
        Cli {
            output: OutputFormat::Json,
            command: Some(Commands::Reports(EndpointCommand::Run(EndpointArgs {
                endpoint: None,
                method: Some(HttpMethod::Get),
                payload: empty_resource_payload_args(),
                safety: WriteSafetyArgs {
                    dry_run: false,
                    yes: true,
                },
                include: Vec::new(),
            }))),
        },
        config.clone(),
    )
    .await
    .expect("report get");

    execute_with_config(
        Cli {
            output: OutputFormat::Json,
            command: Some(Commands::Charts(EndpointCommand::Run(EndpointArgs {
                endpoint: None,
                method: Some(HttpMethod::Put),
                payload: {
                    let mut args = empty_resource_payload_args();
                    args.fields.push("name=revenue".to_string());
                    args
                },
                safety: WriteSafetyArgs {
                    dry_run: false,
                    yes: true,
                },
                include: Vec::new(),
            }))),
        },
        config.clone(),
    )
    .await
    .expect("chart put");

    execute_with_config(
        Cli {
            output: OutputFormat::Json,
            command: Some(Commands::Utility(EndpointCommand::Run(EndpointArgs {
                endpoint: Some("support/ticket_1".to_string()),
                method: Some(HttpMethod::Delete),
                payload: empty_resource_payload_args(),
                safety: WriteSafetyArgs {
                    dry_run: false,
                    yes: true,
                },
                include: Vec::new(),
            }))),
        },
        config,
    )
    .await
    .expect("utility delete");

    product_create.assert();
    product_update.assert();
    webhook_delete.assert();
    report_get.assert();
    chart_put.assert();
    utility_delete.assert();
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

fn expanded_resource_create_commands() -> Vec<(&'static str, Commands)> {
    vec![
        ("clients", Commands::Clients(create_command())),
        ("payments", Commands::Payments(create_command())),
        ("quotes", Commands::Quotes(create_command())),
        ("credits", Commands::Credits(create_command())),
        ("vendors", Commands::Vendors(create_command())),
        ("expenses", Commands::Expenses(create_command())),
        ("projects", Commands::Projects(create_command())),
        ("tasks", Commands::Tasks(create_command())),
        ("locations", Commands::Locations(create_command())),
        ("products", Commands::Products(create_command())),
        (
            "recurring invoices",
            Commands::RecurringInvoices(create_command()),
        ),
        (
            "purchase orders",
            Commands::PurchaseOrders(create_command()),
        ),
        (
            "recurring expenses",
            Commands::RecurringExpenses(create_command()),
        ),
        (
            "bank transactions",
            Commands::BankTransactions(create_command()),
        ),
        (
            "bank integrations",
            Commands::BankIntegrations(create_command()),
        ),
        (
            "bank transaction rules",
            Commands::BankTransactionRules(create_command()),
        ),
        (
            "expense categories",
            Commands::ExpenseCategories(create_command()),
        ),
        ("tax rates", Commands::TaxRates(create_command())),
        ("payment terms", Commands::PaymentTerms(create_command())),
        ("task statuses", Commands::TaskStatuses(create_command())),
        ("activities", Commands::Activities(create_command())),
        ("system logs", Commands::SystemLogs(create_command())),
        ("documents", Commands::Documents(create_command())),
        ("designs", Commands::Designs(create_command())),
        ("templates", Commands::Templates(create_command())),
        ("users", Commands::Users(create_command())),
        ("companies", Commands::Companies(create_command())),
        (
            "company gateways",
            Commands::CompanyGateways(create_command()),
        ),
        ("company ledger", Commands::CompanyLedger(create_command())),
        ("company users", Commands::CompanyUsers(create_command())),
        ("tokens", Commands::Tokens(create_command())),
        ("webhooks", Commands::Webhooks(create_command())),
        ("imports", Commands::Imports(create_command())),
        ("subscriptions", Commands::Subscriptions(create_command())),
        (
            "client gateway tokens",
            Commands::ClientGatewayTokens(create_command()),
        ),
    ]
}

fn create_command() -> ResourceCommand {
    ResourceCommand::Create(ResourceWriteArgs {
        payload: {
            let mut args = empty_resource_payload_args();
            args.name = Some("Koban Smoke".to_string());
            args
        },
        safety: WriteSafetyArgs {
            dry_run: true,
            yes: false,
        },
        include: Vec::new(),
    })
}
