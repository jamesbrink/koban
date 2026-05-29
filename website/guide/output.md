# Output: tables & JSON

koban has a global `--output` flag with two modes:

- `--output table` (default) — a comfortable, human-readable table.
- `--output json` — a stable JSON shape for pipelines, `grep`, and `jq`.

```sh
# Human-friendly tables (default).
koban clients list

# Stable JSON for scripts and agents.
koban clients list --output json | jq '.data[].display_name'
koban invoices show <id> --output json | jq '.data.balance'
```

## Stable shape for agents

JSON output is intentionally stable so agents and scripts can rely on it. List
responses preserve Invoice Ninja's `data` array and `meta`; single-record
responses preserve the `data` object.

## Pagination guardrails

List commands accept raw Invoice Ninja query filters and sorting:

```sh
koban clients list --filter balance=gt:1000 --filter name=Bob --sort 'name|desc'
koban invoices list --all --limit 100 --output json
```

`--all` stops after 100 pages to avoid accidental unbounded traversal. When that
guardrail is hit, JSON output includes `meta.page_cap_reached` so you can detect
truncation programmatically.

## PDF downloads

PDF downloads write bytes to an explicit file path rather than to stdout.
Existing files are not overwritten unless `--force` is set:

```sh
koban invoices download <invitation_key> --output-file invoice.pdf
koban invoices delivery-note <id> --output-file delivery-note.pdf
koban quotes download <invitation_key> --output-file quote.pdf
koban purchase-orders download <invitation_key> --output-file purchase-order.pdf
```

See [Invoices](/commands/invoices) and [Resource commands](/commands/resources)
for details.
