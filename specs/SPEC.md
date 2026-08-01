---
title: "ODS Specification"
description: "Open Document Spec Working Draft 1 - Core format model, conformance levels, frontmatter canonical sequence, and lifecycle operations."
status: "stable"
order: 0
ods:
  profile: "note"
  status: "stable"
---

# ODS Specification

**Open Document Spec - Working Draft 1**

The key words **MUST**, **MUST NOT**, **SHOULD**, **SHOULD NOT**, and **MAY** are to be interpreted as described in RFC 2119.

Design principles, in priority order:

1. **Human first**: readable in any text editor, forever.
2. **Free at level zero**: standard Markdown is already fully conformant; adopting ODS means enriching existing documents with metadata, never migrating or rewriting them.
3. **Token efficient (DRY)**: every fact has exactly one canonical location to minimize redundancy and optimize AI context consumption.
4. **Graph native**: relationships between documents are declared explicitly as frontmatter metadata, rather than inferred from text links or prose.
5. **Trust from validation**: metadata is only as reliable as the checks that enforce it; the specification does not require any rules that cannot be automatically validated or linted.

---

## Format model

An ODS Document is a Markdown file with optional YAML frontmatter.

The frontmatter is machine-readable metadata for tools. The Markdown body is human-readable content for people and AI agents. Frontmatter values are the normative source for identity, lifecycle, graph edges, resource links, code references, and context loading. Body prose explains purpose, rationale, decisions, and usage; it does not redefine metadata.

Plain Markdown without frontmatter remains a valid Level-0 Document.

---

## 1. Core model

ODS defines four core concepts:

| Concept | Description |
| :--- | :--- |
| **Workspace** | The top-level repository unit; a directory tree of documents marked by a root `index.md`. |
| **Document** | A Markdown file (`.md`), optionally containing YAML frontmatter. |
| **Index** | A generated `index.md` file listing a directory's immediate children. |
| **Resource** | Any non-Markdown file referenced by a document (e.g., CSV, PNG, OpenAPI spec, PDF). |

There are no other core concepts. A workspace may contain any number of subdirectories and files. Concepts like bundles, shared folders, archives, and local groups are represented as ordinary directories—the filesystem tree naturally expresses these relationships. `collection` is not a core concept; if a workspace uses the term, it must define distinct, documented semantics outside of this specification.

---

## 2. Documents and conformance levels

Each conformance level described below is fully valid under ODS.

### Level 0 - plain Markdown

Any `.md` file, with or without frontmatter, is a valid ODS document. Tools MUST accept frontmatter-less documents and MUST NOT require any fields upon document creation.

### Level 1 - typed

A Level-1 document introduces identity frontmatter (such as `profile` and `status`):

```markdown
---
profile: guide
status: draft
---

# Checkout Setup

Steps for configuring checkout in a local development environment.
```

### Level 2 - linked

A Level-2 document establishes graph relationships (see [Graph Specification](/spec/graph)) and resides within an indexed workspace (see [Indexes Specification](/spec/indexes)).

### Level 3 - validated

A Level-3 workspace enforces structural integrity and lint rules both locally and in Continuous Integration (CI). These checks ensure there are no dangling references, no duplicate IDs, all profiles resolve correctly, and indexes remain up to date. Documents marked as `status: stable` or listed in an index SHOULD be held to Level-3 validation ("creation is free; promotion has rules").

---

## 3. Frontmatter

Frontmatter is a single YAML block delimited by `---` at the very beginning of the document. **All fields are optional.** To allow for custom extensions and static site generator (SSG) interoperability, tools MUST ignore unknown fields.

Frontmatter is segregated into **Universal Top-Level Metadata** (read by SSGs, Obsidian, OpenGraph, CMSs) and **ODS Engine Metadata** (nested under an `ods:` map):

### Universal Top-Level Metadata Keys

| Field | Type | Meaning |
| :--- | :--- | :--- |
| `description` | string | Single-line summary of the document. Used by ODS index listings AND extracted by SSGs for HTML `<meta name="description">` and social preview cards. |
| `tags` | list of strings | Free-form search facets up to $N$ items. Normalizes to lowercase. Natively parsed by Obsidian, Logseq, Hugo, Docusaurus, and Rspress. |
| `owner` | string \| list | Responsible individual/team or list of teams up to $N$ owners. |
| `created` | string | Document creation timestamp (`YYYY-MM-DD` or ISO-8601). Optional; for non-Git authors. |
| `updated` | string | Document last update timestamp (`YYYY-MM-DD` or ISO-8601). Optional; accepts `last_updated` alias. |

### ODS Engine Metadata Keys (Nested inside `ods:`)

When scaffolded (`odc ods new`), adopted (`odc ods adopt`), or formatted (`odc ods fmt --write`), keys inside the `ods:` map MUST be emitted in this exact **Canonical Key Sequence**:

| Order | Field | Type | Meaning |
| :-: | :--- | :--- | :--- |
| 1 | `profile` | string | Structural schema (`guide`, `decision`, `feature`, `sop`, `api`, `meeting`, `faq`). Defaults to `note`. |
| 2 | `status` | string | Lifecycle state (`draft`, `stable`, `deprecated`, `archived`). Defaults to `draft`. |
| 3 | `id` | string | Explicit node ID override for graph rename stability. Optional; defaults to workspace-relative path minus `.md`. |
| 4 | `share` | string | ODS CLI Visibility Directive (`public`, `org`, `private`). Filters nodes in `odc ods context`, `odc ods export`, and `ods pack`. |
| 5+ | `depends` | list of refs | Multi-value list up to $N$ prerequisite document paths (`.md`) or extensionless path IDs. |
| 5+ | `related` | list of refs | Multi-value list up to $N$ reference document paths (`.md`) or extensionless path IDs. |
| 5+ | `resources` | list of maps | Multi-value list up to $N$ non-Markdown file asset paths (`path`). |
| 5+ | `code` | list of maps | Multi-value list up to $N$ implementation mappings (`path`, `role`, `symbol`). `symbol` accepts single string or multi-line array up to $N$ symbols. Line numbers are prohibited in `path`. |
| 5+ | `context` | map | AI context scope directives (`max-depth` graph hops, `load` file paths up to $N$, `ignore` directory path masks up to $N$). |

### Root `index.md` Workspace Configuration Keys

| Field | Type | Meaning |
| :--- | :--- | :--- |
| `ods` | string | Root `index.md` only: ODS spec version string marker (e.g. `ods: 0.1`). Defines workspace boundary. |
| `odc` | string | Root `index.md` only: minimum compatible ODS CLI version constraint (e.g. `>=0.0.1`). |
| `profiles` | list of paths | Custom profile catalog paths under workspace root up to $N$ items. |
| `packs` | list of paths | Imported ODS Packs list up to $N$ items. |
| `ignore` | list of paths | Workspace scan exclusion path masks up to $N$ items. |
| `aliases` | map | Section heading aliases mapping. |

> [!IMPORTANT]
> **Frontmatter `title` Prohibition Rule**: Frontmatter MUST NOT contain a `title:` key. The document title exists exclusively as the first `# H1` heading in the Markdown body prose (Single Source of Truth / Token Efficiency).
> 
> **Tooling Tolerance Contract**: The ODS parser MUST NOT error if frontmatter keys inside `ods:` appear out of sequence. However, formatting and scaffolding commands (`odc ods fmt --write`, `odc ods new`, `odc ods adopt`) MUST enforce canonical key sequence (`profile` $\rightarrow$ `status` $\rightarrow$ `id` $\rightarrow$ `share` $\rightarrow$ `depends`, `related`, `resources`, `code`, `context`).

### Minimal and Complete Examples

A minimal typed document specifies `profile` and `status` inside the `ods:` map:

```markdown
---
ods:
  profile: guide
  status: draft
---

# Checkout Setup

Steps for configuring checkout in a local development environment.
```

Here is a complete document example illustrating Universal Top-Level metadata and ODS Engine metadata in canonical sequence:

```markdown
---
# Universal Top-Level Metadata
description: How to process customer refunds in the dashboard.
tags:
  - customer-care
  - billing
owner:
  - support-team
  - billing-ops

# ODS Engine Metadata (Canonical Key Order: profile -> status -> id -> share -> depends, related, resources, code, context)
ods:
  profile: guide
  status: stable
  id: docs/guide/07-examples/ecommerce/support/refund-guide
  share: public

  depends:
    - ../website/cart-checkout.md
    - ../auth/sessions.md
  related:
    - ../products/glow-serum.md

  resources:
    - path: docs/refund-flow.pdf
    - path: docs/architecture-diagram.png

  code:
    - path: apps/web/src/routes/refund.tsx
      role: entrypoint
      symbol: RefundRoute
    - path: apps/web/src/features/refunds/process.ts
      role: implementation
      symbol:
        - processRefund
        - validateRefund
        - RefundSchema
    - path: apps/web/src/features/refunds/process.test.ts
      role: test
      symbol:
        - processRefundTest

  context:
    max-depth: 2
    load:
      - ../website/cart-checkout.md
      - ../resources/users-sample.csv
    ignore:
      - marketing/
      - legacy/
---

# Refund Processing

## Overview

Use this guide when processing customer refunds in the dashboard.
```

---

## Specification Modules

The remaining parts of ODS are detailed in the following sub-specifications:

- **[Identity & Graph Relationships](/spec/graph)**: ID resolution, single source of truth rules, and graph edge semantics.
- **[Indexes & Workspace Scanning](/spec/indexes)**: Folder index rules, root index requirements, and scanning ignores.
- **[Resources & Code References](/spec/resources-and-code)**: Native asset mappings and standardized implementation roles.
- **[Deterministic AI Context](/spec/context)**: How agents load context blocks safely.
- **[Profiles & Catalogs](/spec/profiles)**: Document schemas, expected headings, and custom profile mapping.
- **[Validation & Compatibility](/spec/validation)**: Lint rules matrix, unknown content handling, and tooling expectations.
- **[Non-Goals](/spec/non-goals)**: Out-of-scope concepts and design boundaries.

---

## 4. Lifecycle Operations & Heading Inference

### Atomic Document Lifecycle

Tools MUST support four core atomic lifecycle operations for workspace documents:

1. **Scaffolding (`odc ods new <path>`)**: Instantly creates a document pre-populated with Level-1 frontmatter (`profile`, `status: draft`, `description`), path-derived ID, inferred H2/H3 section placeholders, and automatically updates the parent `index.md` child listing.
2. **Relocation (`odc ods mv <from> <to>`)**: Renames or moves a document or folder while atomically updating path-derived IDs, relative prose links, `depends:`, `related:`, `code[].path`, and child `index.md` entries across the workspace graph.
3. **Archival (`ods archive <path-or-id>`)**: Updates document metadata to `status: archived`, moves the file to an `archive/` subfolder, updates index listings, and preserves graph edges.
4. **Atomic Deletion (`odc ods rm <path-or-id>`)**: Removes the target document from disk, strips the file entry from parent `index.md` files, and scrubs the deleted document ID from all dependent documents' `depends:` and `related:` frontmatter arrays workspace-wide.

### Smart Profile Inference (H2 + H3 Heading Scanning)

When adopting or creating un-typed documents, tools SHOULD scan both **Heading 2 (`##`)** AND **Heading 3 (`###`)** section titles to infer the target `profile`:

- `Goal`, `Objective`, `Scope`, `Requirements`, `Acceptance Criteria`, `Risks` $\rightarrow$ **`profile: feature`**
- `Overview`, `Prerequisites`, `Steps`, `Troubleshooting` $\rightarrow$ **`profile: guide`**
- `Context`, `Decision`, `Alternatives`, `Consequences` $\rightarrow$ **`profile: decision`**
- `Purpose`, `Rules`, `Exceptions`, `Validation`, `Rollback` $\rightarrow$ **`profile: sop`**
- `Request`, `Response`, `Errors`, `Examples`, `Endpoint` $\rightarrow$ **`profile: api`**
- `Attendees`, `Agenda`, `Decisions`, `Action Items` $\rightarrow$ **`profile: meeting`**
- `Q/A`, `Questions`, `Answers`, `FAQ` $\rightarrow$ **`profile: faq`**
