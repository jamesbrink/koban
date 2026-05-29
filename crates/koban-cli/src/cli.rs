use std::{fmt, path::PathBuf};

use clap::{Args, Parser, Subcommand, ValueEnum};

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
  koban products create --name Consulting --price 100 --dry-run
  koban invoices update <invoice_id> --data-file invoice.json --dry-run
  koban search run --field query=acme --dry-run
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
    Table,
    Json,
}

#[derive(Debug, Clone, ValueEnum, PartialEq, Eq)]
pub enum CompletionShell {
    Bash,
    Elvish,
    Fish,
    Nushell,
    #[value(name = "powershell", alias = "power-shell")]
    PowerShell,
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

    /// List, show, and manage locations
    #[command(subcommand)]
    Locations(ResourceCommand),

    /// List, show, and manage products
    #[command(subcommand)]
    Products(ResourceCommand),

    /// List, show, and manage recurring invoices
    #[command(name = "recurring-invoices", subcommand)]
    RecurringInvoices(ResourceCommand),

    /// List, show, and manage purchase orders
    #[command(name = "purchase-orders", subcommand)]
    PurchaseOrders(ResourceCommand),

    /// List, show, and manage recurring expenses
    #[command(name = "recurring-expenses", subcommand)]
    RecurringExpenses(ResourceCommand),

    /// List, show, and manage recurring quotes
    #[command(name = "recurring-quotes", subcommand)]
    RecurringQuotes(ResourceCommand),

    /// List, show, and manage bank transactions
    #[command(name = "bank-transactions", subcommand)]
    BankTransactions(ResourceCommand),

    /// List, show, and manage bank integrations
    #[command(name = "bank-integrations", subcommand)]
    BankIntegrations(ResourceCommand),

    /// List, show, and manage bank transaction rules
    #[command(name = "bank-transaction-rules", subcommand)]
    BankTransactionRules(ResourceCommand),

    /// List, show, and manage group settings
    #[command(name = "group-settings", subcommand)]
    GroupSettings(ResourceCommand),

    /// List, show, and manage expense categories
    #[command(name = "expense-categories", subcommand)]
    ExpenseCategories(ResourceCommand),

    /// List, show, and manage tax rates
    #[command(name = "tax-rates", subcommand)]
    TaxRates(ResourceCommand),

    /// List, show, and manage payment terms
    #[command(name = "payment-terms", subcommand)]
    PaymentTerms(ResourceCommand),

    /// List, show, and manage task schedulers
    #[command(name = "task-schedulers", subcommand)]
    TaskSchedulers(ResourceCommand),

    /// List, show, and manage task statuses
    #[command(name = "task-statuses", subcommand)]
    TaskStatuses(ResourceCommand),

    /// List, show, and inspect activities
    #[command(subcommand)]
    Activities(InspectResourceCommand),

    /// List, show, and inspect system logs
    #[command(name = "system-logs", subcommand)]
    SystemLogs(InspectResourceCommand),

    /// List, show, and manage documents
    #[command(subcommand)]
    Documents(ResourceCommand),

    /// List, show, and manage designs
    #[command(subcommand)]
    Designs(ResourceCommand),

    /// List, show, and manage templates
    #[command(subcommand)]
    Templates(ResourceCommand),

    /// List, show, and manage users
    #[command(subcommand)]
    Users(ResourceCommand),

    /// List, show, and manage companies
    #[command(subcommand)]
    Companies(ResourceCommand),

    /// List, show, and manage company gateways
    #[command(name = "company-gateways", subcommand)]
    CompanyGateways(ResourceCommand),

    /// List and inspect company ledger entries
    #[command(name = "company-ledger", subcommand)]
    CompanyLedger(InspectResourceCommand),

    /// List, show, and manage company users
    #[command(name = "company-users", subcommand)]
    CompanyUsers(ResourceCommand),

    /// List, show, and manage API tokens
    #[command(subcommand)]
    Tokens(ResourceCommand),

    /// List, show, and manage webhooks
    #[command(subcommand)]
    Webhooks(ResourceCommand),

    /// List, show, and manage subscriptions
    #[command(subcommand)]
    Subscriptions(ResourceCommand),

    /// List, show, and manage client gateway tokens
    #[command(name = "client-gateway-tokens", subcommand)]
    ClientGatewayTokens(ResourceCommand),

    /// Query reports
    #[command(subcommand)]
    Reports(EndpointCommand),

    /// Query charts
    #[command(subcommand)]
    Charts(EndpointCommand),

    /// Search across Invoice Ninja records
    #[command(subcommand)]
    Search(EndpointCommand),

    /// Call utility endpoints such as ping, health-check, refresh, and preview
    #[command(subcommand)]
    Utility(EndpointCommand),

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

    /// Create a record from guided fields or JSON
    Create(ResourceWriteArgs),

    /// Update a record by hashed ID
    Update(UpdateResourceArgs),

    /// Delete a record by hashed ID
    Delete(ConfirmableIdArgs),

    /// Run a bulk resource action
    Bulk(BulkArgs),

    /// Upload documents to a resource
    Upload(UploadArgs),

    /// Run a custom resource action
    Action(ResourceActionArgs),

    /// Save a resource PDF by invitation key when the API supports it
    Download(DownloadArgs),
}

#[derive(Debug, Subcommand)]
pub enum InspectResourceCommand {
    List(ListArgs),
    Show(ShowArgs),
}

#[derive(Debug, Subcommand)]
pub enum EndpointCommand {
    /// Send a read-like POST or GET request to a named endpoint
    Run(EndpointArgs),
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
pub struct ResourceWriteArgs {
    #[command(flatten)]
    pub payload: ResourcePayloadArgs,

    #[command(flatten)]
    pub safety: WriteSafetyArgs,

    /// Related resources to include, comma-separated; repeatable
    #[arg(long, value_name = "name[,name]", value_delimiter = ',', action = clap::ArgAction::Append)]
    pub include: Vec<String>,
}

#[derive(Debug, Args)]
pub struct UpdateResourceArgs {
    /// Invoice Ninja hashed ID
    pub id: String,

    #[command(flatten)]
    pub payload: ResourcePayloadArgs,

    #[command(flatten)]
    pub safety: WriteSafetyArgs,

    /// Related resources to include, comma-separated; repeatable
    #[arg(long, value_name = "name[,name]", value_delimiter = ',', action = clap::ArgAction::Append)]
    pub include: Vec<String>,
}

#[derive(Debug, Args)]
pub struct ResourcePayloadArgs {
    #[arg(long, value_name = "JSON", conflicts_with_all = ["data_file", "stdin"])]
    pub data: Option<String>,

    #[arg(long = "data-file", value_name = "PATH", conflicts_with_all = ["data", "stdin"])]
    pub data_file: Option<PathBuf>,

    #[arg(long, conflicts_with_all = ["data", "data_file"])]
    pub stdin: bool,

    #[arg(long = "field", value_name = "key=value", action = clap::ArgAction::Append)]
    pub fields: Vec<String>,

    #[arg(long)]
    pub name: Option<String>,

    #[arg(long)]
    pub number: Option<String>,

    #[arg(long)]
    pub client_id: Option<String>,

    #[arg(long)]
    pub vendor_id: Option<String>,

    /// Project hashed ID
    #[arg(long)]
    pub project_id: Option<String>,

    /// Date, usually YYYY-MM-DD
    #[arg(long)]
    pub date: Option<String>,

    /// Due date, usually YYYY-MM-DD
    #[arg(long)]
    pub due_date: Option<String>,

    /// Amount or rate
    #[arg(long)]
    pub amount: Option<String>,

    /// Price for product-like records
    #[arg(long)]
    pub price: Option<String>,

    /// Quantity for product-like or document-like records
    #[arg(long)]
    pub quantity: Option<String>,

    /// Public client-facing notes
    #[arg(long)]
    pub public_notes: Option<String>,

    /// Private internal notes
    #[arg(long)]
    pub private_notes: Option<String>,

    /// Line item as comma-separated key=value pairs; repeatable
    #[arg(long = "line-item", value_name = "key=value,...", action = clap::ArgAction::Append)]
    pub line_items: Vec<String>,
}

#[derive(Debug, Args)]
pub struct ResourceActionArgs {
    /// Invoice Ninja hashed ID
    pub id: String,

    /// Action path segment, such as archive, restore, email, convert, start, or stop
    #[arg(long)]
    pub action: String,

    #[command(flatten)]
    pub payload: ResourcePayloadArgs,

    #[command(flatten)]
    pub safety: WriteSafetyArgs,

    /// Related resources to include, comma-separated; repeatable
    #[arg(long, value_name = "name[,name]", value_delimiter = ',', action = clap::ArgAction::Append)]
    pub include: Vec<String>,
}

#[derive(Debug, Args)]
pub struct EndpointArgs {
    /// Endpoint path under /api/v1, such as search, reports, ping, or preview
    #[arg(long)]
    pub endpoint: Option<String>,

    /// HTTP method to use
    #[arg(long, value_enum)]
    pub method: Option<HttpMethod>,

    #[command(flatten)]
    pub payload: ResourcePayloadArgs,

    #[command(flatten)]
    pub safety: WriteSafetyArgs,

    /// Related resources to include, comma-separated; repeatable
    #[arg(long, value_name = "name[,name]", value_delimiter = ',', action = clap::ArgAction::Append)]
    pub include: Vec<String>,
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
pub enum HttpMethod {
    /// GET request
    Get,
    /// POST request
    Post,
    /// PUT request
    Put,
    /// DELETE request
    Delete,
}

impl HttpMethod {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Delete => "DELETE",
        }
    }
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
    /// Print the JSON request preview without calling Invoice Ninja
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
