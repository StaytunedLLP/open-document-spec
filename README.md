---
ods:
  profile: note
  status: draft
---

# Open Document Spec (ODS)

[![ci](https://github.com/StaytunedLLP/open-document-spec/actions/workflows/ci.yml/badge.svg)](https://github.com/StaytunedLLP/open-document-spec/actions/workflows/ci.yml)

Open Document Spec (ODS) is a lightweight, human-first convention built on plain Markdown. It enriches Markdown repositories with machine-readable YAML frontmatter metadata, explicit graph relationships, and deterministic AI context loading—managed by the **Open Document Spec CLI (`ods`)**.

Your files remain `.md` files forever. Root markers:

| Key | Role |
|---|---|
| `ods:` | ODS **spec** version / workspace boundary |
| `custom-profiles:` | Single-source array registering workspace custom profile schema definitions |
| `okf_version: "0.2"` | Google **OKF** bundle (enabled via `ods lint --okf`) |

```bash
ods <command>         # ODS native commands: ods lint, ods new, ods status, ods adopt, ods profile
ods lint --okf        # validate Google OKF v0.2 knowledge bundles using --okf flag
ods update            # binary self-update
```

---

## Who is ODS For? (Built for Tech & Non-Tech Alike)

ODS is **not just for software engineers**. Anyone who creates, manages, or reads Markdown documentation can use ODS:

- 👩‍💼 **Product Managers & Business Owners**: Link feature PRDs and acceptance criteria to business goals.
- ⚖️ **Legal, HR & Compliance**: Enforce mandatory policy sections and isolate sensitive secrets with `share: private`.
- 🔬 **Researchers & Writers**: Build deterministic citation graphs across research papers and notes.
- 💻 **Developers & DevOps**: Bind technical specs to source code symbols with sub-5ms Rust graph traversal.

> **Zero Terminal Skills Needed**: Non-technical users can use ODS entirely through **AI Assistant Skills** (Claude, Antigravity, ChatGPT, Cursor). Simply tell your AI Assistant: _"Install ODS skill and index my documents folder."_

---

## Quickstart in 3 Steps

### 1. Enable ODS (Primary: Skill-First for AI Assistants)

The recommended, zero-friction way to start with ODS is via the **ODS Skill** (`skills/ods/SKILL.md`) in your AI Coding Assistant.

Add the Open Document Spec skill to your AI environment. The skill detects your OS, downloads the native **`ods`** binary, registers the OS background service, and validates health:

```text
==> Open Document Spec is installed and running on your machine!
==> Version: ods v0.0.12
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
ods init .      # ODS: writes ods: 0.1
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

## Custom Profiles Specification

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

## License

Apache License 2.0. See [LICENSE](LICENSE).
