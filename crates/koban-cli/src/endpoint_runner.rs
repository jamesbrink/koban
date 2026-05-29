use koban::{ApiClient, KobanError, Result};

use crate::{
    cli::{EndpointArgs, EndpointCommand, HttpMethod, OutputFormat},
    commands::push_include,
    invoice::{render_dry_run, require_confirmation},
    payload::resource_payload,
    render::render_value,
};

pub(crate) async fn execute_endpoint(
    client: &ApiClient,
    output: OutputFormat,
    default_endpoint: &str,
    command: EndpointCommand,
) -> Result<String> {
    match command {
        EndpointCommand::Run(args) => {
            execute_endpoint_run(client, output, default_endpoint, args).await
        }
    }
}

async fn execute_endpoint_run(
    client: &ApiClient,
    output: OutputFormat,
    default_endpoint: &str,
    args: EndpointArgs,
) -> Result<String> {
    let custom_endpoint = args.endpoint.is_some();
    let endpoint = args
        .endpoint
        .unwrap_or_else(|| default_endpoint.to_string());
    validate_endpoint_path(&endpoint)?;
    let method = args
        .method
        .unwrap_or_else(|| default_method(default_endpoint));
    if (default_endpoint == "ping" || custom_endpoint) && !matches!(method, HttpMethod::Get) {
        return Err(KobanError::InvalidPayload {
            message: "custom and utility endpoint runners are read-only; use --method get"
                .to_string(),
        });
    }
    let path = format!("api/v1/{endpoint}");
    let body = resource_payload(
        args.payload,
        matches!(method, HttpMethod::Post | HttpMethod::Put),
    )?;
    let has_body = body.as_object().is_some_and(|body| !body.is_empty());
    if has_body && matches!(method, HttpMethod::Get | HttpMethod::Delete) {
        return Err(KobanError::InvalidPayload {
            message: format!(
                "{} endpoint commands do not send request bodies; use --method post or --method put for payload fields",
                method.label()
            ),
        });
    }
    let mut query = Vec::new();
    push_include(&mut query, args.include);
    if args.safety.dry_run {
        return render_dry_run(
            method.label(),
            &path,
            &query,
            has_body.then_some(&body),
            None,
        );
    }

    if !matches!(method, HttpMethod::Get) {
        let action = format!("endpoint {}", method.label().to_ascii_lowercase());
        require_confirmation(&action, &args.safety)?;
    }

    let json = match method {
        HttpMethod::Get => client.get_json(&path, &query).await?,
        HttpMethod::Post => client.post_json(&path, &query, &body).await?,
        HttpMethod::Put => client.put_json(&path, &query, &body).await?,
        HttpMethod::Delete => client.delete_json(&path, &query).await?,
    };
    render_value(output, None, &json)
}

fn default_method(default_endpoint: &str) -> HttpMethod {
    if default_endpoint == "ping" {
        HttpMethod::Get
    } else {
        HttpMethod::Post
    }
}

fn validate_endpoint_path(path: &str) -> Result<()> {
    let is_safe = !path.is_empty()
        && !path.starts_with('/')
        && !path.contains("..")
        && path
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'/'));

    if is_safe {
        Ok(())
    } else {
        Err(KobanError::InvalidPayload {
            message: "endpoint must be a relative /api/v1 path".to_string(),
        })
    }
}
