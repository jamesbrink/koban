---
applyTo: "**/*.md"
---

Keep docs current with the actual CLI, release workflow, and devshell helpers.
`AGENTS.md` is the source of truth for agent instructions; `CLAUDE.md` must stay
a symlink to `AGENTS.md`.

When changing development guidance, keep AGENTS.md and Copilot instructions in
sync on TDD, coverage expectations, and code-health checks. Avoid documenting
workflow commands that are not present in the devshell or CI.

When documenting Invoice Ninja behavior, preserve the guarded-write safety
posture: read commands are safe by default, implemented invoice writes use
`--dry-run`/`--yes`, and live write smoke tests must be explicit. Document the
public demo API as the preferred target for live smoke tests:
`https://demo.invoiceninja.com` with token `TOKEN`.
