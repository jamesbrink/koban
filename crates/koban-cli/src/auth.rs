//! `koban auth` — store, inspect, and remove Invoice Ninja credentials.
//!
//! Login is dispatched before any token resolution (it is how a token is
//! obtained), so it never requires an existing credential.

use std::io::{IsTerminal, Read};

use koban::{ApiClient, Config, DEFAULT_BASE_URL, KobanError, Result};
use serde_json::{Value, json};

use crate::cli::{AuthCommand, AuthLoginArgs, OutputFormat};
use crate::config_store::{self, TokenSource};

pub async fn execute(output: OutputFormat, command: AuthCommand) -> Result<String> {
    match command {
        AuthCommand::Login(args) => login(output, args).await,
        AuthCommand::Logout => logout(output),
        AuthCommand::Status => status(output),
    }
}

async fn login(output: OutputFormat, args: AuthLoginArgs) -> Result<String> {
    let token = resolve_input_token(args.token)?;
    if token.is_empty() {
        return Err(KobanError::MissingToken);
    }

    // Determine the effective base URL: an explicit --base-url, else the
    // previously stored one, else the default. We verify against this exact URL
    // and persist it, so a re-login can't report success against one host while
    // later commands target another.
    let base_url = match args.base_url.clone() {
        Some(base_url) => base_url,
        None => config_store::stored_base_url()?.unwrap_or_else(|| DEFAULT_BASE_URL.to_string()),
    };
    let config = Config::from_values(&base_url, token.clone())?;

    if !args.no_verify {
        // A cheap, safe, read-only call that requires a valid token.
        ApiClient::new(config)
            .get_json("api/v1/statics", &[])
            .await?;
    }

    let path = config_store::save(Some(base_url.clone()), &token, args.keychain)?;
    let storage = if args.keychain {
        "keychain"
    } else {
        "config file"
    };

    match output {
        OutputFormat::Json => to_json(&json!({
            "status": "logged_in",
            "storage": storage,
            "verified": !args.no_verify,
            "base_url": base_url,
            "config_path": path.display().to_string(),
        })),
        OutputFormat::Table => Ok(format!(
            "Stored Invoice Ninja token in the {storage}. Config: {}",
            path.display()
        )),
    }
}

fn logout(output: OutputFormat) -> Result<String> {
    let removed = config_store::clear()?;

    match output {
        OutputFormat::Json => to_json(&json!({
            "status": if removed { "logged_out" } else { "no_credentials" },
            "removed": removed,
        })),
        OutputFormat::Table => Ok(if removed {
            "Removed stored Invoice Ninja credentials.".to_string()
        } else {
            "No stored credentials to remove.".to_string()
        }),
    }
}

fn status(output: OutputFormat) -> Result<String> {
    let status = config_store::status()?;
    let authenticated = status.source != TokenSource::None;

    match output {
        OutputFormat::Json => to_json(&json!({
            "authenticated": authenticated,
            "source": status.source.label(),
            "base_url": status.base_url,
            "config_path": status.config_path.display().to_string(),
        })),
        OutputFormat::Table => Ok(if authenticated {
            format!(
                "Authenticated via {} (base URL {}).",
                status.source.label(),
                status.base_url
            )
        } else {
            "Not authenticated. Run `koban auth login` or set INVOICE_NINJA_API_TOKEN.".to_string()
        }),
    }
}

/// Obtain the token from `--token`, piped stdin, or an interactive prompt.
fn resolve_input_token(token: Option<String>) -> Result<String> {
    if let Some(token) = token {
        return Ok(token.trim().to_string());
    }

    let stdin = std::io::stdin();
    if !stdin.is_terminal() {
        // Piped input (the agent/CI path): read stdin and use the first line.
        let mut buffer = String::new();
        stdin
            .lock()
            .read_to_string(&mut buffer)
            .map_err(|source| KobanError::Credential {
                message: format!("could not read token from stdin: {source}"),
            })?;
        return Ok(buffer.lines().next().unwrap_or_default().trim().to_string());
    }

    // Interactive terminal: prompt without echoing the secret.
    let token = rpassword::prompt_password("Invoice Ninja API token: ").map_err(|source| {
        KobanError::Credential {
            message: format!("could not read token: {source}"),
        }
    })?;
    Ok(token.trim().to_string())
}

fn to_json(value: &Value) -> Result<String> {
    serde_json::to_string_pretty(value).map_err(|source| KobanError::Credential {
        message: format!("could not render JSON: {source}"),
    })
}
