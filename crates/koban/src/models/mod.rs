//! Typed domain models for Invoice Ninja resources.
//!
//! Invoice Ninja wraps responses in a `data` envelope (`{ "data": ... }` for a
//! single record, `{ "data": [...], "meta": {...} }` for a collection). The
//! [`Data`] and [`Paginated`] envelopes model that shape, and the entity structs
//! ([`Invoice`], [`Client`], ...) model the common fields of each resource.
//!
//! Every entity is forward-compatible: each field defaults when absent, numeric
//! fields tolerate the API returning numbers or strings, and any field that is
//! not modelled is preserved in the `extra` map. Adding a field to one of these
//! structs is therefore never a breaking change, and unmodelled resources can be
//! read through [`ApiClient::resource`](crate::ApiClient::resource) with a
//! caller-defined type or `serde_json::Value`.

mod entities;

pub use entities::{
    Client, Contact, Credit, Expense, Invoice, InvoiceItem, Payment, Product, Project, Quote, Task,
    Vendor,
};

use serde::{Deserialize, Serialize};

/// A single-record envelope: `{ "data": T }`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Data<T> {
    /// The wrapped record.
    pub data: T,
}

/// A collection envelope: `{ "data": [T], "meta": { "pagination": ... } }`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paginated<T> {
    /// The records on this page.
    #[serde(default = "Vec::new")]
    pub data: Vec<T>,
    /// Pagination metadata, when present.
    #[serde(default)]
    pub meta: Option<Meta>,
}

/// Response metadata accompanying a [`Paginated`] collection.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Meta {
    /// Pagination counters for the collection.
    pub pagination: Option<Pagination>,
}

/// Pagination counters returned in [`Meta`].
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Pagination {
    /// Total number of records across all pages.
    pub total: Option<u64>,
    /// Number of records on the current page.
    pub count: Option<u64>,
    /// Records requested per page.
    pub per_page: Option<u64>,
    /// The current page number.
    pub current_page: Option<u64>,
    /// Total number of pages.
    pub total_pages: Option<u64>,
}

/// Deserialization helpers that tolerate the Invoice Ninja API returning numeric
/// fields as either JSON numbers or strings.
pub(crate) mod de {
    use serde::{Deserialize, Deserializer};
    use serde_json::Value;

    /// Deserialize an `f64` from a JSON number, a numeric string, or null.
    pub(crate) fn f64_flexible<'de, D>(deserializer: D) -> Result<f64, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(match Value::deserialize(deserializer)? {
            Value::Number(number) => number.as_f64().unwrap_or_default(),
            Value::String(text) => text.trim().parse().unwrap_or_default(),
            _ => 0.0,
        })
    }

    /// Deserialize an optional `i64` (e.g. a Unix timestamp) from a JSON number,
    /// a numeric string, or null.
    pub(crate) fn i64_opt_flexible<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(match Value::deserialize(deserializer)? {
            Value::Number(number) => number.as_i64(),
            Value::String(text) => text.trim().parse().ok(),
            _ => None,
        })
    }
}
