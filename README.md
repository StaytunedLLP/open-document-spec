---
ods:
  profile: note
  status: stable
---

# Open Document Spec (ODS)

[![ci](https://github.com/StaytunedLLP/open-document-spec/actions/workflows/ci.yml/badge.svg)](https://github.com/StaytunedLLP/open-document-spec/actions/workflows/ci.yml)
[![GitHub Marketplace](https://img.shields.io/badge/Marketplace-Open--Document--Spec-blue.svg?logo=github)](https://github.com/marketplace)
[![Compliance](https://img.shields.io/badge/ODS-Level--3-green.svg)](https://github.com/StaytunedLLP/open-document-spec)
[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE)

**Open Document Spec (ODS)** is a lightweight, human-first convention built on plain Markdown. It enriches Markdown repositories with machine-readable YAML frontmatter metadata, explicit graph relationships, and sub-5ms deterministic AI context loading—managed by the native **Open Document Spec CLI (`ods`)**.

Your files remain `.md` files forever.

---

## 🌟 Key Features & Capabilities

ODS turns flat Markdown folders into an intelligent, validated, and auto-healing document graph:

| Feature | Command / Key | Role & Description |
|---|---|---|
| ⚡ **Deterministic Bounded Context Graph** | `ods context <doc-id>` | Bounded AI reading scope (<5ms) following `depends:` and `related:` chains for AI Coding Assistants. |
| 📋 **Custom Profile Schema Engine** | `ods profile` / `custom-profiles:` | Single-source profile schema registration in `index.md`, enforcing `expected_keys` and `H2`/`H3` section hierarchies. |
| 🔄 **Smart Rename & Dependency Healing** | `ods mv <src> <dst>` | Renames files while automatically rewriting graph dependencies, relative body links, and code references. |
| 🪄 **Legacy Markdown Adoption** | `ods adopt <dir>` | Scans legacy non-ODS Markdown folders and automatically drafts frontmatter schemas (`status: draft`). |
| 🏷️ **Tag Taxonomy & Governance** | `ods tag` | Workspace-wide tag cataloging, alias resolution, tag suggestions, and tag renaming. |
| 🌐 **Google OKF v0.2 Interoperability** | `ods lint --okf` | Native validation for Google Open Knowledge Format (OKF) v0.2 knowledge bundles (`okf_version: "0.2"`). |
| 🔒 **Secret & Pack Isolation** | `share: private` / `ods share` | Protects private secrets and exports sanitized public workspace packs (`ods pack`). |
| ⚙️ **Background OS Daemon & Watcher** | `ods setup` / `ods service` | Real-time file system watcher maintaining sub-5ms graph indexes continuously in the background. |
| 🤖 **Zero-Terminal AI Skill** | `skills/ods/SKILL.md` | Skill-first integration for AI Assistants (Antigravity, Claude, Cursor, ChatGPT). |
| 🚀 **GitHub Marketplace Action** | `uses: StaytunedLLP/open-document-spec@v1` | Automated CI linting and GitHub PR inline code annotations. |

---

## 👥 Who is ODS For?

ODS is built for cross-functional collaboration between technical and non-technical teams:

- 👩‍💼 **Product Managers & Business Owners**: Bind feature PRDs and acceptance criteria directly to code implementations and goals.
- ⚖️ **Legal, HR & Compliance**: Enforce mandatory policy section trees and isolate sensitive secrets with `share: private`.
- 🔬 **Researchers & Writers**: Build deterministic, machine-readable citation graphs across research papers and notes.
- 💻 **Developers & DevOps**: Connect technical specs to source code symbols with sub-5ms Rust graph traversal.

> **Zero Terminal Skills Needed**: Non-technical users can use ODS entirely through **AI Assistant Skills** (Antigravity, Claude, ChatGPT, Cursor). Simply tell your AI Assistant: _"Install ODS skill and index my documents folder."_

---

## 🚀 Quickstart in 3 Steps

### 1. Enable ODS (Primary: Skill-First for AI Assistants)

The recommended, zero-friction way to start with ODS is via the **ODS Skill** (`skills/ods/SKILL.md`) in your AI Coding Assistant.

Tell your AI Assistant: _"Install Open Document Spec skill"_. The skill detects your OS, downloads the native **`ods`** binary, registers the OS background service, and verifies workspace health:

```text
==> Open Document Spec is installed and running on your machine!
==> Version: ods v0.0.13
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
ods init .      # ODS: writes ods: 0.1 spec marker
ods setup       # Verify workspace, check updates, and register background OS service
```

### 3. Scaffold, Validate & Track Health

```bash
ods profile init rfc               # Scaffold custom profile schema in .ods/profiles/rfc.md
ods profile list                   # List standard and registered custom profiles
ods new rfc docs/rfcs/001.md       # Scaffold new document from profile template
ods lint                           # Level-3 graph & section integrity check
ods status                         # Profile Coverage Health score summary
ods context <doc-id-or-path>       # Bounded AI reading scope (<5ms)
ods mv <src.md> <dst.md>           # Rename file & auto-rewrite workspace deps
ods adopt docs/                    # Auto-draft frontmatter on legacy Markdown
```

---

## 📘 Custom Profiles Specification

Custom profiles define domain document schemas, expected frontmatter keys, and starter templates.

### Single-Source Registration in Root `index.md`

Custom profiles are registered explicitly in root `index.md` under `custom-profiles:`:

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

### Request Payload

### Response Payload

## Verification & Testing
```

- **`ods.profile: custom-profile`**: Identifies the file explicitly as a Custom Profile Schema Definition.
- **`expected_keys`**: List of frontmatter fields enforced on target documents during `ods lint`.
- **H2 & H3 Hierarchies**: `ods lint` validates parent `## H2` and child `### H3` section trees.

---

## 🛠️ ODS CLI Command Reference

| Command | Usage | Role |
|---|---|---|
| `ods init` | `ods init [path]` | Initialize root `index.md` with `ods: 0.1` spec marker. |
| `ods setup` | `ods setup [path]` | Verify workspace health, check updates, and register background OS service. |
| `ods lint` | `ods lint [path]` | Run Level-3 document graph, frontmatter, and profile section validation. |
| `ods lint --okf` | `ods lint --okf [path]` | Validate Google OKF v0.2 knowledge bundles (`okf_version: "0.2"`). |
| `ods status` | `ods status [path]` | Display workspace health score and profile coverage breakdown. |
| `ods context` | `ods context <doc-id>` | Generate sub-5ms bounded AI context graph for prompt feeding. |
| `ods mv` | `ods mv <src> <dst>` | Move/rename Markdown file and auto-heal graph links and references. |
| `ods adopt` | `ods adopt [path]` | Auto-draft frontmatter on unindexed legacy Markdown files. |
| `ods profile` | `ods profile list/init` | List registered profiles or scaffold new custom profile schemas. |
| `ods tag` | `ods tag catalog/rename` | Query tag catalog, suggest synonyms, or perform global tag renames. |
| `ods share` | `ods share [path]` | Export public documentation pack while stripping `share: private` files. |
| `ods update` | `ods update` | Binary self-update to latest released version from GitHub Releases. |

---

## 🤖 GitHub Marketplace Action Integration

Automate document graph verification in your GitHub CI/CD pipeline:

```yaml
name: Documentation Graph CI

on:
  push:
    branches: [ main ]
  pull_request:

jobs:
  validate-docs:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v7
      - name: Verify Open Document Spec Graph
        uses: StaytunedLLP/open-document-spec@v1
        with:
          version: 'latest'
          command: 'lint'
```

---

## 📄 License

Apache License 2.0. See [LICENSE](LICENSE).
