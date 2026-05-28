//! Library crate for `koban`.
//!
//! The public surface is intentionally small while the CLI and Invoice Ninja
//! client model settle.

use std::{
    env, fmt, fs,
    io::{self, Read},
    path::{Path, PathBuf},
};

mod update;

use clap::{Args, CommandFactory, Parser, Subcommand, ValueEnum};
use miette::Diagnostic;
use reqwest::StatusCode;
use serde_json::{Value, json};
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
    long_about = "koban is an Invoice Ninja CLI for humans and AI agents.",
    arg_required_else_help = true,
    propagate_version = true,
    term_width = 100,
    after_help = "\
Examples:
  koban statics --output json
  koban clients list --page 1 --per-page 20
  koban clients show <client_id> --output json
  koban invoices create --client-id <client_id> --line-item product_key=Consulting,quantity=1,cost=100 --dry-run
  koban invoices update <invoice_id> --data-file invoice.json --dry-run
  koban invoices delete <invoice_id> --yes
  koban invoices download <invitation_key> --output-file invoice.pdf
  koban invoices template --output json
  koban invoices edit-template <invoice_id> --output json
  koban update --check

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
    /// Show reference data such as countries, currencies, and statuses
    #[command(after_help = "\
Examples:
  koban statics
  koban statics --output json")]
    Statics,

    /// List, show, and inspect clients
    #[command(subcommand)]
    Clients(ResourceCommand),

    /// List, show, create, update, and manage invoices
    #[command(subcommand)]
    Invoices(Box<InvoiceCommand>),

    /// List, show, and inspect payments
    #[command(subcommand)]
    Payments(ResourceCommand),

    /// List, show, and inspect quotes
    #[command(subcommand)]
    Quotes(ResourceCommand),

    /// List, show, and inspect credits
    #[command(subcommand)]
    Credits(ResourceCommand),

    /// List, show, and inspect vendors
    #[command(subcommand)]
    Vendors(ResourceCommand),

    /// List, show, and inspect expenses
    #[command(subcommand)]
    Expenses(ResourceCommand),

    /// List, show, and inspect projects
    #[command(subcommand)]
    Projects(ResourceCommand),

    /// List, show, and inspect tasks
    #[command(subcommand)]
    Tasks(ResourceCommand),

    /// Check or install GitHub release updates
    #[command(after_long_help = "\
Upgrade koban in place when installed from a release tarball. For other install
sources the command prints the right upgrade recipe and exits:

  Nix:       nix profile upgrade koban   (or flake update)
  cargo:     cargo install --git https://github.com/jamesbrink/koban --tag vX.Y.Z --force koban
  Homebrew:  brew upgrade koban

The latest tag is resolved by following the /releases/latest redirect, so this
command does not hit api.github.com and avoids anonymous API rate limits. Use
--nightly to install the rolling nightly build produced from main.")]
    Update {
        /// Report whether an update is available without writing to disk
        #[arg(long)]
        check: bool,

        /// Reinstall or downgrade even when the target matches the current version
        #[arg(long)]
        force: bool,

        /// Install a specific release tag instead of the latest release
        #[arg(long, value_name = "TAG")]
        tag: Option<String>,

        /// Install the rolling nightly build from the nightly GitHub release
        #[arg(long, conflicts_with = "tag")]
        nightly: bool,
    },

    /// Print shell completion scripts
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
    /// List records with pagination, filters, and sorting
    #[command(after_help = "\
Examples:
  koban clients list --page 1 --per-page 20
  koban invoices list --include client --output json")]
    List(ListArgs),

    /// Show one record by hashed ID
    #[command(after_help = "\
Examples:
  koban clients show k9avmeG1P0 --output json
  koban payments show k9avmeG1P0")]
    Show(ShowArgs),

    /// Show the default object template from GET /create
    #[command(
        alias = "blank",
        alias = "new-template",
        after_help = "\
Examples:
  koban clients template --output json
  koban invoices template --include client --output json"
    )]
    Template(TemplateArgs),

    /// Show the editable object template from GET /{id}/edit
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

#[derive(Debug, Subcommand)]
pub enum InvoiceCommand {
    /// List invoices with pagination, filters, and sorting
    #[command(after_help = "\
Examples:
  koban invoices list --page 1 --per-page 20
  koban invoices list --filter status_id=gt:1 --sort 'date|desc' --output json")]
    List(ListArgs),

    /// Show one invoice by hashed ID
    #[command(after_help = "\
Examples:
  koban invoices show k9avmeG1P0 --output json
  koban invoices show k9avmeG1P0 --include client")]
    Show(ShowArgs),

    /// Show the default invoice template from GET /create
    #[command(
        alias = "blank",
        alias = "new-template",
        after_help = "\
Examples:
  koban invoices template --output json
  koban invoices template --include client --output json"
    )]
    Template(TemplateArgs),

    /// Show the editable invoice template from GET /{id}/edit
    #[command(
        name = "edit-template",
        alias = "edit-form",
        after_help = "\
Examples:
  koban invoices edit-template k9avmeG1P0 --output json
  koban invoices edit-template k9avmeG1P0 --include client"
    )]
    EditTemplate(ShowArgs),

    /// Create a draft invoice
    #[command(after_help = "\
Examples:
  koban invoices create --client-id k9avmeG1P0 --line-item product_key=Consulting,quantity=1,cost=100 --dry-run
  koban invoices create --data-file invoice.json --include client
  printf '%s' '{\"client_id\":\"k9avmeG1P0\",\"line_items\":[]}' | koban invoices create --stdin --dry-run")]
    Create(InvoiceWriteArgs),

    /// Update an invoice by hashed ID
    #[command(after_help = "\
Examples:
  koban invoices update k9avmeG1P0 --data-file invoice.json --dry-run
  koban invoices update k9avmeG1P0 --public-notes 'Thanks again' --mark-sent --yes")]
    Update(UpdateInvoiceArgs),

    /// Delete an invoice by hashed ID
    #[command(after_help = "\
Examples:
  koban invoices delete k9avmeG1P0 --dry-run
  koban invoices delete k9avmeG1P0 --yes")]
    Delete(ConfirmableIdArgs),

    /// Run a bulk invoice action
    #[command(after_help = "\
Examples:
  koban invoices bulk --action archive --id inv_1 --id inv_2 --dry-run
  koban invoices bulk --action email --email-type invoice --id inv_1 --yes")]
    Bulk(BulkArgs),

    /// Upload documents to an invoice
    #[command(after_help = "\
Examples:
  koban invoices upload k9avmeG1P0 --file contract.pdf --dry-run
  koban invoices upload k9avmeG1P0 --file contract.pdf --yes")]
    Upload(UploadArgs),

    /// Run a single-invoice action
    #[command(after_help = "\
Examples:
  koban invoices action k9avmeG1P0 --action mark_paid --dry-run
  koban invoices action k9avmeG1P0 --action email --yes")]
    Action(InvoiceActionArgs),

    /// Save an invoice PDF by invitation key
    #[command(after_help = "\
Examples:
  koban invoices download invitation_key --output-file invoice.pdf
  koban invoices download invitation_key --output-file invoice.pdf --force")]
    Download(DownloadArgs),

    /// Save a delivery note PDF by invoice ID
    #[command(
        name = "delivery-note",
        after_help = "\
Examples:
  koban invoices delivery-note k9avmeG1P0 --output-file delivery-note.pdf
  koban invoices delivery-note k9avmeG1P0 --output-file delivery-note.pdf --force"
    )]
    DeliveryNote(DownloadArgs),
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

    /// Raw Invoice Ninja filter in key=value form; repeatable
    #[arg(long = "filter", value_name = "key=value", action = clap::ArgAction::Append)]
    pub filters: Vec<String>,

    /// Raw Invoice Ninja sort expression, such as name|asc or date|desc
    #[arg(long, value_name = "field|asc")]
    pub sort: Option<String>,

    /// Fetch pages until the API returns fewer rows than requested
    #[arg(long)]
    pub all: bool,

    /// Maximum number of rows to emit
    #[arg(long, value_parser = clap::value_parser!(u32).range(1..))]
    pub limit: Option<u32>,
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

#[derive(Debug, Args)]
pub struct DownloadArgs {
    /// Invoice invitation key for `download`, or invoice hashed ID for `delivery-note`
    pub id: String,

    /// File path to write the downloaded PDF to
    #[arg(long = "output-file", short = 'o', value_name = "PATH")]
    pub output_file: PathBuf,

    /// Overwrite the output file if it already exists
    #[arg(long)]
    pub force: bool,

    /// Related resources to include, comma-separated; repeatable
    #[arg(long, value_name = "name[,name]", value_delimiter = ',', action = clap::ArgAction::Append)]
    pub include: Vec<String>,
}

#[derive(Debug, Args)]
pub struct InvoiceWriteArgs {
    #[command(flatten)]
    pub payload: InvoicePayloadArgs,

    #[command(flatten)]
    pub triggers: InvoiceTriggerArgs,

    #[command(flatten)]
    pub safety: WriteSafetyArgs,

    /// Related resources to include, comma-separated; repeatable
    #[arg(long, value_name = "name[,name]", value_delimiter = ',', action = clap::ArgAction::Append)]
    pub include: Vec<String>,
}

#[derive(Debug, Args)]
pub struct UpdateInvoiceArgs {
    /// Invoice Ninja hashed ID
    pub id: String,

    #[command(flatten)]
    pub payload: InvoicePayloadArgs,

    #[command(flatten)]
    pub triggers: InvoiceTriggerArgs,

    #[command(flatten)]
    pub safety: WriteSafetyArgs,

    /// Related resources to include, comma-separated; repeatable
    #[arg(long, value_name = "name[,name]", value_delimiter = ',', action = clap::ArgAction::Append)]
    pub include: Vec<String>,
}

#[derive(Debug, Args)]
pub struct InvoicePayloadArgs {
    /// Raw invoice JSON payload
    #[arg(long, value_name = "JSON", conflicts_with_all = ["data_file", "stdin"])]
    pub data: Option<String>,

    /// Read invoice JSON payload from a file
    #[arg(long = "data-file", value_name = "PATH", conflicts_with_all = ["data", "stdin"])]
    pub data_file: Option<PathBuf>,

    /// Read invoice JSON payload from standard input
    #[arg(long, conflicts_with_all = ["data", "data_file"])]
    pub stdin: bool,

    /// Client hashed ID
    #[arg(long)]
    pub client_id: Option<String>,

    /// Invoice date, usually YYYY-MM-DD
    #[arg(long)]
    pub date: Option<String>,

    /// Due date, usually YYYY-MM-DD
    #[arg(long)]
    pub due_date: Option<String>,

    /// Invoice number
    #[arg(long)]
    pub number: Option<String>,

    /// Purchase order number
    #[arg(long)]
    pub po_number: Option<String>,

    /// Public client-facing notes
    #[arg(long)]
    pub public_notes: Option<String>,

    /// Private internal notes
    #[arg(long)]
    pub private_notes: Option<String>,

    /// Invoice terms
    #[arg(long)]
    pub terms: Option<String>,

    /// Invoice footer
    #[arg(long)]
    pub footer: Option<String>,

    /// Project hashed ID
    #[arg(long)]
    pub project_id: Option<String>,

    /// Line item as comma-separated key=value pairs; repeatable
    #[arg(long = "line-item", value_name = "key=value,...", action = clap::ArgAction::Append)]
    pub line_items: Vec<String>,
}

#[derive(Debug, Args)]
pub struct InvoiceTriggerArgs {
    /// Save and send the invoice email
    #[arg(long)]
    pub send_email: bool,

    /// Save and mark the invoice as sent
    #[arg(long)]
    pub mark_sent: bool,

    /// Save and mark the invoice as paid
    #[arg(long)]
    pub paid: bool,

    /// Amount paid to record with --paid
    #[arg(long)]
    pub amount_paid: Option<String>,

    /// Save and mark the invoice as cancelled
    #[arg(long)]
    pub cancel: bool,

    /// Save the footer as the default footer
    #[arg(long)]
    pub save_default_footer: bool,

    /// Save the terms as the default terms
    #[arg(long)]
    pub save_default_terms: bool,

    /// Retry e-send for the invoice
    #[arg(long)]
    pub retry_e_send: bool,
}

#[derive(Debug, Args)]
pub struct WriteSafetyArgs {
    /// Print the request that would be sent without calling Invoice Ninja
    #[arg(long)]
    pub dry_run: bool,

    /// Confirm a destructive or externally visible mutation
    #[arg(long)]
    pub yes: bool,
}

#[derive(Debug, Args)]
pub struct ConfirmableIdArgs {
    /// Invoice Ninja hashed ID
    pub id: String,

    #[command(flatten)]
    pub safety: WriteSafetyArgs,

    /// Related resources to include, comma-separated; repeatable
    #[arg(long, value_name = "name[,name]", value_delimiter = ',', action = clap::ArgAction::Append)]
    pub include: Vec<String>,
}

#[derive(Debug, Args)]
pub struct BulkArgs {
    /// Bulk action to perform, such as archive, restore, delete, email, or bulk_download
    #[arg(long)]
    pub action: String,

    /// Invoice hashed ID; repeatable
    #[arg(long = "id", value_name = "ID", action = clap::ArgAction::Append, required = true)]
    pub ids: Vec<String>,

    /// Email type for bulk email actions
    #[arg(long)]
    pub email_type: Option<String>,

    #[command(flatten)]
    pub safety: WriteSafetyArgs,

    /// Related resources to include, comma-separated; repeatable
    #[arg(long, value_name = "name[,name]", value_delimiter = ',', action = clap::ArgAction::Append)]
    pub include: Vec<String>,
}

#[derive(Debug, Args)]
pub struct UploadArgs {
    /// Invoice Ninja hashed ID
    pub id: String,

    /// File to upload; repeatable
    #[arg(long = "file", value_name = "PATH", action = clap::ArgAction::Append, required = true)]
    pub files: Vec<PathBuf>,

    #[command(flatten)]
    pub safety: WriteSafetyArgs,

    /// Related resources to include, comma-separated; repeatable
    #[arg(long, value_name = "name[,name]", value_delimiter = ',', action = clap::ArgAction::Append)]
    pub include: Vec<String>,
}

#[derive(Debug, Args)]
pub struct InvoiceActionArgs {
    /// Invoice Ninja hashed ID
    pub id: String,

    /// Invoice action, such as mark_paid, archive, delete, email, or clone_to_quote
    #[arg(long)]
    pub action: String,

    #[command(flatten)]
    pub safety: WriteSafetyArgs,

    /// Related resources to include, comma-separated; repeatable
    #[arg(long, value_name = "name[,name]", value_delimiter = ',', action = clap::ArgAction::Append)]
    pub include: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Resource {
    Clients,
    Invoices,
    Payments,
    Quotes,
    Credits,
    Vendors,
    Expenses,
    Projects,
    Tasks,
}

impl Resource {
    pub fn path(self) -> &'static str {
        match self {
            Self::Clients => "clients",
            Self::Invoices => "invoices",
            Self::Payments => "payments",
            Self::Quotes => "quotes",
            Self::Credits => "credits",
            Self::Vendors => "vendors",
            Self::Expenses => "expenses",
            Self::Projects => "projects",
            Self::Tasks => "tasks",
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

    #[error("list filter is not valid: {value}")]
    #[diagnostic(help("Use key=value, for example `--filter balance=gt:1000`."))]
    InvalidFilter { value: String },

    #[error("invoice payload is not valid: {message}")]
    #[diagnostic(help(
        "Use one raw JSON source (--data, --data-file, or --stdin), or build a payload with guided flags such as --client-id and --line-item."
    ))]
    InvalidPayload { message: String },

    #[error("confirmation required for {operation}")]
    #[diagnostic(help(
        "Review the command with --dry-run, then rerun with --yes when you intentionally want to perform this mutation."
    ))]
    ConfirmationRequired { operation: String },

    #[error("could not write download file: {message}")]
    #[diagnostic(help(
        "Choose a path in an existing directory. Use --force if you intentionally want to overwrite an existing file."
    ))]
    File { message: String },

    #[error("update failed: {message}")]
    #[diagnostic(help(
        "Run `koban update --check` to inspect the latest release without modifying the installed binary."
    ))]
    Update { message: String },
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

    pub fn endpoint(&self, path: &str, query: &[(String, String)]) -> Result<Url> {
        let mut url = self
            .config
            .base_url
            .join(path.trim_start_matches('/'))
            .map_err(|source| KobanError::InvalidEndpoint {
                path: path.to_string(),
                source,
            })?;

        if !query.is_empty() {
            url.query_pairs_mut().extend_pairs(
                query
                    .iter()
                    .map(|(key, value)| (key.as_str(), value.as_str())),
            );
        }

        Ok(url)
    }

    pub async fn get_json(&self, path: &str, query: &[(String, String)]) -> Result<Value> {
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

    pub async fn post_json(
        &self,
        path: &str,
        query: &[(String, String)],
        body: &Value,
    ) -> Result<Value> {
        let url = self.endpoint(path, query)?;
        let endpoint = endpoint_label(&url);
        let response = self
            .http
            .post(url)
            .header("X-API-TOKEN", &self.config.api_token)
            .header("X-Requested-With", REQUESTED_WITH)
            .json(body)
            .send()
            .await
            .map_err(|source| KobanError::Transport {
                message: redact(source.to_string(), &self.config.api_token),
            })?;

        self.json_response(response, endpoint).await
    }

    pub async fn put_json(
        &self,
        path: &str,
        query: &[(String, String)],
        body: &Value,
    ) -> Result<Value> {
        let url = self.endpoint(path, query)?;
        let endpoint = endpoint_label(&url);
        let response = self
            .http
            .put(url)
            .header("X-API-TOKEN", &self.config.api_token)
            .header("X-Requested-With", REQUESTED_WITH)
            .json(body)
            .send()
            .await
            .map_err(|source| KobanError::Transport {
                message: redact(source.to_string(), &self.config.api_token),
            })?;

        self.json_response(response, endpoint).await
    }

    pub async fn delete_json(&self, path: &str, query: &[(String, String)]) -> Result<Value> {
        let url = self.endpoint(path, query)?;
        let endpoint = endpoint_label(&url);
        let response = self
            .http
            .delete(url)
            .header("X-API-TOKEN", &self.config.api_token)
            .header("X-Requested-With", REQUESTED_WITH)
            .send()
            .await
            .map_err(|source| KobanError::Transport {
                message: redact(source.to_string(), &self.config.api_token),
            })?;

        self.json_response(response, endpoint).await
    }

    pub async fn put_multipart(
        &self,
        path: &str,
        query: &[(String, String)],
        files: &[PathBuf],
    ) -> Result<Value> {
        let url = self.endpoint(path, query)?;
        let endpoint = endpoint_label(&url);
        let mut form = reqwest::multipart::Form::new();

        for path in files {
            let bytes = fs::read(path).map_err(|source| KobanError::File {
                message: format!("could not read {}: {source}", path.display()),
            })?;
            let file_name = path
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_else(|| "document".to_string());
            let part = reqwest::multipart::Part::bytes(bytes).file_name(file_name);
            form = form.part("documents[]", part);
        }

        let response = self
            .http
            .put(url)
            .header("X-API-TOKEN", &self.config.api_token)
            .header("X-Requested-With", REQUESTED_WITH)
            .multipart(form)
            .send()
            .await
            .map_err(|source| KobanError::Transport {
                message: redact(source.to_string(), &self.config.api_token),
            })?;

        self.json_response(response, endpoint).await
    }

    pub async fn get_bytes(&self, path: &str, query: &[(String, String)]) -> Result<Vec<u8>> {
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
        let bytes = response
            .bytes()
            .await
            .map_err(|source| KobanError::Decode {
                message: redact(source.to_string(), &self.config.api_token),
            })?;

        if status.is_success() {
            Ok(bytes.to_vec())
        } else {
            let body = String::from_utf8_lossy(&bytes).to_string();
            Err(api_error(status, endpoint, body, &self.config.api_token))
        }
    }

    async fn json_response(&self, response: reqwest::Response, endpoint: String) -> Result<Value> {
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
    let output = cli.output;
    let command = cli.command;

    match command {
        Some(Commands::Update {
            check,
            force,
            tag,
            nightly,
        }) => update::run(check, force, tag, nightly),
        command => {
            let config = Config::from_env()?;
            execute_with_config(Cli { output, command }, config).await
        }
    }
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
        Some(Commands::Invoices(command)) => execute_invoice(&client, output, *command).await,
        Some(Commands::Payments(command)) => {
            execute_resource(&client, output, Resource::Payments, command).await
        }
        Some(Commands::Quotes(command)) => {
            execute_resource(&client, output, Resource::Quotes, command).await
        }
        Some(Commands::Credits(command)) => {
            execute_resource(&client, output, Resource::Credits, command).await
        }
        Some(Commands::Vendors(command)) => {
            execute_resource(&client, output, Resource::Vendors, command).await
        }
        Some(Commands::Expenses(command)) => {
            execute_resource(&client, output, Resource::Expenses, command).await
        }
        Some(Commands::Projects(command)) => {
            execute_resource(&client, output, Resource::Projects, command).await
        }
        Some(Commands::Tasks(command)) => {
            execute_resource(&client, output, Resource::Tasks, command).await
        }
        Some(Commands::Update {
            check,
            force,
            tag,
            nightly,
        }) => update::run(check, force, tag, nightly),
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
        ResourceCommand::List(args) => execute_list(client, output, resource, args).await,
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

async fn execute_invoice(
    client: &ApiClient,
    output: OutputFormat,
    command: InvoiceCommand,
) -> Result<String> {
    match command {
        InvoiceCommand::List(args) => execute_list(client, output, Resource::Invoices, args).await,
        InvoiceCommand::Show(args) => {
            execute_resource(
                client,
                output,
                Resource::Invoices,
                ResourceCommand::Show(args),
            )
            .await
        }
        InvoiceCommand::Template(args) => {
            execute_resource(
                client,
                output,
                Resource::Invoices,
                ResourceCommand::Template(args),
            )
            .await
        }
        InvoiceCommand::EditTemplate(args) => {
            execute_resource(
                client,
                output,
                Resource::Invoices,
                ResourceCommand::EditTemplate(args),
            )
            .await
        }
        InvoiceCommand::Create(args) => execute_invoice_create(client, output, args).await,
        InvoiceCommand::Update(args) => execute_invoice_update(client, output, args).await,
        InvoiceCommand::Delete(args) => execute_invoice_delete(client, output, args).await,
        InvoiceCommand::Bulk(args) => execute_invoice_bulk(client, output, args).await,
        InvoiceCommand::Upload(args) => execute_invoice_upload(client, output, args).await,
        InvoiceCommand::Action(args) => execute_invoice_action(client, output, args).await,
        InvoiceCommand::Download(args) => {
            execute_download(client, "api/v1/invoice", "download", args).await
        }
        InvoiceCommand::DeliveryNote(args) => {
            execute_download(client, "api/v1/invoices", "delivery_note", args).await
        }
    }
}

async fn execute_list(
    client: &ApiClient,
    output: OutputFormat,
    resource: Resource,
    args: ListArgs,
) -> Result<String> {
    let mut base_query = Vec::new();
    push_include(&mut base_query, args.include);
    push_sort(&mut base_query, args.sort);
    push_filters(&mut base_query, args.filters)?;

    if !args.all {
        let mut query = base_query;
        query.push(("page".to_string(), args.page.to_string()));
        query.push(("per_page".to_string(), args.per_page.to_string()));

        let json = client
            .get_json(&format!("api/v1/{}", resource.path()), &query)
            .await?;
        let json = apply_limit_to_response(json, args.limit);
        return render_value(output, Some(resource), &json);
    }

    let json = fetch_all_pages(
        client,
        resource,
        &base_query,
        args.page,
        args.per_page,
        args.limit,
    )
    .await?;
    render_value(output, Some(resource), &json)
}

async fn execute_invoice_create(
    client: &ApiClient,
    output: OutputFormat,
    args: InvoiceWriteArgs,
) -> Result<String> {
    validate_invoice_triggers(&args.triggers)?;
    let body = invoice_payload(args.payload, true, false)?;
    let mut query = Vec::new();
    push_include(&mut query, args.include);
    push_invoice_triggers(&mut query, &args.triggers);

    if args.triggers.requires_confirmation() {
        require_confirmation(
            "invoice create with email, paid, cancel, or retry action",
            &args.safety,
        )?;
    }

    if args.safety.dry_run {
        return render_dry_run("POST", "api/v1/invoices", &query, Some(&body), None);
    }

    let json = client.post_json("api/v1/invoices", &query, &body).await?;
    render_value(output, Some(Resource::Invoices), &json)
}

async fn execute_invoice_update(
    client: &ApiClient,
    output: OutputFormat,
    args: UpdateInvoiceArgs,
) -> Result<String> {
    validate_invoice_triggers(&args.triggers)?;
    let body = invoice_payload(args.payload, false, args.triggers.has_any())?;
    let mut query = Vec::new();
    push_include(&mut query, args.include);
    push_invoice_triggers(&mut query, &args.triggers);

    if args.triggers.requires_confirmation() {
        require_confirmation(
            "invoice update with email, paid, cancel, or retry action",
            &args.safety,
        )?;
    }

    let path = format!("api/v1/invoices/{}", args.id);
    if args.safety.dry_run {
        return render_dry_run("PUT", &path, &query, Some(&body), None);
    }

    let json = client.put_json(&path, &query, &body).await?;
    render_value(output, Some(Resource::Invoices), &json)
}

async fn execute_invoice_delete(
    client: &ApiClient,
    output: OutputFormat,
    args: ConfirmableIdArgs,
) -> Result<String> {
    require_confirmation("invoice delete", &args.safety)?;
    let mut query = Vec::new();
    push_include(&mut query, args.include);

    let path = format!("api/v1/invoices/{}", args.id);
    if args.safety.dry_run {
        return render_dry_run("DELETE", &path, &query, None, None);
    }

    let json = client.delete_json(&path, &query).await?;
    render_value(output, Some(Resource::Invoices), &json)
}

async fn execute_invoice_bulk(
    client: &ApiClient,
    output: OutputFormat,
    args: BulkArgs,
) -> Result<String> {
    require_confirmation("invoice bulk action", &args.safety)?;
    let mut query = Vec::new();
    push_include(&mut query, args.include);

    let mut body = serde_json::Map::new();
    body.insert("action".to_string(), Value::String(args.action));
    body.insert(
        "ids".to_string(),
        Value::Array(args.ids.into_iter().map(Value::String).collect()),
    );
    if let Some(email_type) = args.email_type {
        body.insert("email_type".to_string(), Value::String(email_type));
    }
    let body = Value::Object(body);

    if args.safety.dry_run {
        return render_dry_run("POST", "api/v1/invoices/bulk", &query, Some(&body), None);
    }

    let json = client
        .post_json("api/v1/invoices/bulk", &query, &body)
        .await?;
    render_value(output, Some(Resource::Invoices), &json)
}

async fn execute_invoice_upload(
    client: &ApiClient,
    output: OutputFormat,
    args: UploadArgs,
) -> Result<String> {
    require_confirmation("invoice document upload", &args.safety)?;
    for file in &args.files {
        ensure_upload_file(file)?;
    }

    let mut query = Vec::new();
    push_include(&mut query, args.include);
    let path = format!("api/v1/invoices/{}/upload", args.id);

    if args.safety.dry_run {
        return render_dry_run("PUT", &path, &query, None, Some(&args.files));
    }

    let json = client.put_multipart(&path, &query, &args.files).await?;
    render_value(output, Some(Resource::Invoices), &json)
}

async fn execute_invoice_action(
    client: &ApiClient,
    output: OutputFormat,
    args: InvoiceActionArgs,
) -> Result<String> {
    require_confirmation("invoice action", &args.safety)?;
    validate_path_segment("invoice action", &args.action)?;
    let mut query = Vec::new();
    push_include(&mut query, args.include);
    let path = format!("api/v1/invoices/{}/{}", args.id, args.action);

    if args.safety.dry_run {
        return render_dry_run("GET", &path, &query, None, None);
    }

    let json = client.get_json(&path, &query).await?;
    render_value(output, Some(Resource::Invoices), &json)
}

async fn execute_download(
    client: &ApiClient,
    base_path: &str,
    action: &str,
    args: DownloadArgs,
) -> Result<String> {
    let mut query = Vec::new();
    push_include(&mut query, args.include);
    ensure_download_path(&args.output_file, args.force)?;
    write_download_file(
        &args.output_file,
        client
            .get_bytes(&format!("{base_path}/{}/{action}", args.id), &query)
            .await?,
        args.force,
    )?;
    Ok(format!("Wrote {}", args.output_file.display()))
}

async fn fetch_all_pages(
    client: &ApiClient,
    resource: Resource,
    base_query: &[(String, String)],
    start_page: u32,
    per_page: u32,
    limit: Option<u32>,
) -> Result<Value> {
    let mut page = start_page;
    let mut pages_fetched = 0_u32;
    let mut rows = Vec::new();

    loop {
        let mut query = base_query.to_vec();
        query.push(("page".to_string(), page.to_string()));
        query.push(("per_page".to_string(), per_page.to_string()));

        let json = client
            .get_json(&format!("api/v1/{}", resource.path()), &query)
            .await?;
        let page_rows = response_rows(&json)
            .into_iter()
            .cloned()
            .collect::<Vec<_>>();
        let page_len = page_rows.len();
        pages_fetched += 1;

        for row in page_rows {
            if limit.is_some_and(|limit| rows.len() >= limit as usize) {
                break;
            }
            rows.push(row);
        }

        if page_len < per_page as usize || limit.is_some_and(|limit| rows.len() >= limit as usize) {
            break;
        }
        page += 1;
    }

    Ok(json!({
        "data": rows,
        "meta": {
            "pages_fetched": pages_fetched,
            "limit": limit,
        }
    }))
}

fn write_download_file(path: &Path, bytes: Vec<u8>, force: bool) -> Result<()> {
    ensure_download_path(path, force)?;
    fs::write(path, bytes).map_err(|source| KobanError::File {
        message: source.to_string(),
    })
}

fn ensure_download_path(path: &Path, force: bool) -> Result<()> {
    if path.exists() && !force {
        return Err(KobanError::File {
            message: format!("{} already exists", path.display()),
        });
    }

    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
        && !parent.exists()
    {
        return Err(KobanError::File {
            message: format!("parent directory {} does not exist", parent.display()),
        });
    }

    Ok(())
}

fn ensure_upload_file(path: &Path) -> Result<()> {
    let metadata = fs::metadata(path).map_err(|source| KobanError::File {
        message: format!("could not read {}: {source}", path.display()),
    })?;
    if !metadata.is_file() {
        return Err(KobanError::File {
            message: format!("{} is not a file", path.display()),
        });
    }
    Ok(())
}

fn require_confirmation(operation: &str, safety: &WriteSafetyArgs) -> Result<()> {
    if safety.dry_run || safety.yes {
        Ok(())
    } else {
        Err(KobanError::ConfirmationRequired {
            operation: operation.to_string(),
        })
    }
}

fn validate_invoice_triggers(triggers: &InvoiceTriggerArgs) -> Result<()> {
    if triggers.amount_paid.is_some() && !triggers.paid {
        return Err(KobanError::InvalidPayload {
            message: "--amount-paid requires --paid".to_string(),
        });
    }
    Ok(())
}

fn validate_path_segment(label: &str, value: &str) -> Result<()> {
    let is_safe = !value.is_empty()
        && value != "."
        && value != ".."
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'));

    if is_safe {
        Ok(())
    } else {
        Err(KobanError::InvalidPayload {
            message: format!("{label} must be a safe single path segment"),
        })
    }
}

fn invoice_payload(
    args: InvoicePayloadArgs,
    require_payload: bool,
    allow_empty_for_trigger: bool,
) -> Result<Value> {
    let has_raw = args.data.is_some() || args.data_file.is_some() || args.stdin;
    let has_guided = args.has_guided_fields();

    if has_raw && has_guided {
        return Err(KobanError::InvalidPayload {
            message: "raw JSON input cannot be combined with guided invoice flags".to_string(),
        });
    }

    if let Some(data) = args.data {
        return parse_json_payload(&data);
    }
    if let Some(path) = args.data_file {
        let data = fs::read_to_string(&path).map_err(|source| KobanError::InvalidPayload {
            message: format!("could not read {}: {source}", path.display()),
        })?;
        return parse_json_payload(&data);
    }
    if args.stdin {
        let mut data = String::new();
        io::stdin()
            .read_to_string(&mut data)
            .map_err(|source| KobanError::InvalidPayload {
                message: format!("could not read standard input: {source}"),
            })?;
        return parse_json_payload(&data);
    }

    if has_guided {
        return guided_invoice_payload(args);
    }

    if allow_empty_for_trigger {
        return Ok(json!({}));
    }

    if require_payload {
        Err(KobanError::InvalidPayload {
            message: "create requires JSON input or guided invoice flags".to_string(),
        })
    } else {
        Err(KobanError::InvalidPayload {
            message: "update requires JSON input, guided invoice flags, or a trigger flag"
                .to_string(),
        })
    }
}

fn parse_json_payload(data: &str) -> Result<Value> {
    let value =
        serde_json::from_str::<Value>(data).map_err(|source| KobanError::InvalidPayload {
            message: format!("JSON could not be parsed: {source}"),
        })?;
    if !value.is_object() {
        return Err(KobanError::InvalidPayload {
            message: "invoice payload must be a JSON object".to_string(),
        });
    }
    Ok(value)
}

fn guided_invoice_payload(args: InvoicePayloadArgs) -> Result<Value> {
    let mut body = serde_json::Map::new();
    insert_string(&mut body, "client_id", args.client_id);
    insert_string(&mut body, "date", args.date);
    insert_string(&mut body, "due_date", args.due_date);
    insert_string(&mut body, "number", args.number);
    insert_string(&mut body, "po_number", args.po_number);
    insert_string(&mut body, "public_notes", args.public_notes);
    insert_string(&mut body, "private_notes", args.private_notes);
    insert_string(&mut body, "terms", args.terms);
    insert_string(&mut body, "footer", args.footer);
    insert_string(&mut body, "project_id", args.project_id);

    if !args.line_items.is_empty() {
        let line_items = args
            .line_items
            .into_iter()
            .map(|line_item| parse_line_item(&line_item))
            .collect::<Result<Vec<_>>>()?;
        body.insert("line_items".to_string(), Value::Array(line_items));
    }

    Ok(Value::Object(body))
}

fn insert_string(map: &mut serde_json::Map<String, Value>, key: &str, value: Option<String>) {
    if let Some(value) = value {
        map.insert(key.to_string(), Value::String(value));
    }
}

fn parse_line_item(input: &str) -> Result<Value> {
    let mut item = serde_json::Map::new();
    for part in input.split(',') {
        let Some((key, value)) = part.split_once('=') else {
            return Err(KobanError::InvalidPayload {
                message: format!("line item part `{part}` must use key=value"),
            });
        };
        let key = key.trim();
        if key.is_empty() {
            return Err(KobanError::InvalidPayload {
                message: format!("line item part `{part}` has an empty key"),
            });
        }
        item.insert(key.to_string(), parse_scalar(value.trim()));
    }

    if item.is_empty() {
        return Err(KobanError::InvalidPayload {
            message: "line item cannot be empty".to_string(),
        });
    }

    Ok(Value::Object(item))
}

fn parse_scalar(value: &str) -> Value {
    if value.eq_ignore_ascii_case("true") {
        Value::Bool(true)
    } else if value.eq_ignore_ascii_case("false") {
        Value::Bool(false)
    } else if value.eq_ignore_ascii_case("null") {
        Value::Null
    } else if let Ok(number) = value.parse::<serde_json::Number>() {
        Value::Number(number)
    } else {
        Value::String(value.to_string())
    }
}

impl InvoicePayloadArgs {
    fn has_guided_fields(&self) -> bool {
        self.client_id.is_some()
            || self.date.is_some()
            || self.due_date.is_some()
            || self.number.is_some()
            || self.po_number.is_some()
            || self.public_notes.is_some()
            || self.private_notes.is_some()
            || self.terms.is_some()
            || self.footer.is_some()
            || self.project_id.is_some()
            || !self.line_items.is_empty()
    }
}

impl InvoiceTriggerArgs {
    fn has_any(&self) -> bool {
        self.send_email
            || self.mark_sent
            || self.paid
            || self.amount_paid.is_some()
            || self.cancel
            || self.save_default_footer
            || self.save_default_terms
            || self.retry_e_send
    }

    fn requires_confirmation(&self) -> bool {
        self.send_email
            || self.paid
            || self.amount_paid.is_some()
            || self.cancel
            || self.retry_e_send
    }
}

fn push_invoice_triggers(query: &mut Vec<(String, String)>, triggers: &InvoiceTriggerArgs) {
    push_bool_query(query, "send_email", triggers.send_email);
    push_bool_query(query, "mark_sent", triggers.mark_sent);
    push_bool_query(query, "paid", triggers.paid);
    if let Some(amount_paid) = &triggers.amount_paid {
        query.push(("amount_paid".to_string(), amount_paid.clone()));
    }
    push_bool_query(query, "cancel", triggers.cancel);
    push_bool_query(query, "save_default_footer", triggers.save_default_footer);
    push_bool_query(query, "save_default_terms", triggers.save_default_terms);
    push_bool_query(query, "retry_e_send", triggers.retry_e_send);
}

fn push_bool_query(query: &mut Vec<(String, String)>, key: &str, enabled: bool) {
    if enabled {
        query.push((key.to_string(), "true".to_string()));
    }
}

fn render_dry_run(
    method: &str,
    path: &str,
    query: &[(String, String)],
    body: Option<&Value>,
    files: Option<&[PathBuf]>,
) -> Result<String> {
    let value = json!({
        "dry_run": true,
        "method": method,
        "path": path,
        "query": query.iter().map(|(key, value)| json!({"key": key, "value": value})).collect::<Vec<_>>(),
        "body": body,
        "files": files.map(|files| files.iter().map(|file| file.display().to_string()).collect::<Vec<_>>()),
    });
    serde_json::to_string_pretty(&value).map_err(|source| KobanError::Decode {
        message: source.to_string(),
    })
}

fn push_include(query: &mut Vec<(String, String)>, include: Vec<String>) {
    let include = include
        .into_iter()
        .map(|part| part.trim().to_string())
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();

    if !include.is_empty() {
        query.push(("include".to_string(), include.join(",")));
    }
}

fn push_sort(query: &mut Vec<(String, String)>, sort: Option<String>) {
    if let Some(sort) = sort
        .map(|sort| sort.trim().to_string())
        .filter(|sort| !sort.is_empty())
    {
        query.push(("sort".to_string(), sort));
    }
}

fn push_filters(query: &mut Vec<(String, String)>, filters: Vec<String>) -> Result<()> {
    for filter in filters {
        let Some((key, value)) = filter.split_once('=') else {
            return Err(KobanError::InvalidFilter { value: filter });
        };
        let key = key.trim();
        if key.is_empty() {
            return Err(KobanError::InvalidFilter { value: filter });
        }
        query.push((key.to_string(), value.trim().to_string()));
    }
    Ok(())
}

fn apply_limit_to_response(mut value: Value, limit: Option<u32>) -> Value {
    let Some(limit) = limit else {
        return value;
    };
    let limit = limit as usize;

    if let Some(Value::Array(items)) = value.get_mut("data") {
        items.truncate(limit);
    } else if let Some(items) = value.as_array_mut() {
        items.truncate(limit);
    }

    value
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
                Resource::Quotes => Row::quote(item),
                Resource::Credits => Row::credit(item),
                Resource::Vendors => Row::vendor(item),
                Resource::Expenses => Row::expense(item),
                Resource::Projects => Row::project(item),
                Resource::Tasks => Row::task(item),
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

    fn quote(value: &Value) -> Self {
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
            status: quote_status(value),
            amount: field(value, &["amount"]),
            balance: dash(),
            date: first_date_field(value, &[&["due_date"], &["date"], &["created_at"]]),
        }
    }

    fn credit(value: &Value) -> Self {
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
            status: first_field(value, &[&["status"], &["status_id"]]),
            amount: field(value, &["amount"]),
            balance: field(value, &["balance"]),
            date: first_date_field(value, &[&["date"], &["created_at"]]),
        }
    }

    fn vendor(value: &Value) -> Self {
        Self {
            id: field(value, &["id"]),
            number: field(value, &["number", "vendor_number"]),
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

    fn expense(value: &Value) -> Self {
        Self {
            id: field(value, &["id"]),
            number: field(value, &["number", "transaction_id"]),
            name: first_field(
                value,
                &[
                    &["vendor", "display_name"],
                    &["client", "display_name"],
                    &["category", "name"],
                    &["description"],
                ],
            ),
            status: first_field(value, &[&["status"], &["status_id"]]),
            amount: field(value, &["amount"]),
            balance: dash(),
            date: first_date_field(value, &[&["date"], &["created_at"]]),
        }
    }

    fn project(value: &Value) -> Self {
        Self {
            id: field(value, &["id"]),
            number: field(value, &["number"]),
            name: first_field(
                value,
                &[&["name"], &["client", "display_name"], &["client_id"]],
            ),
            status: first_field(value, &[&["status"], &["status_id"]]),
            amount: field(value, &["budgeted_hours"]),
            balance: dash(),
            date: first_date_field(value, &[&["due_date"], &["created_at"]]),
        }
    }

    fn task(value: &Value) -> Self {
        Self {
            id: field(value, &["id"]),
            number: field(value, &["number"]),
            name: first_field(
                value,
                &[&["description"], &["project", "name"], &["client_id"]],
            ),
            status: first_field(value, &[&["status"], &["status_id"]]),
            amount: field(value, &["time"]),
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

fn quote_status(value: &Value) -> String {
    match value.get("status_id").and_then(Value::as_i64) {
        Some(1) => "draft".to_string(),
        Some(2) => "sent".to_string(),
        Some(3) => "approved".to_string(),
        Some(4) => "converted".to_string(),
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

    Some(if looks_like_reasonable_unix_seconds(timestamp) {
        timestamp
    } else {
        timestamp.div_euclid(1_000)
    })
}

fn format_unix_date(timestamp: i64) -> Option<String> {
    let days = timestamp.div_euclid(86_400);
    let (year, month, day) = civil_from_days(days)?;
    Some(format!("{year:04}-{month:02}-{day:02}"))
}

fn looks_like_reasonable_unix_seconds(timestamp: i64) -> bool {
    let days = timestamp.div_euclid(86_400);
    civil_from_days(days).is_some_and(|(year, _, _)| (1900..=9999).contains(&year))
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
                    ("page".to_string(), "2".to_string()),
                    ("per_page".to_string(), "15".to_string()),
                    ("include".to_string(), "activities,ledger".to_string()),
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
    async fn get_bytes_reports_api_and_transport_errors() {
        let server = MockServer::start();
        let failing_download = server.mock(|when, then| {
            when.method(GET).path("/api/v1/invoice/invitation/download");
            then.status(404)
                .body(r#"{"message":"missing secret-token"}"#);
        });

        let client =
            ApiClient::new(Config::from_values(server.base_url(), "secret-token").expect("config"));
        let error = client
            .get_bytes("api/v1/invoice/invitation/download", &[])
            .await
            .expect_err("api failure");
        let message = error.to_string();
        assert!(matches!(error, KobanError::Api { .. }));
        assert!(message.contains("[REDACTED]"), "got: {message}");
        assert!(!message.contains("secret-token"), "got: {message}");
        failing_download.assert();

        let client = ApiClient::new(
            Config::from_values("http://127.0.0.1:9", "secret-token").expect("config"),
        );
        let error = client
            .get_bytes("api/v1/invoice/invitation/download", &[])
            .await
            .expect_err("transport failure");
        assert!(matches!(error, KobanError::Transport { .. }));
    }

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
    fn upload_file_requires_existing_regular_file() {
        let tempdir = tempfile::tempdir().expect("tempdir");
        let existing = tempdir.path().join("document.txt");
        std::fs::write(&existing, b"upload").expect("seed file");
        ensure_upload_file(&existing).expect("regular files are uploadable");

        let missing = tempdir.path().join("missing.txt");
        let error = ensure_upload_file(&missing).expect_err("missing file");
        assert!(matches!(error, KobanError::File { .. }));

        let error = ensure_upload_file(tempdir.path()).expect_err("directory");
        assert!(matches!(error, KobanError::File { .. }));
    }

    #[test]
    fn invoice_payload_reports_missing_and_malformed_sources() {
        let create_error = invoice_payload(empty_payload_args(), true, false)
            .expect_err("create requires payload");
        assert!(matches!(create_error, KobanError::InvalidPayload { .. }));
        assert!(
            create_error
                .to_string()
                .contains("create requires JSON input")
        );

        let update_error = invoice_payload(empty_payload_args(), false, false)
            .expect_err("update requires payload");
        assert!(
            update_error
                .to_string()
                .contains("update requires JSON input")
        );

        let trigger_only = invoice_payload(empty_payload_args(), false, true).expect("empty body");
        assert_eq!(trigger_only, serde_json::json!({}));

        let mut invalid_json = empty_payload_args();
        invalid_json.data = Some("{not json".to_string());
        let error = invoice_payload(invalid_json, true, false).expect_err("invalid JSON");
        assert!(error.to_string().contains("JSON could not be parsed"));

        let mut missing_file = empty_payload_args();
        missing_file.data_file = Some(PathBuf::from("/tmp/koban-missing-payload.json"));
        let error = invoice_payload(missing_file, true, false).expect_err("missing file");
        assert!(error.to_string().contains("could not read"));
    }

    #[test]
    fn guided_invoice_payload_handles_all_common_fields_and_line_item_scalars() {
        let mut args = empty_payload_args();
        args.client_id = Some("client_1".to_string());
        args.date = Some("2026-05-28".to_string());
        args.due_date = Some("2026-06-28".to_string());
        args.number = Some("INV-1".to_string());
        args.po_number = Some("PO-1".to_string());
        args.public_notes = Some("public".to_string());
        args.private_notes = Some("private".to_string());
        args.terms = Some("Net 30".to_string());
        args.footer = Some("footer".to_string());
        args.project_id = Some("project_1".to_string());
        args.line_items = vec![
            "product_key=Consulting,quantity=1,cost=99.5,is_amount_discount=false,optional=null"
                .to_string(),
        ];

        let payload = invoice_payload(args, true, false).expect("payload");
        assert_eq!(payload["client_id"], "client_1");
        assert_eq!(payload["due_date"], "2026-06-28");
        assert_eq!(payload["line_items"][0]["quantity"], 1);
        assert_eq!(payload["line_items"][0]["cost"], 99.5);
        assert_eq!(payload["line_items"][0]["is_amount_discount"], false);
        assert!(payload["line_items"][0]["optional"].is_null());
    }

    #[test]
    fn line_item_parser_reports_bad_parts() {
        let error = parse_line_item("not-a-pair").expect_err("missing equals");
        assert!(error.to_string().contains("must use key=value"));

        let error = parse_line_item("=value").expect_err("empty key");
        assert!(error.to_string().contains("empty key"));
    }

    #[test]
    fn invoice_trigger_helpers_build_query_and_confirm_risky_actions() {
        let triggers = InvoiceTriggerArgs {
            send_email: true,
            mark_sent: true,
            paid: true,
            amount_paid: Some("12.50".to_string()),
            cancel: true,
            save_default_footer: true,
            save_default_terms: true,
            retry_e_send: true,
        };
        assert!(triggers.has_any());
        assert!(triggers.requires_confirmation());

        let mut query = Vec::new();
        push_invoice_triggers(&mut query, &triggers);
        assert!(query.contains(&("send_email".to_string(), "true".to_string())));
        assert!(query.contains(&("amount_paid".to_string(), "12.50".to_string())));
        assert!(query.contains(&("retry_e_send".to_string(), "true".to_string())));

        let safety = WriteSafetyArgs {
            dry_run: false,
            yes: false,
        };
        let error = require_confirmation("invoice action", &safety).expect_err("confirmation");
        assert!(matches!(error, KobanError::ConfirmationRequired { .. }));

        require_confirmation(
            "invoice action",
            &WriteSafetyArgs {
                dry_run: true,
                yes: false,
            },
        )
        .expect("dry run allowed");

        let invalid = InvoiceTriggerArgs {
            amount_paid: Some("12.50".to_string()),
            ..empty_trigger_args()
        };
        let error = validate_invoice_triggers(&invalid).expect_err("amount requires paid");
        assert!(error.to_string().contains("--amount-paid requires --paid"));
    }

    #[test]
    fn dry_run_output_includes_body_query_and_files() {
        let files = vec![PathBuf::from("/tmp/document.pdf")];
        let output = render_dry_run(
            "PUT",
            "api/v1/invoices/invoice_1/upload",
            &[("include".to_string(), "documents".to_string())],
            Some(&serde_json::json!({"client_id": "client_1"})),
            Some(&files),
        )
        .expect("dry run");
        assert!(output.contains("\"method\": \"PUT\""), "got: {output}");
        assert!(
            output.contains("\"client_id\": \"client_1\""),
            "got: {output}"
        );
        assert!(output.contains("/tmp/document.pdf"), "got: {output}");
    }

    #[test]
    fn path_segment_validation_rejects_route_changing_actions() {
        validate_path_segment("invoice action", "mark_paid").expect("known safe action");
        validate_path_segment("invoice action", "clone-to-quote").expect("hyphens allowed");

        for bad in [
            "",
            ".",
            "..",
            "../clients",
            "mark/paid",
            "email?include=client",
        ] {
            let error = validate_path_segment("invoice action", bad).expect_err("unsafe action");
            assert!(
                error.to_string().contains("safe single path segment"),
                "got: {error}"
            );
        }
    }

    fn empty_payload_args() -> InvoicePayloadArgs {
        InvoicePayloadArgs {
            data: None,
            data_file: None,
            stdin: false,
            client_id: None,
            date: None,
            due_date: None,
            number: None,
            po_number: None,
            public_notes: None,
            private_notes: None,
            terms: None,
            footer: None,
            project_id: None,
            line_items: Vec::new(),
        }
    }

    fn empty_trigger_args() -> InvoiceTriggerArgs {
        InvoiceTriggerArgs {
            send_email: false,
            mark_sent: false,
            paid: false,
            amount_paid: None,
            cancel: false,
            save_default_footer: false,
            save_default_terms: false,
            retry_e_send: false,
        }
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
