# Quickstart

## 1. Point koban at an Invoice Ninja API

koban is environment-first. The fastest way to try it is the public Invoice
Ninja demo API, which accepts the demo token `TOKEN`:

```sh
export INVOICE_NINJA_BASE_URL="https://demo.invoiceninja.com"
export INVOICE_NINJA_API_TOKEN="TOKEN"
```

For your own account, point at hosted production (`https://invoicing.co`, the
default) or your self-hosted base URL, and use a real API token. See
[Configuration](/guide/configuration) for details.

## 2. Read some data

```sh
koban statics --output json
koban clients list --page 1 --per-page 20
koban clients show <id> --output json
koban invoices list
koban invoices list --filter status_id=gt:1 --sort 'date|desc' --all --limit 50
koban invoices show <id>
```

List commands accept raw Invoice Ninja query filters and sorting via `--filter`
and `--sort`. `--all` paginates (capped at 100 pages); `--limit` bounds the row
count.

## 3. Discover a schema

The `template` and `edit-template` commands return default/editable payloads
straight from Invoice Ninja's read-only `GET /create` and `GET /{id}/edit`
routes — handy for seeing the exact field shape before you write:

```sh
koban invoices template --output json
koban invoices edit-template <id> --output json
```

## 4. Preview a write, then confirm

Writes are guarded. Use `--dry-run` to preview the exact request, then `--yes`
to actually send it:

```sh
# Preview — nothing leaves your machine.
koban invoices create --client-id <client_id> \
  --line-item product_key=Consulting,quantity=1,cost=100 \
  --dry-run

# Send it.
koban invoices create --client-id <client_id> \
  --line-item product_key=Consulting,quantity=1,cost=100 \
  --yes
```

Read more about the guardrails in [Safety](/guide/safety), and the full command
shape in the [Commands overview](/commands/).
