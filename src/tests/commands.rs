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
