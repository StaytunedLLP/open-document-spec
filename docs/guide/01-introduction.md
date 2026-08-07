---
description: "Principles, binary compliance, learning path, and core benefits of Open Document Spec."
status: "stable"
order: 1
ods:
  profile: "note"
  status: "stable"
---

# Introduction to Open Document Spec (ODS)

Open Document Spec (ODS) is a Markdown-first documentation specification for repositories that need human-readable files and deterministic navigation for people and AI agents.

Documents stay plain `.md` files. Metadata is optional YAML frontmatter. Nothing requires a new file extension or proprietary store.

## 🧭 The 5W1H Framework

ODS is built on six foundational dimensions:

- ❓ **WHAT**: A human-first Markdown specification and native Rust CLI (`ods`) that turns flat `.md` files into a validated document graph.
- 💡 **WHY**: Reduces documentation drift, bounds AI context with `ods context`, and keeps relationships explicit.
- 👥 **WHO**: Developers, PMs, writers, compliance leads, and coding agents (Cursor, Claude, ChatGPT, …).
- 📍 **WHERE**: Local terminals, editors (`ods lsp`), CI, and optional background service (`ods serve`).
- ⏰ **WHEN**: From day-one setup (`ods init`) through refactors (`ods mv`) and PR gates (`ods lint`).
- 🛠️ **HOW**: CLI-first discovery and validation — no nested index lockfiles.

---

## Novice-to-expert roadmap

| Tier | Audience | Focus | Primary commands |
| --- | --- | --- | --- |
| **1 · Setup** | New workspace | Install, `ods.toml`, frontmatter, lint | `ods init` · `ods setup` · `ods lint` |
| **2 · Graph** | Day-to-day docs | `depends` / `related`, code links, moves | `ods overview` · `ods find` · `ods mv` · `ods new` |
| **3 · Profiles** | Custom shapes | Profiles, packs, context | `ods profile` · `ods context` · `ods pack` |
| **4 · Service** | Teams / CI | Serve, hooks, SARIF | `ods serve` · `ods lint --format sarif` |

---

## Core design principles

1. **Human first** — Readable in any text editor.
2. **Plain Markdown is valid** — Adoption is enrichment, never forced migration.
3. **Token efficient** — Every fact has one canonical location; agents use progressive CLI discovery.
4. **Graph native** — Relationships are explicit frontmatter, not guessed from prose.
5. **Trust from validation** — Binary **compliant | non-compliant** via `ods lint`.

---

## Compliance (binary)

| State | Meaning |
| --- | --- |
| **Plain Markdown** | Files open anywhere; no workspace marker required |
| **ODS workspace** | Root **`ods.toml`** with `spec` (e.g. `"0.1"`) |
| **Compliant** | `ods lint` reports zero errors |
| **Non-compliant** | Fix diagnostics, then re-lint |

There is **no** Level 0–3 ladder. Normative rules: [`specs/ods/intro.md`](../../specs/ods/intro.md), [`specs/ods/core.md`](../../specs/ods/core.md), [`specs/ods/validation.md`](../../specs/ods/validation.md).

---

## Learning path

| Step | Doc | You will |
| --- | --- | --- |
| 1 | This introduction | Principles and compliance |
| 2 | [Quickstart](02-quickstart.md) | Init, lint, context |
| 3 | [Adoption](03-adoption.md) | Enrich an existing repo |
| 4 | [Tooling](04-tooling.md) | CLI and CI |
| 5 | [Profiles](05-profiles.md) | Document shapes |
| 6 | [Advanced](06-advanced.md) | Agents and large workspaces |
| 7 | [FAQ](faq.md) | Troubleshooting |

**Discovery:** use `ods overview` → `ods find` / `ods tag` / `ods tree` → `ods context <id>`. Do **not** commit nested `index.ods.md` files.

---

## What you get

- **Clear shapes** — Standard profiles (guide, feature, decision, …).
- **CLI navigation** — Progressive discovery without index lockfiles.
- **Explicit edges** — `depends` and `related` for agents and tools.
- **Bounded AI context** — `ods context` with optional `--max-tokens`.
- **Share-aware publish** — `ods share` filters by `share:`.
- **Safer refactors** — `ods mv` rewrites graph refs.
- **Low-memory service** — `ods serve` targets **≤ 10 MB** RSS by default (`service.max_rss_mb`).
