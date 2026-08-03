---
description: "Index placement, governance model, root configuration keys, and tooling scan ignore defaults."
status: "stable"
order: 4
ods:
  profile: "note"
  status: "stable"
---

# ODS Indexes and Workspace Scanning

This document defines how indexes are structured, how version compatibility is declared, how imported ODS Packs are declared, and how tools scan a workspace.

---

## 1. Indexes: progressive disclosure

Index files (`index.md`) provide a clear, hierarchical navigation path for both humans and AI: start at the root index, descend one directory level at a time, and retrieve only the documents along the path to the target.

### 1.1 Placement and scope

- An `index.md` in the root directory designates the boundary of a workspace. This file serves as the sole workspace marker; there is no separate manifest file.
- Any directory MAY have its own `index.md`.
- This structure applies recursively: any subdirectory within the workspace can hold folders, files, and its own local `index.md` to guide navigation.
- An `index.md` file MUST only list the immediate children (documents and subdirectories) of its own directory. It MUST NOT recursively list deeper descendants; nested navigation is handled by child `index.md` files.

### 1.2 Content

An index is a Markdown list of links, each with an optional one-line summary:

```markdown
---
profile: index
---

# Product Documentation

- [features/](features/index.md) - customer-facing capabilities
- [api/](api/index.md) - HTTP API reference
- [architecture.md](architecture.md) - system overview and trade-offs
```

Rules:

- Indexes MUST NOT contain document counts, size statistics, or other metadata that would require updates on every commit.
- Indexes SHOULD be generated automatically by tooling (`ods index`), treated similarly to lockfiles—reviewed and committed, but never modified by hand. The generator MUST extract the summary for a child document from that document's frontmatter `description` field, if available. Hand-written index summaries are deprecated.
- `status` is optional on indexes and is usually omitted on generated navigation files.
- The **root** index MUST declare the ODS spec version and CLI compatibility requirement once for the whole workspace:

```yaml
---
profile: index
ods: 0.1
ods: ">=0.0.1"
packs:
  - vendor/engineering-pack
---
```

An ODS-compliant workspace is identified by this root `ods` field or by entry in the global machine-level workspace registry (`~/.ods/odsconfig.toml`, with legacy read of `~/.ods/odcconfig.toml` if present). ODS follows a strict **Dual-Scope Workspace Governance Model**:

1. **OS / Machine Level (`~/.ods/odsconfig.toml`)**: A machine-wide registry managed via `ods workspaces [add|remove|list|path]` that tracks active ODS repositories for the developer's background watch service (`ods start`). The file contains a `[workspaces]` key with a list of absolute directory paths:
   ```toml
   [workspaces]
   paths = [
     "/home/user/projects/my-docs",
     "/home/user/projects/ecommerce"
   ]
   ```
2. **Project / Repository Level (Root `index.md` Frontmatter)**: All project-level policy, spec compatibility (`ods:`), CLI requirements (`ods:`), multi-spec auto-activation & key suppression (`specs:` / `okf_lint:`), profile catalogs (`profiles:`), pack imports (`packs:`), scan excludes (`ignore:`), and heading aliases (`aliases:`) are declared directly on the root `index.md` frontmatter. ODS strictly enforces a **Zero Config-File Guarantee** inside repositories—no proprietary `.odsconfig`, `workspace.toml`, or `.odsignore` files are created inside project trees.

Ordinary documents MUST NOT carry `ods:` or `ods:`. Nested navigation indexes SHOULD omit both when they are part of the same workspace. Child directories inside imported ODS Packs (`ods-profiles/`, `skills/`, `templates/`) maintain nested `index.md` files (`profile: index`) without requiring an `ods:` key, inheriting their workspace boundary from the pack root.

---

## 2. Tooling workspace scan (ignore defaults)

Tools that walk a workspace MUST NOT treat the following directory names as documentation content (case-insensitive base names): `target`, `node_modules`, `dist`, `build`, `.artifacts`, `.git`, `.hg`, `.svn`, `.jj`, `__pycache__`, `.venv`, `venv`, `vendor`, and any name that starts with `.`.

In addition, the **root** `index.md` MAY declare workspace-level excludes:

```yaml
---
profile: index
ods: 0.1
ods: ">=0.0.1"
ignore:
  - src
  - apps/web
---
```

Each `ignore` entry is a **workspace-relative path prefix** (with or without a trailing `/`). Tools MUST NOT collect Markdown under those prefixes, MUST NOT require `index.md` files there, and MUST NOT list ignored trees as children of parent indexes. Implementation code and package READMEs typically live outside the ODS document graph this way.

Tools MAY also respect the repository's existing `.gitignore` when scanning (for build and dependency noise). ODS does **not** define a separate ignore or config file format such as `.odsignore`; workspace policy stays on the root `index.md` (`ods`, `profiles`, `packs`, `ignore`, `aliases`). Documents remain plain `.md` only.

Index child validation SHOULD consider only top-level Markdown list links (unordered list items with a single link), not every link in prose or tables. Indexes SHOULD be generated by tooling (`ods index` and equivalent LSP structure sync) and treated like lockfiles.
