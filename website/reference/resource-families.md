# Resource families

koban exposes Invoice Ninja resources through three command profiles:

- **Full** — the complete [resource command shape](/commands/resources): `list`,
  `show`, `template`, `edit-template`, `create`, `update`, `delete`, `bulk`,
  `action`, `upload`.
- **Inspect-only** — `list` and `show` only. No generic write commands.
- **Invoices** — the full shape plus `download` and `delivery-note`.

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
| `bank-transactions`      | Full         | generic        |
| `bank-integrations`      | Full         | generic        |
| `bank-transaction-rules` | Full         | generic        |
| `expense-categories`     | Full         | generic        |
| `tax-rates`              | Full         | generic        |
| `payment-terms`          | Full         | generic        |
| `task-statuses`          | Full         | generic        |
| `documents`              | Full         | generic        |
| `designs`                | Full         | generic        |
| `templates`              | Full         | generic        |
| `users`                  | Full         | generic        |
| `companies`              | Full         | generic        |
| `company-gateways`       | Full         | generic        |
| `company-users`          | Full         | generic        |
| `tokens`                 | Full         | generic        |
| `webhooks`               | Full         | generic        |
| `subscriptions`          | Full         | generic        |
| `client-gateway-tokens`  | Full         | generic        |
| `activities`             | Inspect-only | generic        |
| `system-logs`            | Inspect-only | generic        |
| `company-ledger`         | Inspect-only | generic        |
| `imports`                | Inspect-only | generic        |

## Endpoint runners

These are not resource families; they run named endpoints (see
[Endpoint runners](/commands/endpoints)):

| CLI name  | Command          |
| --------- | ---------------- |
| `reports` | `run`            |
| `charts`  | `run`            |
| `search`  | `run`            |
| `utility` | `run`            |
| `statics` | (top-level read) |
