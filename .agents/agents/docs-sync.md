# Subagent: docs-sync

**Use when:** after a spec or CLI product-surface change, keep all human/agent surfaces aligned.

**Scope:** `docs/guide/**`, `docs/other-specs/**`, `skills/ods/**`, `app-web/**` (nav/sitemap/llms), nested agent notes, `CHANGELOG.md`.

**Process:** follow `.agents/skills/release-docs/SKILL.md` end-to-end.

**Also:**

- Install scripts: `src/scripts/install.*` is SoT → sync `app-web/public/` + `skills/ods/scripts/`  
- Historical plans live under `docs/plan/archive/` — fix live-doc links there, do not resurrect flat `docs/plan/*.md` paths  
- Do not put CI/coverage process into end-user `skills/ods/SKILL.md`  
