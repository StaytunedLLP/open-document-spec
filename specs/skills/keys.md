---
description: "SKILL.md frontmatter keys validated by ods lint --skills."
ods:
  profile: "note"
  status: "stable"
  depends:
    - intro.md
---

# Skills · Keys

Frontmatter keys for Agent Skills packages as enforced by `ods lint --skills`.

---

## Required

| Field | Constraints | Purpose |
| :--- | :--- | :--- |
| `name` | 1–64 chars; `[a-z0-9-]`; no leading/trailing/consecutive `-`; **equals parent directory name** | Skill identity |
| `description` | 1–1024 chars, non-empty | What the skill does and when to use it |

```yaml
---
name: pdf-processing
description: Extract PDF text, fill forms, merge files. Use when handling PDFs.
---
```

---

## Optional

| Field | Constraints | Purpose |
| :--- | :--- | :--- |
| `license` | string | License name or path to license file |
| `compatibility` | max 500 chars | Environment requirements (product, OS packages, network) |
| `metadata` | string→string map | Extra structured labels (author, version, …) |
| `allowed-tools` | space-separated string | Experimental pre-approved tools list |

```yaml
---
name: pdf-processing
description: Extract PDF text, fill forms, merge files. Use when handling PDFs.
license: Apache-2.0
compatibility: Requires pdftotext on PATH.
metadata:
  author: example-org
  version: "1.0"
allowed-tools: bash read_file
---
```

---

## Rules

- Unknown keys are **preserved** (not rejected).
- Body length > 500 lines → progressive-disclosure **warning** (move detail to `references/`).
- `name` mismatch with directory name → **error**.

## CLI

```bash
ods lint --skills ./pdf-processing
ods init --skills ./new-skill
```

Root workspace auto-enable (ODS root index):

```yaml
specs:
  skills:
    enabled: true
    lint:
      check_keys: true
```
