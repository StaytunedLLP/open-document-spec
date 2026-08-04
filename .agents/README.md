# `.agents/` — maintainer agent surface

Portable agent config for this repository (Grok, Claude Code, Cursor, and similar tools often discover `.agents/`).

| Path | Role |
|---|---|
| `skills/` | Maintainer skills (edit specs, multi-spec, release docs) |
| `rules/` | Always-on short rules (also mirrored by nested `AGENTS.md`) |
| `agents/` | Custom subagent / persona prompts |
| `hooks/` | Optional lifecycle hooks (lint after spec edits) |

**End-user product skill** remains `skills/ods/` (installable via `ods skill install`).  
**Always-on product locks** remain root [`AGENTS.md`](../AGENTS.md).

## Start here for agents

1. Root `AGENTS.md` — CLI surface, schema-driven keys, quality gates, odc policy
2. Spec intro — `specs/ods/intro.md`
3. Spec keys — `specs/ods/keys.md`
4. Schema how-to — `docs/maintainer/schema-driven-keys.md` when adding keys/dialects
5. This folder’s skills when the task matches

## Pre-handoff (engine work)

```bash
SKIP_RELEASE_BUILD=true ./src/scripts/check-local.sh
ODS_COVERAGE_FAIL_UNDER_LINES=90 ./src/scripts/coverage.sh
```
