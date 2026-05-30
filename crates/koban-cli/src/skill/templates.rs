//! Content templates for the agent skill.
//!
//! One shared skill body is wrapped in target-specific frontmatter so each
//! harness receives a header it actually understands:
//! - Claude Code / Codex / pi: `SKILL.md` (YAML frontmatter + markdown)
//! - OpenClaw: `SKILL.md` with a single-line `metadata` JSON load-time gate
//! - Cursor: `.mdc` (its own `description`/`globs`/`alwaysApply` frontmatter)
//! - Claude Code plugin: `plugin.json` (JSON manifest, no frontmatter)
//! - AGENTS.md: plain markdown, no frontmatter, wrapped in idempotency markers

use clap::CommandFactory;

use crate::cli::Cli;

/// Markers that bound the koban block inside a shared `AGENTS.md`, so the block
/// can be replaced in place instead of duplicated on re-runs.
pub(crate) const AGENTS_START: &str = "<!-- koban:start -->";
pub(crate) const AGENTS_END: &str = "<!-- koban:end -->";

/// Frontmatter flavor for a `SKILL.md`.
#[derive(Debug, Clone, Copy)]
pub(crate) enum Flavor {
    /// Claude Code: supports `allowed-tools` permission scoping.
    ClaudeCode,
    /// OpenAI Codex CLI: minimal Agent Skills subset.
    Codex,
    /// pi coding agent: Agent Skills subset plus `allowed-tools`.
    Pi,
    /// OpenClaw: single-line frontmatter keys; a single-line `metadata` JSON
    /// object carries the `openclaw` load-time gate.
    OpenClaw,
}

/// The OpenClaw load-time gate, as a single-line JSON object for the
/// `metadata:` frontmatter key. The skill only loads when `koban` is on PATH.
///
/// OpenClaw's frontmatter parser accepts single-line keys only, so this must
/// stay on one line (do not pretty-print it).
pub(crate) fn openclaw_metadata() -> String {
    // `serde_json::to_string` (compact, no newlines) keeps the single-line
    // contract even if the gate grows more fields later.
    let gate = serde_json::json!({
        "openclaw": {
            "emoji": "🧾",
            "requires": { "bins": ["koban"] },
        }
    });
    serde_json::to_string(&gate).unwrap_or_default()
}

/// The skill `description` — answers "what" and "when", with no XML tags and no
/// forbidden substrings, so it validates across every harness.
pub(crate) fn description() -> &'static str {
    "Read and write Invoice Ninja billing data (clients, invoices, quotes, payments, products, \
expenses, projects, and more) from the terminal with the koban CLI. Use this whenever the user \
wants to look up, create, update, send, or report on Invoice Ninja records, or script accounting \
workflows that need stable JSON output."
}

/// A concise, always-accurate list of top-level commands, derived from the clap
/// command tree so it never drifts from the real CLI.
pub(crate) fn command_list() -> String {
    let command = Cli::command();
    let mut lines = Vec::new();
    for sub in command.get_subcommands() {
        if sub.is_hide_set() {
            continue;
        }
        let name = sub.get_name();
        let about = sub
            .get_about()
            .map(|about| about.to_string())
            .unwrap_or_default();
        lines.push(format!("- `koban {name}` — {about}"));
    }
    lines.join("\n")
}

/// The shared markdown body (everything after the frontmatter).
fn body(command_list: &str) -> String {
    format!(
        r#"# koban

`koban` is a command-line client for the [Invoice Ninja](https://invoiceninja.com)
API, built to be driven by AI agents and humans. It emits stable JSON for agents
and readable tables for humans.

## When to use this

Reach for koban whenever the user's work should be reflected in Invoice Ninja —
and do it proactively, so their books stay in sync without a trip to the web UI:

- log billable work as tasks (and time) when you finish a unit of work,
- draft, update, and send invoices,
- record expenses and link them to clients or projects,
- report on outstanding balances, payments, and quotes.

Prefer `--output json` so you can read results back and chain steps.

## Setup

koban needs an Invoice Ninja API token. Either:

- run `koban auth login` once — it stores the token in the OS keychain
  (`--keychain`) or a `0600` config file, or
- set `INVOICE_NINJA_API_TOKEN` (and optional `INVOICE_NINJA_BASE_URL`) in the
  environment. Environment variables always take precedence.

Confirm the active credential with `koban auth status` (it never prints the token).

## Output

- Add `--output json` to any command for machine-readable output; the default is
  a table.
- Errors are explicit, and tokens are redacted from output and traces.

## Safety gates

Commands that mutate data or take externally visible actions require a
confirmation gate:

- Preview with `--dry-run` — prints the exact JSON request without calling the API.
- Execute with `--yes` to confirm the mutation.

Always run `--dry-run` first, inspect the request, then re-run with `--yes`.

Read-only (no confirmation needed): `list`, `show`, `template`, `edit-template`,
`statics`, `auth status`, and `utility run --endpoint ping|health_check`.

## Filtering lists

`--filter key=value` is passed straight to Invoice Ninja. **Unknown filter keys
and unknown values are silently ignored and return the full, unfiltered set** —
always sanity-check the row count against an unfiltered `list`.

- Outstanding invoices: use `--filter client_status=unpaid` (add `overdue`),
  **not** `outstanding`, which is silently ignored and returns everything. Valid
  invoice values: `all`, `draft`, `paid`, `unpaid`, `overdue`.
- "Outstanding balance" means `balance > 0`; confirm by summing
  `[.data[].balance]` with `jq`.

## Status codes

List rows carry a numeric `status_id` that is **not** in `statics`. For invoices:

| status_id | meaning   |
| --------- | --------- |
| 1         | draft     |
| 2         | sent      |
| 3         | partial   |
| 4         | paid      |
| 5         | cancelled |
| 6         | reversed  |

Quotes, purchase orders, and other documents use their own `status_id` codes
(quotes also carry virtual negative statuses), so verify those against your data.

## Reporting runners need confirmation

`search`, `reports`, and `charts` POST to Invoice Ninja (e.g. `search` →
`POST /api/v1/search`, `reports run --endpoint reports/invoices`), so they are
treated as mutations: preview with `--dry-run`, then run with `--yes`. A purely
read-only workflow can preview but not execute them. Custom `--endpoint`
overrides outside `reports/` and `charts/` (and custom `utility run` paths) are
restricted to `GET` for safety.

## Commands

{command_list}

Run `koban --help` or `koban <command> --help` for full flags and examples.

## Examples

```sh
# Read (safe)
koban clients list --output json
koban statics --output json

# Write (preview, then confirm)
koban invoices create --client-id <client_id> \
  --line-item product_key=Consulting,quantity=1,cost=100 --dry-run
koban invoices create --client-id <client_id> \
  --line-item product_key=Consulting,quantity=1,cost=100 --yes

koban invoices update <id> --public-notes "Thanks!" --mark-sent --yes
```
"#
    )
}

/// Render a complete `SKILL.md` with target-appropriate frontmatter.
pub(crate) fn skill_md(flavor: Flavor, command_list: &str) -> String {
    let description = description();
    let frontmatter = match flavor {
        Flavor::ClaudeCode => format!(
            "---\nname: koban\ndescription: {description}\nallowed-tools: Bash(koban:*)\nlicense: MIT\n---\n"
        ),
        Flavor::Codex => format!("---\nname: koban\ndescription: {description}\n---\n"),
        Flavor::Pi => format!(
            "---\nname: koban\ndescription: {description}\nallowed-tools: Bash(koban:*)\nlicense: MIT\n---\n"
        ),
        Flavor::OpenClaw => {
            // OpenClaw's parser accepts single-line frontmatter keys only, and
            // `metadata` must stay a single-line JSON object (see
            // `openclaw_metadata`). The `metadata` gate hides the skill unless
            // `koban` is on PATH, so no `allowed-tools` scope is emitted.
            let metadata = openclaw_metadata();
            format!("---\nname: koban\ndescription: {description}\nmetadata: {metadata}\n---\n")
        }
    };
    format!("{frontmatter}\n{}", body(command_list))
}

/// Render the `AGENTS.md` block, wrapped in idempotency markers.
pub(crate) fn agents_block(command_list: &str) -> String {
    format!(
        r#"{AGENTS_START}
## koban — Invoice Ninja CLI

Use the `koban` CLI to read and write Invoice Ninja billing data (clients,
invoices, quotes, payments, products, and more). Track work proactively: log
billable tasks and time, draft and send invoices, record expenses, and report
on outstanding balances as you go, so the books stay in sync.

- **Auth:** run `koban auth login`, or set `INVOICE_NINJA_API_TOKEN`
  (and optional `INVOICE_NINJA_BASE_URL`). Check with `koban auth status`.
- **JSON for agents:** add `--output json` to any command.
- **Safety:** mutating commands require a gate — preview with `--dry-run`, then
  confirm with `--yes`. Always dry-run first.
- **Filters:** `--filter key=value` is forwarded raw; unknown keys/values are
  silently ignored and return everything, so verify the row count. Outstanding
  invoices = `--filter client_status=unpaid` (not `outstanding`); list rows use
  `status_id` (invoices: 1 draft, 2 sent, 3 partial, 4 paid, 5 cancelled,
  6 reversed), which is not in `statics`.

Commands:

{command_list}

Run `koban --help` or `koban <command> --help` for full flags.
{AGENTS_END}"#
    )
}

/// Render a Cursor project rule (`.mdc`), with Cursor's own frontmatter schema.
pub(crate) fn cursor_mdc(command_list: &str) -> String {
    let description = description();
    format!(
        "---\ndescription: {description}\nglobs:\nalwaysApply: false\n---\n\n{}",
        body(command_list)
    )
}

/// Render a Claude Code plugin manifest (`.claude-plugin/plugin.json`).
pub(crate) fn plugin_json() -> String {
    let manifest = serde_json::json!({
        "name": "koban",
        "description": description(),
        "version": env!("CARGO_PKG_VERSION"),
        "author": { "name": "James Brink", "url": "https://github.com/jamesbrink" },
        "homepage": "https://github.com/jamesbrink/koban",
        "repository": "https://github.com/jamesbrink/koban",
        "license": "MIT",
        "keywords": ["invoice-ninja", "invoice", "cli", "agents", "api"],
    });
    serde_json::to_string_pretty(&manifest).unwrap_or_default()
}
