# Errors & features

## `Result` and `KobanError`

Library functions return `koban::Result<T>`, an alias for
`std::result::Result<T, KobanError>`. `KobanError` is a `thiserror` enum with one
variant per failure mode:

| Variant                | When it occurs                                           |
| ---------------------- | -------------------------------------------------------- |
| `MissingToken`         | `INVOICE_NINJA_API_TOKEN` is not configured.             |
| `InvalidBaseUrl`       | The base URL could not be parsed.                        |
| `InsecureBaseUrl`      | The base URL is not HTTPS (and is not a local host).     |
| `InvalidEndpoint`      | An API URL could not be built for a path.                |
| `Transport`            | The request could not reach Invoice Ninja.               |
| `Api`                  | Invoice Ninja returned a non-success HTTP status.        |
| `Decode`               | A response could not be decoded into the expected shape. |
| `InvalidFilter`        | A list filter value was rejected.                        |
| `InvalidPayload`       | A write payload failed validation.                       |
| `ConfirmationRequired` | A guarded operation needs explicit confirmation.         |
| `File`                 | A download file could not be written.                    |
| `Update`               | A self-update step failed.                               |

Tokens are never embedded in error messages — the `redact` helper is used to
strip them from any text that might contain one.

```rust
use koban::{ApiClient, Config, KobanError};

match ApiClient::new(Config::from_env()?).invoices().list().await {
    Ok(invoices) => println!("{} invoices", invoices.len()),
    Err(KobanError::Api { status, .. }) => eprintln!("API error: HTTP {status}"),
    Err(err) => eprintln!("{err}"),
}
```

## The `miette` feature

By default the library depends only on `thiserror`. Enable the optional `miette`
feature to derive `miette::Diagnostic` on `KobanError`, adding diagnostic help
text suitable for rich CLI error reporting:

```toml
[dependencies]
koban = { version = "0.1", features = ["miette"] }
```

The `koban-cli` crate enables this feature; most library consumers can leave it
off and keep the dependency tree lean.

## `redact`

`koban::redact(text, token)` returns a copy of `text` with `token` replaced by a
redaction marker. koban uses it internally so tokens never leak into errors,
traces, or logs; it is re-exported so you can apply the same guarantee to your
own logging.
