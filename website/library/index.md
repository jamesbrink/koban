# Using koban as a library

The `koban` crate is a standalone Invoice Ninja API client that other Rust
applications can depend on, independent of the CLI:

```sh
cargo add koban
```

## Quickstart

```rust
use koban::{ApiClient, Config};

#[tokio::main]
async fn main() -> koban::Result<()> {
    // Reads INVOICE_NINJA_API_TOKEN and optional INVOICE_NINJA_BASE_URL.
    let client = ApiClient::new(Config::from_env()?);

    // Typed resource accessors return the built-in models.
    for invoice in client.invoices().list().await? {
        println!("{} -> {}", invoice.number, invoice.balance);
    }

    // Or work with any resource and your own type / serde_json::Value.
    let client_record = client.clients().get("client_id").await?;
    println!("{}", client_record.display_name);

    Ok(())
}
```

## Configuration

`Config` reads from the environment with `Config::from_env()`, or you can build
one directly. The relevant env constants are re-exported: `API_TOKEN_ENV`,
`BASE_URL_ENV`, and `DEFAULT_BASE_URL`. See the
[Environment reference](/reference/environment).

## Three layers of API

koban gives you three ways to talk to Invoice Ninja, from highest to lowest
level:

1. **Typed resource accessors** — `client.invoices()`, `client.clients()`, … —
   return typed models. See [Resource accessors](/library/resources).
2. **Generic typed methods** — `client.resource::<T>(Resource::TaxRates)` works
   for any `T: DeserializeOwned`, including your own types or `serde_json::Value`.
3. **Raw JSON escape hatch** — `client.get_json(...)`, `post_json`, `put_json`,
   and `delete_json` return `serde_json::Value` for non-resource endpoints or
   routes outside the generic CRUD shape.

## What gets re-exported

`lib.rs` re-exports the full public surface:

- `ApiClient` and `Config` (+ `API_TOKEN_ENV`, `BASE_URL_ENV`, `DEFAULT_BASE_URL`)
- `Resource` (the resource enum) and `Resources` (the accessor handle)
- `KobanError` and `Result`
- `redact` (token redaction helper)
- Models: `Invoice`, `InvoiceItem`, `Client`, `Contact`, `Payment`, `Quote`,
  `Credit`, `Product`, `Expense`, `Vendor`, `Project`, `Task`, and the envelopes
  `Data`, `Paginated`, `Meta`, `Pagination`.

By default the library depends only on `thiserror` for its error type; enable the
`miette` feature for diagnostic help text on `KobanError` (see
[Errors & features](/library/errors)).
