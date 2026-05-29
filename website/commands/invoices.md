# Invoices

Invoices follow the [shared resource command shape](/commands/resources) and add
a few first-class extras: PDF downloads and richer create/update flags.

## Read & template

```sh
koban invoices list
koban invoices list --filter status_id=gt:1 --sort 'date|desc' --all --limit 50
koban invoices show <id>
koban invoices show <id> --include client
koban invoices template --output json
koban invoices edit-template <id> --output json
```

## Create & update

```sh
koban invoices create --client-id <client_id> \
  --line-item product_key=Consulting,quantity=1,cost=100 --dry-run
koban invoices create --data-file invoice.json --include client
koban invoices update <id> --data-file invoice.json --dry-run
koban invoices update <id> --public-notes "Thanks again" --mark-sent --yes
```

Invoice `create` / `update` keep a lighter workflow for ordinary draft edits,
but require `--yes` when they mark sent, send email, mark paid, cancel, save
default footer/terms, retry e-send, or otherwise cause externally visible state
changes.

## Delete, bulk & action

```sh
koban invoices delete <id> --dry-run
koban invoices delete <id> --yes
koban invoices bulk --action archive --id <id> --id <id> --dry-run
koban invoices action <id> --action mark_paid --dry-run
koban invoices action <id> --action mark_paid --yes
```

## Uploads

```sh
koban invoices upload <id> --file contract.pdf --dry-run
koban invoices upload <id> --file contract.pdf --yes
```

## PDF downloads

Invoice PDF downloads use read-only `GET` routes and write bytes to explicit
file paths. They remain first-class because their route shape is documented and
invitation-key based. Existing files are not overwritten unless `--force` is set.

```sh
koban invoices download <invitation_key> --output-file invoice.pdf
koban invoices delivery-note <id> --output-file delivery-note.pdf
```
