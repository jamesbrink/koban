---
name: koban
description: Read and write Invoice Ninja billing data (clients, invoices, quotes, payments, products, expenses, projects, and more) from the terminal with the koban CLI. Use this whenever the user wants to look up, create, update, send, or report on Invoice Ninja records, or script accounting workflows that need stable JSON output.
allowed-tools: Bash(koban:*)
license: MIT
metadata: {"openclaw":{"emoji":"🧾","requires":{"bins":["koban"]}}}
---

# koban — Invoice Ninja CLI

`koban` is a command-line client for the [Invoice Ninja](https://invoiceninja.com) API, built for humans and AI agents. It emits stable JSON for agents and readable tables for humans.

## Quick reference

```bash
koban auth status                         # Show active credential source, never the token
koban clients list --output json          # List clients as stable JSON
koban invoices list --filter client_status=unpaid --output json
koban invoices show <id> --output json
koban invoices create --client-id <id> --line-item product_key=Consulting,quantity=1,cost=100 --dry-run
koban invoices create --client-id <id> --line-item product_key=Consulting,quantity=1,cost=100 --yes
koban payments list --output json
koban reports run --endpoint reports/invoices --dry-run
koban skill install                       # Install this skill for detected agents
koban skill install --project             # Install project-level skill copies
koban skill list                          # Show supported agents, paths, and install status
```

## When to use this skill

Use koban whenever the user's work should be reflected in Invoice Ninja:

- look up clients, invoices, quotes, payments, products, expenses, projects, tasks, and related records;
- log billable work as tasks and time entries when you finish a unit of work;
- draft, update, and send invoices or quotes;
- record expenses and connect them to clients or projects;
- report on outstanding balances, payments, and quote/invoice status.

Prefer `--output json` for agent workflows so results can be parsed reliably.

## Auth and setup

koban needs an Invoice Ninja API token. Either:

- run `koban auth login` once, optionally with `--keychain`, or
- set `INVOICE_NINJA_API_TOKEN` and optional `INVOICE_NINJA_BASE_URL` in the environment.

Environment variables always win over stored credentials. Confirm the active source with `koban auth status`; it never prints the token.

If `koban` is not on `PATH`, install it with the repository installer:

```bash
curl -fsSL https://raw.githubusercontent.com/jamesbrink/koban/main/install.sh | sh
```

It is also available as `cargo install koban-cli` and as prebuilt binaries on GitHub releases.

## Safety rules

Mutating or externally visible commands require an explicit confirmation gate:

1. Preview with `--dry-run` and inspect the JSON request.
2. Execute only after the preview is correct by re-running with `--yes`.

Read-only commands do not need confirmation, including `list`, `show`, `template`, `edit-template`, `statics`, `auth status`, and safe utility endpoints such as `ping` and `health_check`.

`search`, `reports`, and `charts` POST to Invoice Ninja, so koban treats their runners as mutations. Use `--dry-run` first, then `--yes` only when execution is intended.

## Filters and status pitfalls

`--filter key=value` is forwarded to Invoice Ninja. Unknown filter keys and values are silently ignored and can return the full unfiltered set, so always sanity-check row counts.

- Outstanding invoices: use `--filter client_status=unpaid`, optionally with `overdue`; do **not** use `outstanding`.
- "Outstanding balance" means `balance > 0`; confirm by summing `.data[].balance` with `jq`.
- List rows carry numeric `status_id` values that are not in `statics`.

Invoice `status_id` values:

| status_id | meaning   |
| --------- | --------- |
| 1         | draft     |
| 2         | sent      |
| 3         | partial   |
| 4         | paid      |
| 5         | cancelled |
| 6         | reversed  |

Quotes, purchase orders, and other documents use their own status codes; verify against actual data.

## Common commands

- `koban statics` — reference data such as countries, currencies, and statuses
- `koban clients` — list, show, and inspect clients
- `koban invoices` — list, show, create, update, and manage invoices
- `koban payments` — list, show, and inspect payments
- `koban quotes` — list, show, and inspect quotes
- `koban credits` — list, show, and inspect credits
- `koban vendors` — list, show, and inspect vendors
- `koban expenses` — list, show, and inspect expenses
- `koban projects` — list, show, and inspect projects
- `koban tasks` — list, show, and inspect tasks
- `koban products` — list, show, and manage products
- `koban reports` — query reports with confirmation gates
- `koban charts` — query charts with confirmation gates
- `koban search` — search across Invoice Ninja records with confirmation gates
- `koban utility` — utility endpoints such as ping, health-check, refresh, and preview
- `koban auth` — store, inspect, and remove credentials
- `koban skill` — install, list, show, and uninstall this agent skill
- `koban update` — check or install GitHub release updates

Run `koban --help` or `koban <command> --help` for the full command surface and flags.

## Skill management

The koban binary embeds this `SKILL.md` and can install it where supported agents look:

```bash
koban skill install                       # User-wide for detected agents, fallback ~/.agents/skills
koban skill install --project             # Current project: .claude/skills + .agents/skills
koban skill install claude codex          # Specific agents only
koban skill install --all                 # Every supported user/project path
koban skill install copilot --dir ~/repo  # Project install into another repo
koban skill list                          # Supported agents, paths, install status
koban skill show                          # Print this SKILL.md
koban skill uninstall --project           # Remove project-level installed copies
```

Supported agent names: `claude`, `codex`, `pi`, `openclaw`, `copilot`, `cursor`, `gemini`, `amp`, `goose`, and `agents`. Explicit project installs for `openclaw` use the repository `skills/koban/SKILL.md` workspace path.

Install overwrites only `koban/SKILL.md` in the target skill directory and never touches sibling files. Uninstall removes only that file and removes the now-empty `koban/` directory when possible.

## Agent patterns

Always ask for JSON when reading data:

```bash
koban clients list --output json | jq '.data[] | {id, name}'
koban invoices list --filter client_status=unpaid --output json | jq '[.data[].balance] | add'
```

For writes, preview first:

```bash
koban invoices update <invoice_id> --public-notes "Thanks!" --dry-run
koban invoices update <invoice_id> --public-notes "Thanks!" --yes
```

Do not print, log, or store API tokens. Redact credentials from errors, traces, fixtures, and docs.
