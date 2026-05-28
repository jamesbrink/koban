//! `koban` is a client library for the [Invoice Ninja](https://invoiceninja.com)
//! API, built for humans and AI agents.
//!
//! The library exposes an [`ApiClient`] for talking to an Invoice Ninja instance,
//! a [`Config`] for environment-first credential handling, the [`Resource`]
//! catalogue of API resource families, typed [`models`] for the common
//! resources, and a redaction helper for keeping tokens out of logs and errors.
//!
//! Typed access via the resource accessors and [`models`]:
//!
//! ```no_run
//! use koban::{ApiClient, Config};
//!
//! # async fn run() -> koban::Result<()> {
//! let client = ApiClient::new(Config::from_env()?);
//! let invoices = client.invoices().list().await?;
//! for invoice in &invoices {
//!     println!("{} -> {}", invoice.number, invoice.balance);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! The raw JSON methods ([`ApiClient::get_json`] and friends) remain available as
//! a low-level escape hatch for endpoints or fields the typed layer does not
//! model.
//!
//! Token handling is environment-first via `INVOICE_NINJA_API_TOKEN` and the
//! optional `INVOICE_NINJA_BASE_URL`. Enable the `miette` feature to attach
//! diagnostic help text to [`KobanError`].

mod api;
mod config;
mod error;
pub mod models;
mod resource;
mod typed;
mod util;

pub use api::ApiClient;
pub use config::{API_TOKEN_ENV, BASE_URL_ENV, Config, DEFAULT_BASE_URL};
pub use error::{KobanError, Result};
pub use models::{
    Client, Contact, Credit, Data, Expense, Invoice, InvoiceItem, Meta, Paginated, Pagination,
    Payment, Product, Project, Quote, Task, Vendor,
};
pub use resource::Resource;
pub use typed::Resources;
pub use util::redact;

#[cfg(test)]
mod tests;
