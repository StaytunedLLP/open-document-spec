# Specification: Single-Source Custom Profiles, H2/H3 Section Hierarchy, and Profile Metadata

This document specifies the design for **Strict Single-Source Custom Profile Registration**, **Nested H2 & H3 Section Hierarchy Validation**, and the **Frontmatter Metadata Rules** for custom profile definition files in Open Document Spec (ODS).

---

## 1. Strict Single-Source Custom Profile Registration (`custom-profiles:` in Root `index.md`)

Custom profiles MUST be explicitly registered in the root `index.md` frontmatter using the **`custom-profiles:`** array. There is **only one way** to define and activate custom profiles in an ODS workspace—no hidden folder magic or implicit file scanning.

### Root `index.md` Registration Example

```yaml
---
profile: index
ods: 0.1
custom-profiles:
  - .ods/profiles/rfc.md
  - docs/profiles/api_endpoint.md
  - docs/profiles/architecture_record.md
---

# Workspace Root Index
```

### Strict Registration Rules

1. **Explicit File Paths Only**: Every custom profile `.md` file must be explicitly listed under `custom-profiles:` in root `index.md`.
2. **Zero Folder Auto-Discovery**: Dropping a `.md` file inside `.ods/profiles/` will **not** register it automatically. It MUST be listed in `custom-profiles:`.
3. **Deterministic Graph & Linting**: Keeps the workspace document graph completely explicit, deterministic, and easy for AI agents and linters to verify.

---

## 2. Supporting Both H2 (`##`) and H3 (`###`) Section Hierarchies

Profile definition files can declare nested sub-sections using standard Markdown header levels (`##` H2 and `###` H3).

### Example Profile Definition (`docs/profiles/api_endpoint.md`)

```markdown
---
name: api_endpoint
description: "REST/GraphQL API Endpoint Specification"
expected_keys:
  - service
  - endpoint_url
  - auth_level
ods:
  profile: custom-profile
  status: stable
---

# API Endpoint Profile

## Overview
General introduction and purpose of the endpoint.

## Specification
Technical contract details.

### Request Payload | Request Body
Headers, query parameters, and JSON payload schema.

### Response Payload | Response Body
HTTP status codes, success response, and error schema.

## Verification & Testing
Steps to manually or automatically test the endpoint.
```

---

## 3. Profile Naming Rules (`name:` Key & Filename Stem Fallback)

How is a profile referenced in consumer documents?

### The Rule

1. **Explicit `name:` Key**: The `name:` field in the profile definition frontmatter explicitly defines the profile identifier referenced by consumer documents (`ods.profile: <name>`).
2. **Filename Stem Fallback**: If `name:` is omitted in the profile definition file, the profile identifier defaults to the file stem (e.g. `api_endpoint.md` $\rightarrow$ `api_endpoint`).

---

## 4. Key Naming: Strict Single Standard (`expected_keys`)

In custom profile definitions, frontmatter expected keys MUST be declared using **`expected_keys`** (or `expected-keys`).

- **No Ambiguous Fallbacks**: `expected_keys` is the sole canonical key name. No redundant `custom-profile-keys` fallback is permitted.
- **Coverage**: Defines the frontmatter keys (both standard domain keys like `author`, `reviewer` and custom domain keys like `endpoint_url`) that `ods lint` enforces on target documents.

---

## 5. Complete End-to-End Workflow Example

### Step 1: Explicitly Register in Root `index.md`

```yaml
---
profile: index
ods: 0.1
custom-profiles:
  - docs/profiles/api_endpoint.md
---
```

### Step 2: Create Profile Definition (`docs/profiles/api_endpoint.md`)

```markdown
---
name: api_endpoint
description: "API Contract Spec"
expected_keys:
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

## Verification
```

### Step 3: Scaffold & Validate Target Document

Author runs:
```bash
ods new api_endpoint docs/apis/users-v1.md --title "Users API v1"
```

Generates target document (`docs/apis/users-v1.md`):

```markdown
---
endpoint_url: "/api/v1/users"
ods:
  profile: api_endpoint
  status: draft
---

# Users API v1

## Overview

## Specification

### Request Payload

### Response Payload

## Verification
```

Running `ods lint` verifies both H2 sections (`Overview`, `Specification`, `Verification`) and H3 sub-sections (`Request Payload`, `Response Payload`) under `Specification`, as well as the presence of `endpoint_url` declared in `expected_keys`!
