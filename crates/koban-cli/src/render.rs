use serde_json::Value;
use tabled::{Table, Tabled, settings::Style};

use koban::{KobanError, Resource, Result};

use crate::cli::OutputFormat;

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

pub(crate) fn render_table(resource: Option<Resource>, value: &Value) -> String {
    if resource.is_none() {
        if value.get("data").is_some() {
            return render_rows(None, value);
        }
        return render_statics_table(value);
    }

    render_rows(resource, value)
}

fn render_rows(resource: Option<Resource>, value: &Value) -> String {
    let rows = response_rows(value)
        .into_iter()
        .map(|item| match resource {
            Some(Resource::Clients) => Row::client(item),
            Some(Resource::Invoices) => Row::invoice(item),
            Some(Resource::Payments) => Row::payment(item),
            Some(Resource::Quotes) => Row::quote(item),
            Some(Resource::Credits) => Row::credit(item),
            Some(Resource::Vendors) => Row::vendor(item),
            Some(Resource::Expenses) => Row::expense(item),
            Some(Resource::Projects) => Row::project(item),
            Some(Resource::Tasks) => Row::task(item),
            Some(Resource::Products) => Row::product(item),
            Some(Resource::PurchaseOrders) => Row::purchase_order(item),
            Some(Resource::RecurringInvoices) => Row::invoice_like(item),
            Some(Resource::RecurringExpenses) => Row::expense(item),
            Some(Resource::BankTransactions) => Row::bank_transaction(item),
            Some(_) | None => Row::generic(item),
        })
        .collect::<Vec<_>>();

    if rows.is_empty() {
        return format!("No {} found.", resource.map_or("records", Resource::label));
    }

    let mut table = Table::new(rows);
    table.with(Style::rounded());
    table.to_string()
}

pub(crate) fn render_statics_table(value: &Value) -> String {
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

pub(crate) fn value_kind(value: &Value) -> &'static str {
    match value {
        Value::Array(_) => "array",
        Value::Object(_) => "object",
        Value::String(_) => "string",
        Value::Number(_) => "number",
        Value::Bool(_) => "boolean",
        Value::Null => "null",
    }
}

pub(crate) fn value_len(value: &Value) -> usize {
    match value {
        Value::Array(items) => items.len(),
        Value::Object(map) => map.len(),
        _ => 1,
    }
}

pub(crate) fn response_rows(value: &Value) -> Vec<&Value> {
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

    fn product(value: &Value) -> Self {
        Self {
            id: field(value, &["id"]),
            number: field(value, &["product_key"]),
            name: first_field(value, &[&["notes"], &["name"], &["custom_value1"]]),
            status: first_field(value, &[&["status"], &["status_id"]]),
            amount: first_field(value, &[&["price"], &["cost"]]),
            balance: dash(),
            date: date_field(value, &["created_at"]),
        }
    }

    fn purchase_order(value: &Value) -> Self {
        Self {
            id: field(value, &["id"]),
            number: field(value, &["number"]),
            name: first_field(
                value,
                &[
                    &["vendor", "display_name"],
                    &["vendor", "name"],
                    &["vendor_id"],
                ],
            ),
            status: first_field(value, &[&["status"], &["status_id"]]),
            amount: field(value, &["amount"]),
            balance: field(value, &["balance"]),
            date: first_date_field(value, &[&["due_date"], &["date"], &["created_at"]]),
        }
    }

    fn invoice_like(value: &Value) -> Self {
        let mut row = Self::invoice(value);
        row.status = first_field(value, &[&["status"], &["status_id"], &["frequency_id"]]);
        row
    }

    fn bank_transaction(value: &Value) -> Self {
        Self {
            id: field(value, &["id"]),
            number: first_field(value, &[&["transaction_id"], &["number"]]),
            name: first_field(
                value,
                &[&["description"], &["bank_account_id"], &["vendor_id"]],
            ),
            status: first_field(value, &[&["status"], &["status_id"]]),
            amount: field(value, &["amount"]),
            balance: field(value, &["balance"]),
            date: first_date_field(value, &[&["date"], &["created_at"]]),
        }
    }

    fn generic(value: &Value) -> Self {
        Self {
            id: field(value, &["id"]),
            number: first_field(
                value,
                &[&["number"], &["name"], &["key"], &["token"], &["email"]],
            ),
            name: first_field(
                value,
                &[
                    &["display_name"],
                    &["name"],
                    &["description"],
                    &["email"],
                    &["title"],
                ],
            ),
            status: first_field(value, &[&["status"], &["status_id"], &["is_deleted"]]),
            amount: first_field(value, &[&["amount"], &["balance"], &["price"]]),
            balance: field(value, &["balance"]),
            date: first_date_field(value, &[&["date"], &["updated_at"], &["created_at"]]),
        }
    }
}

pub(crate) fn invoice_status(value: &Value) -> String {
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

pub(crate) fn quote_status(value: &Value) -> String {
    match value.get("status_id").and_then(Value::as_i64) {
        Some(1) => "draft".to_string(),
        Some(2) => "sent".to_string(),
        Some(3) => "approved".to_string(),
        Some(4) => "converted".to_string(),
        _ => first_field(value, &[&["status"], &["status_id"]]),
    }
}

pub(crate) fn first_field(value: &Value, paths: &[&[&str]]) -> String {
    paths
        .iter()
        .map(|path| field(value, path))
        .find(|value| value != "-")
        .unwrap_or_else(dash)
}

pub(crate) fn first_date_field(value: &Value, paths: &[&[&str]]) -> String {
    paths
        .iter()
        .map(|path| date_field(value, path))
        .find(|value| value != "-")
        .unwrap_or_else(dash)
}

pub(crate) fn date_field(value: &Value, path: &[&str]) -> String {
    let Some(value) = nested_value(value, path) else {
        return dash();
    };

    unix_timestamp(value)
        .and_then(format_unix_date)
        .unwrap_or_else(|| field_value(value))
}

pub(crate) fn field(value: &Value, path: &[&str]) -> String {
    nested_value(value, path).map_or_else(dash, field_value)
}

pub(crate) fn nested_value<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
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

pub(crate) fn field_value(value: &Value) -> String {
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

pub(crate) fn unix_timestamp(value: &Value) -> Option<i64> {
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

pub(crate) fn format_unix_date(timestamp: i64) -> Option<String> {
    let days = timestamp.div_euclid(86_400);
    let (year, month, day) = civil_from_days(days)?;
    Some(format!("{year:04}-{month:02}-{day:02}"))
}

pub(crate) fn looks_like_reasonable_unix_seconds(timestamp: i64) -> bool {
    let days = timestamp.div_euclid(86_400);
    civil_from_days(days).is_some_and(|(year, _, _)| (1900..=9999).contains(&year))
}

pub(crate) fn civil_from_days(days: i64) -> Option<(i32, u32, u32)> {
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

pub(crate) fn dash() -> String {
    "-".to_string()
}
