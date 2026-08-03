---
description: "Standard profiles, custom profiles, precedence rules, profile mapping matrix, and ODS Packs."
status: "stable"
order: 5
ods:
  profile: "note"
  status: "stable"
---

# Profiles, Catalogs, and ODS Packs

Profiles answer **what kind of document** a file is. They define expected H2 section headings checked during validation.

ODS uses a two-tiered **Standard (Built-in) vs. Workspace (Custom)** layering model for profiles and tags:

| Layer | Source | Enforcement |
| :--- | :--- | :--- |
| **Standard Profiles** | Built into the specification / `ods` binary (12 core profiles) | Always available; section lint for known shapes |
| **Custom Profiles** | Explicitly declared under `custom-profiles:` in root `index.ods.md`, or imported **ODS Packs** (`packs:`) | Additive catalogs; unknown name → warning (falls back to **Default Profile (`note`)**) |

Prefer **Standard Profiles** first (`feature`, `guide`, `api`, `architecture`, `decision`, `sop`, `policy`, `meeting`, `faq`, `checklist`, `index`, `note`). Introduce a **Custom Profile** or **ODS Pack** only when a repeated document class is worth standardizing across teams.

---

## Resolution Precedence

When resolving profile definitions (Zero Folder Auto-Discovery):

```text
1. Standard Profiles (built-in 12 core profiles)           // always loaded first
2. Explicit custom profile catalog paths listed in custom-profiles: // workspace-local
3. Custom profiles in imported ODS Packs listed in packs:   // vendor / linked packs
```

- First definition of a name wins.
- Later duplicate definitions trigger a conflict warning (Level 3 CI should fail).
- Custom profiles are **additive**; they cannot overwrite or replace a Standard Profile name.
- Custom profiles are recognized **strictly** when explicitly declared in `custom-profiles:` or imported `packs:`. Unlisted directories are not auto-discovered.

---

## ODS Packs (`ods pack`)

A **Profile** defines a single document structural schema (`profile: decision`). A **Pack** is a reusable ODS workspace bundling **Custom Profiles**, **AI Agent Skills**, **SOPs**, **Templates**, and **Governance Rules** across repositories and machines.

Workspaces declare custom profiles and imported ODS Packs in their root `index.md` / `index.ods.md`:

```yaml
---
profile: index
ods: 0.1
ignore:
  - src
custom-profiles:
  - .ods/profiles/rfc.md
packs:
  - vendor/engineering-pack
  - ../shared-company-pack
---
```

### Multi-Transport `ods pack add`

`ods pack add <source> [--auto-update frequency]` supports Git-native transport across five sources:

1. **GitHub Shorthand**: `ods pack add owner/repo` (Git clone into `vendor/repo`).
2. **HTTPS Git URL**: `ods pack add https://github.com/acme/pack.git` (Remote HTTPS clone).
3. **SSH Git URL**: `ods pack add git@github.com:acme/pack.git` (Remote SSH clone).
4. **Local Path / Link**: `ods pack add ../shared-company-pack` (Monorepo & local pack links/symlinks).
5. **File URL**: `ods pack add file:///opt/packs/security` (Air-gapped enterprise CI volume mounts).

---

## Profile Mapping Matrix ("Which Profile Should I Use?")

| Document Intent | Recommended Standard Profile | Custom Profile / Pack Option |
| :--- | :--- | :--- |
| **Product Requirements / PRD** | `feature` | — |
| **System Design / Technical Architecture** | `architecture` | — |
| **Interface / API Specification** | `api` | — |
| **Architecture Decision / Choice / ADR** | `decision` | — |
| **Standard Operating Procedure / Runbook** | `sop` | — |
| **Rules / Policy / Governance** | `policy` | — |
| **Tutorial / How-To Guide** | `guide` | — |
| **Audit / Release Gates** | `checklist` | — |
| **Meeting Notes / Discussion Summary** | `meeting` | — |
| **Frequently Asked Questions** | `faq` | — |
| **Directory Navigation** | `index` | — |
| **General Prose / Formal Spec / Note** | `note` | — |


### Profile resolution order

1. Built-in standard profiles
2. Explicit custom profiles listed in root `custom-profiles:`
3. Custom profiles in imported `packs:`
