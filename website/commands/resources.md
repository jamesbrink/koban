# Resource commands

Every resource family shares the same verbs. The examples below use `clients`
and `invoices`, but the shape is identical across resources (subject to the
[inspect-only restrictions](/reference/resource-families)).

## Read

```sh
koban statics --output json
koban clients list --page 1 --per-page 20
koban clients show <id> --output json
koban clients template --output json
koban clients edit-template <id> --output json
```

- `list` accepts raw Invoice Ninja query filters and sorting via repeatable
  `--filter key=value` and `--sort 'field|dir'`, plus `--page`, `--per-page`,
  `--all`, and `--limit`. `--all` stops after 100 pages; JSON output includes
  `meta.page_cap_reached` when that guardrail is hit.
- `template` and `edit-template` use Invoice Ninja's read-only `GET /create` and
  `GET /{id}/edit` routes. They return default/editable payloads for schema
  discovery; they do not create or update records.

```sh
koban clients list --filter balance=gt:1000 --filter name=Bob --sort 'name|desc'
koban invoices list --all --limit 100 --output json
```

## Write

Write commands accept either one raw JSON source **or** guided flags — the two
cannot be combined.

### Raw JSON

```sh
koban invoices create --data '{"client_id":"...","line_items":[]}' --dry-run
koban invoices create --data-file invoice.json --include client
printf '%s' '{"public_notes":"Updated"}' | koban invoices update <id> --stdin
```

`--data`, `--data-file`, and `--stdin` are the three raw-JSON sources.

### Guided flags

Resource writes expose broad guided fields such as `--name`, `--number`,
`--client-id`, `--vendor-id`, `--project-id`, `--date`, `--due-date`,
`--amount`, `--price`, `--quantity`, notes flags, a repeatable
`--field key=value`, and a repeatable `--line-item key=value,...` for
document-like resources.

```sh
koban products create --name Consulting --price 100 --dry-run
koban products update <id> --field notes="Hourly support" --dry-run
koban products delete <id> --dry-run
koban clients create --field name=Acme --field contacts.email=ap@example.test --dry-run
koban invoices create --client-id <client_id> \
  --line-item product_key=Consulting,quantity=1,cost=100 --dry-run
```

Generic `--field` values parse JSON-like scalars (`true`, `false`, `null`, and
numbers); quote a value to force a JSON string, e.g. `--field number='"1000"'`.
Raw JSON cannot be combined with guided fields or `--line-item`.

## Bulk and single-record actions

```sh
koban invoices bulk --action archive --id <id> --id <id> --dry-run
koban invoices action <id> --action mark_paid --dry-run
koban recurring-invoices action <id> --action start --dry-run
```

Generic resource single-record actions are sent through Invoice Ninja's bulk
action endpoint with a one-item `ids` list, matching the upstream API shape.

## Uploads

```sh
koban invoices upload <id> --file contract.pdf --dry-run
koban invoices upload <id> --file contract.pdf --yes
```

## Guardrails

Generic `create` / `update` / `delete` / `bulk` / `upload` / `action` commands
require `--yes` unless `--dry-run` is used. See [Safety](/guide/safety) for the
full set of guardrails.
