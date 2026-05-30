# koban

[![CI](https://github.com/jamesbrink/koban/actions/workflows/ci.yml/badge.svg)](https://github.com/jamesbrink/koban/actions/workflows/ci.yml)
[![koban on crates.io](https://img.shields.io/crates/v/koban.svg?label=koban)](https://crates.io/crates/koban)
[![koban-cli on crates.io](https://img.shields.io/crates/v/koban-cli.svg?label=koban-cli)](https://crates.io/crates/koban-cli)
[![docs.rs](https://img.shields.io/docsrs/koban?label=docs.rs)](https://docs.rs/koban)
[![Docs](https://img.shields.io/badge/docs-website-D4AF37)](https://jamesbrink.online/koban/)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

`koban` is a small, scriptable Rust CLI and client library for
[Invoice Ninja](https://www.invoiceninja.com/), built for humans at a terminal
and AI agents that need stable JSON output, explicit errors, and shell
completions.

The name is a nod to the _koban_ (小判), the Edo-period oval gold coin ninja were
paid in — a short, currency-flavored name for a tool that drives an invoicing
API.

📖 **Documentation: <https://jamesbrink.online/koban/>**

The project is a Cargo workspace with two crates:

- [`koban`](https://crates.io/crates/koban) — the reusable Invoice Ninja API
  **client library** (`cargo add koban`).
- [`koban-cli`](https://crates.io/crates/koban-cli) — the **command-line tool**,
  which installs a `koban` binary (`cargo install koban-cli`).

The CLI exposes a broad Invoice Ninja API surface with guarded write commands,
stable JSON for automation, human-friendly tables, shell completions, direct
binary installs, and a reusable Rust client library.

## Install

For direct binary installs on macOS and Linux:

```sh
curl -fsSL https://raw.githubusercontent.com/jamesbrink/koban/main/install.sh | sh
```

Installer options:

```sh
# A specific release tag, into a custom directory
curl -fsSL https://raw.githubusercontent.com/jamesbrink/koban/main/install.sh | \
  KOBAN_VERSION="v0.2.0" KOBAN_INSTALL_DIR="$HOME/.local/bin" sh

# The rolling nightly build from main
curl -fsSL https://raw.githubusercontent.com/jamesbrink/koban/main/install.sh | \
  KOBAN_VERSION="nightly" sh
```

`KOBAN_VERSION` accepts `latest` (default, newest stable release), `nightly`
(the rolling prerelease built from `main`), or a tag such as `v0.2.0`. Pass
`--help` to the script (`sh install.sh --help`) to print usage.

The installer downloads GitHub release tarballs, verifies `SHA256SUMS` when
available, installs `koban` into `~/.local/bin` by default, and prints the
installed version. It uses the same macOS/Linux asset names as release CI and
`koban update`.

Other install paths:

```sh
cargo install koban-cli
nix run github:jamesbrink/koban -- --help
```

## Use with AI agents

koban is built to be driven by AI coding agents. One command teaches your agent
the whole CLI:

```sh
koban skill install --target all          # Claude Code, Codex, and an AGENTS.md block
koban skill install --target claude-code  # or pick a single harness
koban skill generate                      # write to ./koban-skills to review first
```

Once the skill is installed, an agent can **track your work in Invoice Ninja
automatically** — logging billable tasks and time, drafting and sending
invoices, recording expenses, and reporting on outstanding balances — while it
works, instead of you context-switching to the web UI. The skill teaches the
agent koban's stable JSON output and its `--dry-run`/`--yes` safety gates, so
every write is previewed before it happens.

Supported harnesses include Claude Code, OpenAI Codex CLI, pi, Cursor, OpenClaw,
Claude Desktop, and any tool that reads `AGENTS.md` (Windsurf, Gemini CLI, Aider,
Copilot, Zed, …). See the
[agent skill docs](https://jamesbrink.online/koban/commands/skill).

## Current CLI

```sh
koban --help
koban --version
koban update --check
koban completions zsh
koban completions bash
koban completions fish
koban completions nushell

# Authentication (stores the token in a 0600 config file or the OS keychain)
koban auth login --token <token>
koban auth login --keychain
echo "$TOKEN" | koban auth login --no-verify
koban auth status
koban auth logout

# Agent skill (teach Claude Code, Codex, pi, Cursor, ... how to drive koban)
koban skill generate                       # write to ./koban-skills for review
koban skill install --target claude-code   # into ./.claude/skills/koban
koban skill install --global --target all  # into ~/.claude, ~/.agents, AGENTS.md
koban skill install --target openclaw       # into ./skills/koban (OpenClaw workspace)
# OpenClaw users can also install straight from Git:
#   openclaw skills install git:jamesbrink/koban
koban skill install --target claude-desktop # build koban.zip to upload
```

Credentials resolve in this order, so agents and CI stay deterministic:
`INVOICE_NINJA_API_TOKEN` env → OS keychain → stored config file. Set
`KOBAN_CONFIG_DIR` to override where the config file lives.

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
koban invoices update <id> --public-notes "Thanks again" --mark-sent --yes
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
koban reports run --data-file report.json --dry-run
```

Generic resource single-record actions use Invoice Ninja's bulk endpoint when
that is the published upstream shape. Resources with official single-record
action routes, such as payments, quotes, purchase orders, recurring invoices,
and recurring quotes, use `GET /api/v1/{resource}/{id}/{action}`.
Endpoint runner payload flags are accepted only for `POST` and `PUT`; `GET` and
`DELETE` reject payloads so dry-runs cannot show a body that the live request
would ignore.

## Use as a library

The `koban` crate is a standalone Invoice Ninja API client that other Rust
applications can depend on:

```sh
cargo add koban
```

```rust
use koban::{ApiClient, Config};

#[tokio::main]
async fn main() -> koban::Result<()> {
    // Reads INVOICE_NINJA_API_TOKEN and optional INVOICE_NINJA_BASE_URL.
    let client = ApiClient::new(Config::from_env()?);

    // Typed resource accessors return the built-in models.
    for invoice in client.invoices().list().await? {
        println!("{} -> {}", invoice.number, invoice.balance);
    }

    // Or work with any resource and your own type / serde_json::Value.
    let client_record = client.clients().get("client_id").await?;
    println!("{}", client_record.display_name);

    Ok(())
}
```

The typed models (`Invoice`, `Client`, `Payment`, ...) are forward-compatible:
unknown fields are preserved in an `extra` map, and the raw JSON methods
(`client.get_json(...)`, `post_json`, ...) remain available as a low-level escape
hatch. The library depends only on `thiserror` for its error type by default;
enable the `miette` feature for diagnostic help text on `KobanError`.

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
koban quotes download <invitation_key>
koban purchase-orders download <invitation_key>
koban search run
koban reports run --endpoint reports/invoices
koban charts run --endpoint charts/totals
koban utility run
```

Custom endpoint overrides are read-only and only send `GET` requests. Use
first-class resource commands for mutations.

`<resource>` includes `clients`, `invoices`, `payments`, `quotes`, `credits`,
`vendors`, `expenses`, `projects`, `tasks`, `locations`, `products`,
`recurring-invoices`, `purchase-orders`, `recurring-expenses`,
`recurring-quotes`, `bank-transactions`, `bank-integrations`,
`bank-transaction-rules`, `group-settings`, `expense-categories`, `tax-rates`,
`payment-terms`, `task-schedulers`, `task-statuses`, `activities`,
`system-logs`, `documents`, `designs`, `templates`, `users`, `companies`,
`company-gateways`, `company-ledger`, `company-users`, `tokens`, `webhooks`,
`subscriptions`, and `client-gateway-tokens`.

Inspect-only/audit groups `activities`, `system-logs`, and `company-ledger`
expose only safe reads. Import/preimport endpoints are not listable resource
families in the official OpenAPI spec, so they are intentionally left for a
dedicated guarded workflow.

Some official resources publish narrower route sets than the generic command
shape. Koban rejects unsupported commands locally before networking, such as
`documents upload`, `tax-rates create`, or `templates list`.

The `template` and `edit-template` commands use Invoice Ninja's read-only
`GET /create` and `GET /{id}/edit` routes. They return default/editable payloads
for schema discovery; they do not create or update records.

List commands accept raw Invoice Ninja query filters and sorting:

```sh
koban clients list --filter balance=gt:1000 --filter name=Bob --sort 'name|desc'
koban invoices list --all --limit 100 --output json
```

`--all` stops after 100 pages to avoid accidental unbounded traversal. JSON
output includes `meta.page_cap_reached` when that guardrail is hit.

Invoice download commands also use read-only `GET` routes and write PDF bytes to
explicit file paths. Existing files are not overwritten unless `--force` is set.

Write commands accept either one raw JSON source or guided flags. Resource
writes expose broad guided fields such as `--name`, `--number`, `--client-id`,
`--vendor-id`, `--project-id`, `--date`, `--due-date`, `--amount`, `--price`,
`--quantity`, notes, repeatable `--field key=value`, and repeatable
`--line-item key=value,...` for document-like resources. Raw JSON cannot be
combined with guided fields or `--line-item`. Generic `--field` values parse
JSON-like scalars (`true`, `false`, `null`, and numbers); quote a field value to
force a JSON string, such as `--field number='"1000"'`.

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
`--yes` unless `--dry-run` is used. Generic endpoint runner defaults may use
non-GET methods with `--yes`, but custom `--endpoint` overrides are GET-only.
Invoice create/update keep
their lighter workflow for ordinary draft edits, but require `--yes` when they
mark sent, send email, mark paid, cancel, save default footer/terms, retry
e-send, or otherwise cause externally visible state changes.

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
koban           cargo run -p koban-cli -- ...
koban-help      show koban help
smoke-statics   safe live GET /api/v1/statics smoke test
smoke-invoice-write-demo  explicit demo-only invoice create/update/delete smoke test
smoke-all-demo  explicit demo-only smoke test for every implemented command family
docs-dev        start the VitePress dev server for the docs site
docs-build      build the docs site (static output in website/.vitepress/dist)
docs-preview    preview the built documentation site
docs-fmt        format documentation with prettier
docs-fmt-check  check documentation formatting (matches CI)
```

The documentation website lives in `website/` (VitePress) and is published to
GitHub Pages at <https://jamesbrink.online/koban/> by
`.github/workflows/pages.yml` on pushes to `main` that touch `website/`.

To keep this repo safe for AI agents, the devshell **forces koban to the public
demo endpoint by default**: it exports the demo `INVOICE_NINJA_BASE_URL` /
`INVOICE_NINJA_API_TOKEN` (overriding any `.env` or inherited values) and points
`KOBAN_CONFIG_DIR` at a gitignored repo-local `.koban/`, so a stored
`koban auth login` credential is never resolved here. An agent loading the koban
skill can only reach the demo account.

To use live credentials for an intentional check, enter the shell with
`KOBAN_ALLOW_LIVE=1`:

```sh
KOBAN_ALLOW_LIVE=1 nix develop
```

Under `KOBAN_ALLOW_LIVE=1`, `INVOICE_NINJA_API_TOKEN` and
`INVOICE_NINJA_BASE_URL` are loaded from a local gitignored `.env` file when not
already set in the shell, and the stored credential is reachable:

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

Releases are managed by release-please across the workspace, keeping the `koban`
library and `koban-cli` binary crates on a single linked version. When a release
is cut, CI builds unsigned CLI tarballs for macOS and Linux, uploads
`SHA256SUMS`, and publishes both crates to crates.io (library first, then the
CLI) using `CARGO_REGISTRY_TOKEN`. The binary release keeps the prefix-free
`vX.Y.Z` tag that `install.sh` and `koban update` rely on.

The nightly workflow builds the current `main` branch into a rolling
`nightly` prerelease. It uses a `nightly-staging` release while compiling so
the previous nightly stays available to updater clients until the new assets
are ready.

The `install.sh` script is the supported `curl | sh` path for direct installs:

```sh
curl -fsSL https://raw.githubusercontent.com/jamesbrink/koban/main/install.sh | sh
curl -fsSL https://raw.githubusercontent.com/jamesbrink/koban/main/install.sh | KOBAN_VERSION=v0.2.0 sh
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
koban update --tag v0.2.0
koban update --nightly
```

## API Notes

The working notes in [docs/invoice-ninja-api.md](docs/invoice-ninja-api.md) are
grounded in the current Invoice Ninja documentation and should be refreshed
before adding new networked command groups.
