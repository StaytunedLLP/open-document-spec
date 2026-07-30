---
title: "Open Document Specification (ODS)"
description: "Normative specifications for Open Document Spec (ODS), covering file formats, frontmatter, graph edges, context engine, and Level 0-3 validation."
---

# Open Document Specification (ODS)

This directory contains the normative specifications for Open Document Spec.

**CLI (OpenDocify):** ODS document operations use the **`odc ods`** namespace (e.g. `odc ods lint`). OKF is a sibling native engine under **`odc okf`**. See [frontmatter keys: ODS vs OKF](../docs/specs/frontmatter-keys-ods-vs-okf.md).

## ODS modules

- [SPEC.md](SPEC.md) — Core Normative Specification
- [context.md](context.md) — Context Engine & Reading List Bounds
- [graph.md](graph.md) — Dependency Graph & Edge Types
- [indexes.md](indexes.md) — Directory Indexes & Progressive Disclosure
- [non-goals.md](non-goals.md) — Scope Boundaries & Non-Goals
- [profiles.md](profiles.md) — Standard Profiles & Heading Schemas
- [resources-and-code.md](resources-and-code.md) — Resource Lists & Code Reference Mapping
- [validation.md](validation.md) — Conformance Levels 0-3 & Validation Rules

## Multi-spec (OpenDocify)

- [ODS vs OKF frontmatter keys](../docs/specs/frontmatter-keys-ods-vs-okf.md) — which keys exist in which spec, purpose, CLI mapping
- [ODC migration plan](../docs/plan/ods_to_odc_migration_and_cli_architecture.md)
- [Native OKF plan](../docs/plan/okf_native_support.md)
