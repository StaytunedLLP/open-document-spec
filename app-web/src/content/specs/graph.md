---
title: "ODS Graph Specification"
description: "Identity, single source of truth rules, and graph relationships in ODS."
status: "stable"
order: 2
ods:
  profile: "note"
  status: "stable"
---

# ODS Graph Specification

This document defines identity, single source of truth rules, and graph relationships in ODS.

---

## 1. Identity: IDs are paths

A document's ID **is its workspace-relative path, without the `.md` extension**, using `/` separators:

```
features/login.md        ->  features/login
guides/setup/oauth.md    ->  guides/setup/oauth
```

Instead of manually generating IDs, the filesystem path implicitly defines the document's identity.

Rules:

- IDs MUST be unique within a workspace. This uniqueness is guaranteed automatically when using path-derived IDs.
- An explicit `ods.id:` field inside the nested `ods:` map overrides the path-derived ID. Its primary purpose is **stability across renames**: when a document needs to be moved or renamed, you can preserve the existing ID using `ods.id:` to avoid breaking external references, or use tooling (e.g., `ods mv`) to update references.
- References to non-existent IDs are considered dangling references, which trigger lint errors in Level-3 workspaces.
- IDs are case-insensitive and normalized to lowercase alphanumeric characters, hyphens, and slashes (`a-z`, `0-9`, `-`, `/`). Tools MUST resolve all references case-insensitively and normalize them to lowercase.
- Path separators MUST be normalized to forward slashes (`/`) across all operating systems.

---

## 2. Single Source of Truth (Token Efficiency Rules)

- A document's title MUST only exist once: as the first `# H1` header in the markdown body. Frontmatter MUST NOT define a duplicate title field.
- Machine-readable relationships MUST reside exclusively in the frontmatter. The document body MUST NOT contain redundant lists of dependencies or related links. Prose within the body may explain the *why* behind a relationship, but the edge declaration belongs solely in the frontmatter.
- Metadata fields (such as `status` and `owner`) exist only in the frontmatter and MUST NOT be restated in the document body.
- Information should have a single canonical source. Instead of duplicating knowledge across documents, reference or link to the authoritative document.

---

## 3. The graph: `depends` and `related`

ODS defines exactly **two** edge types inside the `ods:` frontmatter block. Richer vocabularies (e.g., `implements`, `extends`, and `replaces`) are deliberately excluded from the core; use `ods.related` until a future extension standardizes them from observed need.

```yaml
ods:
  depends:
    - ../auth/sessions.md
    - ../api/tokens.md
  related:
    - ../policy/customer-policy.md
```

Semantics:

- **`depends`**: Indicates that this document cannot be fully understood, implemented, or acted upon without first reading the target document. This is a directional relationship. When an AI parses this document, it SHOULD load its dependencies transitively (up to `context.max-depth` if specified).
- **`related`**: Points to content that serves as useful reference or further reading. This is a non-binding relationship; an AI agent MAY load it optionally.

Rules:

- References in `depends` or `related` SHOULD use editor-jumpable `.md` paths to Markdown Documents. Tools MUST also resolve legacy extensionless document IDs for backward compatibility.
- `ods fmt --refs md-paths` SHOULD rewrite resolvable legacy Document references in `depends`, `related`, and Document entries in `context.load` to canonical `.md` paths. Fragment anchors (e.g., `#heading-name`) and query parameters MUST be preserved during rewriting. It MUST NOT rewrite `id`, `resources` item `path`, `code` item `path`, `ignore`, `context.ignore`, or external URLs (`http://`, `https://`).
- `ods lint --canonical-refs` SHOULD warn on resolvable extensionless Document references and suggest the canonical `.md` path. Default lint MUST remain backward-compatible and accept both forms.
- The dependency graph MUST NOT contain cyclic dependencies in Level-3 workspaces (resulting in a validation error).
- Relationships are declared only on the dependent side. Backlinks and reverse lookups ("Which documents depend on this one?") are computed dynamically by tooling; they MUST NOT be written manually, as hardcoded backlinks are prone to becoming outdated.
