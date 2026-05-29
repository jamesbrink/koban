# Safety & guardrails

koban is designed so that destructive or externally visible operations are hard
to trigger by accident.

## Dry-run previews and confirmation gates

- `--dry-run` previews the exact request koban _would_ send — method, URL, and
  body — without contacting the API. Nothing leaves your machine.
- `--yes` is required to actually perform a guarded mutation.

Generic resource `create` / `update` / `delete` / `bulk` / `upload` / `action`
commands require `--yes` unless `--dry-run` is used.

```sh
koban invoices delete <id> --dry-run    # preview
koban invoices delete <id> --yes        # confirm and send
```

Invoice `create` / `update` keep a lighter workflow for ordinary draft edits,
but still require `--yes` when they cause externally visible state changes — for
example marking sent, sending email, marking paid, cancelling, saving default
footer/terms, or retrying e-send.

## Read-only by default for risky surfaces

- **Inspect-only resources** — `activities`, `system-logs`, and
  `company-ledger` — expose only safe reads. Import/preimport endpoints are not
  listable resource families and need a dedicated guarded workflow before they
  become first-class commands.
- **`template` / `edit-template`** use Invoice Ninja's read-only `GET /create`
  and `GET /{id}/edit` routes. They return schema/default payloads and never
  create or update records.
- **Custom endpoint overrides** (`--endpoint`) are read-only and only send `GET`
  requests. Use first-class resource commands for mutations.
- **PDF downloads** use read-only `GET` routes and write to explicit file paths.
  Existing files are not overwritten unless `--force` is set.

## Token redaction

Tokens are read from the environment and never printed by default — not in
output, errors, traces, fixtures, or docs.

## Live smoke testing

Read-only live smoke tests should use the public demo endpoint
(`https://demo.invoiceninja.com`, token `TOKEN`) by default. Live _write_ smoke
tests must be explicit and should target the public demo API. High-risk
endpoints — purges, refunds, merges, imports, scheduler, and admin utility
routes — should only be exercised by a dedicated smoke helper that creates and
cleans up its own demo data. Production or personal accounts should only be used
for intentional checks.
