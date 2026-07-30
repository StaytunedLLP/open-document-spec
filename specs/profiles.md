---
title: "ODS Profiles and Catalogs"
description: "Standard document profiles, workspace custom catalogs, ODS pack resolution, and section header mapping rules."
status: "stable"
order: 1
ods:
  profile: "note"
  status: "stable"
---

# ODS Profiles and Catalogs

This document defines standard document profiles, workspace custom catalogs, ODS pack resolution, and section header mapping rules.

---

## 1. Profiles, Catalogs, and ODS Packs

Profiles define the nature of a document (e.g., a guide, decision record, or system architecture) and specify the H2 section headings tools should expect. They act as structural validation schemas, not copy-paste templates.

ODS uses a two-tiered **Standard (Built-in) vs. Workspace (Custom)** layering for profiles and tags, with distinct enforcement:

| Layer | Profiles | Tags |
| :--- | :--- | :--- |
| **Standard (Spec Level)** | **Standard Profiles** built into the specification (always available) | **Suggested Tags**: Optional built-in suggestion set for tools (never required) |
| **Workspace (Custom Level)** | **Custom Profiles** under `ods-profiles/`, root `profiles:`, or imported **ODS Packs** (`packs:`) | **Workspace Tags**: Observed `tags:` values on documents |
| Unknown values | SHOULD warn; fall back to **Default Profile (`note`)** section checks | MUST NOT error |

Rules:

- The `profile` field is optional. If omitted, the document defaults to the **Default Profile (`note`)**. On ordinary documents it lives at `ods.profile` inside the nested `ods:` map (position #1 in the canonical key sequence — see `specs/SPEC.md`); only the root `index.md` keeps `profile:` flat at the top level, alongside the other root-only workspace marker keys.
- **Standard Profiles** are built into the specification and are always available across all ODS workspaces.
- **Custom Profiles & Profile Catalogs:** Workspaces can declare custom profile catalogs or import reusable **ODS Packs** in the root `index.md` frontmatter:

```yaml
---
profile: index
ods: 0.1
odc: ">=0.0.1"
profiles:
  - ods-profiles
  - docs/profiles
packs:
  - vendor/engineering-pack
---
```

- An **ODS Pack** is a versioned repository or directory containing reusable document profiles (`ods-profiles/`), skills (`skills/`), or templates (`templates/`).
- Remote Git packs (HTTPS, SSH, or shorthand like `user/repo`) are synced by tooling to `~/.odc/packs/` (legacy `~/.ods/packs/` is still read) and referenced in root `index.md`. Local path packs are linked relatively.
- Profile definitions MUST resolve in the following order of precedence: Standard Profiles, catalog roots listed in root `profiles:`, imported ODS Packs listed in `packs:`, and finally the default `ods-profiles/` directory.
- If a profile name is defined in multiple catalogs, tools SHOULD use the first definition loaded and issue a conflict warning. Level-3 workspaces SHOULD treat duplicate profile definitions as validation errors.
- Profile catalogs are workspace-local utilities. They do not participate in the document graph and SHOULD be excluded from ordinary index listings.
- Each profile is defined in a Markdown file. The profile's identifier is derived from the file name (excluding the `.md` extension). If a catalog contains an `index.md` file, the profile name is taken from its parent directory name.
- Second-level headings (`## H2`) in a profile document declare the expected sections. A heading can define multiple alternative names separated by pipes, for example: `## Goal | Objective | Purpose`.
- A profile catalog file can also specify section-level aliases in its frontmatter using an `aliases:` list.
- Encountering an unrecognized profile SHOULD produce a warning, and tools SHOULD treat the document as a `note` (the Default Profile) for validation purposes until a definition is provided.
- Custom profiles are purely additive; they define structural requirements for specific document types without altering standard profiles or establishing complex inheritance hierarchies.

---

## 2. Standard Profiles

| Profile | Meaning | Expected sections |
| :--- | :--- | :--- |
| `note` | Free-form prose with no fixed structure (Default Profile). | none |
| `feature` | A capability, requirement set, or PRD-style document. | Goal, Scope, Requirements, Acceptance Criteria, Risks |
| `guide` | A tutorial or how-to. | Overview, Prerequisites, Steps, Troubleshooting |
| `api` | Reference for an interface; the canonical machine artifact stays a resource. | Overview, Request, Response, Errors, Examples |
| `architecture` | How a system is structured and why. | Overview, Components, Data Flow, Trade-offs |
| `decision` | A recorded decision, ADR, RFC outcome, or policy choice. | Context, Decision, Alternatives, Consequences |
| `sop` | A standard operating procedure or runbook. | Purpose, Prerequisites, Steps, Validation, Rollback |
| `policy` | A rule set people must follow. | Purpose, Scope, Rules, Exceptions |
| `meeting` | Notes and outcomes from a meeting or discussion. | Attendees, Agenda, Decisions, Action Items |
| `faq` | Questions and answers. | question/answer pairs; no fixed H2 list |
| `checklist` | A verifiable list of items or gates (preflight, release, audit). | Overview, Items, Verification, Notes |
| `index` | Navigation files for directories. | none |

> [!NOTE]
> There is no standard `specs` profile in core ODS. Specification documents should use `note`, `decision`, or `guide` depending on the document's purpose.

---

## 3. Workspace Tag Operations

Documents MAY declare free-form strings in their frontmatter `tags:` list. Tools MUST normalize tags to lowercase on ingest.

Tooling SHOULD provide query and refactoring operations for workspace tags:
- **`ods tags`**: Inspect all unique tags in the workspace and their frequency across documents.
- **`ods find --tag <name>`**: Retrieve all documents carrying a given tag.
- **`ods tag rename <old> <new>`**: Safely rewrite occurrences of `<old>` to `<new>` in document frontmatter across the workspace without altering body content.

---

## 4. Section Aliases

| Canonical | Recognized aliases |
| :--- | :--- |
| Goal | Objective, Objectives, Purpose |
| Scope | In Scope, Boundaries |
| Requirements | Functional Requirements, Needs |
| Acceptance Criteria | Acceptance, Success Criteria, Definition of Done |
| Overview | Introduction, Summary, Background |
| Prerequisites | Requirements, Before You Begin |
| Steps | Instructions, Procedure, Process |
| Troubleshooting | Common Issues, FAQ |
| Context | Background |
| Alternatives | Options, Options Considered |
| Consequences | Outcome, Implications |
| Validation | Verification, Checks |
| Rollback | Recovery, Revert |
| Rules | Standards, Requirements |
| Action Items | Actions, Next Steps, TODO |
| Risks | Risks and Mitigations, Concerns |
| Trade-offs | Tradeoffs, Pros and Cons |
