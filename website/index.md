---
layout: home

hero:
  name: koban
  text: Invoice Ninja from the terminal
  tagline:
    A small, scriptable Rust CLI and client library for Invoice Ninja, built for
    humans at a terminal and AI agents that need stable JSON, explicit errors,
    and shell completions.
  actions:
    - theme: brand
      text: Get Started →
      link: /guide/
    - theme: alt
      text: Commands
      link: /commands/
    - theme: alt
      text: GitHub
      link: https://github.com/jamesbrink/koban

features:
  - title: Drop-in agent skill
    details: koban skill install teaches Claude Code, Codex, Cursor, and any
      AGENTS.md-aware tool to drive koban — so your agent tracks work in Invoice
      Ninja automatically, with guarded writes it always previews first.
  - title: One durable command shape
    details:
      Every resource family follows the same verbs — list, show, template,
      edit-template, create, update, delete, bulk, action, upload — across
      clients, invoices, payments, quotes, and 30+ more.
  - title: Built for humans and agents
    details:
      Comfy table output by default; --output json emits a stable shape for
      pipelines, grep, and jq. Errors are explicit and never print your token.
  - title: Guarded writes
    details:
      Mutations require --yes, and --dry-run previews the exact request before
      anything leaves your machine. Inspect-only resources never expose write
      verbs.
  - title: A real client library too
    details: cargo add koban gives you a typed Invoice Ninja client — ApiClient,
      Config::from_env(), and forward-compatible models that preserve unknown
      fields.
  - title: Shell completions everywhere
    details:
      Generate completions for bash, zsh, fish, nushell, elvish, and PowerShell
      straight from the binary.
  - title: Single binary, easy install
    details:
      curl | sh installer with checksum verification, cargo install koban-cli,
      or nix run. Self-update with koban update, including nightly builds.
---

## At a glance

```sh
# Point koban at the public Invoice Ninja demo API.
export INVOICE_NINJA_BASE_URL="https://demo.invoiceninja.com"
export INVOICE_NINJA_API_TOKEN="TOKEN"

koban clients list --per-page 20
koban invoices list --filter status_id=gt:1 --sort 'date|desc' --all
koban invoices show <id> --output json | jq '.data.balance'

# Writes are guarded: preview first, then confirm.
koban invoices create --client-id <id> \
  --line-item product_key=Consulting,quantity=1,cost=100 --dry-run
koban invoices action <id> --action mark_paid --yes
```

Use it as a library from any Rust app:

```rust
use koban::{ApiClient, Config};

#[tokio::main]
async fn main() -> koban::Result<()> {
    let client = ApiClient::new(Config::from_env()?);
    for invoice in client.invoices().list().await? {
        println!("{} -> {}", invoice.number, invoice.balance);
    }
    Ok(())
}
```

## Track your work automatically with AI agents

Install koban as a skill and your coding agent drives it for you:

```sh
koban skill install --target all
```

Now an agent can log billable tasks and time, draft and send invoices, record
expenses, and report on outstanding balances in Invoice Ninja **as it works** —
using koban's stable JSON and its `--dry-run`/`--yes` safety gates so every
write is previewed first. Works with Claude Code, OpenAI Codex CLI, pi, Cursor,
Claude Desktop, and any `AGENTS.md`-aware tool.

[Read the agent skill guide →](/commands/skill)

<div class="tip custom-block" style="padding-top: 8px">

New here? Start with the [Installation](/guide/installation) and
[Quickstart](/guide/quickstart) guides, or jump to the
[Commands overview](/commands/).

</div>
