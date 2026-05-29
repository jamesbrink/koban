# Environment

## Variables

| Variable                  | Required | Default                | Notes                                              |
| ------------------------- | -------- | ---------------------- | -------------------------------------------------- |
| `INVOICE_NINJA_API_TOKEN` | no\*     | —                      | Sent as `X-API-TOKEN`. Never printed.              |
| `INVOICE_NINJA_BASE_URL`  | no       | `https://invoicing.co` | Hosted production or self-hosted base URL.         |
| `KOBAN_CONFIG_DIR`        | no       | platform config dir    | Overrides where `koban auth` stores `config.json`. |

\* A token is required, but it may come from `INVOICE_NINJA_API_TOKEN`, the OS
keychain, or the stored config file — see [Authentication](/commands/auth). The
environment variable always takes precedence.

For the public demo API, set the base URL to
`https://demo.invoiceninja.com` and the token to `TOKEN`.

## Library constants

The library re-exports the variable names and the default so consumers don't
hard-code strings:

| Constant           | Value                       |
| ------------------ | --------------------------- |
| `API_TOKEN_ENV`    | `"INVOICE_NINJA_API_TOKEN"` |
| `BASE_URL_ENV`     | `"INVOICE_NINJA_BASE_URL"`  |
| `DEFAULT_BASE_URL` | `"https://invoicing.co"`    |

```rust
use koban::{Config, DEFAULT_BASE_URL};

let config = Config::from_env()?; // reads the two env vars above
assert!(DEFAULT_BASE_URL.starts_with("https://"));
```

`Config::from_env()` requires `INVOICE_NINJA_API_TOKEN`; a missing token yields
`KobanError::MissingToken`. The base URL must use HTTPS — the one exception is
local hosts (`localhost`, `127.0.0.1`, `::1`), which may use plain HTTP for
self-hosted development. Any other non-HTTPS URL yields
`KobanError::InsecureBaseUrl`.

## Installer & updater variables

`install.sh` reads two additional variables (see [Installation](/guide/installation)):

| Variable            | Default        | Purpose                 |
| ------------------- | -------------- | ----------------------- |
| `KOBAN_VERSION`     | `latest`       | Release tag to install. |
| `KOBAN_INSTALL_DIR` | `~/.local/bin` | Install directory.      |

## Token redaction

Tokens are never emitted in output, errors, traces, fixtures, or docs. The
`koban::redact` helper strips a token from arbitrary text and is used internally
wherever a message might otherwise contain one.
