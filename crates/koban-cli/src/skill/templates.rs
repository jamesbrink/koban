//! Content templates for the agent skill.
//!
//! One shared skill body is wrapped in target-specific frontmatter so each
//! harness receives a header it actually understands:
//! - Claude Code / Codex / pi: `SKILL.md` (YAML frontmatter + markdown)
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
    };
    format!("{frontmatter}\n{}", body(command_list))
}

/// Render the `AGENTS.md` block, wrapped in idempotency markers.
pub(crate) fn agents_block(command_list: &str) -> String {
    format!(
        r#"{AGENTS_START}
## koban — Invoice Ninja CLI

Use the `koban` CLI to read and write Invoice Ninja billing data (clients,
invoices, quotes, payments, products, and more).

- **Auth:** run `koban auth login`, or set `INVOICE_NINJA_API_TOKEN`
  (and optional `INVOICE_NINJA_BASE_URL`). Check with `koban auth status`.
- **JSON for agents:** add `--output json` to any command.
- **Safety:** mutating commands require a gate — preview with `--dry-run`, then
  confirm with `--yes`. Always dry-run first.

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
