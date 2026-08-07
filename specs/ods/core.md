---
description: "Open Document Spec Working Draft 1 — format model, conformance levels, and lifecycle operations."
ods:
  profile: "note"
  status: "stable"
  depends:
    - intro.md
    - keys.md
  related:
    - graph.md
    - indexes.md
    - profiles.md
    - validation.md
---

# ODS · Core

**Open Document Spec — Working Draft 1**

The key words **MUST**, **MUST NOT**, **SHOULD**, **SHOULD NOT**, and **MAY** are to be interpreted as described in RFC 2119.

End-user overview: [intro.md](intro.md). Full key dictionary: [keys.md](keys.md).

## Design principles (priority order)

1. **Human first**: readable in any text editor, forever.
2. **Free at level zero**: standard Markdown is already fully conformant; adopting ODS means enriching documents, never forcing a migration rewrite.
3. **Token efficient (DRY)**: every fact has exactly one canonical location.
4. **Graph native**: relationships are explicit frontmatter, not inferred from prose alone.
5. **Trust from validation**: the specification does not require rules that cannot be automatically validated or linted.

---

## Format model

An ODS Document is a Markdown file with optional YAML frontmatter.

- **Frontmatter** is machine-readable metadata for tools (identity, lifecycle, graph edges, assets, context).
- **Body** is human-readable prose (purpose, rationale, decisions, usage). Body MUST NOT redefine metadata already present in frontmatter.
- Plain Markdown without frontmatter remains a valid **Level-0** document.

Frontmatter is a single YAML block delimited by `---` at the start of the file. **All fields are optional.** Tools MUST ignore unknown fields for extension and SSG interoperability.

Key placement (universal top-level vs nested `ods:`) is normative in [keys.md](keys.md). Tools MUST emit engine keys under `ods:` and universal keys at top level when scaffolding or migrating (`ods fmt --migrate`).

Frontmatter MUST NOT contain a `title:` key. The document title exists only as the first `# H1` in the body.

---

## Core concepts

| Concept | Description |
| :--- | :--- |
| **Workspace** | Directory tree marked by a root `ods.toml` (or `index.md`) with scalar `ods:` (spec version, e.g. `0.1`). |
| **Document** | A Markdown file (`.md`), optionally with YAML frontmatter. |
| **Index** | Generated navigation file listing a directory's **immediate** children (`ods.toml` preferred). |
| **Resource** | Any non-Markdown file referenced via `ods.resources` (CSV, PNG, PDF, OpenAPI, …). |

There are no other core concepts. Bundles, archives, and groups are ordinary directories. `collection` is not a core concept.

---

## Compliance (binary)

ODS workspaces are either **compliant** or **non-compliant**. There is no Level 0–3 ladder.

| State | Meaning |
| :--- | :--- |
| **ODS workspace** | Root `ods.toml` with `spec` present |
| **Compliant** | `ods lint` reports zero errors (graph integrity, refs, assets, profiles) |
| **Non-compliant** | One or more lint errors; fix and re-lint |
| **Plain Markdown** | `.md` without workspace marker — not an ODS workspace until `ods init` |

Example document frontmatter:

```markdown
---
ods:
  profile: guide
  status: draft
---

# Checkout Setup
```

Parsers MUST accept legacy top-level flat `profile` / `status` for migration; canonical emit nests them under `ods:`. See [validation.md](validation.md) and [indexes.md](indexes.md) (`ods.toml` + CLI discovery).

---

## Specification modules

| Module | Topic |
| :--- | :--- |
| [intro.md](intro.md) | Purpose and reading map |
| [keys.md](keys.md) | Frontmatter dictionary |
| [profiles.md](profiles.md) | Profiles, catalogs, packs |
| [graph.md](graph.md) | IDs, SSOT, depends/related |
| [context.md](context.md) | AI context scope |
| [indexes.md](indexes.md) | Indexes and workspace scan |
| [assets.md](assets.md) | Resources and code references |
| [validation.md](validation.md) | Lint rules and tooling contract |
| [scope.md](scope.md) | Intentional non-goals |

---

## Lifecycle operations

Tools MUST support four atomic document operations:

1. **Scaffold (`ods new <path>`)** — create Level-1 frontmatter (`profile`, `status: draft`, optional `description`), path-derived ID, profile section placeholders, keep graph refs current.
2. **Relocate (`ods mv <from> <to>`)** — move/rename while rewriting path-derived IDs, prose links, `depends`, `related`, `code[].path`, and index entries.
3. **Archive (`ods archive <path-or-id>`)** — set `status: archived`, keep graph edges current, preserve graph edges (implementation MAY move under `archive/`).
4. **Delete (`ods rm <path-or-id>`)** — remove file, drop from parent indexes, scrub ID from all `depends` / `related` arrays.

### Smart profile inference

When adopting or creating untyped documents, tools SHOULD scan `##` and `###` headings to infer `profile`:

| Headings (examples) | Profile |
| :--- | :--- |
| Goal, Scope, Requirements, Acceptance Criteria, Risks | `feature` |
| Overview, Prerequisites, Steps, Troubleshooting | `guide` |
| Context, Decision, Alternatives, Consequences | `decision` |
| Purpose, Rules, Exceptions, Validation, Rollback | `sop` |
| Request, Response, Errors, Examples, Endpoint | `api` |
| Attendees, Agenda, Decisions, Action Items | `meeting` |
| Q/A, Questions, Answers, FAQ | `faq` |
