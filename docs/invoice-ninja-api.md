# Invoice Ninja API Notes

These notes capture the API facts `koban` should design around. They are not a
replacement for the upstream docs; refresh them before implementing networked
commands.

## Sources Checked

- Developer guide: https://invoiceninja.github.io/docs/developer-guide
- API reference: https://invoiceninja.github.io/docs/api-reference/invoice-ninja-api-reference
- Clients API: https://invoiceninja.github.io/docs/api-reference/clients
- Invoices API: https://invoiceninja.github.io/docs/api-reference/invoices
- Payments API: https://invoiceninja.github.io/docs/api-reference/payments
- Search API: https://invoiceninja.github.io/docs/api-reference/search

## API Shape

- Invoice Ninja v5 uses the `/api/v1` namespace.
- Hosted production is `https://invoicing.co`; the public demo is
  `https://demo.invoiceninja.com`.
- Self-hosted installs use the same API namespace under their own base URL.
- The current API reference checked during this pass reported version `5.12.55`.
- v5 IDs are obfuscated string IDs, not v4-style integer primary keys.

## Required Headers

- `X-API-TOKEN: <token>` authenticates normal API requests.
- `X-Requested-With: XMLHttpRequest` is documented as required.
- `Content-Type: application/json` is required for JSON `POST` and `PUT`
  requests.
- `X-API-SECRET` is optional and only assessed on `/api/v1/login`.

## First Endpoint Families

The first networked release should focus on read-only commands against common
resources:

- Clients: `GET /api/v1/clients`, `GET /api/v1/clients/{id}`
- Invoices: `GET /api/v1/invoices`, `GET /api/v1/invoices/{id}`
- Payments: list/show endpoints from the Payments API
- Search: the cross-entity search endpoint after basic pagination and error
  handling are in place

Creation and mutation commands should come later, once request modeling,
validation, dry-run behavior, and JSON input conventions are settled.

## CLI Design Consequences

- Prefer `KOBAN_BASE_URL` and `KOBAN_API_TOKEN` for the first auth path.
- Always support `--output json` for networked commands before considering them
  stable enough for agents.
- Model IDs as strings everywhere.
- Preserve server error details, but redact request tokens and secrets.
- Handle pagination explicitly; the API reference says index routes default to
  20 records and can use `?per_page=50`.
