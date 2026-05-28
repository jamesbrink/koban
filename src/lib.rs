//! Library crate for `koban`.
//!
//! The public surface is intentionally small while the CLI and Invoice Ninja
//! client model settle.

use std::{env, fmt};

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
  koban invoices template --output json
  koban invoices edit-template <invoice_id> --output json

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

    /// Fetch a blank/default object template with GET /create
    #[command(
        alias = "blank",
        alias = "new-template",
        after_help = "\
Examples:
  koban clients template --output json
  koban invoices template --include client --output json"
    )]
    Template(TemplateArgs),

    /// Fetch the editable object template with GET /{id}/edit
    #[command(
        name = "edit-template",
        alias = "edit-form",
        after_help = "\
Examples:
  koban clients edit-template k9avmeG1P0 --output json
  koban payments edit-template k9avmeG1P0"
    )]
    EditTemplate(ShowArgs),
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

#[derive(Debug, Args)]
pub struct TemplateArgs {
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

        let mut base_url =
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

        if !base_url.path().ends_with('/') {
            let mut path = base_url.path().to_string();
            path.push('/');
            base_url.set_path(&path);
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
        ResourceCommand::Template(args) => {
            let mut query = Vec::new();
            push_include(&mut query, args.include);

            let json = client
                .get_json(&format!("api/v1/{}/create", resource.path()), &query)
                .await?;
            render_value(output, Some(resource), &json)
        }
        ResourceCommand::EditTemplate(args) => {
            let mut query = Vec::new();
            push_include(&mut query, args.include);

            let json = client
                .get_json(
                    &format!("api/v1/{}/{}/edit", resource.path(), args.id),
                    &query,
                )
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
        .map(
            |item| match resource.expect("statics are rendered before resource row dispatch") {
                Resource::Clients => Row::client(item),
                Resource::Invoices => Row::invoice(item),
                Resource::Payments => Row::payment(item),
            },
        )
        .collect::<Vec<_>>();

    if rows.is_empty() {
        return format!("No {} found.", resource.map_or("records", Resource::path));
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
            balance: field(value, &["balance"]),
            date: date_field(value, &["created_at"]),
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
            amount: field(value, &["amount"]),
            balance: field(value, &["balance"]),
            date: first_date_field(value, &[&["due_date"], &["date"], &["created_at"]]),
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
            amount: field(value, &["amount"]),
            balance: dash(),
            date: first_date_field(value, &[&["date"], &["created_at"]]),
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

fn first_field(value: &Value, paths: &[&[&str]]) -> String {
    paths
        .iter()
        .map(|path| field(value, path))
        .find(|value| value != "-")
        .unwrap_or_else(dash)
}

fn first_date_field(value: &Value, paths: &[&[&str]]) -> String {
    paths
        .iter()
        .map(|path| date_field(value, path))
        .find(|value| value != "-")
        .unwrap_or_else(dash)
}

fn date_field(value: &Value, path: &[&str]) -> String {
    let Some(value) = nested_value(value, path) else {
        return dash();
    };

    unix_timestamp(value)
        .and_then(format_unix_date)
        .unwrap_or_else(|| field_value(value))
}

fn field(value: &Value, path: &[&str]) -> String {
    nested_value(value, path).map_or_else(dash, field_value)
}

fn nested_value<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut current = value;
    for segment in path {
        if let Ok(index) = segment.parse::<usize>() {
            current = current.get(index)?;
        } else {
            current = current.get(*segment)?;
        }
    }

    Some(current)
}

fn field_value(value: &Value) -> String {
    match value {
        Value::Null => dash(),
        Value::String(value) if value.trim().is_empty() => dash(),
        Value::String(value) => value.clone(),
        Value::Number(value) => value.to_string(),
        Value::Bool(value) => value.to_string(),
        Value::Array(items) => format!("{} items", items.len()),
        Value::Object(map) => format!("{} fields", map.len()),
    }
}

fn unix_timestamp(value: &Value) -> Option<i64> {
    let timestamp = match value {
        Value::Number(number) => number.as_i64()?,
        Value::String(value) => value.trim().parse::<i64>().ok()?,
        _ => return None,
    };

    Some(if timestamp.abs() >= 1_000_000_000_000 {
        timestamp / 1_000
    } else {
        timestamp
    })
}

fn format_unix_date(timestamp: i64) -> Option<String> {
    let days = timestamp.div_euclid(86_400);
    let (year, month, day) = civil_from_days(days)?;
    Some(format!("{year:04}-{month:02}-{day:02}"))
}

fn civil_from_days(days: i64) -> Option<(i32, u32, u32)> {
    let z = days.checked_add(719_468)?;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let mut year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    if month <= 2 {
        year += 1;
    }

    Some((
        year.try_into().ok()?,
        month.try_into().ok()?,
        day.try_into().ok()?,
    ))
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
    use httpmock::{Method::GET, MockServer};

    #[test]
    fn config_defaults_to_invoice_ninja_base_url() {
        let config = Config::from_values(DEFAULT_BASE_URL, "token").expect("config");
        assert_eq!(config.base_url.as_str(), "https://invoicing.co/");
    }

    #[test]
    fn completion_shell_display_uses_documented_names() {
        assert_eq!(CompletionShell::Bash.to_string(), "bash");
        assert_eq!(CompletionShell::Elvish.to_string(), "elvish");
        assert_eq!(CompletionShell::Fish.to_string(), "fish");
        assert_eq!(CompletionShell::Nushell.to_string(), "nushell");
        assert_eq!(CompletionShell::PowerShell.to_string(), "powershell");
        assert_eq!(CompletionShell::Zsh.to_string(), "zsh");
    }

    #[test]
    fn config_preserves_self_hosted_path_prefix_without_trailing_slash() {
        let config =
            Config::from_values("https://example.com/invoiceninja", "token").expect("config");
        let client = ApiClient::new(config);
        let url = client.endpoint("api/v1/clients", &[]).expect("url");
        assert_eq!(
            url.as_str(),
            "https://example.com/invoiceninja/api/v1/clients"
        );
    }

    #[test]
    fn config_preserves_self_hosted_path_prefix_with_trailing_slash() {
        let config =
            Config::from_values("https://example.com/invoiceninja/", "token").expect("config");
        let client = ApiClient::new(config);
        let url = client.endpoint("api/v1/clients", &[]).expect("url");
        assert_eq!(
            url.as_str(),
            "https://example.com/invoiceninja/api/v1/clients"
        );
    }

    #[test]
    fn config_rejects_empty_token() {
        let error = Config::from_values(DEFAULT_BASE_URL, "").expect_err("missing token");
        assert!(matches!(error, KobanError::MissingToken));
    }

    #[test]
    fn config_reports_invalid_base_url() {
        let error = Config::from_values("not a url", "token").expect_err("invalid URL");
        assert!(matches!(error, KobanError::InvalidBaseUrl { .. }));
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
    fn endpoint_accepts_leading_slash_paths() {
        let client =
            ApiClient::new(Config::from_values("http://localhost:1234", "token").expect("config"));
        let url = client.endpoint("/api/v1/statics", &[]).expect("url");
        assert_eq!(url.as_str(), "http://localhost:1234/api/v1/statics");
    }

    #[test]
    fn redacts_token_from_text() {
        assert_eq!(
            redact("bad token secret-token failed", "secret-token"),
            "bad token [REDACTED] failed"
        );
    }

    #[test]
    fn redaction_is_noop_without_token() {
        assert_eq!(redact("nothing to hide", ""), "nothing to hide");
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
        let value = serde_json::json!({
            "data": [{
                "id": "abc",
                "display_name": "Ada",
                "balance": 12.5,
                "created_at": 1744305114
            }]
        });
        let output =
            render_value(OutputFormat::Table, Some(Resource::Clients), &value).expect("table");
        assert!(output.contains("Ada"), "got: {output}");
        assert!(output.contains("12.5"), "got: {output}");
        assert!(output.contains("2025-04-10"), "got: {output}");
    }

    #[test]
    fn table_output_reports_empty_resource_lists() {
        let value = serde_json::json!({"data": []});
        let output =
            render_value(OutputFormat::Table, Some(Resource::Clients), &value).expect("table");
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

        let scalar =
            render_value(OutputFormat::Table, None, &serde_json::json!(true)).expect("table");
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
            "date": "2026-05-16"
        });

        assert_eq!(date_field(&value, &["created_at"]), "2025-04-10");
        assert_eq!(date_field(&value, &["updated_at"]), "2024-11-04");
        assert_eq!(date_field(&value, &["date"]), "2026-05-16");
        assert_eq!(date_field(&value, &["missing"]), "-");
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

    #[test]
    fn api_errors_extract_arrays_objects_and_plain_text() {
        let array_error = api_error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "/api/v1/clients".to_string(),
            r#"{"errors":["name is required",{"email":"invalid"}]}"#.to_string(),
            "token",
        );
        assert!(array_error.to_string().contains("name is required"));

        let object_error = api_error(
            StatusCode::BAD_REQUEST,
            "/api/v1/clients".to_string(),
            r#"{"error":{"message":"bad"}}"#.to_string(),
            "token",
        );
        assert!(object_error.to_string().contains("message"));

        let text_error = api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "/api/v1/clients".to_string(),
            "plain failure".to_string(),
            "token",
        );
        assert!(text_error.to_string().contains("plain failure"));

        let numeric_message = api_error(
            StatusCode::BAD_REQUEST,
            "/api/v1/clients".to_string(),
            r#"{"message":123}"#.to_string(),
            "token",
        );
        assert!(numeric_message.to_string().contains("\"message\":123"));
    }

    #[tokio::test]
    async fn get_json_reports_transport_errors() {
        let client = ApiClient::new(
            Config::from_values("http://127.0.0.1:9", "secret-token").expect("config"),
        );
        let error = client
            .get_json("api/v1/statics", &[])
            .await
            .expect_err("transport failure");
        let message = error.to_string();
        assert!(matches!(error, KobanError::Transport { .. }));
        assert!(!message.contains("secret-token"), "got: {message}");
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
            config,
        )
        .await
        .expect("execute none");
        assert!(output.is_empty());
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
                command: Some(Commands::Invoices(ResourceCommand::Show(ShowArgs {
                    id: "invoice_1".to_string(),
                    include: Vec::new(),
                }))),
            },
            config.clone(),
        )
        .await
        .expect("invoice show");
        assert!(output.contains("sent"), "got: {output}");

        let output = execute_with_config(
            Cli {
                output: OutputFormat::Json,
                command: Some(Commands::Payments(ResourceCommand::List(ListArgs {
                    page: 1,
                    per_page: 20,
                    include: vec!["client".to_string(), " ".to_string()],
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
                command: Some(Commands::Invoices(ResourceCommand::EditTemplate(
                    ShowArgs {
                        id: "invoice_1".to_string(),
                        include: vec!["client".to_string()],
                    },
                ))),
            },
            config,
        )
        .await
        .expect("invoice edit template");
        assert!(output.contains("invoice_1"), "got: {output}");

        clients.assert();
        invoices.assert();
        payments.assert();
        client_template.assert();
        invoice_edit_template.assert();
    }

    #[tokio::test]
    async fn get_json_reports_decode_errors() {
        let server = MockServer::start();
        let invalid_json = server.mock(|when, then| {
            when.method(GET).path("/api/v1/statics");
            then.status(200).body("not json");
        });

        let client =
            ApiClient::new(Config::from_values(server.base_url(), "token").expect("config"));
        let error = client
            .get_json("api/v1/statics", &[])
            .await
            .expect_err("decode failure");
        assert!(matches!(error, KobanError::Decode { .. }));
        invalid_json.assert();
    }
}
