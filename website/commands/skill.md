# Agent skill

`koban skill` emits a reusable "skill" that teaches an AI coding harness how to
drive koban — its auth, JSON output, and `--dry-run`/`--yes` safety gates. One
shared skill body is wrapped in the frontmatter each harness expects.

Install it once and your agent can **track work in Invoice Ninja automatically**
— logging billable tasks and time, drafting and sending invoices, recording
expenses, and reporting on outstanding balances as it works, instead of you
switching to the web UI.

## Generate vs install

```sh
koban skill generate                       # write to ./koban-skills for review
koban skill generate --dir ./out           # choose the output directory
koban skill install --target claude-code   # write into ./.claude/skills/koban
koban skill install --global --target all  # write into your home config
koban skill install --force --target codex # overwrite existing files
```

`generate` writes files into a directory (default `./koban-skills`) so you can
inspect them first, then prints a hint reminding you to activate them with
`koban skill install` (or copy them into place manually). `install` writes into
the live harness locations — the current project by default, or your user-level
config with `--global`.

## Targets

`--target` is repeatable and defaults to `all`
(`claude-code` + `codex` + `agents-md`).

| Target           | Output                                | Covers                                                |
| ---------------- | ------------------------------------- | ----------------------------------------------------- |
| `claude-code`    | `.claude/skills/koban/SKILL.md`       | Claude Code                                           |
| `codex`          | `.agents/skills/koban/SKILL.md`       | OpenAI Codex CLI                                      |
| `pi`             | `.pi/skills/koban/SKILL.md`           | pi coding agent (native location)                     |
| `agents-md`      | `AGENTS.md` block                     | Cursor, Windsurf, Gemini CLI, Aider, Copilot, Zed …   |
| `claude-desktop` | `koban.zip`                           | Claude Desktop (upload via Settings)                  |
| `cursor`         | `.cursor/rules/koban.mdc`             | Cursor (project rule)                                 |
| `openclaw`       | `skills/koban/SKILL.md`               | OpenClaw (workspace skill, gated on the koban binary) |
| `plugin`         | `koban/.claude-plugin/plugin.json`    | Claude Code plugin bundle                             |
| `all`            | `claude-code` + `codex` + `agents-md` | the practical default bundle                          |

The `.agents/skills/` directory written by the `codex` target is also read by
pi, so most pi setups are already covered by `codex`. Use the dedicated `pi`
target only when you want the skill in pi's native `.pi/skills/` location.

With `--global`, file targets install into `~/.claude`, `~/.agents`,
`~/.pi/agent`, `~/.codex/AGENTS.md`, and `~/.openclaw/skills` (OpenClaw's shared
location). The `claude-desktop` zip and `plugin` bundle are project artifacts and
are written to the output directory regardless of `--global`.

## OpenClaw

`--target openclaw` writes an [OpenClaw](https://openclaw.ai) skill to the
workspace `skills/koban/SKILL.md` (or `~/.openclaw/skills/koban/SKILL.md` with
`--global`). Its frontmatter carries a single-line `metadata` gate
(`requires.bins: ["koban"]`), so OpenClaw only loads the skill when the `koban`
binary is on `PATH`. It stays opt-in — `--target all` does not include it.

## AGENTS.md is idempotent

The `agents-md` target splices a marked block into an existing `AGENTS.md`
(between `<!-- koban:start -->` and `<!-- koban:end -->`), preserving the rest of
the file and replacing the block in place on re-runs — never duplicating it.

## Claude Desktop

`--target claude-desktop` produces `koban.zip` containing `koban/SKILL.md`.
Upload it in Claude Desktop via **Settings → Capabilities → Skills**.

::: tip
Skills uploaded to claude.ai / the Skills API run in a sandbox **without network
access**, so koban cannot reach Invoice Ninja there. The generated skill is
documentation in that context; run koban from a real shell (Claude Code, Codex,
pi, your terminal) for it to work.
:::
