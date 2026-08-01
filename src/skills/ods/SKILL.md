---
name: ods
description: >-
  Install, run, update, and author with Open Document Spec (`ods`) for ODS workspaces.
  Use `ods <cmd>` for all document workflows (ods lint, ods new, ods status, ods adopt, ods profile).
  Supports OKF bundles via `ods lint --okf` flag.
---

# ODS — Open Document Spec

## 0. Glossary

| Term | Meaning |
|---|---|
| **`ods`** | The primary CLI binary and tool standard |
| **`ods:` frontmatter** | ODS format root/nested engine keys (`ods: { profile: rfc, status: draft }`) |
| **`custom-profiles:`** | Root `index.md` array declaring custom profile schema definitions |

ODS is plain Markdown with **optional** YAML frontmatter, powered by a high-performance Rust engine binary named **`ods`**. A **workspace** is any directory tree whose **root `index.md` carries an `ods:` key** (e.g. `ods: 0.1`).

---

## 1. Setup & Service Management

The primary, zero-friction method to run ODS is via the `ods` CLI binary:

- `scripts/bootstrap.sh` (macOS / Linux) or `scripts/bootstrap.ps1` (Windows) — installs latest `ods` release binary.
- `ods setup [path]` — check release freshness, detect workspace boundary, and ensure background service (`ods serve`).
- `ods update` — self-update the `ods` CLI binary.

---

## 2. Core Operational Specification

### Frontmatter Layout & Key Placement Rules

1. **Custom Domain Keys (`author`, `reviewer`, `target_release`, `service`, `team`, `title`, `tags`)**:
   - Placed at the **top-level** of frontmatter (outside `ods:`).
   - Keeps custom metadata clean, human-readable, and compatible with Static Site Generators (Docusaurus, Astro, Hugo, Next.js).
2. **ODS Engine Keys (`profile`, `status`, `id`, `share`, `depends`, `related`, `code`, `resources`)**:
   - Nested inside the **`ods:` map** (`ods.profile: rfc`, `ods.status: draft`).

```yaml
---
title: "Distributed Redis Caching Strategy"
author: "Alice Smith"
reviewer: "Bob Jones"
target_release: "v2.4"
ods:
  profile: rfc
  status: draft
---
```

---

## 3. Custom Profiles Specification

Custom profiles define domain schemas, required frontmatter keys, and starter templates.

### Single-Source Registration (`custom-profiles:` in Root `index.md`)

Custom profiles MUST be explicitly registered in root `index.md` under `custom-profiles:`:

```yaml
---
profile: index
ods: 0.1
custom-profiles:
  - .ods/profiles/rfc.md
  - docs/profiles/api_endpoint.md
---
```

### Custom Profile Schema Definition File (`docs/profiles/api_endpoint.md`)

```markdown
---
name: api_endpoint
description: "REST/GraphQL API Endpoint Specification"
expected_keys:
  - service
  - endpoint_url
ods:
  profile: custom-profile
  status: stable
---

# API Endpoint Profile

## Overview

## Specification

### Request Payload | Request Body

### Response Payload | Response Body

## Verification & Testing
```

- **`ods.profile: custom-profile`**: Identifies the file as a Custom Profile Schema Definition.
- **`expected_keys`**: List of frontmatter keys enforced on target documents by `ods lint`.
- **H2 & H3 Hierarchies**: `ods lint` validates parent `## H2` and child `### H3` section trees.

---

## 4. CLI Workflow Matrix (`ods`)

| Command | Action |
|---|---|
| **`ods lint`** | Validates structural integrity, graph links, and profile H2/H3 section trees (`--format text\|json\|sarif`). |
| **`ods stats`** | Displays workspace document telemetry, health score, graph density, and profile breakdown (`--format text\|json`). |
| **`ods completion <shell>`** | Generates shell autocompletion scripts for `bash`, `zsh`, `fish`, `powershell`. |
| **`ods schema`** | Exports JSON Schema (`ods.schema.json`) for IDE frontmatter autocomplete and validation (`--write`, `--out`). |
| **`ods tree`** | Displays visual ASCII/Unicode hierarchy tree of index navigation and dependency graphs (`--format text\|json`). |
| **`ods diff [target]`** | Compares document graph dependencies and frontmatter changes against git commits or branches. |
| **`ods clean`** | Cleans diagnostic reports (`.ods/ods-errors.md`), coverage files (`.ods/coverage.md`), and cache files. |
| **`ods setup --git-hooks`** | Installs `.git/hooks/pre-commit` hook for automated commit linting. |
| **`ods lint --okf`** | Validates OKF document bundles using the `--okf` flag. |
| **`ods new <path>`** | Scaffolds a new document using profile template and frontmatter. |
| **`ods profile list`** | Lists all standard profiles + registered custom profiles. |
| **`ods profile init <name>`** | Scaffolds a new custom profile schema in `.ods/profiles/<name>.md`. |
| **`ods status` / `ods coverage`** | Displays workspace Profile Coverage Health score (% profiled docs vs notes). |
| **`ods adopt`** | Adds minimal frontmatter (`ods: { profile: note, status: draft }`) to plain `.md` files. |
| **`ods init`** | Initializes root `index.md` (`ods: 0.1`). |
| **`ods pack`** | Manages reusable ODS Packs. |
