# Sample Profile Definition & Document Frontmatter Layout

In Open Document Spec (ODS), metadata fields are intentionally separated between **Custom Domain Keys** (top-level) and **ODS Engine Keys** (nested under `ods:`).

---

## 1. Custom Profile Schema Definition File (`.ods/profiles/rfc.md`)

This file defines the profile validation schema and starter body template:

```yaml
---
description: "Request for Comments (RFC) technical architecture proposal and decision record"
expected_keys:
  - author
  - reviewer
  - target_release
aliases:
  Context:
    - Background
    - Problem Statement
    - Motivation
  Proposal:
    - Technical Design
    - Architecture Overview
  Alternatives:
    - Options Considered
    - Rejected Approaches
  Consequences:
    - Impact & Trade-offs
    - Risks
---

# RFC: [Short Descriptive Title]

## Context | Background | Motivation
<!-- 
Provide context around why this proposal is needed:
- What problem are we trying to solve?
- Why is the status quo insufficient?
-->

## Proposal | Technical Design
<!-- 
Describe the proposed technical architecture and implementation details.
-->

## Alternatives | Options Considered
<!-- 
List alternative designs or approaches that were evaluated.
-->

## Consequences | Impact & Trade-offs
<!-- 
Outline positive outcomes, risks, and trade-offs of adopting this proposal.
-->

## Decision | Outcome
<!-- 
Record the final decision status (e.g., Approved, Deferred, Rejected).
-->
```

---

## 2. Scaffolded Document Created by `ods new rfc docs/rfcs/001-caching.md`

When an author creates a document using this profile, `ods` places **Custom Keys at the top-level** and **Engine Keys under `ods:`**:

```yaml
---
title: "Distributed Redis Caching Strategy"
author: "Alice Smith"
reviewer: "Bob Jones"
target_release: "v2.4"
ods:
  profile: rfc
  status: draft
---

# RFC: Distributed Redis Caching Strategy

## Context

## Proposal

## Alternatives

## Consequences

## Decision
```

---

## Key Placement Summary Table

| Key Type | Examples | Canonical Frontmatter Location | Why? |
| :--- | :--- | :--- | :--- |
| **Custom Domain Keys** | `author`, `reviewer`, `target_release`, `service`, `team`, `title`, `tags` | **Top-Level** (outside `ods:`) | Keeps custom metadata clean and fully compatible with Static Site Generators (Next.js, Astro, Docusaurus). |
| **ODS Engine Keys** | `profile`, `status`, `id`, `share`, `depends`, `related`, `code`, `resources` | **Nested `ods:` Map** (`ods.profile: rfc`) | Reserved namespace for Open Document Spec engine graph indexing, validation, and linting. |

> [!NOTE]
> **Permissive Parser Guarantee**: `ods` parser accepts both flat top-level `profile: rfc` and nested `ods.profile: rfc`. Running `ods fmt --migrate` automatically canonicalizes engine keys under `ods:` while preserving custom keys at the top level.
