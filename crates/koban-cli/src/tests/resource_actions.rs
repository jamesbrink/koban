use super::*;

#[tokio::test]
async fn resource_actions_hit_official_live_routes_when_confirmed() {
    let server = MockServer::start();
    let config = Config::from_values(server.base_url(), "token").expect("config");

    let recurring_start = server.mock(|when, then| {
        when.method(GET)
            .path("/api/v1/recurring_invoices/recurring_1/start");
        then.status(200)
            .json_body(serde_json::json!({"data": {"id": "recurring_1"}}));
    });
    let product_archive = server.mock(|when, then| {
        when.method(POST).path("/api/v1/products/bulk");
        then.status(200)
            .json_body(serde_json::json!({"data": [{"id": "product_1"}]}));
    });

    execute_with_config(
        Cli {
            output: OutputFormat::Json,
            command: Some(Commands::RecurringInvoices(ResourceCommand::Action(
                ResourceActionArgs {
                    id: "recurring_1".to_string(),
                    action: "start".to_string(),
                    payload: empty_resource_payload_args(),
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
    .expect("recurring invoice action");

    execute_with_config(
        Cli {
            output: OutputFormat::Json,
            command: Some(Commands::Products(ResourceCommand::Action(
                ResourceActionArgs {
                    id: "product_1".to_string(),
                    action: "archive".to_string(),
                    payload: {
                        let mut args = empty_resource_payload_args();
                        args.fields.push("reason=duplicate".to_string());
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
        config,
    )
    .await
    .expect("product bulk action");

    recurring_start.assert();
    product_archive.assert();
}

#[tokio::test]
async fn get_resource_actions_reject_payloads_before_network() {
    let config = Config::from_values("http://localhost:1234", "token").expect("config");

    let error = execute_with_config(
        Cli {
            output: OutputFormat::Json,
            command: Some(Commands::RecurringInvoices(ResourceCommand::Action(
                ResourceActionArgs {
                    id: "recurring_1".to_string(),
                    action: "start".to_string(),
                    payload: {
                        let mut args = empty_resource_payload_args();
                        args.fields.push("reason=now".to_string());
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
    .expect_err("GET action payload should fail");
    assert!(matches!(error, KobanError::InvalidPayload { .. }));
    assert!(
        error.to_string().contains("GET resource actions"),
        "got: {error}"
    );
}

#[tokio::test]
async fn unsupported_pdf_downloads_fail_before_network() {
    let config = Config::from_values("http://localhost:1234", "token").expect("config");
    let tempdir = tempfile::tempdir().expect("tempdir");

    let error = execute_with_config(
        Cli {
            output: OutputFormat::Json,
            command: Some(Commands::Clients(ResourceCommand::Download(DownloadArgs {
                id: "client_invitation".to_string(),
                output_file: tempdir.path().join("client.pdf"),
                force: false,
                include: Vec::new(),
            }))),
        },
        config,
    )
    .await
    .expect_err("clients do not have PDF downloads");
    assert!(
        error.to_string().contains("does not support PDF downloads"),
        "got: {error}"
    );
}
