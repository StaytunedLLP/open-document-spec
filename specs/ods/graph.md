---
description: "Identity, single source of truth rules, and graph relationships in ODS."
ods:
  profile: "note"
  status: "stable"
  depends:
    - keys.md
  related:
    - context.md
    - validation.md
---

# ODS · Graph

Identity, single source of truth (SSOT), and graph edges (`depends` / `related`).

Key shapes: [keys.md](keys.md). Context loading walks this graph: [context.md](context.md).

---

## 1. Identity: IDs are paths

A document's ID **is its workspace-relative path without the `.md` extension**, using `/` separators:

```
features/login.md        →  features/login
guides/setup/oauth.md    →  guides/setup/oauth
```

Rules:

- IDs MUST be unique within a workspace (automatic with path-derived IDs).
- Explicit `ods.id` overrides the path-derived ID for **rename stability**. Prefer `ods mv` to rewrite references when moving files.
- References to missing IDs are dangling (Level-3 error).
- IDs are case-insensitive; tools MUST normalize to lowercase `a-z`, `0-9`, `-`, `/`.
- Path separators MUST normalize to `/` on all platforms.

---

## 2. Single source of truth

- Title exists **once**: first `# H1` in the body. Frontmatter MUST NOT define `title:`.
- Machine-readable relationships live **only** in frontmatter (`depends` / `related`). Body MAY explain *why*, not restate edge lists.
- Lifecycle fields (`status`, `owner`, …) live in frontmatter only—do not restate them as metadata in the body.
- Prefer linking to the authoritative document over duplicating knowledge.

---

## 3. Edges: `depends` and `related`

ODS defines exactly **two** edge types under `ods:` (not top-level). Richer vocabularies (`implements`, `extends`, `replaces`) are out of core—use `related` until a future extension standardizes them.

```yaml
ods:
  depends:
    - ../auth/sessions.md
    - ../api/tokens.md
  related:
    - ../policy/customer-policy.md
```

| Edge | Meaning |
| :--- | :--- |
| **`depends`** | Directional prerequisite. This document cannot be fully understood or acted on without the target. Agents SHOULD load dependencies transitively (up to `context.max-depth` when set). |
| **`related`** | Soft reference / further reading. Agents MAY load optionally. |

Rules:

- Prefer editor-jumpable `.md` paths. Tools MUST still resolve legacy extensionless IDs.
- `ods fmt --refs md-paths` SHOULD rewrite resolvable legacy IDs in `depends`, `related`, and `context.load` to `.md` paths (preserve fragments/queries). MUST NOT rewrite `id`, `resources`/`code` item `path`, `ignore`, `context.ignore`, or external URLs.
- `ods lint --canonical-refs` SHOULD warn on extensionless refs; default lint accepts both forms.
- Level-3: dependency graph MUST NOT contain cycles.
- Declare edges only on the dependent side. Backlinks are computed by tooling—never hand-write reverse lists.
