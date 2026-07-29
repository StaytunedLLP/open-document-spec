---
ods:
  profile: note
  status: draft
---

# Open  Document Spec (ODS)

[![ci](https://github.com/StaytunedLLP/open-document-spec/actions/workflows/ci.yml/badge.svg)](https://github.com/StaytunedLLP/open-document-spec/actions/workflows/ci.yml)

Open Document Spec (ODS) is a lightweight, human-first convention built on plain Markdown. It enriches Markdown repositories with machine-readable YAML frontmatter metadata, explicit graph relationships, and deterministic AI context loading—all managed via a single native CLI binary (**`ods`**).

Your files remain `.md` files forever. `ods` validates links, updates maps, and runs a machine background service so path renames and indexes stay consistent automatically.

---

## Who is ODS For? (Built for Tech & Non-Tech Alike)

ODS is **not just for software engineers**. Anyone who creates, manages, or reads Markdown documentation can use ODS:

- 👩‍💼 **Product Managers & Business Owners**: Link feature PRDs and acceptance criteria to business goals.
- ⚖️ **Legal, HR & Compliance**: Enforce mandatory policy sections and isolate sensitive secrets with `share: private`.
- 🔬 **Researchers & Writers**: Build deterministic citation graphs across research papers and notes.
- 💻 **Developers & DevOps**: Bind technical specs to source code symbols with sub-5ms Rust graph traversal.

> **Zero Terminal Skills Needed**: Non-technical users can use ODS entirely through **AI Assistant Skills** (Claude, Antigravity, ChatGPT, Cursor). Simply tell your AI Assistant: _"Install ODS skill and index my documents folder."_

## Quickstart in 3 Steps

### 1. Enable ODS (Primary: Skill-First for AI Assistants)

The recommended, zero-friction way to start with ODS is via the **ODS Skill** in your AI Coding Assistant (Claude Code, Antigravity, Cursor, Codex, etc.).

Add the `ods` skill to your AI environment. The skill automatically detects your operating system (macOS, Linux, Windows), downloads the native `ods` binary, registers the OS background service, and validates health:

```text
==> ODS is installed and running now in your machine!
==> Version: ods v0.1.x
```

<details>
<summary><b>Optional: Manual Direct CLI Installation (Without AI Assistant)</b></summary>

**macOS / Linux**:

```bash
export GH_TOKEN="$(gh auth token)"
curl -fsSL -H "Authorization: Bearer ${GH_TOKEN}" \
  https://raw.githubusercontent.com/StaytunedLLP/open-document-spec/main/src/scripts/install.sh | bash
```

**Windows (PowerShell)**:

```powershell
$env:GH_TOKEN = (gh auth token)
irm -Headers @{ Authorization = "Bearer $env:GH_TOKEN" } `
  https://raw.githubusercontent.com/StaytunedLLP/open-document-spec/main/src/scripts/install.ps1 | iex
```

</details>

### 2. Initialize & Start Service

```bash
mkdir my-docs && cd my-docs
ods init .      # Initialize ODS workspace boundary (creates root index.md)
ods setup       # Verify workspace, check updates, and register background OS service
```

### 3. Validate, Load AI Context & Audit Token Savings

```bash
ods lint                      # Level-3 graph and reference integrity check
ods context <doc-id-or-path>  # Generate bounded AI reading list (<5ms)
ods mv <src.md> <dst.md>      # Rename file & auto-rewrite all workspace dependencies
ods adopt docs/               # Auto-draft frontmatter headers across legacy repos
ods bench stats               # Calculate token & API cost ROI report (~94% savings)
```

---

## Standard ODS Profile Types

ODS standardizes document structures via `profile` definitions enforced by `ods lint`:

| Profile                | Primary Purpose                       | Enforced H2 Sections                          |
| ---------------------- | ------------------------------------- | --------------------------------------------- |
| **`profile: spec`**    | Technical & API Specifications        | `## Summary`, `## Requirements`, `## Schema`  |
| **`profile: guide`**   | Operational Runbooks & Onboarding     | `## Overview`, `## Prerequisites`, `## Steps` |
| **`profile: feature`** | Product PRDs & Feature Requirements   | `## User Goal`, `## Acceptance Criteria`      |
| **`profile: note`**    | Architectural Decision Records (ADRs) | `## Context`, `## Decision`, `## Impact`      |

---

## Core Value Proposition

| Need                  | Without ODS                                 | With ODS                                                                                      |
| --------------------- | ------------------------------------------- | --------------------------------------------------------------------------------------------- |
| **Link Integrity**    | Broken links rot when files move            | `ods lint` catches dangling refs; `ods mv` auto-rewrites paths on move                        |
| **Navigation**        | Hand-crafted lists rot over time            | `ods index` auto-generates child lists in `index.md`                                          |
| **AI Context**        | Context bloat or missing files              | `ods context` generates deterministic, bounded loading lists (~94% token savings)             |
| **Token ROI & Cost**  | Large context windows waste AI tokens       | `ods bench stats / strip / restore` benchmarks token savings                                  |
| **Code Mapping**      | Loose descriptions disconnected from source | `code:` frontmatter maps docs to source files, symbols (`init_pool()`), and roles             |
| **Schema Validation** | Inconsistent document layouts               | Standard `profile` definitions (`spec`, `guide`, `feature`, `note`) enforce section structure |
| **Vibecoding Sync**   | Context loss when switching AI tools        | **Pro Viber ($10/mo)** provides seamless cloud context sync for vibecoding & vibedesigning    |

---

## Documentation Directory

For complete documentation, tutorials, specification rules, and guides, refer to the following resources:

| Guide                                                                                         | Description                                                                                                    |
| --------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------- |
| **[01-introduction.md](docs/guide/01-introduction.md)**                                       | Core design principles, Level 0–3 adoption ladder, and value overview.                                         |
| **[02-quickstart.md](docs/guide/02-quickstart.md)**                                           | End-to-end hands-on walkthrough for new and existing projects.                                                 |
| **[03-adoption.md](docs/guide/03-adoption.md)**                                               | Step-by-step guide to adopting ODS on an existing Markdown repository.                                         |
| **[04-tooling.md](docs/guide/04-tooling.md)**                                                 | Comprehensive CLI command reference, OS background service setup, CI integration, and updates.                 |
| **[05-profiles.md](docs/guide/05-profiles.md)**                                               | Standard document profiles (`guide`, `feature`, `decision`, etc.) and custom ODS Packs.                        |
| **[06-advanced.md](docs/guide/06-advanced.md)**                                               | AI context engineering, multi-workspace tracking, and low-memory deployment modes.                             |
| **[07-troubleshooting-and-diagnostics.md](docs/guide/07-troubleshooting-and-diagnostics.md)** | Diagnostics, `ods-error.md` catalog, Git merge conflict handling, and daemon troubleshooting.                  |
| **[08-enterprise-deployment.md](docs/guide/08-enterprise-deployment.md)**                     | Enterprise ODS Pack distribution, multi-repo governance, CI/CD pipelines, security controls, and ROI modeling. |
| **[Enterprise Use Cases](docs/guide/use-cases/index.md)**                                     | Real-world enterprise patterns: Secrets isolation, cross-repo contracts, and compliance audits.                |
| **[ROI & Cost Savings](docs/guide/roi-calculator.md)**                                        | Token reduction economics and productivity savings models scaling to 50k+ developers.                          |
| **[Benchmarks](ods-test/benchmarks/README.md)**                                               | Evaluation suite and benchmark script comparing flat vs ODS context token usage.                               |
| **[Open-Source Strategy](docs/strategy/open-source-strategy.md)**                             | Topography, repository organization, and governance strategy.                                                  |
| **[Enterprise Pricing & Services](docs/strategy/enterprise-services-and-pricing.md)**         | Enterprise offerings, commercial service catalog, and SaaS pricing model.                                      |
| **[Repository Visibility Guide](docs/maintainer/repository-visibility-guide.md)**             | Maintainer guide on public open-source vs private commercial repository boundaries.                            |
| **[FEATURE_MATRIX.md](docs/FEATURE_MATRIX.md)**                                               | Complete matrix of frontmatter keys, specs, and auto-update behaviors.                                         |
| **[specs/SPEC.md](specs/SPEC.md)**                                                            | Normative Open Document Specification (the formal spec rules).                                                 |
| **[CONTRIBUTING.md](CONTRIBUTING.md#versioning--release-workflow-maintainers)**               | Developer guide, build instructions, and maintainer release workflows.                                         |

---

## Maintainers

Release cuts, version bumping, and CI publishing automation are documented in [CONTRIBUTING.md](CONTRIBUTING.md#versioning--release-workflow-maintainers).
