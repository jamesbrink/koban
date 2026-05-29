# Typed models

The library ships first-class models for the common Invoice Ninja resources:

`Invoice` (with `InvoiceItem`), `Client` (with `Contact`), `Payment`, `Quote`,
`Credit`, `Product`, `Expense`, `Vendor`, `Project`, and `Task`.

Each derives `Serialize`, `Deserialize`, `Debug`, `Clone`, and `Default`.

## Forward compatible

Models are designed so adding fields later is non-breaking:

- Every field uses `#[serde(default)]`, so absent fields deserialize to their
  default rather than erroring.
- Unmodeled fields are captured verbatim in an `extra` map
  (`#[serde(flatten)]`), so nothing is dropped on round-trip. Adding a new field
  to your account's payload won't break deserialization.

```rust
let invoice = client.invoices().get("inv_1").await?;
println!("{}", invoice.number);
// Fields koban doesn't model yet are still available:
if let Some(custom) = invoice.extra.get("custom_value1") {
    println!("custom_value1 = {custom}");
}
```

Numeric fields tolerate Invoice Ninja sending numbers as either JSON numbers or
strings (e.g. `"50.25"` parses to `50.25`).

## Response envelopes

Invoice Ninja wraps responses in a `data` key. The library models that with two
envelope types:

- `Data<T>` — a single record: `{ "data": T }`.
- `Paginated<T>` — a collection: `{ "data": [T], "meta": Option<Meta> }`, where
  `Meta` carries `Pagination` (total, count, current page, total pages, per
  page, …).

The typed accessors unwrap these for you — `list()` returns `Vec<T>` and `get()`
returns `T` — while `list_paginated()` exposes the `Meta`. See
[Resource accessors](/library/resources).

## Resources without built-in models

Koban intentionally keeps first-class structs focused on the common workflow
objects above. Other resource families are still reachable through the generic
resource API using your own type or `serde_json::Value`:

```rust
use koban::Resource;

let tax_rates = client
    .resource::<serde_json::Value>(Resource::TaxRates)
    .list()
    .await?;
```

This is a typed-model gap, not a CLI/API coverage gap. The full list of generic
`Resource` variants is in [Resource families](/reference/resource-families).
