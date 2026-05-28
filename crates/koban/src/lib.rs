//! `koban` is a client library for the [Invoice Ninja](https://invoiceninja.com)
//! API, built for humans and AI agents.
//!
//! The library exposes an [`ApiClient`] for talking to an Invoice Ninja instance,
//! a [`Config`] for environment-first credential handling, the [`Resource`]
//! catalogue of API resource families, and a redaction helper for keeping tokens
//! out of logs and errors.
//!
//! ```no_run
//! use koban::{ApiClient, Config};
//!
//! # async fn run() -> koban::Result<()> {
//! let config = Config::from_env()?;
//! let client = ApiClient::new(config);
//! let invoices = client.get_json("api/v1/invoices", &[]).await?;
//! println!("{invoices}");
//! # Ok(())
//! # }
//! ```
//!
//! Token handling is environment-first via `INVOICE_NINJA_API_TOKEN` and the
//! optional `INVOICE_NINJA_BASE_URL`. Enable the `miette` feature to attach
//! diagnostic help text to [`KobanError`].

mod api;
mod config;
mod error;
mod resource;
mod util;

pub use api::ApiClient;
pub use config::{API_TOKEN_ENV, BASE_URL_ENV, Config, DEFAULT_BASE_URL};
pub use error::{KobanError, Result};
pub use resource::Resource;
pub use util::redact;

#[cfg(test)]
mod tests;
