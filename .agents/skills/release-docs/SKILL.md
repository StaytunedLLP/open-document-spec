---
name: release-docs
description: >-
  Keep guide, site, skill, llms.txt, and CHANGELOG in sync after spec or CLI
  surface changes.
---

# Release docs sync

When `specs/**`, CLI flags, key placement, or **user-facing error catalog** change:

1. **Schema registry** — if keys changed, update `src/ods-core/src/spec/schema.rs` in the same change set
1b. **Error catalog** — if CLI/lifecycle messages changed: `src/ods-core/src/error/messages.rs` + `docs/guide/07-troubleshooting-and-diagnostics.md` + skill error directive
2. **Specs** — `specs/{ods,okf,skills}/` consistent indexes
3. **Guide** — `docs/guide/*` links and summaries (no forked key rules)
4. **Other-specs** — comparisons point at `specs`, not old flat paths; archive plans under `docs/plan/archive/`
5. **Site** — `app-web` Layout nav, sitemap, `spec.astro` default entry, redirects
6. **Skill** — `skills/ods/SKILL.md` + `skills/ods/references/*`
7. **Agents** — root `AGENTS.md`, `.agents/rules` if product locks changed
8. **llms.txt** — `app-web/public/llms.txt` entrypoints (intro + keys first)
9. **Install scripts** — `src/scripts/install.*` is source of truth; sync `app-web/public/` + `skills/ods/scripts/`
10. **CHANGELOG** — Unreleased bullet

## Grep dead paths + false CLI claims

```bash
# from repo root
grep -RInE 'SPEC\.md|resources-and-code|/spec/spec|non-goals\.md' \
  --include='*.md' --include='*.astro' --include='*.ts' --include='*.txt' \
  docs app-web/src app-web/public skills AGENTS.md README.md || true

# Do not teach export-as-routine-context or missing flags
grep -RInE 'export graph.*token|related.*chains.*context' skills/ods README.md docs/guide || true
```

After skill edits: confirm `ods skill install` bundle in `skill_command.rs` still lists current `references/*`.

## Optional verify

```bash
ods lint
cd app-web && npm run build
```
