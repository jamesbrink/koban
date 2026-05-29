# Resource accessors

`ApiClient` exposes typed accessor methods that return a `Resources<'_, T>`
handle scoped to one resource. The built-in accessors are:

`clients()`, `invoices()`, `payments()`, `quotes()`, `credits()`, `products()`,
`expenses()`, `vendors()`, `projects()`, and `tasks()`.

For any other resource, use the generic `resource::<T>(Resource)` method with a
[`Resource`](/reference/resource-families) variant.

## Methods on `Resources<T>`

| Method                   | Returns        | Description                               |
| ------------------------ | -------------- | ----------------------------------------- |
| `get(id)`                | `T`            | Fetch one record (unwraps `Data<T>`).     |
| `list()`                 | `Vec<T>`       | List records (unwraps the `data` array).  |
| `list_paginated(&query)` | `Paginated<T>` | List with access to pagination `Meta`.    |
| `create(&body)`          | `T`            | Create a record from a serializable body. |
| `update(id, &body)`      | `T`            | Update a record.                          |
| `delete(id)`             | `T`            | Delete a record.                          |
| `resource()`             | `Resource`     | The resource this handle targets.         |

## Examples

```rust
use koban::{ApiClient, Config};
use serde_json::json;

let client = ApiClient::new(Config::from_env()?);

// Typed list and get.
let invoices = client.invoices().list().await?;
let invoice = client.invoices().get("inv_1").await?;

// Create / update / delete round-trip through the Data envelope.
let created = client.clients().create(&json!({ "name": "Acme" })).await?;
let updated = client
    .clients()
    .update(&created.id, &json!({ "name": "Acme Inc" }))
    .await?;
client.clients().delete(&created.id).await?;

// Pagination metadata.
let page = client
    .clients()
    .list_paginated(&[("per_page".to_string(), "20".to_string())])
    .await?;
if let Some(pagination) = page.meta.and_then(|m| m.pagination) {
    println!("total = {:?}", pagination.total);
}
```

## Generic resources & custom types

The generic handle works for any `T: DeserializeOwned` — a built-in model, your
own struct, or `serde_json::Value`:

```rust
use koban::Resource;

// Your own type.
#[derive(serde::Deserialize)]
struct TaxRate {
    name: String,
    rate: f64,
}

let rows = client.resource::<TaxRate>(Resource::TaxRates).list().await?;

// Or untyped.
let raw = client
    .resource::<serde_json::Value>(Resource::TaxRates)
    .list()
    .await?;
```

## Raw JSON escape hatch

When a route isn't covered by a typed method, drop down to the raw `ApiClient`
methods, which return `serde_json::Value`:

```rust
let value = client.get_json("/api/v1/statics", &[]).await?;
```

`post_json`, `put_json`, and `delete_json` are available too.
