# Invoice Ninja API Reference For Koban

This document is Koban's working API reference. It is intentionally conservative
because development touches accounting APIs. Koban's implemented surface now
spans the official resource families with guarded write commands. Prefer the
public demo API for live smoke tests, and use production or personal accounts
only for intentional checks.

Last researched: 2026-05-28.

## Primary Sources

- Developer guide: https://invoiceninja.github.io/docs/developer-guide
- API reference: https://invoiceninja.github.io/docs/api-reference/invoice-ninja-api-reference
- Interactive/OpenAPI docs: https://api-docs.invoicing.co/
- Clients API: https://invoiceninja.github.io/docs/api-reference/clients
- List clients: https://invoiceninja.github.io/docs/api-reference/get-clients
- Invoices API: https://invoiceninja.github.io/docs/api-reference/invoices
- List invoices: https://invoiceninja.github.io/docs/api-reference/get-invoices
- Payments API: https://invoiceninja.github.io/docs/api-reference/payments
- List payments: https://invoiceninja.github.io/docs/api-reference/get-payments
- Quotes API: https://invoiceninja.github.io/docs/api-reference/quotes
- Credits API: https://invoiceninja.github.io/docs/api-reference/credits
- Vendors API: https://invoiceninja.github.io/docs/api-reference/vendors
- Expenses API: https://invoiceninja.github.io/docs/api-reference/expenses
- Projects API: https://invoiceninja.github.io/docs/api-reference/projects
- Tasks API: https://invoiceninja.github.io/docs/api-reference/tasks
- Search API: https://invoiceninja.github.io/docs/api-reference/search
- Search endpoint: https://invoiceninja.github.io/docs/api-reference/post-search
- Statics endpoint: https://invoiceninja.github.io/docs/api-reference/get-statics
- Products API: https://invoiceninja.github.io/docs/api-reference/products
- Recurring invoices API: https://invoiceninja.github.io/docs/api-reference/recurring-invoices
- Purchase orders API: https://invoiceninja.github.io/docs/api-reference/purchase-orders
- Recurring expenses API: https://invoiceninja.github.io/docs/api-reference/recurring-expenses

## API Shape

- Invoice Ninja v5 uses the `/api/v1` namespace.
- Hosted production base URL: `https://invoicing.co`.
- Hosted demo base URL: `https://demo.invoiceninja.com`.
- Hosted demo API token: `TOKEN`.
- Self-hosted installs use the same `/api/v1` namespace under their own base URL.
- The official API reference currently reports version `5.12.55`.
- The docs say API requests must use HTTPS.
- v5 IDs are hashed/obfuscated strings, not v4 integer IDs. Model every ID as a
  string.
- The API is large and resource-oriented. The reference covers clients, invoices,
  recurring invoices, payments, quotes, credits, projects, tasks, vendors,
  purchase orders, expenses, bank transactions, reports, activities, documents,
  webhooks, statics, search, and more.

## Authentication And Headers

Required headers for normal API requests:

```text
X-API-TOKEN: <token>
X-Requested-With: XMLHttpRequest
```

Required header for JSON writes:

```text
Content-Type: application/json
```

Notes:

- `X-API-TOKEN` replaced the old v4 `X-Ninja-Token` header.
- `X-API-SECRET` is optional and only assessed on `/api/v1/login`.
- Koban should load the token from `INVOICE_NINJA_API_TOKEN`.
- Koban should default `INVOICE_NINJA_BASE_URL` to `https://invoicing.co`.
- Local smoke testing should prefer
  `INVOICE_NINJA_BASE_URL=https://demo.invoiceninja.com` and
  `INVOICE_NINJA_API_TOKEN=TOKEN`.
- Never log request headers verbatim. Redact tokens and secrets in errors,
  traces, debug output, and agent-facing JSON.

## Responses, Errors, And Rate Limits

The API reference documents these common response classes:

- `200`: success.
- `400`: bad request.
- `401`: authentication failed or missing.
- `403`: authorization failure.
- `404`: resource not found.
- `405`: unsupported method.
- `409`: conflict.
- `422`: validation error.
- `429`: rate limit exceeded.
- `5XX`: server-side failure.

List endpoints document these response headers:

- `X-MINIMUM-CLIENT-VERSION`: API/client compatibility version.
- `X-RateLimit-Remaining`: remaining requests in the current window.
- `X-RateLimit-Limit`: total requests in the current window.

Koban should preserve useful server messages for humans and agents, but should
wrap them in a stable local error envelope for `--output json`.

## Pagination, Filtering, Sorting, Includes

Index routes default to `20` records per page. The documented pagination
parameters are:

```text
per_page=<count>
page=<page>
```

Client list examples show:

```text
/api/v1/clients?per_page=15&page=2
/api/v1/clients?sort=name|desc
/api/v1/clients?balance=gt:1000
/api/v1/clients?balance=gt:1000&name=Bob
/api/v1/clients?include=activities,ledger,system_logs
```

Implemented design:

- List commands expose explicit `--page` and `--per-page` flags.
- List commands expose repeatable raw `--filter key=value` flags and a raw
  `--sort field|direction` flag.
- List commands expose `--all` and `--limit` for controlled pagination.
- List/show/template/edit-template/create/update/delete/bulk/upload/action
  commands expose repeatable, comma-separated `--include` flags when useful.
- Single-page JSON output preserves the API response. Multi-page JSON output uses
  a local `data` plus `meta.pages_fetched` envelope.
- Avoid unbounded traversal unless the user explicitly passes `--all`.

## Implemented Resource Parity

Koban exposes the same command shape for the main resource-oriented API groups:

```text
koban <resource> list
koban <resource> show <id>
koban <resource> template
koban <resource> edit-template <id>
koban <resource> create
koban <resource> update <id>
koban <resource> delete <id>
koban <resource> bulk
koban <resource> upload <id>
koban <resource> action <id>
```

The resource set includes `clients`, `invoices`, `payments`, `quotes`,
`credits`, `vendors`, `expenses`, `projects`, `tasks`, `locations`, `products`,
`recurring-invoices`, `purchase-orders`, `recurring-expenses`,
`bank-transactions`, `bank-integrations`, `bank-transaction-rules`,
`expense-categories`, `tax-rates`, `payment-terms`, `task-statuses`,
`activities`, `system-logs`, `documents`, `designs`, `templates`, `users`,
`companies`, `company-gateways`, `company-ledger`, `company-users`, `tokens`,
`webhooks`, `imports`, `subscriptions`, and `client-gateway-tokens`.

Inspect-only/high-risk resources `activities`, `system-logs`, `company-ledger`,
and `imports` expose only `list` and `show`. They intentionally do not expose
the generic write command family.

The `create` and `edit` routes above return blank/default or editable objects;
they are read-only `GET` routes despite their names. Koban exposes them as
`template` and `edit-template` commands for schema discovery instead of
user-facing `create` or `edit` verbs.

Utility-style endpoints are exposed through endpoint runners:

```text
koban search run
koban reports run
koban charts run
koban utility run
```

`search`, `reports`, and `charts` default to their matching endpoint names.
Custom `--endpoint` overrides are read-only and only send `GET` requests.
`utility run` defaults to `ping` and is always read-only.

## Implemented Write Endpoints

Resource manipulation follows the official REST shape:

```text
POST /api/v1/{resource}
PUT /api/v1/{resource}/{id}
DELETE /api/v1/{resource}/{id}
POST /api/v1/{resource}/bulk
POST /api/v1/{resource}/{id}/upload
GET|POST /api/v1/{resource}/{id}/{action}
```

Create and update accept either one raw JSON source (`--data`, `--data-file`, or
`--stdin`) or guided flags. Raw JSON cannot be combined with guided fields or
`--line-item`. Generic guided flags cover common fields:
`--name`, `--number`, `--client-id`, `--vendor-id`, `--project-id`, `--date`,
`--due-date`, `--amount`, `--price`, `--quantity`, notes, repeatable
`--field key=value`, and repeatable `--line-item key=value,...`.

Invoice commands keep a specialized guided payload and trigger model for
invoice-specific workflows:

Invoice create/update may also send documented trigger query flags:

```text
send_email=true
mark_sent=true
paid=true
amount_paid=<amount>
cancel=true
save_default_footer=true
save_default_terms=true
retry_e_send=true
```

Koban intentionally does not expose the documented `redirect` trigger.

Note: the interactive docs label invoice document upload as `POST`, but the
public demo API currently accepts `PUT /api/v1/invoices/{id}/upload`; Koban uses
the live-compatible method for invoices and `POST` for generic resource uploads.
Generic multipart uploads include Invoice Ninja's `_method=POST` form field and
use the documented `documents` file field.

Safety rules:

- Every write supports `--dry-run`.
- Generic resource `create`, `update`, `delete`, `bulk`, `upload`, and `action`
  commands require `--yes` unless `--dry-run` is used.
- Generic endpoint runner defaults require `--yes` for non-GET methods unless
  `--dry-run` is used. Custom `--endpoint` overrides are read-only.
- `utility run` is read-only and rejects non-GET methods.
- Generic endpoint runner payload flags are valid only with `POST` and `PUT`;
  `GET` and `DELETE` reject payloads instead of silently dropping request
  bodies.
- `create` and `update` require `--yes` when invoice-specific triggers mark
  sent, send email, mark paid, record an amount paid, cancel, save default
  footer/terms, or retry e-send.
- Mocked tests are required for every write path. Live write smoke tests must be
  explicitly opted in and should use the public demo endpoint.
- `smoke-all-demo` is the repeatable full live smoke helper for the implemented
  command families. It hard-codes the public demo URL and token internally,
  live-reads every supported demo resource, dry-runs every expanded resource
  write family, and only runs when `KOBAN_LIVE_WRITE_SMOKE=1`.

## High-Risk Endpoints

Purge, merge, refund, import, scheduler, support, and admin utility endpoints
can have irreversible side effects. Koban exposes the plumbing needed to call
them, but live smoke tests for these paths must be demo-only, opt-in, and tied
to fixtures that create and clean up their own records.

## Invoice Statuses

The invoice reference documents these status values:

| Status | Meaning |
| --- | --- |
| `1` | Draft |
| `2` | Sent |
| `3` | Partially paid |
| `4` | Paid |
| `5` | Cancelled |
| `6` | Reversed |
| `-1` | Overdue |
| `-2` | Unpaid/not yet due |

Koban should render friendly names in table output while preserving raw numeric
values in JSON output.

## Implemented Starting Point

The current implementation is a guarded API foundation:

1. A small HTTP client module with `base_url`, `api_token`, and default headers.
2. Config loading from environment only:
   `INVOICE_NINJA_API_TOKEN` and optional `INVOICE_NINJA_BASE_URL`.
3. `koban statics` uses `GET /api/v1/statics` as the smallest authenticated
   smoke test, preferably against the public demo endpoint.
4. Resource `list/show/template/edit-template/create/update/delete/bulk/upload/
   action` commands across the documented resource families.
5. List pagination with `--page`, `--per-page`, `--all`, and `--limit`.
6. Raw filtering and sorting with `--filter key=value` and `--sort field|dir`.
7. Read-only invoice PDF downloads for invoice PDFs and delivery notes.
8. Guarded write commands with generic guided payloads plus specialized invoice
   create/update triggers.

Safety rules for this milestone:

- Read commands and downloads issue `GET` requests.
- Inspect-only/high-risk resources stay list/show-only until Koban adds
  resource-specific safe write workflows.
- Write commands require explicit payloads, `--dry-run` previews, and
  `--yes` where the mutation is destructive or externally visible.
- No automatic pagination across multiple pages unless `--all` is explicit.
- Imports, purge, refund, merge, scheduler, and admin utility live smoke tests
  must be explicit and demo-only.
- Live smoke tests should use the public demo endpoint by default:
  `https://demo.invoiceninja.com` with token `TOKEN`.
- Production or personal accounts are acceptable only for intentional checks.
- JSON output should be stable enough for AI agents.
- Human table output should hide noisy raw fields by default.
- Error output must redact `INVOICE_NINJA_API_TOKEN`.

Implemented command shape:

```text
koban statics --output json
koban clients list --page 1 --per-page 20 --output table
koban clients list --filter balance=gt:1000 --filter name=Bob --sort 'name|desc'
koban clients show <client_id> --output json
koban clients template --output json
koban clients edit-template <client_id> --output json
koban invoices list --page 1 --per-page 20 --output table
koban invoices list --all --limit 100 --output json
koban invoices show <invoice_id> --output json
koban invoices template --output json
koban invoices edit-template <invoice_id> --output json
koban invoices create --client-id <client_id> --line-item product_key=Consulting,quantity=1,cost=100 --dry-run
koban invoices create --data-file invoice.json --include client
koban invoices update <invoice_id> --data-file invoice.json --dry-run
koban invoices update <invoice_id> --public-notes "Thanks again" --mark-sent --yes
koban invoices delete <invoice_id> --dry-run
koban invoices delete <invoice_id> --yes
koban invoices bulk --action archive --id <invoice_id> --id <invoice_id> --dry-run
koban invoices action <invoice_id> --action mark_paid --dry-run
koban invoices upload <invoice_id> --file contract.pdf --dry-run
koban invoices download <invitation_key> --output-file invoice.pdf
koban invoices delivery-note <invoice_id> --output-file delivery-note.pdf
koban payments list --page 1 --per-page 20 --output table
koban payments show <payment_id> --output json
koban payments template --output json
koban payments edit-template <payment_id> --output json
koban products create --name Consulting --price 100 --dry-run
koban products update <product_id> --field notes="Hourly support" --dry-run
koban recurring-invoices action <recurring_invoice_id> --action start --dry-run
koban search run --field query=acme --dry-run
koban reports run --data-file report.json --dry-run
koban quotes list --output table
koban credits show <credit_id> --output json
koban vendors template --output json
koban expenses edit-template <expense_id> --output json
koban projects list --filter client_id=<client_id>
koban tasks list --all --limit 50
```

Recurring invoice single-record actions are represented as
`POST /api/v1/recurring_invoices/bulk` with the requested action and a one-item
`ids` list, because the upstream API documents recurring lifecycle actions on
the bulk endpoint.

## Open Questions

- Should Koban add a first-class `--demo` flag or `koban smoke` command that
  temporarily uses `https://demo.invoiceninja.com` with token `TOKEN` without
  writing those values to a user's shell environment?
- Should Koban prefer Invoice Ninja's response shape verbatim for agent JSON, or
  normalize list/show responses behind Koban's own `data`, `meta`, and `links`
  envelope?
- Should statics be cached on disk, and if so where should Koban keep cache files
  on macOS/Linux?
- Which guided fields should become resource-specific first-class flags beyond
  the current common field set?
