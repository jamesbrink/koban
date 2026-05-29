# Resource families

koban exposes Invoice Ninja resources through three command profiles:

- **Full** — the complete [resource command shape](/commands/resources): `list`,
  `show`, `template`, `edit-template`, `create`, `update`, `delete`, `bulk`,
  `action`, `upload`.
- **Inspect-only** — `list` and `show` only. No generic write commands.
- **Invoices** — the full shape plus `download` and `delivery-note`.
- **Guarded partial** — generic commands are present, but commands for routes
  not published by the official API fail locally with an explicit message.
  Some payable/document resources also expose `download`.

In the **library**, a subset of resources has typed model accessors
(`client.invoices()`, etc.); every resource is reachable via the generic
`resource::<T>(Resource)` method. See [Resource accessors](/library/resources).

## CLI resources

| CLI name                 | Profile      | Typed accessor |
| ------------------------ | ------------ | -------------- |
| `clients`                | Full         | `clients()`    |
| `invoices`               | Invoices     | `invoices()`   |
| `payments`               | Full         | `payments()`   |
| `quotes`                 | Full         | `quotes()`     |
| `credits`                | Full         | `credits()`    |
| `vendors`                | Full         | `vendors()`    |
| `expenses`               | Full         | `expenses()`   |
| `projects`               | Full         | `projects()`   |
| `tasks`                  | Full         | `tasks()`      |
| `products`               | Full         | `products()`   |
| `locations`              | Full         | generic        |
| `recurring-invoices`     | Full         | generic        |
| `purchase-orders`        | Full         | generic        |
| `recurring-expenses`     | Full         | generic        |
| `recurring-quotes`       | Full         | generic        |
| `bank-transactions`      | Full         | generic        |
| `bank-integrations`      | Full         | generic        |
| `bank-transaction-rules` | Full         | generic        |
| `group-settings`         | Full         | generic        |
| `expense-categories`     | Full         | generic        |
| `tax-rates`              | Partial      | generic        |
| `payment-terms`          | Full         | generic        |
| `task-schedulers`        | Full         | generic        |
| `task-statuses`          | Full         | generic        |
| `documents`              | Partial      | generic        |
| `designs`                | Full         | generic        |
| `templates`              | Partial      | generic        |
| `users`                  | Full         | generic        |
| `companies`              | Partial      | generic        |
| `company-gateways`       | Full         | generic        |
| `company-users`          | Partial      | generic        |
| `tokens`                 | Full         | generic        |
| `webhooks`               | Full         | generic        |
| `subscriptions`          | Full         | generic        |
| `client-gateway-tokens`  | Partial      | generic        |
| `activities`             | Inspect-only | generic        |
| `system-logs`            | Inspect-only | generic        |
| `company-ledger`         | Inspect-only | generic        |

`download` is supported for `quotes`, `credits`, `recurring-invoices`, and
`purchase-orders` using the official invitation-key PDF routes.

Import/preimport endpoints are not listable resource families in the official
OpenAPI spec. Koban keeps them out of the normal resource table until there is a
dedicated guarded workflow.

## Endpoint runners

These are not resource families; they run named endpoints (see
[Endpoint runners](/commands/endpoints)):

| CLI name  | Command          |
| --------- | ---------------- |
| `reports` | `run --endpoint reports/...` |
| `charts`  | `run --endpoint charts/...`  |
| `search`  | `run`                       |
| `utility` | `run`            |
| `statics` | (top-level read) |
