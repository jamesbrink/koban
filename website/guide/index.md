# What is koban?

`koban` is a small, scriptable Rust CLI for [Invoice Ninja](https://www.invoiceninja.com/).
The goal is a tool that feels good for humans at a terminal and predictable for
AI agents that need stable JSON output, explicit errors, and shell completions.

The project is a Cargo workspace with two crates:

- [`koban`](https://crates.io/crates/koban) — the reusable Invoice Ninja API
  **client library** (`cargo add koban`).
- [`koban-cli`](https://crates.io/crates/koban-cli) — the **command-line tool**,
  which installs a `koban` binary (`cargo install koban-cli`).

The CLI boots, reports its version, generates shell completions, exposes a broad
Invoice Ninja API surface, and includes guarded write commands.

## Design principles

- **One durable command shape.** Every resource family follows the same verbs —
  `list`, `show`, `template`, `edit-template`, `create`, `update`, `delete`,
  `bulk`, `action`, and `upload`. Learn it once; it works across 30+ resources.
- **Humans and agents both.** Human output uses a comfortable table layout;
  `--output json` emits a stable shape for pipelines, `grep`, and `jq`.
- **Guarded writes.** Mutations require `--yes`, and `--dry-run` previews the
  exact request before anything leaves your machine. Inspect-only resources never
  expose write verbs.
- **Token safety.** Tokens are read from the environment and never printed —
  not in output, errors, traces, or logs.

## Why "koban"?

A _koban_ (小判) was an oval gold coin minted during Japan's Edo period — and the
currency ninja were literally paid in. The name nods to both the ninja theme and
to money, which suits a tool for an invoicing API. It is short, clean on
crates.io, and unambiguous.

## Next steps

- [Installation](/guide/installation) — `curl | sh`, `cargo install`, or Nix.
- [Quickstart](/guide/quickstart) — your first commands against the demo API.
- [Commands overview](/commands/) — the full command shape and resource families.
- [Use as a library](/library/) — depend on `koban` from your own Rust app.
