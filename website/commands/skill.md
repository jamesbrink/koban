# Agent skill

`koban skill` installs the embedded Agent Skills standard `SKILL.md` that teaches
AI coding agents how to drive koban — auth, stable JSON output, filters, and the
`--dry-run`/`--yes` safety gates.

Install it once and your agent can **track work in Invoice Ninja automatically**
— logging billable tasks and time, drafting and sending invoices, recording
expenses, and reporting on outstanding balances as it works.

## Common usage

```sh
koban skill install                       # user-wide for detected agents
koban skill install --project             # current project: .claude + .agents
koban skill install claude codex          # install for specific agents
koban skill install --all                 # install every known path
koban skill install copilot --dir ~/repo  # project install into another repo
koban skill list                          # paths and install status
koban skill show                          # print the embedded SKILL.md
koban skill uninstall --project           # remove project-level installed copies
```

With no agent names, a user-wide install targets agents detected on this machine
(their config directory exists), falling back to `~/.agents/skills` when none are
found. A project install (`--project` or `--dir`) defaults to the `.claude/skills`
and `.agents/skills` pair, which covers the supported project-level readers.

Installs overwrite only `<skills dir>/koban/SKILL.md` and never touch sibling
files. `uninstall` removes only that file and removes the `koban/` directory only
when it is empty.

## Supported agents

| Agent name | User-wide skill directory      | Project skill directory |
| ---------- | ------------------------------ | ----------------------- |
| `claude`   | `~/.claude/skills/`            | `.claude/skills/`       |
| `codex`    | `~/.codex/skills/`             | `.agents/skills/`       |
| `pi`       | `~/.pi/agent/skills/`          | `.pi/skills/`           |
| `openclaw` | `~/.openclaw/skills/`          | `skills/`               |
| `copilot`  | `~/.copilot/skills/`           | `.github/skills/`       |
| `cursor`   | `~/.cursor/skills/`            | `.agents/skills/`       |
| `gemini`   | `~/.gemini/skills/`            | `.agents/skills/`       |
| `amp`      | `~/.config/amp/skills/`        | `.agents/skills/`       |
| `goose`    | `~/.config/goose/skills/`      | `.agents/skills/`       |
| `agents`   | `~/.agents/skills/`            | `.agents/skills/`       |

Agents that share a directory are deduplicated, so for example:

```sh
koban skill install codex cursor gemini --project
```

writes one `.agents/skills/koban/SKILL.md` file.

## Repository copy

A canonical copy is checked into the koban repo at `skills/koban/SKILL.md` for
repository-based skill installers. CI verifies it stays byte-identical to the
embedded skill printed by:

```sh
koban skill show
```
