# koban

`koban` is an early-stage Rust CLI for [Invoice Ninja](https://www.invoiceninja.com/).
The goal is a small, scriptable tool that feels good for humans at a terminal
and predictable for AI agents that need stable JSON output, explicit errors, and
shell completions.

The crate name is claimed on crates.io as `koban` at `0.0.1`. This repository is
still early work: the CLI boots, reports its version, generates shell
completions, exposes a broad Invoice Ninja API surface, and includes guarded
write commands.

## Install

For direct binary installs on macOS and Linux:

```sh
curl -fsSL https://raw.githubusercontent.com/jamesbrink/koban/main/install.sh | sh
```

Installer options:

```sh
curl -fsSL https://raw.githubusercontent.com/jamesbrink/koban/main/install.sh | \
  KOBAN_VERSION="v0.1.0" KOBAN_INSTALL_DIR="$HOME/.local/bin" sh
```

The installer downloads GitHub release tarballs, verifies `SHA256SUMS` when
available, installs `koban` into `~/.local/bin` by default, and prints the
installed version. It uses the same macOS/Linux asset names as release CI and
`koban update`.

Other install paths:

```sh
cargo install koban
nix run github:jamesbrink/koban -- --help
```

## Current CLI

```sh
koban --help
koban --version
koban update --check
koban completions zsh
koban completions bash
koban completions fish
koban completions nushell
```

The implemented API commands cover read workflows, guided/JSON writes,
bulk/custom actions, and uploads across Invoice Ninja resource families.
Invoice-specific PDF downloads remain first-class because their route shape is
documented and invitation-key based:

```sh
koban statics --output json
koban clients list --page 1 --per-page 20
koban clients show <id> --output json
koban clients template --output json
koban clients edit-template <id> --output json
koban invoices list
koban invoices list --filter status_id=gt:1 --sort 'date|desc' --all --limit 50
koban invoices show <id>
koban invoices template --output json
koban invoices edit-template <id> --output json
koban invoices create --client-id <client_id> --line-item product_key=Consulting,quantity=1,cost=100 --dry-run
koban invoices create --data-file invoice.json --include client
koban invoices update <id> --data-file invoice.json --dry-run
koban invoices update <id> --public-notes "Thanks again" --mark-sent
koban invoices delete <id> --dry-run
koban invoices delete <id> --yes
koban invoices bulk --action archive --id <id> --id <id> --dry-run
koban invoices action <id> --action mark_paid --dry-run
koban invoices upload <id> --file contract.pdf --dry-run
koban invoices download <invitation_key> --output-file invoice.pdf
koban invoices delivery-note <id> --output-file delivery-note.pdf
koban payments list
koban payments show <id>
koban payments template --output json
koban payments edit-template <id> --output json
koban quotes list
koban quotes show <id>
koban quotes template --output json
koban quotes edit-template <id> --output json
koban credits list
koban credits show <id>
koban credits template --output json
koban credits edit-template <id> --output json
koban vendors list
koban vendors show <id>
koban vendors template --output json
koban vendors edit-template <id> --output json
koban expenses list
koban expenses show <id>
koban expenses template --output json
koban expenses edit-template <id> --output json
koban projects list
koban projects show <id>
koban projects template --output json
koban projects edit-template <id> --output json
koban tasks list
koban tasks show <id>
koban tasks template --output json
koban tasks edit-template <id> --output json
koban products create --name Consulting --price 100 --dry-run
koban products update <id> --field notes="Hourly support" --dry-run
koban products delete <id> --dry-run
koban recurring-invoices action <id> --action start --dry-run
koban search run --field query=acme --dry-run
koban reports run --endpoint reports --data-file report.json --dry-run
```

Recurring invoice single-record actions are sent through Invoice Ninja's bulk
action endpoint with a one-item `ids` list, matching the upstream API shape.
Endpoint runner payload flags are accepted only for `POST` and `PUT`; `GET` and
`DELETE` reject payloads so dry-runs cannot show a body that the live request
would ignore.

## Invoice Ninja Direction

Invoice Ninja v5 exposes an API under `/api/v1`. Hosted production is
`https://invoicing.co`, and self-hosted installs use the same namespace under
their own base URL. Invoice Ninja also provides a public demo API at
`https://demo.invoiceninja.com` with the demo token `TOKEN`; use that target for
live smoke tests whenever possible.

Authentication is token based. Requests require `X-API-TOKEN`, and the developer
guide also documents `X-Requested-With: XMLHttpRequest` as a required security
header. JSON write requests must send `Content-Type: application/json`.

The implemented `koban` API surface is intentionally boring and durable. The
core resource command shape is:

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
koban search run
koban reports run
koban charts run
koban utility run
```

`<resource>` includes `clients`, `invoices`, `payments`, `quotes`, `credits`,
`vendors`, `expenses`, `projects`, `tasks`, `locations`, `products`,
`recurring-invoices`, `purchase-orders`, `recurring-expenses`,
`bank-transactions`, `bank-integrations`, `bank-transaction-rules`,
`expense-categories`, `tax-rates`, `payment-terms`, `task-statuses`,
`activities`, `system-logs`, `documents`, `designs`, `templates`, `users`,
`companies`, `company-gateways`, `company-ledger`, `company-users`, `tokens`,
`webhooks`, `imports`, `subscriptions`, and `client-gateway-tokens`.

The `template` and `edit-template` commands use Invoice Ninja's read-only
`GET /create` and `GET /{id}/edit` routes. They return default/editable payloads
for schema discovery; they do not create or update records.

List commands accept raw Invoice Ninja query filters and sorting:

```sh
koban clients list --filter balance=gt:1000 --filter name=Bob --sort 'name|desc'
koban invoices list --all --limit 100 --output json
```

Invoice download commands also use read-only `GET` routes and write PDF bytes to
explicit file paths. Existing files are not overwritten unless `--force` is set.

Write commands accept either one raw JSON source or guided flags. Resource
writes expose broad guided fields such as `--name`, `--number`, `--client-id`,
`--vendor-id`, `--project-id`, `--date`, `--due-date`, `--amount`, `--price`,
`--quantity`, notes, repeatable `--field key=value`, and repeatable
`--line-item key=value,...` for document-like resources:

```sh
koban invoices create --data '{"client_id":"...","line_items":[]}' --dry-run
koban invoices create --data-file invoice.json --include client
printf '%s' '{"public_notes":"Updated"}' | koban invoices update <id> --stdin
koban invoices create --client-id <client_id> \
  --line-item product_key=Consulting,quantity=1,cost=100 \
  --dry-run
koban products create --name Consulting --price 100 --dry-run
koban clients create --field name=Acme --field contacts.email=ap@example.test --dry-run
```

Generic resource create/update/delete/bulk/upload/action commands require
`--yes` unless `--dry-run` is used. Generic endpoint runners also require
`--yes` for non-GET methods unless they are dry runs. Invoice create/update keep
their lighter workflow for ordinary draft edits, but require `--yes` when they
send email, mark paid, cancel, retry e-send, or otherwise cause externally
visible state changes.

## Configuration Plan

The intended configuration model is environment-first:

```sh
export INVOICE_NINJA_BASE_URL="https://invoicing.co"
export INVOICE_NINJA_API_TOKEN="..."
```

`INVOICE_NINJA_BASE_URL` is optional and defaults to `https://invoicing.co`.
Tokens must never be printed by default, and human-facing output should have a
matching JSON mode before it ships.

For demo smoke tests:

```sh
export INVOICE_NINJA_BASE_URL="https://demo.invoiceninja.com"
export INVOICE_NINJA_API_TOKEN="TOKEN"
```

## Safety

Read-only live smoke tests should use the public demo endpoint above by default.
Write support is implemented with `--dry-run` and `--yes` guardrails, but live
write smoke tests must be explicit and should target the public demo API.
Purges, refunds, merges, imports, scheduler, and admin utility endpoints should
only be exercised when a dedicated smoke helper creates and cleans up its own
demo data. Production or personal accounts should only be used for intentional
checks.

## Development

This repo pins Rust in `rust-toolchain.toml` and exposes the same toolchain
through the Nix flake.

```sh
cargo fmt --all -- --check
cargo check
cargo clippy -- -D warnings
cargo test
cargo build --release
nix flake check
```

With Nix:

```sh
nix develop
nix build
nix run . -- --help
```

Inside `nix develop`, the devshell menu exposes helper commands:

```text
build           cargo build (debug)
build-release   cargo build --release
check           cargo check
clippy          cargo clippy -- -D warnings
fmt             cargo fmt
fmt-check       cargo fmt --all -- --check
run-tests       cargo test
ci-local        run the Rust-side CI sequence
code-health     check Rust source files against module size budgets
coverage        cargo llvm-cov summary, or --html for a report
koban           cargo run -- ...
koban-help      show koban help
smoke-statics   safe live GET /api/v1/statics smoke test
smoke-invoice-write-demo  explicit demo-only invoice create/update/delete smoke test
smoke-all-demo  explicit demo-only smoke test for every implemented command family
```

The devshell also loads `INVOICE_NINJA_API_TOKEN` and
`INVOICE_NINJA_BASE_URL` from a local gitignored `.env` file when those
variables are not already set in the shell. For routine live smoke testing, use
the demo values:

```dotenv
INVOICE_NINJA_API_TOKEN=TOKEN
INVOICE_NINJA_BASE_URL=https://demo.invoiceninja.com
```

The demo write smoke helpers are intentionally opt-in and hard-code the public
demo API internally so they cannot inherit a production or personal endpoint:

```sh
KOBAN_LIVE_WRITE_SMOKE=1 smoke-invoice-write-demo
KOBAN_LIVE_WRITE_SMOKE=1 smoke-all-demo
```

`smoke-all-demo` live-reads every resource that the public demo API exposes,
allows documented demo-only 404s for unsupported reference pages, dry-runs every
expanded resource write family, and performs a create/update/upload/action/
download/bulk/delete invoice lifecycle with cleanup.

The flake exports `packages.default`, `packages.koban`, `apps.default`,
`apps.koban`, `checks.koban`, and a development shell for Linux and Darwin on
both x86_64 and aarch64.

## Releases

Releases are managed by release-please. When a release is cut, CI builds
unsigned CLI tarballs for macOS and Linux, uploads `SHA256SUMS`, and publishes
the crate to crates.io using `CARGO_REGISTRY_TOKEN`.

The nightly workflow builds the current `main` branch into a rolling
`nightly` prerelease. It uses a `nightly-staging` release while compiling so
the previous nightly stays available to updater clients until the new assets
are ready.

The `install.sh` script is the supported `curl | sh` path for direct installs:

```sh
curl -fsSL https://raw.githubusercontent.com/jamesbrink/koban/main/install.sh | sh
curl -fsSL https://raw.githubusercontent.com/jamesbrink/koban/main/install.sh | KOBAN_VERSION=v0.1.0 sh
curl -fsSL https://raw.githubusercontent.com/jamesbrink/koban/main/install.sh | KOBAN_VERSION=nightly sh
curl -fsSL https://raw.githubusercontent.com/jamesbrink/koban/main/install.sh | KOBAN_INSTALL_DIR=/usr/local/bin sh
```

`koban update` downloads those GitHub release tarballs and verifies checksums
for direct installs. Package-manager installs are left alone and get an upgrade
recipe instead:

```sh
koban update --check
koban update --nightly --check
koban update
koban update --tag v0.1.0
koban update --nightly
```

## API Notes

The working notes in [docs/invoice-ninja-api.md](docs/invoice-ninja-api.md) are
grounded in the current Invoice Ninja documentation and should be refreshed
before adding new networked command groups.
