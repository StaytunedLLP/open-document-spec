---
description: "Concepts and features ODS intentionally excludes from its core design to keep adoption frictionless, validation simple, and AI navigation highly deterministic."
status: "stable"
order: 7
ods:
  profile: "note"
  status: "stable"
---

# ODS Non-Goals

This document lists concepts and features ODS intentionally excludes from its core design to keep adoption frictionless, validation simple, and AI navigation highly deterministic.

---

## Logical Exclusions

### 1. Document Format & Identity
- **No new file extension**: Standard `.md` Markdown is used. New extensions create editor friction, break git diffing, and complicate ingestion.
- **No frontmatter `title` field**: Document title MUST exist only once as the first `# H1` body header (Single Source of Truth / Token Efficiency).
- **No un-prefixed flat engine keys or flat key prefixing**: Engine metadata is grouped in a nested `ods:` map (`ods: { profile, status, share, id, depends, related, resources, code, context }`) to avoid collisions with static site generators (Rspress, Docusaurus, Astro) and visual clutter.
- **No separate type taxonomy**: The `ods.profile` field uniquely identifies a document's classification; adding a parallel `type` field introduces redundant taxonomy bureaucracy.
- **No per-document spec version / profile version**: Version info belongs in the root index and profile definitions, not duplicated in every file causing commit churn.
- **No lifecycle field**: The `ods.status` field is the single source of truth for document maturity; adding `lifecycle` creates a duplicate status system.
- **No updated timestamp**: Hand-maintained timestamps rot quickly; git commit logs provide the single authoritative history.
- **No closed tag registries / allowlists**: Tags are free-form search facets. Hardcoded tag registries cause Synonym bureaucracy and friction; soft completion suggestions are sufficient.

### 2. Manifests & Configs
- **No manifest files**: Navigation, workspaces, metadata, and profiles are resolved directly from the directory structure and root `index.md`. There are no workspace, context, or collection manifests.
- **No lock files or derived stats**: Derived statistics (like complete counts) rot instantly on commits; layout is resolved dynamically, avoiding lockfile merge conflicts.
- **No required `llms.txt`**: LLM indexes can be generated dynamically from `index.md`; ODS avoids maintaining a secondary output file in the repository.
- **No mandatory enterprise configs / namespaces**: Third-party registries (like Jira/GitHub tickets) are supported via namespaced custom frontmatter keys rather than standardizing them in the core schema.

### 3. Graph Edges & Relationships
- **No extra relationship vocabulary**: The core graph supports only `depends` (directional prerequisites) and `related` (references). Edges like `implements`, `extends`, or `replaces` are excluded to keep tooling simple.
- **No universal `url:` link field**: Prose links inside the Markdown body are sufficient for external URLs; frontmatter is reserved for graph structure.
- **No inline source comments / code frontmatter**: Frontmatter belongs in `.md` files. Parsing annotations inside source code (`.ts`, `.rs`) adds language-specific dependencies and drifts quickly.
- **No line numbers as code identity**: Code references target file paths and symbols; line numbers change during normal editing, leading to stale metadata.
- **No project-customizable code roles**: Standard roles (`entrypoint`, `implementation`, `test`, `schema`, `migration`, `config`, `infrastructure`, `pipeline`) cover the Pareto set. Custom roles break AI agent portability.

### 4. Templating & Customization
- **No profile inheritance / customization**: Profiles act as flat validation schemas. Inheritance and hiding rules complicate validators and obscure document structure.
- **No standardized templates / rendering engine**: snippets and outline generator scripts are local tools. Standardizing templates freezes layout options too early.
- **No template engines**: ODS describes documents and validates metadata; it is not a static site generator or rendering pipeline.
- **No separate asset bucket**: Attached artifacts are declared as path-only items in `resources`; adding a separate asset key duplicates resources.
- **No typed code schema inside `resources`**: `resources` is path-only. Code references require symbol and role context and belong in the `code` array.
- **No special shareability protocol**: Git repositories and path directories are the default transport mechanism; no custom sync protocol is defined.
- **No `collection` concept**: Standard subdirectories are sufficient to group documents; collection is excluded to avoid naming ambiguity.
- **No dedicated `specs` profile**: Specification documents should use `note`, `decision`, or `guide` profiles based on their intent.
