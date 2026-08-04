---
name: spec-author
description: >-
  Edit Open Document Spec modules under specs/{ods,okf,skills}. Use when
  changing normative or author-facing dialect docs, renames, or reading maps.
---

# Spec author (maintainer)

## Layout

```
specs/
  index.ods.md
  ods/   intro keys core profiles graph context indexes assets validation scope
  okf/   intro keys
  skills/ intro keys
```

## Rules

1. Read `specs/ods/intro.md` and `keys.md` before inventing frontmatter.
2. Keep **one idea per file**. Do not re-dump the full key table into `core.md`.
3. ODS voice:
   - `intro` — plain language for end users
   - `keys` — dictionary + examples
   - `core` / others — precise; RFC 2119 where normative
4. Link with **relative** paths inside the dialect folder.
5. After edits, grep for dead paths (`SPEC.md`, `non-goals`, `resources-and-code`, flat `/spec/graph`).
6. Sync skill copies under `skills/ods/references/` when ODS intro/keys/core/scope change.
7. Update `app-web` nav + `sitemap.xml.ts` + redirects in `astro.config.mjs` if routes change.

## Checklist

- [ ] File name is one word and matches content
- [ ] Frontmatter `order` / `description` set for site sorting
- [ ] Indexes (`index.ods.md`) list new children
- [ ] If keys changed: update `src/ods-core/src/spec/schema.rs` in the same PR (registry is engine SoT)
- [ ] Sync `skills/ods/references/` when ODS intro/keys/core/scope change
- [ ] CHANGELOG Unreleased note
