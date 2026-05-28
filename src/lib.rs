//! Library crate for `koban`.
//!
//! The public surface is intentionally small while the CLI and Invoice Ninja
//! client model settle.

use std::{env, ffi::OsString, fmt};

use clap::{Args, CommandFactory, Parser, Subcommand, ValueEnum};
use miette::Diagnostic;
use reqwest::StatusCode;
use serde_json::Value;
use tabled::{Table, Tabled, settings::Style};
use thiserror::Error;
use url::Url;

pub const DEFAULT_BASE_URL: &str = "https://invoicing.co";
pub const API_TOKEN_ENV: &str = "INVOICE_NINJA_API_TOKEN";
pub const BASE_URL_ENV: &str = "INVOICE_NINJA_BASE_URL";
const REQUESTED_WITH: &str = "XMLHttpRequest";

#[derive(Debug, Parser)]
#[command(
    name = "koban",
    version,
    about = "Invoice Ninja from the terminal",
    long_about = "koban is a read-only Invoice Ninja CLI for humans and AI agents.",
    arg_required_else_help = true,
    propagate_version = true,
    next_line_help = true,
    after_help = "\
Examples:
  koban statics --output json
  koban clients list --page 1 --per-page 20
  koban clients show <client_id> --output json

Environment:
  INVOICE_NINJA_API_TOKEN  Required API token
  INVOICE_NINJA_BASE_URL   Optional API base URL, defaults to https://invoicing.co"
)]
pub struct Cli {
    /// Output format for commands that return data
    #[arg(long, value_enum, default_value_t = OutputFormat::Table, global = true)]
    pub output: OutputFormat,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

impl Cli {
    pub fn parse_from_args<I, T>(args: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        Self::parse_from(args)
    }
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
pub enum OutputFormat {
    /// Human-readable tables
    Table,
    /// Machine-readable JSON
    Json,
}

#[derive(Debug, Clone, ValueEnum, PartialEq, Eq)]
pub enum CompletionShell {
    /// Bash completion script
    Bash,
    /// Elvish completion script
    Elvish,
    /// Fish completion script
    Fish,
    /// Nushell completion script
    Nushell,
    /// PowerShell completion script
    #[value(name = "powershell", alias = "power-shell")]
    PowerShell,
    /// Zsh completion script
    Zsh,
}

impl fmt::Display for CompletionShell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Bash => "bash",
            Self::Elvish => "elvish",
            Self::Fish => "fish",
            Self::Nushell => "nushell",
            Self::PowerShell => "powershell",
            Self::Zsh => "zsh",
        };
        f.write_str(value)
    }
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Print account statics such as countries, currencies, and invoice status values
    #[command(after_help = "\
Examples:
  koban statics
  koban statics --output json")]
    Statics,

    /// Read clients
    #[command(subcommand)]
    Clients(ResourceCommand),

    /// Read invoices
    #[command(subcommand)]
    Invoices(ResourceCommand),

    /// Read payments
    #[command(subcommand)]
    Payments(ResourceCommand),

    /// Generate shell completions
    #[command(after_long_help = "\
Setup examples:

  zsh:
    source <(koban completions zsh)

  bash:
    source <(koban completions bash)

  fish:
    koban completions fish | source

  nushell:
    koban completions nushell | save ~/.config/nushell/completions/koban.nu")]
    Completions {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: CompletionShell,
    },
}

#[derive(Debug, Subcommand)]
pub enum ResourceCommand {
    /// List records without mutating Invoice Ninja data
    #[command(after_help = "\
Examples:
  koban clients list --page 1 --per-page 20
  koban invoices list --include client --output json")]
    List(ListArgs),

    /// Show one record by its Invoice Ninja hashed ID
    #[command(after_help = "\
Examples:
  koban clients show k9avmeG1P0 --output json
  koban payments show k9avmeG1P0")]
    Show(ShowArgs),
}

#[derive(Debug, Args)]
pub struct ListArgs {
    /// Page number to request
    #[arg(long, default_value_t = 1, value_parser = clap::value_parser!(u32).range(1..))]
    pub page: u32,

    /// Records per page to request
    #[arg(long, default_value_t = 20, value_parser = clap::value_parser!(u32).range(1..=100))]
    pub per_page: u32,

    /// Related resources to include, comma-separated; repeatable
    #[arg(long, value_name = "name[,name]", value_delimiter = ',', action = clap::ArgAction::Append)]
    pub include: Vec<String>,
}

#[derive(Debug, Args)]
pub struct ShowArgs {
    /// Invoice Ninja hashed ID
    pub id: String,

    /// Related resources to include, comma-separated; repeatable
    #[arg(long, value_name = "name[,name]", value_delimiter = ',', action = clap::ArgAction::Append)]
    pub include: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Resource {
    Clients,
    Invoices,
    Payments,
}

impl Resource {
    pub fn path(self) -> &'static str {
        match self {
            Self::Clients => "clients",
            Self::Invoices => "invoices",
            Self::Payments => "payments",
        }
    }

    fn title(self) -> &'static str {
        match self {
            Self::Clients => "clients",
            Self::Invoices => "invoices",
            Self::Payments => "payments",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub base_url: Url,
    pub api_token: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let api_token = env::var(API_TOKEN_ENV).map_err(|_| KobanError::MissingToken)?;
        let base_url = env::var(BASE_URL_ENV).unwrap_or_else(|_| DEFAULT_BASE_URL.to_string());
        Self::from_values(base_url, api_token)
    }

    pub fn from_values(base_url: impl AsRef<str>, api_token: impl Into<String>) -> Result<Self> {
        let api_token = api_token.into();
        if api_token.trim().is_empty() {
            return Err(KobanError::MissingToken);
        }

        let base_url =
            Url::parse(base_url.as_ref()).map_err(|source| KobanError::InvalidBaseUrl {
                value: base_url.as_ref().to_string(),
                source,
            })?;

        let is_local = matches!(base_url.host_str(), Some("localhost" | "127.0.0.1" | "::1"));
        if base_url.scheme() != "https" && !is_local {
            return Err(KobanError::InsecureBaseUrl {
                value: base_url.to_string(),
            });
        }

        Ok(Self {
            base_url,
            api_token,
        })
    }
}

#[derive(Debug, Error, Diagnostic)]
pub enum KobanError {
    #[error("Invoice Ninja API token is not configured")]
    #[diagnostic(help(
        "Set INVOICE_NINJA_API_TOKEN in your shell, then retry the command. Koban will not call Invoice Ninja without an explicit token."
    ))]
    MissingToken,

    #[error("Invoice Ninja base URL is not valid: {value}")]
    #[diagnostic(help(
        "Set INVOICE_NINJA_BASE_URL to a full URL such as https://invoicing.co or your self-hosted Invoice Ninja domain."
    ))]
    InvalidBaseUrl {
        value: String,
        #[source]
        source: url::ParseError,
    },

    #[error("Invoice Ninja base URL must use HTTPS: {value}")]
    #[diagnostic(help(
        "Use an HTTPS Invoice Ninja URL. Plain HTTP is allowed only for localhost mock tests."
    ))]
    InsecureBaseUrl { value: String },

    #[error("could not build Invoice Ninja API URL for {path}")]
    InvalidEndpoint {
        path: String,
        #[source]
        source: url::ParseError,
    },

    #[error("could not reach Invoice Ninja: {message}")]
    #[diagnostic(help(
        "Check INVOICE_NINJA_BASE_URL, your network, and whether the Invoice Ninja API is reachable."
    ))]
    Transport { message: String },

    #[error("Invoice Ninja returned HTTP {status} for {endpoint}: {message}")]
    Api {
        status: u16,
        endpoint: String,
        message: String,
    },

    #[error("Invoice Ninja returned a response Koban could not decode: {message}")]
    Decode { message: String },
}

pub type Result<T> = std::result::Result<T, KobanError>;

pub struct ApiClient {
    config: Config,
    http: reqwest::Client,
}

impl ApiClient {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            http: reqwest::Client::new(),
        }
    }

    pub fn endpoint(&self, path: &str, query: &[(&str, String)]) -> Result<Url> {
        let mut url = self
            .config
            .base_url
            .join(path.trim_start_matches('/'))
            .map_err(|source| KobanError::InvalidEndpoint {
                path: path.to_string(),
                source,
            })?;

        if !query.is_empty() {
            url.query_pairs_mut()
                .extend_pairs(query.iter().map(|(key, value)| (*key, value.as_str())));
        }

        Ok(url)
    }

    pub async fn get_json(&self, path: &str, query: &[(&str, String)]) -> Result<Value> {
        let url = self.endpoint(path, query)?;
        let endpoint = endpoint_label(&url);
        let response = self
            .http
            .get(url)
            .header("X-API-TOKEN", &self.config.api_token)
            .header("X-Requested-With", REQUESTED_WITH)
            .send()
            .await
            .map_err(|source| KobanError::Transport {
                message: redact(source.to_string(), &self.config.api_token),
            })?;

        let status = response.status();
        let body = response.text().await.map_err(|source| KobanError::Decode {
            message: redact(source.to_string(), &self.config.api_token),
        })?;

        if !status.is_success() {
            return Err(api_error(status, endpoint, body, &self.config.api_token));
        }

        serde_json::from_str(&body).map_err(|source| KobanError::Decode {
            message: redact(source.to_string(), &self.config.api_token),
        })
    }
}

pub async fn execute(cli: Cli) -> Result<String> {
    let config = Config::from_env()?;
    execute_with_config(cli, config).await
}

pub async fn execute_with_config(cli: Cli, config: Config) -> Result<String> {
    let client = ApiClient::new(config);
    let output = cli.output;

    match cli.command {
        Some(Commands::Statics) => {
            let json = client.get_json("api/v1/statics", &[]).await?;
            render_value(output, None, &json)
        }
        Some(Commands::Clients(command)) => {
            execute_resource(&client, output, Resource::Clients, command).await
        }
        Some(Commands::Invoices(command)) => {
            execute_resource(&client, output, Resource::Invoices, command).await
        }
        Some(Commands::Payments(command)) => {
            execute_resource(&client, output, Resource::Payments, command).await
        }
        Some(Commands::Completions { .. }) | None => Ok(String::new()),
    }
}

async fn execute_resource(
    client: &ApiClient,
    output: OutputFormat,
    resource: Resource,
    command: ResourceCommand,
) -> Result<String> {
    match command {
        ResourceCommand::List(args) => {
            let mut query = vec![
                ("page", args.page.to_string()),
                ("per_page", args.per_page.to_string()),
            ];
            push_include(&mut query, args.include);

            let json = client
                .get_json(&format!("api/v1/{}", resource.path()), &query)
                .await?;
            render_value(output, Some(resource), &json)
        }
        ResourceCommand::Show(args) => {
            let mut query = Vec::new();
            push_include(&mut query, args.include);

            let json = client
                .get_json(&format!("api/v1/{}/{}", resource.path(), args.id), &query)
                .await?;
            render_value(output, Some(resource), &json)
        }
    }
}

fn push_include(query: &mut Vec<(&str, String)>, include: Vec<String>) {
    let include = include
        .into_iter()
        .map(|part| part.trim().to_string())
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();

    if !include.is_empty() {
        query.push(("include", include.join(",")));
    }
}

pub fn render_value(
    output: OutputFormat,
    resource: Option<Resource>,
    value: &Value,
) -> Result<String> {
    match output {
        OutputFormat::Json => {
            serde_json::to_string_pretty(value).map_err(|source| KobanError::Decode {
                message: source.to_string(),
            })
        }
        OutputFormat::Table => Ok(render_table(resource, value)),
    }
}

fn render_table(resource: Option<Resource>, value: &Value) -> String {
    if resource.is_none() {
        return render_statics_table(value);
    }

    let rows = response_rows(value)
        .into_iter()
        .map(|item| match resource {
            Some(Resource::Clients) => Row::client(item),
            Some(Resource::Invoices) => Row::invoice(item),
            Some(Resource::Payments) => Row::payment(item),
            None => Row::statics(item),
        })
        .collect::<Vec<_>>();

    if rows.is_empty() {
        return format!("No {} found.", resource.map_or("records", Resource::title));
    }

    let mut table = Table::new(rows);
    table.with(Style::rounded());
    table.to_string()
}

fn render_statics_table(value: &Value) -> String {
    let Some(map) = value.as_object() else {
        return "No statics found.".to_string();
    };

    let rows = map
        .iter()
        .map(|(name, value)| StaticRow {
            name: name.clone(),
            kind: value_kind(value).to_string(),
            entries: value_len(value),
        })
        .collect::<Vec<_>>();

    if rows.is_empty() {
        return "No statics found.".to_string();
    }

    let mut table = Table::new(rows);
    table.with(Style::rounded());
    table.to_string()
}

fn value_kind(value: &Value) -> &'static str {
    match value {
        Value::Array(_) => "array",
        Value::Object(_) => "object",
        Value::String(_) => "string",
        Value::Number(_) => "number",
        Value::Bool(_) => "boolean",
        Value::Null => "null",
    }
}

fn value_len(value: &Value) -> usize {
    match value {
        Value::Array(items) => items.len(),
        Value::Object(map) => map.len(),
        _ => 1,
    }
}

fn response_rows(value: &Value) -> Vec<&Value> {
    match value.get("data") {
        Some(Value::Array(items)) => items.iter().collect(),
        Some(item @ Value::Object(_)) => vec![item],
        _ => match value {
            Value::Array(items) => items.iter().collect(),
            Value::Object(_) => vec![value],
            _ => Vec::new(),
        },
    }
}

#[derive(Tabled)]
struct StaticRow {
    name: String,
    kind: String,
    entries: usize,
}

#[derive(Tabled)]
struct Row {
    id: String,
    number: String,
    name: String,
    status: String,
    amount: String,
    balance: String,
    date: String,
}

impl Row {
    fn client(value: &Value) -> Self {
        Self {
            id: field(value, &["id"]),
            number: field(value, &["number", "client_number"]),
            name: first_field(
                value,
                &[&["display_name"], &["name"], &["contacts", "0", "email"]],
            ),
            status: field(value, &["status"]),
            amount: dash(),
            balance: moneyish(value, &["balance"]),
            date: field(value, &["created_at"]),
        }
    }

    fn invoice(value: &Value) -> Self {
        Self {
            id: field(value, &["id"]),
            number: field(value, &["number"]),
            name: first_field(
                value,
                &[
                    &["client", "display_name"],
                    &["client", "name"],
                    &["client_id"],
                ],
            ),
            status: invoice_status(value),
            amount: moneyish(value, &["amount"]),
            balance: moneyish(value, &["balance"]),
            date: first_field(value, &[&["due_date"], &["date"], &["created_at"]]),
        }
    }

    fn payment(value: &Value) -> Self {
        Self {
            id: field(value, &["id"]),
            number: field(value, &["number"]),
            name: first_field(
                value,
                &[
                    &["client", "display_name"],
                    &["client", "name"],
                    &["client_id"],
                ],
            ),
            status: field(value, &["status"]),
            amount: moneyish(value, &["amount"]),
            balance: dash(),
            date: first_field(value, &[&["date"], &["created_at"]]),
        }
    }

    fn statics(value: &Value) -> Self {
        Self {
            id: field(value, &["id"]),
            number: dash(),
            name: first_field(value, &[&["name"], &["value"], &["label"]]),
            status: dash(),
            amount: dash(),
            balance: dash(),
            date: dash(),
        }
    }
}

fn invoice_status(value: &Value) -> String {
    match value.get("status_id").and_then(Value::as_i64) {
        Some(1) => "draft".to_string(),
        Some(2) => "sent".to_string(),
        Some(3) => "partially paid".to_string(),
        Some(4) => "paid".to_string(),
        Some(5) => "cancelled".to_string(),
        Some(6) => "reversed".to_string(),
        Some(-1) => "overdue".to_string(),
        Some(-2) => "unpaid".to_string(),
        _ => first_field(value, &[&["status"], &["status_id"]]),
    }
}

fn moneyish(value: &Value, path: &[&str]) -> String {
    let raw = field(value, path);
    if raw == "-" {
        return raw;
    }
    raw
}

fn first_field(value: &Value, paths: &[&[&str]]) -> String {
    paths
        .iter()
        .map(|path| field(value, path))
        .find(|value| value != "-")
        .unwrap_or_else(dash)
}

fn field(value: &Value, path: &[&str]) -> String {
    let mut current = value;
    for segment in path {
        if let Ok(index) = segment.parse::<usize>() {
            current = match current.get(index) {
                Some(value) => value,
                None => return dash(),
            };
        } else {
            current = match current.get(*segment) {
                Some(value) => value,
                None => return dash(),
            };
        }
    }

    match current {
        Value::Null => dash(),
        Value::String(value) if value.trim().is_empty() => dash(),
        Value::String(value) => value.clone(),
        Value::Number(value) => value.to_string(),
        Value::Bool(value) => value.to_string(),
        Value::Array(items) => format!("{} items", items.len()),
        Value::Object(map) => format!("{} fields", map.len()),
    }
}

fn dash() -> String {
    "-".to_string()
}

fn api_error(status: StatusCode, endpoint: String, body: String, token: &str) -> KobanError {
    let message = serde_json::from_str::<Value>(&body)
        .ok()
        .and_then(|value| {
            first_json_string(&value, &["message"])
                .or_else(|| first_json_string(&value, &["error"]))
                .or_else(|| first_json_string(&value, &["errors"]))
        })
        .unwrap_or(body);

    KobanError::Api {
        status: status.as_u16(),
        endpoint,
        message: redact(message, token),
    }
}

fn first_json_string(value: &Value, path: &[&str]) -> Option<String> {
    let mut current = value;
    for segment in path {
        current = current.get(*segment)?;
    }
    match current {
        Value::String(value) => Some(value.clone()),
        Value::Array(items) => Some(
            items
                .iter()
                .map(|item| match item {
                    Value::String(value) => value.clone(),
                    other => other.to_string(),
                })
                .collect::<Vec<_>>()
                .join(", "),
        ),
        Value::Object(_) => Some(current.to_string()),
        _ => None,
    }
}

fn endpoint_label(url: &Url) -> String {
    match url.query() {
        Some(query) => format!("{}?{query}", url.path()),
        None => url.path().to_string(),
    }
}

pub fn redact(input: impl Into<String>, token: &str) -> String {
    let input = input.into();
    if token.trim().is_empty() {
        input
    } else {
        input.replace(token, "[REDACTED]")
    }
}

pub fn command() -> clap::Command {
    Cli::command()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_defaults_to_invoice_ninja_base_url() {
        let config = Config::from_values(DEFAULT_BASE_URL, "token").expect("config");
        assert_eq!(config.base_url.as_str(), "https://invoicing.co/");
    }

    #[test]
    fn config_rejects_empty_token() {
        let error = Config::from_values(DEFAULT_BASE_URL, "").expect_err("missing token");
        assert!(matches!(error, KobanError::MissingToken));
    }

    #[test]
    fn config_rejects_non_local_http() {
        let error = Config::from_values("http://example.com", "token").expect_err("insecure URL");
        assert!(matches!(error, KobanError::InsecureBaseUrl { .. }));
    }

    #[test]
    fn endpoint_builds_pagination_and_include_query() {
        let client =
            ApiClient::new(Config::from_values("http://localhost:1234", "token").expect("config"));
        let url = client
            .endpoint(
                "api/v1/clients",
                &[
                    ("page", "2".to_string()),
                    ("per_page", "15".to_string()),
                    ("include", "activities,ledger".to_string()),
                ],
            )
            .expect("url");
        assert_eq!(
            url.as_str(),
            "http://localhost:1234/api/v1/clients?page=2&per_page=15&include=activities%2Cledger"
        );
    }

    #[test]
    fn redacts_token_from_text() {
        assert_eq!(
            redact("bad token secret-token failed", "secret-token"),
            "bad token [REDACTED] failed"
        );
    }

    #[test]
    fn json_output_preserves_api_shape() {
        let value = serde_json::json!({"data": [{"id": "abc", "display_name": "Ada"}]});
        let output =
            render_value(OutputFormat::Json, Some(Resource::Clients), &value).expect("json");
        assert!(output.contains("\"display_name\": \"Ada\""));
    }

    #[test]
    fn table_output_renders_client_fields() {
        let value =
            serde_json::json!({"data": [{"id": "abc", "display_name": "Ada", "balance": 12.5}]});
        let output =
            render_value(OutputFormat::Table, Some(Resource::Clients), &value).expect("table");
        assert!(output.contains("Ada"), "got: {output}");
        assert!(output.contains("12.5"), "got: {output}");
    }

    #[test]
    fn table_output_renders_statics_summary() {
        let value = serde_json::json!({
            "countries": [{"id": "840", "name": "United States"}],
            "currencies": [{"id": "1", "name": "US Dollar"}]
        });
        let output = render_value(OutputFormat::Table, None, &value).expect("table");
        assert!(output.contains("countries"), "got: {output}");
        assert!(output.contains("currencies"), "got: {output}");
        assert!(output.contains("array"), "got: {output}");
    }

    #[test]
    fn api_errors_redact_tokens() {
        let error = api_error(
            StatusCode::UNAUTHORIZED,
            "/api/v1/clients".to_string(),
            r#"{"message":"token secret-token is bad"}"#.to_string(),
            "secret-token",
        );
        let message = error.to_string();
        assert!(message.contains("[REDACTED]"), "got: {message}");
        assert!(!message.contains("secret-token"), "got: {message}");
    }
}
