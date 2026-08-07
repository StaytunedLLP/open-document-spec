---
description: "What ODS intentionally excludes from core design — boundaries by module theme."
ods:
  profile: "note"
  status: "stable"
  depends:
    - intro.md
  related:
    - keys.md
    - graph.md
    - assets.md
---

# ODS · Scope

Concepts and features ODS **intentionally excludes** from core. Grouped to match the other modules so you can see “what belongs where” and “what never will.”

Purpose of ODS: [intro.md](intro.md).

---

## Keys & identity

- **No new file extension** — plain `.md` only.
- **No frontmatter `title`** — title is the first `# H1` only.
- **No un-prefixed flat engine keys as the long-term API** — engine metadata lives under nested `ods:`; legacy flat keys are migration-only.
- **No nesting universal keys under `ods:`** — `tags`, `description`, `owner` stay top-level.
- **No parallel `type` taxonomy** — `ods.profile` is the classification.
- **No per-document spec/profile version** — version lives on root `ods.toml` / profile defs.
- **No separate `lifecycle` field** — use `ods.status`.
- **No required hand-maintained `updated` timestamps** — git is authoritative (optional timestamps allowed for non-git authors).
- **No closed tag registries** — free-form top-level tags.

## Indexes & config

- **No nested index lockfiles** — root `ods.toml` + directory tree + CLI discovery are enough.
- **No lock files or derived folder indexes** — no per-commit counts in generated navigation files.
- **No required `llms.txt`** — can be generated; not part of the core workspace contract.
- **No mandatory enterprise namespaces in core schema** — use custom top-level keys when needed.
- **No `.odsignore` as the primary policy file** — use root `ignore:`.

## Graph

- **No extra relationship vocabulary in core** — only `depends` and `related`.
- **No universal frontmatter `url:` field** — external URLs stay in body prose.
- **No hand-written backlinks** — tooling computes reverse edges.

## Assets (resources & code)

- **No frontmatter inside source code** — annotations stay in `.md`.
- **No line numbers as code identity** — paths + symbols only.
- **No project-custom code roles** — fixed role enum for agent portability.
- **No typed schema inside `resources`** — path-only; code needs `code`.
- **No separate asset bucket key** — use `resources`.

## Profiles & templating

- **No profile inheritance / customization engines** — flat schemas only.
- **No standardized rendering/template engine** — ODS is not an SSG.
- **No dedicated `specs` profile** — use `note`, `decision`, or `guide`.
- **No `collection` concept** — directories are enough.

## Transport

- **No special shareability protocol** — git and directories are the transport; `share` only filters export/context.
