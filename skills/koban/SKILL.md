---
name: koban
description: Read and write Invoice Ninja billing data (clients, invoices, quotes, payments, products, expenses, projects, and more) from the terminal with the koban CLI. Use this whenever the user wants to look up, create, update, send, or report on Invoice Ninja records, or script accounting workflows that need stable JSON output.
metadata: {"openclaw":{"emoji":"🧾","requires":{"bins":["koban"]}}}
---

# koban

`koban` is a command-line client for the [Invoice Ninja](https://invoiceninja.com)
API, built to be driven by AI agents and humans. It emits stable JSON for agents
and readable tables for humans.

## When to use this

Reach for koban whenever the user's work should be reflected in Invoice Ninja —
and do it proactively, so their books stay in sync without a trip to the web UI:

- log billable work as tasks (and time) when you finish a unit of work,
- draft, update, and send invoices,
- record expenses and link them to clients or projects,
- report on outstanding balances, payments, and quotes.

Prefer `--output json` so you can read results back and chain steps.

## Install

If `koban` is not already on your `PATH`, install it (the script auto-detects
your OS/arch and verifies checksums):

```sh
curl -fsSL https://raw.githubusercontent.com/jamesbrink/koban/main/install.sh | sh
```

It is also on crates.io (`cargo install koban-cli`) and ships prebuilt binaries
on each [GitHub release](https://github.com/jamesbrink/koban/releases).

## Setup

koban needs an Invoice Ninja API token. Either:

- run `koban auth login` once — it stores the token in the OS keychain
  (`--keychain`) or a `0600` config file, or
- set `INVOICE_NINJA_API_TOKEN` (and optional `INVOICE_NINJA_BASE_URL`) in the
  environment. Environment variables always take precedence.

Confirm the active credential with `koban auth status` (it never prints the token).

## Output

- Add `--output json` to any command for machine-readable output; the default is
  a table.
- Errors are explicit, and tokens are redacted from output and traces.

## Safety gates

Commands that mutate data or take externally visible actions require a
confirmation gate:

- Preview with `--dry-run` — prints the exact JSON request without calling the API.
- Execute with `--yes` to confirm the mutation.

Always run `--dry-run` first, inspect the request, then re-run with `--yes`.

Read-only (no confirmation needed): `list`, `show`, `template`, `edit-template`,
`statics`, `auth status`, and `utility run --endpoint ping|health_check`.

## Filtering lists

`--filter key=value` is passed straight to Invoice Ninja. **Unknown filter keys
and unknown values are silently ignored and return the full, unfiltered set** —
always sanity-check the row count against an unfiltered `list`.

- Outstanding invoices: use `--filter client_status=unpaid` (add `overdue`),
  **not** `outstanding`, which is silently ignored and returns everything. Valid
  invoice values: `all`, `draft`, `paid`, `unpaid`, `overdue`.
- "Outstanding balance" means `balance > 0`; confirm by summing
  `[.data[].balance]` with `jq`.

## Status codes

List rows carry a numeric `status_id` that is **not** in `statics`. For invoices:

| status_id | meaning   |
| --------- | --------- |
| 1         | draft     |
| 2         | sent      |
| 3         | partial   |
| 4         | paid      |
| 5         | cancelled |
| 6         | reversed  |

Quotes, purchase orders, and other documents use their own `status_id` codes
(quotes also carry virtual negative statuses), so verify those against your data.

## Reporting runners need confirmation

`search`, `reports`, and `charts` POST to Invoice Ninja (e.g. `search` →
`POST /api/v1/search`, `reports run --endpoint reports/invoices`), so they are
treated as mutations: preview with `--dry-run`, then run with `--yes`. A purely
read-only workflow can preview but not execute them. Custom `--endpoint`
overrides outside `reports/` and `charts/` (and custom `utility run` paths) are
restricted to `GET` for safety.

## Commands

- `koban statics` — Show reference data such as countries, currencies, and statuses
- `koban clients` — List, show, and inspect clients
- `koban invoices` — List, show, create, update, and manage invoices
- `koban payments` — List, show, and inspect payments
- `koban quotes` — List, show, and inspect quotes
- `koban credits` — List, show, and inspect credits
- `koban vendors` — List, show, and inspect vendors
- `koban expenses` — List, show, and inspect expenses
- `koban projects` — List, show, and inspect projects
- `koban tasks` — List, show, and inspect tasks
- `koban locations` — List, show, and manage locations
- `koban products` — List, show, and manage products
- `koban recurring-invoices` — List, show, and manage recurring invoices
- `koban purchase-orders` — List, show, and manage purchase orders
- `koban recurring-expenses` — List, show, and manage recurring expenses
- `koban recurring-quotes` — List, show, and manage recurring quotes
- `koban bank-transactions` — List, show, and manage bank transactions
- `koban bank-integrations` — List, show, and manage bank integrations
- `koban bank-transaction-rules` — List, show, and manage bank transaction rules
- `koban group-settings` — List, show, and manage group settings
- `koban expense-categories` — List, show, and manage expense categories
- `koban tax-rates` — List, show, and manage tax rates
- `koban payment-terms` — List, show, and manage payment terms
- `koban task-schedulers` — List, show, and manage task schedulers
- `koban task-statuses` — List, show, and manage task statuses
- `koban activities` — List, show, and inspect activities
- `koban system-logs` — List, show, and inspect system logs
- `koban documents` — List, show, and manage documents
- `koban designs` — List, show, and manage designs
- `koban templates` — List, show, and manage templates
- `koban users` — List, show, and manage users
- `koban companies` — List, show, and manage companies
- `koban company-gateways` — List, show, and manage company gateways
- `koban company-ledger` — List and inspect company ledger entries
- `koban company-users` — List, show, and manage company users
- `koban tokens` — List, show, and manage API tokens
- `koban webhooks` — List, show, and manage webhooks
- `koban subscriptions` — List, show, and manage subscriptions
- `koban client-gateway-tokens` — List, show, and manage client gateway tokens
- `koban reports` — Query reports
- `koban charts` — Query charts
- `koban search` — Search across Invoice Ninja records
- `koban utility` — Call utility endpoints such as ping, health-check, refresh, and preview
- `koban auth` — Store, inspect, and remove Invoice Ninja credentials
- `koban skill` — Generate or install an agent skill that teaches a harness to use koban
- `koban update` — Check or install GitHub release updates
- `koban completions` — Print shell completion scripts

Run `koban --help` or `koban <command> --help` for full flags and examples.

## Examples

```sh
# Read (safe)
koban clients list --output json
koban statics --output json

# Write (preview, then confirm)
koban invoices create --client-id <client_id> \
  --line-item product_key=Consulting,quantity=1,cost=100 --dry-run
koban invoices create --client-id <client_id> \
  --line-item product_key=Consulting,quantity=1,cost=100 --yes

koban invoices update <id> --public-notes "Thanks!" --mark-sent --yes
```
