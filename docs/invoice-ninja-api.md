# Invoice Ninja API Reference For Koban

This document is Koban's working API reference. It is intentionally conservative
because the first development target is James' active Invoice Ninja account. Until
we have explicit write safeguards, Koban should only perform read-only requests
against production data.

Last researched: 2026-05-27.

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
- Search API: https://invoiceninja.github.io/docs/api-reference/search
- Search endpoint: https://invoiceninja.github.io/docs/api-reference/post-search
- Statics endpoint: https://invoiceninja.github.io/docs/api-reference/get-statics

## API Shape

- Invoice Ninja v5 uses the `/api/v1` namespace.
- Hosted production base URL: `https://invoicing.co`.
- Hosted demo base URL: `https://demo.invoiceninja.com`.
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

Design consequences:

- Start with explicit `--page` and `--per-page` flags.
- Add `--include` as a repeatable or comma-separated option once response
  shaping is stable.
- Keep raw filter support available for advanced users, but do not invent a
  broad filter DSL in the first pass.
- Avoid automatic all-page traversal until rate-limit behavior is tested.

## High-Value Read-Only Endpoints

These are suitable for the first networked Koban milestone:

```text
GET /api/v1/clients
GET /api/v1/clients/{id}
GET /api/v1/clients/{id}/edit
GET /api/v1/clients/create

GET /api/v1/invoices
GET /api/v1/invoices/{id}
GET /api/v1/invoices/{id}/edit
GET /api/v1/invoices/create
GET /api/v1/invoice/{invitation_key}/download
GET /api/v1/invoices/{id}/delivery_note

GET /api/v1/payments
GET /api/v1/payments/{id}
GET /api/v1/payments/{id}/edit
GET /api/v1/payments/create

GET /api/v1/statics
```

The `create` and `edit` routes above return blank/default or editable objects;
they are read-only `GET` routes despite their names. They may be useful later for
schema discovery, but Koban's initial CLI should focus on list/show first.

The search endpoint is useful, but it is a `POST`:

```text
POST /api/v1/search
```

Treat search as read-like but not part of the first production-account smoke
test. Add it once Koban has a request body model, a `--dry-run` convention, and
fixtures around token redaction.

## Write And Destructive Endpoints To Avoid Initially

Do not implement these until Koban has confirmation prompts, `--yes`, dry-run
rendering, request validation, tests with a mock server, and clear docs:

```text
POST /api/v1/clients
PUT /api/v1/clients/{id}
DELETE /api/v1/clients/{id}
POST /api/v1/clients/bulk
POST /api/v1/clients/{id}/purge
POST /api/v1/clients/{id}/{mergeable_client_hashed_id}/merge

POST /api/v1/invoices
PUT /api/v1/invoices/{id}
DELETE /api/v1/invoices/{id}
POST /api/v1/invoices/bulk
POST /api/v1/invoices/{id}/upload

POST /api/v1/payments
PUT /api/v1/payments/{id}
DELETE /api/v1/payments/{id}
POST /api/v1/payments/{id}/refund
POST /api/v1/payments/bulk
POST /api/v1/payments/{id}/upload
```

Some custom action endpoints may be harmless and others may send emails, archive
records, reverse state, or otherwise mutate accounting data. Treat all custom,
bulk, upload, merge, purge, refund, email, import, and scheduler endpoints as
unsafe until individually audited.

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

Milestone 1 is now a read-only API foundation:

1. A small HTTP client module with `base_url`, `api_token`, and default headers.
2. Config loading from environment only:
   `INVOICE_NINJA_API_TOKEN` and optional `INVOICE_NINJA_BASE_URL`.
3. `koban statics` uses `GET /api/v1/statics` as the least
   business-data-heavy authenticated smoke test.
4. `koban clients list --page --per-page --output json|table`.
5. `koban clients show <id> --output json|table`.
6. The same list/show pattern for invoices.
7. The same list/show pattern for payments.

Safety rules for this milestone:

- Only issue `GET` requests.
- No automatic pagination across multiple pages by default.
- No file uploads, no imports, no email endpoints, no bulk endpoints.
- No command may mutate production data.
- JSON output should be stable enough for AI agents.
- Human table output should hide noisy raw fields by default.
- Error output must redact `INVOICE_NINJA_API_TOKEN`.

Implemented command shape:

```text
koban statics --output json
koban clients list --page 1 --per-page 20 --output table
koban clients show <client_id> --output json
koban invoices list --page 1 --per-page 20 --output table
koban invoices show <invoice_id> --output json
koban payments list --page 1 --per-page 20 --output table
koban payments show <payment_id> --output json
```

## Open Questions

- Which hosted base URL does James' active account use: `https://invoicing.co`,
  `https://app.invoicing.co`, or a self-hosted domain? The API docs identify
  `https://invoicing.co` as the v5 production API endpoint.
- Should Koban prefer Invoice Ninja's response shape verbatim for agent JSON, or
  normalize list/show responses behind Koban's own `data`, `meta`, and `links`
  envelope?
- Should statics be cached on disk, and if so where should Koban keep cache files
  on macOS/Linux?
- What are the first truly useful human workflows: finding a client, checking
  unpaid invoices, downloading PDFs, or creating draft invoices?
