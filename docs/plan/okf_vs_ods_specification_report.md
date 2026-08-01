---
profile: decision
status: stable
share: public
description: "Comprehensive Comparison Report: OKF v0.2 vs. ODS — Unique Features, Architectural Differences, and Unified ODC CLI Routing."
owner: team:opendocify
tags: [ods, okf, spec, comparison, architecture, odc, report]
---

# Specification Comparison Report: OKF v0.2 vs. ODS (Open Document Spec)

## Executive Summary
This document provides a detailed comparative analysis of **OKF v0.2** (`docs/specs/okf-0.2.md`) and **ODS (Open Document Spec WD 1)** (`specs/`). It highlights what is unique to each specification, why each exists, how they differ structurally, and how the unified **`odc` CLI** routes and supports both formats natively.

---

## 1. High-Level Comparison & Purpose

| Dimension | ODS (Open Document Spec) | OKF v0.2 (Open Knowledge Format) |
| :--- | :--- | :--- |
| **Primary Domain** | **Software Systems & Codebase Engineering** | **Business Knowledge, Data Assets & Metrics** |
| **Target Audience** | Software engineers, architects, coding agents | Data engineers, analysts, on-call responders, AI analytical agents |
| **Core Problem Solved** | Documenting system architecture, APIs, RFCs, and code bindings into a queryable graph | Establishing provenance, trust, freshness, and execution attestation for data & metrics |
| **Root Marker** | `ods: 0.1` and `odc: ">=x.y.z"` in root `index.md` | `okf_version: "0.2"` in root `index.md` |
| **CLI Tooling** | `odc` / `odc ods` | `odc` / `odc okf` |

---

## 2. Core Differences: ODS vs. OKF v0.2

```mermaid
flowchart TD
    subgraph Shared Base
        MD[Markdown Body + YAML Frontmatter]
    end

    subgraph ODS (Systems & Code Graph)
        ODSMap[Nested ods: Map]
        Profiles[Structural Profiles & H2 Headings\nguide, feature, api, sop, decision]
        Graph[Explicit Graph Edges\ndepends, related]
        Code[Code Path & Symbol Bindings\ncode: path, role, symbol]
        Context[Bounded Prompt Assembly\nodc context max-depth, load]
    end

    subgraph OKF v0.2 (Knowledge & Trust Catalog)
        TopLevel[Top-Level Frontmatter Keys\ntype, title, description, resource]
        TypeReq[Required type: Field\nMetric, BigQuery Table, Playbook]
        Trust[Trust & Actor Tracking\ngenerated, verified, human: id]
        Prov[Provenance & Footnotes\nsources, usage_count, last_modified]
        Attest[Attested Computations\nruntime, parameters, executor, attester]
    end

    MD --> ODSMap
    MD --> TopLevel
    ODSMap --> Profiles
    ODSMap --> Graph
    ODSMap --> Code
    ODSMap --> Context

    TopLevel --> TypeReq
    TopLevel --> Trust
    TopLevel --> Prov
    TopLevel --> Attest
```

---

## 3. What is Unique to ODS?

### 1. Nested `ods:` Map (Metadata Segregation)
Engine metadata is neatly grouped under an `ods:` key (`profile`, `status`, `id`, `share`, `depends`, `related`, `code`, `resources`, `context`). This keeps top-level frontmatter clean and prevents collision with Static Site Generators (Docusaurus, Astro, Hugo, Rspress).

### 2. Explicit Graph Edges (`depends` and `related`)
Unlike OKF (which relies on prose markdown links), ODS defines **directional, machine-queryable graph edges** in frontmatter:
- `depends`: Hard prerequisite documents required before reading.
- `related`: Soft cross-reference documents.

### 3. Native Code Bindings (`code:`)
ODS directly maps documentation nodes to source code files, functions, and classes using standard roles: `entrypoint`, `implementation`, `test`, `schema`, `migration`, `config`, `infrastructure`, `pipeline`. Line numbers are prohibited to ensure metadata stability across commits.

### 4. Structural Profile H2 Section Validation
Profiles (`feature`, `guide`, `api`, `architecture`, `decision`, `sop`, `policy`, `meeting`, `faq`, `checklist`) enforce expected `## H2` headings and section aliases.

### 5. Deterministic Context Assembly (`odc context <id>`)
ODS resolves bounded reading lists for AI agents by expanding `depends` edges up to `max-depth` and extracting specified `code` and `resources`.

---

## 4. What is Unique to OKF v0.2?

### 1. Required `type:` Field
Every OKF document MUST specify a `type:` (e.g. `type: BigQuery Table`, `type: Metric`, `type: Playbook`, `type: Attested Computation`). `type` is the only required key across OKF.

### 2. Verifiable Trust Tiers (`generated` and `verified`)
OKF records content creation (`generated: {by, at}`) and verification events (`verified: [{by, at}]`). It tracks actor prefixes (`human:<id>`, `reference_agent/gemini-2.5-pro`, `process:<id>`), allowing consumers to derive trust tiers:
$$\text{Unverified} \longrightarrow \text{Machine-Confirmed} \longrightarrow \text{Human-Reviewed}$$

### 3. Provenance & Credibility Signals (`sources`)
`sources` records input materials, credibility signals (`author`, `usage_count`, `last_modified`), and `usage_window`. Footnote references (`[^id]`) join body claims directly to specific `sources[].id` entries.

### 4. Absolute Staleness Threshold (`stale_after`)
Specifies an absolute expiration date (`YYYY-MM-DD`). A concept is automatically considered stale when `today >= stale_after`.

### 5. Attested Computations (`type: Attested Computation`)
Binds mathematical definitions or metrics to executable runtimes (`bigquery`, `python`, `dbt`), declared `parameters`, execution receipts (`executor`), and deterministic verification scripts (`attester`).

### 6. Standardized Update History (`log.md`)
Defines `log.md` as a reserved filename for date-grouped ISO 8601 update changelogs (`## YYYY-MM-DD`).

---

## 5. Summary Table: Feature Comparison

| Feature / Capability | ODS (Open Document Spec) | OKF v0.2 (Open Knowledge Format) |
| :--- | :--- | :--- |
| **Frontmatter Structure** | Nested `ods:` map | Top-level YAML fields |
| **Document Classification** | `profile` (12 standard schemas) | `type` (freeform, required string) |
| **Graph Edges** | Explicit `depends` & `related` arrays | Markdown body links & footnotes |
| **Code Binding** | `code:` list with fixed roles & symbols | `resource` URI or Attested Computation |
| **Trust & Provenance** | Basic lifecycle `status` | `generated`, `verified`, `sources`, `stale_after` |
| **Attested Computation** | Not included | Native (`runtime`, `executor`, `attester`) |
| **Reserved Files** | Standard `.md` documents | `index.md` (directory) & `log.md` (changelog) |
| **CLI Validation** | `odc ods lint` / `odc lint` | `odc okf lint` / `odc lint` |
| **Audit & Coverage** | `odc audit` / `odc coverage` | `odc okf audit` / `odc coverage` |

---

## 6. How the Unified `odc` Binary Handles Both

The single `odc` binary natively supports both specifications through a **Unified Spec Key Descriptor Engine**:
1. **Auto-Detection**: `odc` inspects the root `index.md` for `ods:` or `okf_version:`.
2. **Command Routing**:
   - `odc lint` automatically applies ODS or OKF rules based on detected root markers.
   - Explicit subcommands (`odc ods lint` vs `odc okf lint`) force spec-specific evaluation.
3. **Universal Operations**: `odc coverage`, `odc audit`, and `odc find` operate seamlessly across both formats using shared frontmatter AST parsing.
