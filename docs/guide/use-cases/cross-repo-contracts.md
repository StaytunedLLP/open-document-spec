---
title: "Cross-Repo Contracts Use Case"
description: "Managing cross-repository API contracts and shared governance using ODS Packs."
status: "stable"
order: 14
ods:
  profile: "note"
  status: "stable"
---

# Cross-Repo Contracts Use Case

ODS Packs (`odc pack`) enable teams to share architectural contracts, custom profiles, and AI agent skills across multiple repositories.

---

## Architecture

```text
acme-engineering-pack/
├── index.md
├── ods-profiles/
│   └── api-contract.md
└── skills/
    └── contract-validator/
```

Workspaces import the shared pack via Git URL:

```bash
odc pack add git@github.com:acme-org/engineering-pack.git --auto-update daily
```
