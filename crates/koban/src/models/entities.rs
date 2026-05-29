//! Typed structs for the most commonly used Invoice Ninja resources.
//!
//! Each struct models the stable, frequently used fields of a resource. Every
//! struct is `#[serde(default)]`, so any field missing from a response falls back
//! to its default rather than failing to deserialize, and every struct keeps an
//! `extra` map that captures fields the struct does not name (for example,
//! timestamps that vary by deployment, or resource-specific columns). Adding a
//! field here is therefore always backwards compatible.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// Shorthand for the flatten map that captures unmodelled fields.
type Extra = Map<String, Value>;

/// A client or vendor contact.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Contact {
    /// Contact identifier.
    pub id: String,
    /// Contact first name.
    pub first_name: String,
    /// Contact last name.
    pub last_name: String,
    /// Contact email address.
    pub email: String,
    /// Contact phone number.
    pub phone: String,
    /// Whether this is the primary contact.
    pub is_primary: bool,
    /// Fields not modelled above, preserved verbatim.
    #[serde(flatten)]
    pub extra: Extra,
}

/// An Invoice Ninja client.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Client {
    /// Client identifier.
    pub id: String,
    /// Internal client name.
    pub name: String,
    /// Display name shown in the UI.
    pub display_name: String,
    /// Client number.
    pub number: String,
    /// Outstanding balance.
    #[serde(deserialize_with = "crate::models::de::f64_flexible")]
    pub balance: f64,
    /// Total paid to date.
    #[serde(deserialize_with = "crate::models::de::f64_flexible")]
    pub paid_to_date: f64,
    /// Primary phone number.
    pub phone: String,
    /// Website URL.
    pub website: String,
    /// VAT/tax number.
    pub vat_number: String,
    /// Notes shown to the client.
    pub public_notes: String,
    /// Internal notes.
    pub private_notes: String,
    /// Client contacts.
    pub contacts: Vec<Contact>,
    /// Whether the client has been soft-deleted.
    pub is_deleted: bool,
    /// Creation timestamp (Unix seconds), when present.
    #[serde(deserialize_with = "crate::models::de::i64_opt_flexible")]
    pub created_at: Option<i64>,
    /// Fields not modelled above, preserved verbatim.
    #[serde(flatten)]
    pub extra: Extra,
}

/// A single line item on an invoice, quote, or credit.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct InvoiceItem {
    /// Product key/identifier.
    pub product_key: String,
    /// Line item notes/description.
    pub notes: String,
    /// Unit cost.
    #[serde(deserialize_with = "crate::models::de::f64_flexible")]
    pub cost: f64,
    /// Quantity.
    #[serde(deserialize_with = "crate::models::de::f64_flexible")]
    pub quantity: f64,
    /// Computed line total.
    #[serde(deserialize_with = "crate::models::de::f64_flexible")]
    pub line_total: f64,
    /// Fields not modelled above, preserved verbatim.
    #[serde(flatten)]
    pub extra: Extra,
}

/// An Invoice Ninja invoice.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Invoice {
    /// Invoice identifier.
    pub id: String,
    /// Invoice number.
    pub number: String,
    /// Owning client identifier.
    pub client_id: String,
    /// Invoice total amount.
    #[serde(deserialize_with = "crate::models::de::f64_flexible")]
    pub amount: f64,
    /// Outstanding balance.
    #[serde(deserialize_with = "crate::models::de::f64_flexible")]
    pub balance: f64,
    /// Status identifier.
    pub status_id: String,
    /// Invoice date.
    pub date: String,
    /// Due date.
    pub due_date: String,
    /// Purchase order number.
    pub po_number: String,
    /// Notes shown on the invoice.
    pub public_notes: String,
    /// Internal notes.
    pub private_notes: String,
    /// Invoice terms.
    pub terms: String,
    /// Invoice footer.
    pub footer: String,
    /// Line items.
    pub line_items: Vec<InvoiceItem>,
    /// Whether the invoice has been soft-deleted.
    pub is_deleted: bool,
    /// Creation timestamp (Unix seconds), when present.
    #[serde(deserialize_with = "crate::models::de::i64_opt_flexible")]
    pub created_at: Option<i64>,
    /// Fields not modelled above, preserved verbatim.
    #[serde(flatten)]
    pub extra: Extra,
}

/// An Invoice Ninja payment.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Payment {
    /// Payment identifier.
    pub id: String,
    /// Payment number.
    pub number: String,
    /// Owning client identifier.
    pub client_id: String,
    /// Payment amount.
    #[serde(deserialize_with = "crate::models::de::f64_flexible")]
    pub amount: f64,
    /// Amount applied to invoices.
    #[serde(deserialize_with = "crate::models::de::f64_flexible")]
    pub applied: f64,
    /// Amount refunded.
    #[serde(deserialize_with = "crate::models::de::f64_flexible")]
    pub refunded: f64,
    /// Status identifier.
    pub status_id: String,
    /// Payment date.
    pub date: String,
    /// External transaction reference.
    pub transaction_reference: String,
    /// Whether the payment has been soft-deleted.
    pub is_deleted: bool,
    /// Fields not modelled above, preserved verbatim.
    #[serde(flatten)]
    pub extra: Extra,
}

/// An Invoice Ninja product.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Product {
    /// Product identifier.
    pub id: String,
    /// Product key/SKU.
    pub product_key: String,
    /// Product notes/description.
    pub notes: String,
    /// Unit price.
    #[serde(deserialize_with = "crate::models::de::f64_flexible")]
    pub price: f64,
    /// Unit cost.
    #[serde(deserialize_with = "crate::models::de::f64_flexible")]
    pub cost: f64,
    /// Default quantity.
    #[serde(deserialize_with = "crate::models::de::f64_flexible")]
    pub quantity: f64,
    /// Whether the product has been soft-deleted.
    pub is_deleted: bool,
    /// Fields not modelled above, preserved verbatim.
    #[serde(flatten)]
    pub extra: Extra,
}

/// An Invoice Ninja quote.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Quote {
    /// Quote identifier.
    pub id: String,
    /// Quote number.
    pub number: String,
    /// Owning client identifier.
    pub client_id: String,
    /// Quote total amount.
    #[serde(deserialize_with = "crate::models::de::f64_flexible")]
    pub amount: f64,
    /// Outstanding balance.
    #[serde(deserialize_with = "crate::models::de::f64_flexible")]
    pub balance: f64,
    /// Status identifier.
    pub status_id: String,
    /// Quote date.
    pub date: String,
    /// Valid-until date.
    pub due_date: String,
    /// Purchase order number.
    pub po_number: String,
    /// Line items.
    pub line_items: Vec<InvoiceItem>,
    /// Whether the quote has been soft-deleted.
    pub is_deleted: bool,
    /// Fields not modelled above, preserved verbatim.
    #[serde(flatten)]
    pub extra: Extra,
}

/// An Invoice Ninja credit.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Credit {
    /// Credit identifier.
    pub id: String,
    /// Credit number.
    pub number: String,
    /// Owning client identifier.
    pub client_id: String,
    /// Credit total amount.
    #[serde(deserialize_with = "crate::models::de::f64_flexible")]
    pub amount: f64,
    /// Outstanding balance.
    #[serde(deserialize_with = "crate::models::de::f64_flexible")]
    pub balance: f64,
    /// Status identifier.
    pub status_id: String,
    /// Credit date.
    pub date: String,
    /// Line items.
    pub line_items: Vec<InvoiceItem>,
    /// Whether the credit has been soft-deleted.
    pub is_deleted: bool,
    /// Fields not modelled above, preserved verbatim.
    #[serde(flatten)]
    pub extra: Extra,
}

/// An Invoice Ninja expense.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Expense {
    /// Expense identifier.
    pub id: String,
    /// Expense number.
    pub number: String,
    /// Expense amount.
    #[serde(deserialize_with = "crate::models::de::f64_flexible")]
    pub amount: f64,
    /// Expense date.
    pub date: String,
    /// Owning vendor identifier.
    pub vendor_id: String,
    /// Associated client identifier.
    pub client_id: String,
    /// Expense category identifier.
    pub category_id: String,
    /// Notes shown on related invoices.
    pub public_notes: String,
    /// Internal notes.
    pub private_notes: String,
    /// Whether the expense should be invoiced.
    pub should_be_invoiced: bool,
    /// Whether the expense has been soft-deleted.
    pub is_deleted: bool,
    /// Fields not modelled above, preserved verbatim.
    #[serde(flatten)]
    pub extra: Extra,
}

/// An Invoice Ninja vendor.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Vendor {
    /// Vendor identifier.
    pub id: String,
    /// Vendor name.
    pub name: String,
    /// Vendor number.
    pub number: String,
    /// VAT/tax number.
    pub vat_number: String,
    /// Primary phone number.
    pub phone: String,
    /// Website URL.
    pub website: String,
    /// Vendor contacts.
    pub contacts: Vec<Contact>,
    /// Whether the vendor has been soft-deleted.
    pub is_deleted: bool,
    /// Fields not modelled above, preserved verbatim.
    #[serde(flatten)]
    pub extra: Extra,
}

/// An Invoice Ninja project.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Project {
    /// Project identifier.
    pub id: String,
    /// Project name.
    pub name: String,
    /// Project number.
    pub number: String,
    /// Owning client identifier.
    pub client_id: String,
    /// Hourly task rate.
    #[serde(deserialize_with = "crate::models::de::f64_flexible")]
    pub task_rate: f64,
    /// Budgeted hours.
    #[serde(deserialize_with = "crate::models::de::f64_flexible")]
    pub budgeted_hours: f64,
    /// Due date.
    pub due_date: String,
    /// Notes shown to the client.
    pub public_notes: String,
    /// Internal notes.
    pub private_notes: String,
    /// Whether the project has been soft-deleted.
    pub is_deleted: bool,
    /// Fields not modelled above, preserved verbatim.
    #[serde(flatten)]
    pub extra: Extra,
}

/// An Invoice Ninja task.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Task {
    /// Task identifier.
    pub id: String,
    /// Task number.
    pub number: String,
    /// Task description.
    pub description: String,
    /// Owning client identifier.
    pub client_id: String,
    /// Owning project identifier.
    pub project_id: String,
    /// Encoded time log entries.
    pub time_log: String,
    /// Whether the task is currently running.
    pub is_running: bool,
    /// Billable rate.
    #[serde(deserialize_with = "crate::models::de::f64_flexible")]
    pub rate: f64,
    /// Whether the task has been soft-deleted.
    pub is_deleted: bool,
    /// Fields not modelled above, preserved verbatim.
    #[serde(flatten)]
    pub extra: Extra,
}
