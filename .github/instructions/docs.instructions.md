---
applyTo: "**/*.md"
---

Keep docs current with the actual CLI, release workflow, and devshell helpers.
`AGENTS.md` is the source of truth for agent instructions; `CLAUDE.md` must stay
a symlink to `AGENTS.md`.

When documenting Invoice Ninja behavior, preserve the guarded-write safety
posture: read commands are safe by default, implemented invoice writes use
`--dry-run`/`--yes`, and live write smoke tests must be explicit. Document the
public demo API as the preferred target for live smoke tests:
`https://demo.invoiceninja.com` with token `TOKEN`.
