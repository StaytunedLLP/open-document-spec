---
title: "ODS Validation and Tooling Contract"
description: "ODS lint rules, CLI tool expectations, and adoption/compatibility guidelines."
status: "stable"
order: 6
ods:
  profile: "note"
  status: "stable"
---

# ODS Validation and Tooling Contract

This document defines ODS lint rules, CLI tool expectations, and adoption/compatibility guidelines.

---

## 1. Validation

Conformance for metadata is defined by **validation, not intention**. The normative lint rules:

| Rule | Level | Severity |
| :--- | :---: | :---: |
| Frontmatter parses as valid YAML | 1+ | error |
| Frontmatter MUST NOT contain a `title:` key (title exists only as H1 body header) | 1+ | error |
| `ods.status` (or fallback `status`) is one of `draft`, `stable`, `deprecated`, or `archived` | 1+ | error |
| `ods.share` (or fallback `share`, when present) is one of `public`, `org`, or `private` | 1+ | error |
| `ods.profile` (or fallback `profile`) resolves to a known profile | 1+ | warning |
| `ods.code[].path` MUST NOT contain line number suffixes (e.g., `:L45`) | 1+ | error |
| No dangling `ods.depends`/`ods.related` refs | 3 | error |
| No duplicate document IDs (`ods.id` or path-derived) | 3 | error |
| No `ods.depends` cycles | 3 | error |
| Resource (`ods.resources`), `ods.context.load`, and root `packs:` paths exist and resolve | 3 | error |
| `ods.code` item `path` exists and `role` is a standard role | 1+ role / 3 path | error |
| Index lists exactly its directory's current children | 3 | error |
| Markdown links in the document body do not dangle | 3 | error |
| Profile's expected sections present | 3 | warning |

Tools SHOULD support selecting lint level (for example `--level 1` vs `--level 3`) so teams can adopt gradually: Level 1 checks frontmatter shape; Level 3 checks graph integrity, indexes, resources, imported packs, and body links.

Tools MUST normalize enum-like field values (`status`, `profile`, `share`) to lowercase on ingest. For backward compatibility during migration, tools MUST fallback to reading top-level legacy flat keys (`profile`, `status`, `share`, `id`, `depends`) if the `ods:` nested block is absent.

A workspace claims Level 3 compliance by enforcing these validation rules in CI. To maintain spec integrity, stable documents that fail Level 3 validation checks SHOULD be demoted to draft status until the violations are resolved.

### 1.1 Consumer behavior for unknown content

ODS consumers must preserve author content whenever doing so does not break the core model:

| Scenario | Behavior |
| :--- | :--- |
| Unknown frontmatter key | Preserve and ignore for core validation. |
| Unknown `profile` value | Warn and treat as `note` (Default Profile) for section checks until a Profile definition is available. |
| Unknown `tags` value | Accept; tags are workspace-observed search facets. |
| Unknown `code` item `role` value | Error; Code Reference roles are fixed by this specification. |
| Invalid `share` value | Error; `share` must be `public`, `org`, or `private`. |
| Dangling `depends` / `related` reference | Level-3 error. |
| Dangling `resources` item `path`, `code` item `path`, or `packs:` item `path` | Level-3 error. |
| Duplicate document ID | Level-3 error. |

### 1.2 Command examples

The expected local and CI validation commands are:

```bash
ods lint .
ods index --check .
ods context support/refund-guide.md
ods pack list
```

---

## 2. Adoption tooling contract

Optional tools MAY implement workspace adoption helpers:

- **Report / dry-run**: scan Markdown, report missing frontmatter, unknown profiles, and alias suggestions without writing files.
- **Write mode**: optionally draft minimal frontmatter (`profile`, `status: draft`) for documents that have no frontmatter, inferring `profile` from headings when possible.

Adoption never rewrites prose body content. Plain Markdown without frontmatter remains valid Level 0.

---

## 3. Compatibility (CLI-versioned workspaces)

Tools that implement tag discovery SHOULD treat the **workspace tag set** as the observed union of document `tags` values after normalization. A built-in suggestion list, when present, is for completions and documentation only and MUST NOT make documents invalid.

For the `ods: 0.1` core field set (`profile`, `status`, `share`, `description`, `id`, `depends`, `related`, `resources`, `code`, `context`, `owner`, `tags`, and root `ods` / `ods-cli` / `profiles` / `packs` / `ignore` / `aliases`):

- A workspace root `ods:` value SHOULD equal the current ODS spec version.
- A workspace root `ods-cli:` value SHOULD be an exact CLI version or minimum range such as `>=0.1.18`.
- Workspace discovery SHOULD remain tolerant of older `ods:` values so setup can upgrade them in place.
- `ods lint` and `ods doctor` MUST report missing or stale root `ods:` values and missing, invalid, or unsatisfied `ods-cli:` values.
- `ods init` and `ods setup` MUST write the current spec version to `ods:` and the current CLI minimum requirement to `ods-cli:`.
- Unknown frontmatter keys MUST continue to be ignored by core tools.
- Breaking changes to core field meaning require a new `ods` version string on the root index.
