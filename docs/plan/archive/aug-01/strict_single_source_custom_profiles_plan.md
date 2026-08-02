# Master Implementation Plan: Dynamic Spec Engine, Custom Profiles, & Strict Single-Source Registration

This document tracks all engineering tasks for Open Document Spec (ODS), including the **Dynamic Spec Schema Engine**, **Custom Profile System**, and **Strict Single-Source Registration**.

---

## 1. Executive Summary & Completed Scope

All core architectural features and refactorings have been **100% executed and verified** (`cargo test --workspace` passed 100%).

---

## 2. Engineering Milestones (100% Completed)

### Phase 1: Dynamic Spec Schema Engine (`ods-core/src/spec/`)
- [x] Create `SpecSchemaRegistry`, `SpecSchema`, `KeyDefinition`, `KeyPlacement`, and `KeyType` (`src/ods-core/src/spec/schema.rs`).
- [x] Register standard ODS core keys (`profile`, `status`, `id`, `share`, `depends`, `related`, `code`, `resources`, `context`) and OKF keys (`okf_version`, `concept_id`, `trust_tier`, `attested`, `date_range`).
- [x] Support declarative key placement rules (`TopLevel` vs `NestedEngineMap` vs `RootIndexOnly`) and value types (`String`, `List`, `Map`, `Enum`).
- [x] Export schema types in `spec/mod.rs` and `lib.rs`.

### Phase 2: Custom Profile Catalog & 4-Tier Inferencing Engine (`ods-core/src/profiles/`)
- [x] Add `name:`, `description:`, and `expected_keys:` frontmatter parsing for custom profile Markdown definitions.
- [x] Implement 4-tier profile inferencing in `resolve_document_profile`:
  - [x] Tier 1: Explicit Frontmatter (`ods.profile: <name>`)
  - [x] Tier 2: Directory Path Convention (`/adrs/` $\rightarrow$ `decision`, `/features/` $\rightarrow$ `feature`, `/apis/` $\rightarrow$ `api`, `/sops/` $\rightarrow$ `sop`, `/rfcs/` $\rightarrow$ `rfc`, `/guides/` $\rightarrow$ `guide`, `/policies/` $\rightarrow$ `policy`)
  - [x] Tier 3: Heading Signature Fuzzy Matching (matches document H2 `##` headings against catalog section definitions)
  - [x] Tier 4: Default Fallback (`note`)
- [x] Implement `render_profile_template` for custom profile scaffolding.

### Phase 3: Comprehensive CLI Command Integration (`ods-cli`)
- [x] **`ods profile list`**: Displays all standard (12) + custom profiles, expected keys, section trees, and source layer tags (`[default ODS]` vs `[project]`).
- [x] **`ods profile init <name>`**: Scaffolds custom profile schema files.
- [x] **`ods lint`**: Enforces schema key placement rules, required `expected_keys`, H2/H3 section hierarchies, and graph links.
- [x] **`ods new <profile> <path>`**: Scaffolds target documents from profile section templates.

### Phase 4: Test Suite Alignment & Integration Verification
- [x] Add unit tests in `ods-core` for dynamic spec schemas, custom profile loading, 4-tier profile inferencing, and template rendering.
- [x] Align integration test expectations in `ods-cli/tests/` (`cli_matrix_core_commands.test.rs`, `profiles.test.rs`, `okf_commands.test.rs`).
- [x] Execute `cargo test --workspace` with 100% clean pass.

### Phase 5: Folder Auto-Discovery Removal (`ods-core/src/profiles.rs`)
- [x] Refactor `profile_catalog_roots` to remove folder auto-discovery loops (`"ods-profiles"`, `".ods/profiles"`, `".ods/profiles"`, global home profiles).
- [x] Ensure custom profile files are loaded exclusively from explicit `custom-profiles:` array in root `index.md` (and imported ODS Packs).

### Phase 6: CLI Profile Init & Test Alignment
- [x] Update `ods profile init <name>` to scaffold custom profile files at `docs/profiles/<name>.md` and instruct authors to register them in root `index.md` under `custom-profiles:`.
- [x] Update integration tests in `ods-core/tests/` and `ods-cli/tests/` to register custom profiles via `custom-profiles:` in test root `index.md`.

### Phase 7: Specifications & User Guides Audit
- [x] Update `specs/profiles.md` and `docs/specs/custom_profiles.md` to mandate explicit `custom-profiles:` registration only.
- [x] Update `README.md`, `CONTRIBUTING.md`, and `skills/ods/SKILL.md`.

---

## 3. Verification Plan Results

```bash
cargo test --workspace
```
- **100% Clean Pass** across all unit and integration tests.
