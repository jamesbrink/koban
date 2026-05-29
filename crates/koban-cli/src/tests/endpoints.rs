use super::*;

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
async fn reports_default_to_post() {
    let config = Config::from_values("http://localhost:1234", "token").expect("config");
    let output = execute_with_config(
        Cli {
            output: OutputFormat::Json,
            command: Some(Commands::Reports(EndpointCommand::Run(EndpointArgs {
                endpoint: None,
                method: None,
                payload: {
                    let mut args = empty_resource_payload_args();
                    args.fields.push("date_range=last30".to_string());
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
    .expect("report default dry run");
    assert!(output.contains("\"method\": \"POST\""), "got: {output}");
    assert!(
        output.contains("\"path\": \"api/v1/reports\""),
        "got: {output}"
    );
}

#[tokio::test]
async fn endpoint_invalid_paths_fail_before_network() {
    let config = Config::from_values("http://localhost:1234", "token").expect("config");

    let error = execute_with_config(
        Cli {
            output: OutputFormat::Json,
            command: Some(Commands::Utility(EndpointCommand::Run(EndpointArgs {
                endpoint: Some("../admin".to_string()),
                method: Some(HttpMethod::Get),
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
    .expect_err("route-changing endpoint path should fail");
    assert!(matches!(error, KobanError::InvalidRequest { .. }));
    assert!(
        error.to_string().contains("relative /api/v1 path"),
        "got: {error}"
    );
}

#[tokio::test]
async fn endpoint_get_and_delete_reject_payloads_instead_of_dropping_them() {
    let config = Config::from_values("http://localhost:1234", "token").expect("config");

    for method in [HttpMethod::Get, HttpMethod::Delete] {
        let error = execute_with_config(
            Cli {
                output: OutputFormat::Json,
                command: Some(Commands::Reports(EndpointCommand::Run(EndpointArgs {
                    endpoint: None,
                    method: Some(method),
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
            config.clone(),
        )
        .await
        .expect_err("bodyless endpoint method should reject payload");
        assert!(matches!(error, KobanError::InvalidRequest { .. }));
        assert!(error.to_string().contains(method.label()), "got: {error}");
    }
}

#[tokio::test]
async fn endpoint_post_and_delete_hit_live_routes_when_confirmed() {
    let server = MockServer::start();
    let config = Config::from_values(server.base_url(), "token").expect("config");

    let report_post = server.mock(|when, then| {
        when.method(POST).path("/api/v1/reports/invoices");
        then.status(200)
            .json_body(serde_json::json!({"data": {"name": "invoices"}}));
    });
    let report_delete = server.mock(|when, then| {
        when.method(httpmock::Method::DELETE)
            .path("/api/v1/reports");
        then.status(200)
            .json_body(serde_json::json!({"data": {"deleted": true}}));
    });

    execute_with_config(
        Cli {
            output: OutputFormat::Json,
            command: Some(Commands::Reports(EndpointCommand::Run(EndpointArgs {
                endpoint: Some("reports/invoices".to_string()),
                method: Some(HttpMethod::Post),
                payload: {
                    let mut args = empty_resource_payload_args();
                    args.fields.push("date_range=last30".to_string());
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
    .expect("report post");

    execute_with_config(
        Cli {
            output: OutputFormat::Json,
            command: Some(Commands::Reports(EndpointCommand::Run(EndpointArgs {
                endpoint: None,
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
    .expect("report delete");

    report_post.assert();
    report_delete.assert();
}

#[tokio::test]
async fn utility_rejects_write_methods() {
    let config = Config::from_values("http://localhost:1234", "token").expect("config");

    for method in [HttpMethod::Post, HttpMethod::Put, HttpMethod::Delete] {
        let error = execute_with_config(
            Cli {
                output: OutputFormat::Json,
                command: Some(Commands::Utility(EndpointCommand::Run(EndpointArgs {
                    endpoint: Some("support/ticket_1".to_string()),
                    method: Some(method),
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
        .expect_err("utility writes should be rejected");
        assert!(matches!(error, KobanError::InvalidRequest { .. }));
        assert!(error.to_string().contains("read-only"), "got: {error}");
    }
}

#[tokio::test]
async fn scoped_report_and_chart_endpoint_overrides_allow_payload_methods() {
    let config = Config::from_values("http://localhost:1234", "token").expect("config");

    let report = execute_with_config(
        Cli {
            output: OutputFormat::Json,
            command: Some(Commands::Reports(EndpointCommand::Run(EndpointArgs {
                endpoint: Some("reports/invoices".to_string()),
                method: Some(HttpMethod::Post),
                payload: {
                    let mut args = empty_resource_payload_args();
                    args.fields.push("date_range=last30".to_string());
                    args
                },
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
    .expect("scoped report endpoint dry run");
    assert!(
        report.contains("\"path\": \"api/v1/reports/invoices\""),
        "got: {report}"
    );
    assert!(report.contains("\"method\": \"POST\""), "got: {report}");
    assert!(report.contains("\"date_range\""), "got: {report}");

    let chart = execute_with_config(
        Cli {
            output: OutputFormat::Json,
            command: Some(Commands::Charts(EndpointCommand::Run(EndpointArgs {
                endpoint: Some("charts/totals".to_string()),
                method: Some(HttpMethod::Put),
                payload: {
                    let mut args = empty_resource_payload_args();
                    args.fields.push("currency_id=1".to_string());
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
    .expect("scoped chart endpoint dry run");
    assert!(
        chart.contains("\"path\": \"api/v1/charts/totals\""),
        "got: {chart}"
    );
    assert!(chart.contains("\"method\": \"PUT\""), "got: {chart}");
    assert!(chart.contains("\"currency_id\""), "got: {chart}");
}

#[tokio::test]
async fn endpoint_overrides_reject_write_methods() {
    let config = Config::from_values("http://localhost:1234", "token").expect("config");

    for endpoint in ["clients/client_1/purge", "reports"] {
        let error = execute_with_config(
            Cli {
                output: OutputFormat::Json,
                command: Some(Commands::Reports(EndpointCommand::Run(EndpointArgs {
                    endpoint: Some(endpoint.to_string()),
                    method: Some(HttpMethod::Post),
                    payload: empty_resource_payload_args(),
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
        .expect_err("custom endpoint writes should be rejected");
        assert!(matches!(error, KobanError::InvalidRequest { .. }));
        assert!(error.to_string().contains("read-only"), "got: {error}");
    }
}
