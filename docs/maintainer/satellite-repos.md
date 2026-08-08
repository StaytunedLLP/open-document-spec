---
profile: note
status: draft
share: public
description: First-cut mapping of monorepo trees to open-doc-spec satellite repositories.
---

# Satellite repositories (first cut)

Product surfaces are being extracted into dedicated repos under
[github.com/open-doc-spec](https://github.com/open-doc-spec). This monorepo remains
the **engine / CLI** source of truth.

## Policy (first cut)

| Rule | Detail |
|------|--------|
| Satellite SoT | After seed PRs merge, **prefer editing the satellite** for that surface |
| Monorepo mirrors | In-tree copies below are **kept for first cut** (tests, local demos, packaging) |
| Hard delete | **Not** done yet — follow-up when CLI/docs no longer need local trees |
| Install scripts | Canonical: monorepo `src/scripts/install.{sh,ps1}` — sync into site/skill copies |

## Mapping

| Surface | Monorepo path (mirror) | Satellite (SoT after merge) | Visibility |
|---------|------------------------|-----------------------------|------------|
| Site / domain app | `app-web/` | [open-doc-spec/opendocify.com](https://github.com/open-doc-spec/opendocify.com) | **Private** |
| Normative specs | `specs/` | [open-doc-spec/ods-spec](https://github.com/open-doc-spec/ods-spec) | Public |
| Benchmark fixtures | `src/fixtures/benchmarks/` | [open-doc-spec/ods-benchmarks](https://github.com/open-doc-spec/ods-benchmarks) | Public |
| End-user skill | `skills/ods/` | [open-doc-spec/ods-skills](https://github.com/open-doc-spec/ods-skills) | Public |
| GitHub Action | `action.yml` + `src/action/` | [open-doc-spec/ods-action](https://github.com/open-doc-spec/ods-action) | Public |
| Engine / CLI | `src/ods-core`, `src/ods-cli`, … | **This repo** (`ods`) | Public |
| Org defaults | `.github/` (this repo only) | [open-doc-spec/.github](https://github.com/open-doc-spec/.github) | Public |

## Notes

- **Specs vs schema**: Markdown under `specs/` is normative documentation.
  Runtime keys remain schema-driven in `src/ods-core/src/spec/schema.rs`.
- **Benchmarks**: Fixtures only move; `ods bench` engine stays here.
- **Action consumers**: Prefer `uses: open-doc-spec/ods-action@v1` once tagged;
  monorepo action remains a mirror for first cut.
- **Skill**: Satellite has `SKILL.md` at repo root; monorepo keeps `skills/ods/`.

Tracking parent: [open-doc-spec/ods#34](https://github.com/open-doc-spec/ods/issues/34).
