use super::*;

#[tokio::test]
async fn newly_supported_resource_families_have_dry_run_writes() {
    let config = Config::from_values("http://localhost:1234", "token").expect("config");

    let cases = [
        (
            "recurring quotes",
            Commands::RecurringQuotes(ResourceCommand::Create(ResourceWriteArgs {
                payload: {
                    let mut args = empty_resource_payload_args();
                    args.client_id = Some("client_1".to_string());
                    args
                },
                safety: WriteSafetyArgs {
                    dry_run: true,
                    yes: false,
                },
                include: Vec::new(),
            })),
            "api/v1/recurring_quotes",
        ),
        (
            "group settings",
            Commands::GroupSettings(ResourceCommand::Create(ResourceWriteArgs {
                payload: {
                    let mut args = empty_resource_payload_args();
                    args.name = Some("Retail clients".to_string());
                    args
                },
                safety: WriteSafetyArgs {
                    dry_run: true,
                    yes: false,
                },
                include: Vec::new(),
            })),
            "api/v1/group_settings",
        ),
        (
            "task schedulers",
            Commands::TaskSchedulers(ResourceCommand::Create(ResourceWriteArgs {
                payload: {
                    let mut args = empty_resource_payload_args();
                    args.name = Some("Weekly sync".to_string());
                    args
                },
                safety: WriteSafetyArgs {
                    dry_run: true,
                    yes: false,
                },
                include: Vec::new(),
            })),
            "api/v1/task_schedulers",
        ),
    ];

    for (label, command, path) in cases {
        let output = execute_with_config(
            Cli {
                output: OutputFormat::Json,
                command: Some(command),
            },
            config.clone(),
        )
        .await
        .unwrap_or_else(|error| panic!("{label} dry run failed: {error}"));
        assert!(output.contains(path), "{label}: {output}");
    }
}

#[tokio::test]
async fn document_resources_download_from_official_invitation_routes() {
    let server = MockServer::start();
    let config = Config::from_values(server.base_url(), "token").expect("config");
    let tempdir = tempfile::tempdir().expect("tempdir");

    let cases = [
        (
            "quote",
            Commands::Quotes(ResourceCommand::Download(DownloadArgs {
                id: "quote_invitation".to_string(),
                output_file: tempdir.path().join("quote.pdf"),
                force: false,
                include: Vec::new(),
            })),
            "/api/v1/quote/quote_invitation/download",
        ),
        (
            "credit",
            Commands::Credits(ResourceCommand::Download(DownloadArgs {
                id: "credit_invitation".to_string(),
                output_file: tempdir.path().join("credit.pdf"),
                force: false,
                include: Vec::new(),
            })),
            "/api/v1/credit/credit_invitation/download",
        ),
        (
            "recurring invoice",
            Commands::RecurringInvoices(ResourceCommand::Download(DownloadArgs {
                id: "recurring_invitation".to_string(),
                output_file: tempdir.path().join("recurring.pdf"),
                force: false,
                include: Vec::new(),
            })),
            "/api/v1/recurring_invoice/recurring_invitation/download",
        ),
        (
            "purchase order",
            Commands::PurchaseOrders(ResourceCommand::Download(DownloadArgs {
                id: "po_invitation".to_string(),
                output_file: tempdir.path().join("purchase-order.pdf"),
                force: false,
                include: Vec::new(),
            })),
            "/api/v1/purchase_order/po_invitation/download",
        ),
    ];

    for (label, command, path) in cases {
        let mock = server.mock(|when, then| {
            when.method(GET).path(path);
            then.status(200).body("%PDF-1.7");
        });
        execute_with_config(
            Cli {
                output: OutputFormat::Json,
                command: Some(command),
            },
            config.clone(),
        )
        .await
        .unwrap_or_else(|error| panic!("{label} download failed: {error}"));
        mock.assert();
    }
}

#[tokio::test]
async fn unsupported_resource_routes_fail_before_network() {
    let config = Config::from_values("http://127.0.0.1:9", "token").expect("config");

    let tax_rate_create = execute_with_config(
        Cli {
            output: OutputFormat::Json,
            command: Some(Commands::TaxRates(ResourceCommand::Create(
                ResourceWriteArgs {
                    payload: {
                        let mut args = empty_resource_payload_args();
                        args.name = Some("GST".to_string());
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
    .expect_err("tax rate create is not an official route");
    assert!(
        tax_rate_create.to_string().contains("does not support"),
        "got: {tax_rate_create}"
    );

    let documents_upload = execute_with_config(
        Cli {
            output: OutputFormat::Json,
            command: Some(Commands::Documents(ResourceCommand::Upload(UploadArgs {
                id: "document_1".to_string(),
                files: vec![PathBuf::from("/tmp/koban-no-network-needed.txt")],
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
    .expect_err("documents upload is not an official route");
    assert!(
        documents_upload.to_string().contains("does not support"),
        "got: {documents_upload}"
    );
}

#[test]
fn route_helpers_cover_special_document_and_bulk_paths() {
    assert_eq!(
        resource_update_path(Resource::PurchaseOrders, "po_1"),
        "api/v1/purchase_order/po_1"
    );
    assert_eq!(
        resource_delete_path(Resource::PurchaseOrders, "po_1"),
        "api/v1/purchase_order/po_1"
    );

    let route = resource_action_route(Resource::Products, "product_1", "archive");
    assert_eq!(route.method, HttpMethod::Post);
    assert_eq!(route.path, "api/v1/products/bulk");
    assert!(route.body);
    assert!(route.is_bulk);

    assert!(resource_download_base_path(Resource::Clients).is_none());
}

#[test]
fn unsupported_capability_errors_name_the_rejected_verb() {
    for (resource, capability, label) in [
        (Resource::Templates, ResourceCapability::List, "list"),
        (Resource::Activities, ResourceCapability::Show, "show"),
        (Resource::TaxRates, ResourceCapability::Template, "template"),
        (
            Resource::Locations,
            ResourceCapability::EditTemplate,
            "edit-template",
        ),
        (Resource::TaxRates, ResourceCapability::Create, "create"),
        (Resource::CompanyUsers, ResourceCapability::Update, "update"),
        (Resource::CompanyUsers, ResourceCapability::Delete, "delete"),
        (Resource::Locations, ResourceCapability::Bulk, "bulk"),
    ] {
        let error =
            require_resource_capability(resource, capability).expect_err("capability rejected");
        assert!(error.to_string().contains(label), "{label}: {error}");
    }
}
