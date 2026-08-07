---
description: "Managing cross-repository API contracts and shared governance using ODS Packs."
ods:
  profile: "note"
  status: "stable"
---

# Cross-Repo Contracts Use Case

ODS Packs (`ods pack`) enable teams to share architectural contracts, custom profiles, and AI agent skills across multiple repositories.

---

## Architecture

```text
acme-engineering-pack/
├── ods.toml
├── ods-profiles/
│   └── api-contract.md
└── skills/
    └── contract-validator/
```

Workspaces import the shared pack via Git URL:

```bash
ods pack add git@github.com:acme-org/engineering-pack.git --auto-update daily
```
