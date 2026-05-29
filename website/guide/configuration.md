# Configuration

koban's configuration model is environment-first.

## Environment variables

```sh
export INVOICE_NINJA_BASE_URL="https://invoicing.co"
export INVOICE_NINJA_API_TOKEN="..."
```

| Variable                  | Required | Default                | Notes                                           |
| ------------------------- | -------- | ---------------------- | ----------------------------------------------- |
| `INVOICE_NINJA_API_TOKEN` | yes      | —                      | Token-based auth, sent as `X-API-TOKEN`.        |
| `INVOICE_NINJA_BASE_URL`  | no       | `https://invoicing.co` | Hosted production or your self-hosted base URL. |

Tokens are never printed by default — not in output, errors, traces, or logs.

## Invoice Ninja endpoints

Invoice Ninja v5 exposes its API under `/api/v1`:

- **Hosted production:** `https://invoicing.co`
- **Self-hosted:** the same `/api/v1` namespace under your own base URL.
- **Public demo:** `https://demo.invoiceninja.com` with the demo token `TOKEN` —
  use this for live smoke tests whenever possible.

Authentication is token based. Requests send `X-API-TOKEN` along with
`X-Requested-With: XMLHttpRequest`; JSON write requests send
`Content-Type: application/json`.

## Demo target

For demo smoke tests:

```sh
export INVOICE_NINJA_BASE_URL="https://demo.invoiceninja.com"
export INVOICE_NINJA_API_TOKEN="TOKEN"
```

## Loading from a .env file (devshell)

Inside the Nix `nix develop` shell, the devshell loads
`INVOICE_NINJA_API_TOKEN` and `INVOICE_NINJA_BASE_URL` from a local,
gitignored `.env` file when those variables are not already set in your shell:

```dotenv
INVOICE_NINJA_API_TOKEN=TOKEN
INVOICE_NINJA_BASE_URL=https://demo.invoiceninja.com
```

See the [Environment reference](/reference/environment) for the constants the
library exposes for these variables.
