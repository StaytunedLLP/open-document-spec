---
description: "What Google OKF v0.2 is, when to use it with the ods CLI, and how it differs from ODS."
ods:
  profile: "note"
  status: "stable"
  related:
    - keys.md
---

# OKF · Intro

**Open Knowledge Format (OKF) v0.2** is a Markdown + YAML convention for **knowledge concepts**: metrics, tables, playbooks, policies, and attested computations—with first-class **provenance**, **trust**, **freshness**, and **lifecycle**.

The `ods` CLI supports OKF **natively** as an extra dialect:

```bash
ods init --okf ./bundle
ods lint --okf
ods context --okf <concept>
```

There is no `ods okf` namespace. ODS remains the default product when no flag is passed.

Upstream-oriented full write-up (long form): `docs/other-specs/okf-0.2.md`.  
Key dictionary for this dialect: [keys.md](keys.md).  
Compare with ODS: `docs/other-specs/frontmatter-keys-ods-vs-okf.md`.

---

## When to use OKF vs ODS

| Use **OKF** when… | Use **ODS** when… |
| :--- | :--- |
| Docs describe **data/knowledge assets** (tables, metrics, catalogs) | Docs describe **software systems** (services, guides, RFCs, APIs) |
| You need **trust / provenance / staleness** signals | You need **typed graph edges** and **code bindings** |
| Bundle root carries `okf_version: "0.2"` | Workspace root carries `ods: 0.1` |
| Relationships are mainly prose links | Relationships are `depends` / `related` arrays |

Hybrid trees may declare both markers; bare `ods lint` stays ODS-first unless `specs.okf.enabled: true` on the root index.

---

## Core ideas

| Term | Meaning |
| :--- | :--- |
| **Bundle** | Self-contained tree of knowledge documents (unit of distribution). |
| **Concept** | One Markdown document describing an asset or idea. |
| **Concept ID** | Path without `.md` (same path-id idea as ODS). |
| **Root marker** | Root `index.md` / `index.ods.md` with `okf_version: "0.2"`. |

OKF standardizes a small frontmatter surface so agent-maintained corpora stay **trustable** without a central schema registry.

---

## What to read next

- [keys.md](keys.md) — OKF key dictionary with examples
- `docs/other-specs/okf-0.2.md` — full OKF v0.2 prose (upstream-aligned)
- [../ods/intro.md](../ods/intro.md) — ODS intro
