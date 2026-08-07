---
description: "What Open Document Spec is, why it exists, core concepts, and a reading map of every ODS module."
ods:
  profile: "note"
  status: "stable"
  related:
    - keys.md
    - core.md
    - profiles.md
    - graph.md
    - context.md
    - indexes.md
    - assets.md
    - validation.md
    - scope.md
---

# ODS · Intro

Open Document Spec (**ODS**) is a **Markdown-first** way to document software and knowledge in a git repository so both **people** and **AI agents** can navigate it reliably.

Your files stay plain `.md`. You add optional YAML frontmatter when you want structure—types, relationships, code links, and AI reading lists. Nothing requires a new file extension or a proprietary database.

**Why it exists:** flat Markdown folders drift, links break, and AI tools waste tokens searching the whole repo. ODS makes identity, relationships, and context **explicit and lintable**, so docs stay trustworthy as code changes.

**Coexistence with SSGs:** ODS is designed for zero-friction coexistence alongside Hugo, Astro, Jekyll, Docusaurus, Next.js, and Obsidian. Non-ODS frontmatter keys, custom metadata, and comments are preserved untouched across all CLI operations.

**How it is used:** the `ods` CLI is the default engine (no `--ods` flag). Extra dialects use flags only: `--okf` (Google OKF v0.2) and `--skills` (Agent Skills packages). Sibling dialects: [`../`](../).

---

## Core concepts (glossary)

| Term | Meaning |
| :--- | :--- |
| **Workspace** | A document tree whose root **`ods.toml`** declares `spec` (e.g. `"0.1"`). |
| **Document** | Any `.md` file. Frontmatter is optional. |
| **Profile** | Document *shape* (guide, feature, decision, …). Controls expected sections; not a file type. |
| **Graph edge** | Explicit frontmatter links: `depends` (must-read first) and `related` (optional reference). |
| **Asset** | Non-Markdown files (`resources`) or source bindings (`code`) attached to a document. |
| **Context** | A bounded AI reading list (`ods.context` + `ods context` CLI). |
| **Compliance** | Binary: **compliant** when `ods lint` is clean; otherwise **non-compliant**. |
| **Discovery** | Progressive CLI: `overview` → `find` / `tag` / `ls` / `tree` → `context` (no nested index files). |

---

## Compliance (not levels)

Plain Markdown is fine without ODS. An **ODS workspace** has `ods.toml`. Run `ods lint` for trust. There is no Level 0–3 ladder. Normative wording: [core.md](core.md), [validation.md](validation.md).

---

## How this folder is organized

Read **intro** (this file) first, then pick a path below. Each module owns one idea.

| File | Purpose |
| :--- | :--- |
| [intro.md](intro.md) | Purpose, concepts, reading map (you are here) |
| [keys.md](keys.md) | Every frontmatter key: placement, purpose, examples |
| [core.md](core.md) | Format model, compliance, lifecycle operations |
| [profiles.md](profiles.md) | Standard and custom document shapes |
| [graph.md](graph.md) | IDs, single source of truth, depends/related |
| [context.md](context.md) | Deterministic AI context loading |
| [indexes.md](indexes.md) | `ods.toml` workspace config + CLI discovery |
| [assets.md](assets.md) | Resources (files) and code references |
| [validation.md](validation.md) | Lint rules and tooling contract |
| [scope.md](scope.md) | What ODS deliberately does **not** include |

---

## Start-here paths

| Goal | Read first |
| :--- | :--- |
| Author frontmatter correctly | [keys.md](keys.md) |
| Understand the formal model | [core.md](core.md) |
| Pick a document shape | [profiles.md](profiles.md) |
| Link documents together | [graph.md](graph.md) |
| Give AI a bounded reading list | [context.md](context.md) |
| Set up folder navigation | [indexes.md](indexes.md) |
| Attach PDFs, CSVs, or source files | [assets.md](assets.md) |
| Enforce quality in CI | [validation.md](validation.md) |
| See intentional non-features | [scope.md](scope.md) |

**CLI / tutorials** (how to run tools) live in the product guide under `docs/guide/`, not in this folder. This tree defines **what the format means**.

---

## Multi-spec map

| Spec folder | CLI | Best for |
| :--- | :--- | :--- |
| [ods/](../ods/) (this folder) | bare `ods …` | Engineering docs, graphs, code bindings |
| [okf/](../okf/) | `ods … --okf` | Knowledge bundles: trust, provenance, freshness |
| [skills/](../skills/) | `ods … --skills` | `SKILL.md` agent skill packages |

There is **no** `--ods` flag and **no** `ods okf` / `ods skills` namespaces.

---

## Design principles (summary)

1. **Human first** — readable in any text editor, forever.
2. **Free at level zero** — plain Markdown is already conformant.
3. **Token efficient** — every fact has one canonical location.
4. **Graph native** — relationships are frontmatter, not guessed from prose.
5. **Trust from validation** — only require what tools can check.

---

## What to read next

- New to ODS? → [keys.md](keys.md) (cheat sheet) then [core.md](core.md)
- Tooling day-to-day? → product guide (`docs/guide/01-introduction.md`, quickstart)
- Implementing a tool? → [validation.md](validation.md) and [core.md](core.md) (RFC language)
