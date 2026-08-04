# `.agents/` — maintainer agent surface

Portable agent config for this repository (Grok, Claude Code, Cursor, and similar tools often discover `.agents/`).

| Path | Role |
|---|---|
| `skills/` | Maintainer skills (edit specs, multi-spec, release docs, **quality-gate**) |
| `rules/` | Always-on short rules (also summarized by root `AGENTS.md`) |
| `agents/` | Custom subagent / persona prompts |
| `hooks/` | Post-edit hints + **pre-handoff** cheap gates |

**End-user product skill** remains `skills/ods/` (installable via `ods skill install`).  
**Always-on product locks** remain root [`AGENTS.md`](../AGENTS.md) (**hand-maintained** in this monorepo).

## Start here

1. Root `AGENTS.md` — locks, **token reliability**, Definition of Done  
2. Rules: `00-cli-surface.md`, `30-schema-keys.md`, `35-token-context.md`, `40-quality-gates.md`  
3. Spec intro/keys — `specs/ods/intro.md`, `keys.md` (on demand only)  
4. **One** skill by task (do not load all):
   - **quality-gate** — sequenced pre-commit / coverage / finish work  
   - **spec-author** — `specs/**`  
   - **multi-spec** — flags / dialects  
   - **release-docs** — site/skill/CHANGELOG sync  

**Budget:** always-on rules + one skill. Prefer `ods context <id>` over full-tree reads.

## Pre-handoff

```bash
.agents/hooks/scripts/pre-handoff.sh
.agents/hooks/scripts/pre-handoff.sh --full
# coverage when needed:
ODS_COVERAGE_HANDOFF=1 .agents/hooks/scripts/pre-handoff.sh --full
```

## `ods agents sync` ownership

- **This monorepo:** root `AGENTS.md` is hand-owned (has `.agents/rules/`). Sync **must not** overwrite it; it only refreshes editor snippets.  
- **User workspaces** (no `.agents/rules/`): `ods agents sync` may write a generated `AGENTS.md`.
