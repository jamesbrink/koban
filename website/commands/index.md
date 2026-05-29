# Commands overview

The implemented `koban` API surface is intentionally boring and durable. Most
work flows through one repeating command shape, plus a handful of special cases
for invoices and endpoint runners.

## The durable command shape

```text
koban statics
koban <resource> list
koban <resource> show <id>
koban <resource> template
koban <resource> edit-template <id>
koban <resource> create
koban <resource> update <id>
koban <resource> delete <id>
koban <resource> bulk
koban <resource> action <id>
koban <resource> upload <id>
koban invoices download <invitation_key>
koban invoices delivery-note <id>
koban quotes download <invitation_key>
koban purchase-orders download <invitation_key>
koban search run
koban reports run --endpoint reports/invoices
koban charts run --endpoint charts/totals
koban utility run
```

See [Resource commands](/commands/resources) for the shared verbs,
[Invoices](/commands/invoices) for the invoice-specific extras, and
[Endpoint runners](/commands/endpoints) for `reports` / `charts` / `search` /
`utility`.

## Global flags

| Flag       | Values          | Default | Purpose                       |
| ---------- | --------------- | ------- | ----------------------------- |
| `--output` | `table`, `json` | `table` | Human tables vs. stable JSON. |

## Resource families

`<resource>` includes:

`clients`, `invoices`, `payments`, `quotes`, `credits`, `vendors`, `expenses`,
`projects`, `tasks`, `locations`, `products`, `recurring-invoices`,
`purchase-orders`, `recurring-expenses`, `recurring-quotes`,
`bank-transactions`, `bank-integrations`, `bank-transaction-rules`,
`group-settings`, `expense-categories`, `tax-rates`, `payment-terms`,
`task-schedulers`, `task-statuses`, `activities`, `system-logs`, `documents`,
`designs`, `templates`, `users`, `companies`, `company-gateways`,
`company-ledger`, `company-users`, `tokens`, `webhooks`, `subscriptions`, and
`client-gateway-tokens`.

Inspect-only/high-risk groups — `activities`, `system-logs`, `company-ledger`,
and similar audit surfaces — expose only safe read commands. Import-style
endpoints are not resource-list commands; use guarded endpoint workflows when
they are added. See the full breakdown in the
[Resource families reference](/reference/resource-families).

## Other commands

```sh
koban --version
koban --help
koban auth login
koban skill install --target claude-code
koban update --check
koban completions zsh
```

- [`koban auth`](/commands/auth) — store, inspect, and remove credentials.
- [`koban skill`](/commands/skill) — generate or install the agent skill.
- [`koban update`](/guide/updating) — self-update from GitHub releases.
- [`koban completions`](/guide/completions) — shell completion scripts.
