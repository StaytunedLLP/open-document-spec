---
profile: note
status: draft
---

# ODS Non-Goals

This document collects the things ODS intentionally does not add to the core model, and why.

The goal is not to be negative. The goal is to keep the spec small enough that:

- humans can read it without learning a new framework
- AI can navigate it without guessing hidden structure
- validators can enforce it without complicated heuristics
- adoption stays cheap enough for real repos

If a concept only helps one team, one workflow, or one tool, it belongs in:

- a profile
- an optional extension
- generated tooling
- or plain git / Markdown conventions

## Core rule

Only adopt a concept if it changes graph behavior, validation behavior, or trusted retrieval enough to justify permanent maintenance.

## 1. No new file extension

Not adopting `.od`, `.odf`, `.ods` as a document format, or any other new document extension.

Why:

- Markdown already works.
- A new extension creates migration friction.
- Discovery should come from `index.md`, not a suffix.

Human and AI effect:

- humans keep using the editor they already have
- AI keeps reading plain Markdown instead of format-specific wrappers
- the repository stays easy to clone, diff, and review

## 2. No manifest file in the core

Not adopting `odf.yaml`, `workspace.yaml`, `collection.yaml`, or any mandatory manifest for the core workspace model.

Why:

- The root `index.md` already marks the workspace.
- A manifest duplicates the workspace marker and creates a second source of truth.
- The core should stay readable without extra configuration files.

What stays out for now:

- workspace manifests
- collection manifests
- profile manifests
- context manifests
- lock files for docs structure
- generated schema files as required sources of truth
- `status.yaml`, `context.yaml`, `profile.yaml`, and similar repo-level config layers

Human and AI effect:

- humans have one obvious entry point
- AI does not need to reconcile a manifest with the filesystem
- navigation stays local to the tree the user is already looking at

## 3. No per-document spec version

Not adopting `odf: v0.4` or any per-file workspace version stamp in every document.

Why:

- It creates churn across the repo.
- Most documents do not need that metadata repeated.
- The root `index.md` carries the workspace compatibility version once.

## 4. No `lifecycle` field

Not adopting a separate `lifecycle` field alongside `status`.

Why:

- It becomes a second status system.
- It overlaps with `status`, directory structure, and validation.
- It adds maintenance without adding a new graph capability.

Use `status` for trust and maturity.

## 5. No `updated` timestamp in canonical metadata

Not adopting `updated:` as a core field.

Why:

- Git already tracks the real history.
- Auto-updated timestamps rot quickly.
- Freshness is better computed than hand-maintained.

## 6. No `aliases`, `audience`, or `applies-to` in the core

Not adopting these as universal core metadata keys.

Why:

- They are useful, but not universal.
- They expand the schema before the core model has settled.
- They are better handled by profiles or optional extensions.

Use them only when a workspace has a concrete need and a validator to enforce it.

## 7. No universal `url:` field for external links

Not adopting a generic `url:` frontmatter key as core metadata.

Why:

- Markdown already links externally.
- A second link field adds schema noise.
- External URLs are usually presentation or reference data, not graph structure.

## 8. No dedicated asset bucket

Not adopting a separate `assets:` concept in the core.

Why:

- Assets are already resources.
- A separate asset bucket duplicates `resources:`.
- The core should not reclassify file types unnecessarily.

## 8.1 No typed code schema inside `resources`

Not using `resources` for source-code mappings with `symbol`, `role`, `language`, or `type`.

Why:

- `resources` is intentionally path-only.
- Code references need implementation-specific meaning that ordinary resources do not.
- Overloading `resources` would blur attached artifacts with implementation context.

Use `code` for implementation mappings. Use `resources` for non-Markdown artifacts that a document references or describes.

## 9. No extra relationship vocabulary in the core

Not adopting `implements`, `extends`, `replaces`, `depends-on`, `related-to`, or similar core edge types.

Why:

- More edge types increase tooling cost.
- The core model should stay small and deterministic.
- Additional semantics can be standardized later if real use demands it.

Keep:

- `depends`
- `related`

## 9.1 No source-file ODS frontmatter or required code comments

Not adopting ODS frontmatter blocks inside `.ts`, `.rs`, or other source files. Not requiring inline comments or backlinks such as `ods-related:` in source code.

Why:

- Markdown frontmatter is the canonical metadata layer.
- Source-file comment formats differ by language and are easy to drift from the document graph.
- Required code comments would make adoption depend on touching implementation files.

Optional code anchors may be explored by tooling later, but they are not canonical ODS metadata.

## 9.2 No line numbers as stable code identity

Not adopting line numbers or line ranges as stable canonical code references.

Why:

- Lines move during ordinary edits.
- Symbol and path references survive refactors better.
- Line-range maintenance would require diff tracking and still produce noisy churn.

Use `code[].symbol` when precision beyond `code[].path` is needed.

## 9.3 No project-customizable code roles

Not adopting workspace role catalogs or custom `code[].role` values.

Why:

- A small fixed vocabulary keeps AI context selection portable.
- Custom roles become framework taxonomies and drift across teams.
- The standard roles cover the Pareto set of implementation, tests, schemas, config, infrastructure, and pipeline automation.

Unknown code roles are errors.

## 10. No profile inheritance in the core

Not adopting profile inheritance, section renaming, or section hiding as core mechanisms.

Why:

- They make validation harder.
- They hide structure from both humans and tools.
- They push ODS toward a template engine instead of a simple document model.

If a profile needs a variant, define a new profile rather than building inheritance chains.

## 11. No standardized templates

Not adopting a canonical template system in the spec.

Why:

- Templates are authoring aids, not the specification.
- Standardizing templates freezes formatting too early.
- Profiles should define structure; tools can generate templates locally.

## 12. No closed tag registry or tag catalog files

Not adopting a required workspace tag allowlist, `tag_policy`, `ods-tags/` catalog trees, or closed-mode unknown-tag errors.

Why:

- `tags` are free-form facets for search/grouping, not structure (see SPEC §3).
- Closed vocabularies create adoption friction and synonym bureaucracy.
- Project tags are observed from document frontmatter; tools may complete from that set plus soft built-in suggestions.
- Org-specific closed lists belong in optional CI scripts, not the core format.

## 13. No mandatory enterprise config file

Not adopting `od.config.toml`, dedicated ignore files for ODS, or any similar required repo config file for custom extension keys.

Why:

- It adds another mandatory file to learn.
- It competes with `index.md` as a workspace entry point.
- Unknown frontmatter keys already provide safe extension space.

If an enterprise wants stricter extension validation, that belongs in optional tooling or CI rules.

## 14. No built-in enterprise namespace registry

Not adopting a core registry for `jira`, `github`, `ticket`, or other enterprise fields.

Why:

- Enterprise workflows differ too much.
- Hardcoding them into the spec makes the core too opinionated.
- Namespaced extension keys are enough.

## 15. No shared-workspace registry

Not adopting a separate registry or marker for shared workspaces.

Why:

- A shared workspace is a structural convention, not a new primitive.
- The tree, links, and git composition already express sharing.
- A registry would create yet another place for drift.

## 16. No special shareability protocol

Not adopting a new core protocol for sharing, publishing, or importing workspaces.

Why:

- Git paths and repo URLs already make workspaces portable.
- The core should remain repository-native.
- Distribution can be layered on later if there is real demand.

## 17. No `collection` as a core concept unless it has unique semantics

Not adopting `collection` as a duplicate name for a folder of docs.

Why:

- If it means the same thing as workspace or directory, it is redundant.
- A second hierarchy without distinct semantics confuses navigation.
- The core should not carry two names for the same organizing idea.

If `collection` is ever kept, it must mean something measurably different from `workspace`.

## 18. No separate type taxonomy from `profile`

Not adopting a `type` layer that sits beside or above `profile`.

Why:

- It splits document identity across multiple systems.
- `profile` already names document kind.
- A parallel taxonomy increases ambiguity and validation cost.

## 19. No required `llms.txt`

Not adopting `llms.txt` as canonical source material.

Why:

- It should be generated from the root index.
- The root `index.md` is the source of truth.
- Generated derivatives should not become another maintained contract.

## 20. No lock file for document structure

Not adopting a lock file for indexes, context, or workspace layout.

Why:

- Those are derivable from the workspace itself.
- A lock file adds churn and maintenance.
- The core already prefers generated over hand-maintained for derived artifacts.

Also not adopting index-level derived stats such as:

- document counts
- coverage percentages
- completeness scores
- other numbers that rot on every commit

## 21. No mandatory profile version on every document

Not adopting per-document profile version stamping in the core.

Why:

- It makes every file carry compatibility noise.
- Profile version belongs with the profile definition, not every instance.
- The document should stay focused on its content and role.

If profile versioning is needed, keep it at the profile definition layer.

## 22. No template engine in the core

Not adopting section rendering, content generation, or template engines as normative features.

Why:

- They are useful authoring tools, but they are not the specification itself.
- A template engine would make the core harder to read and validate.
- ODS should stay a document model, not a site generator.

## 23. Summary

The core spec should stay small and stable.

Keep:

- `index.md` as the workspace marker
- `status` as the maturity signal
- `profile` as the document kind
- `depends` and `related` as the graph
- `resources` for assets and other non-Markdown artifacts

Do not add:

- new file extensions
- manifest files
- `lifecycle`
- `updated`
- extra edge vocabularies
- profile inheritance
- template engines
- derived statistics in indexes

The reason is simple: the fewer moving parts the spec defines, the easier it is for both humans and AI to use it consistently.

## 23. No dedicated `specs` profile in the core

Not adopting a standard `specs` profile for specification prose.

Why:

- spec documents already fit existing profiles (`note`, `decision`, `guide`)
- a dedicated `specs` profile would duplicate meaning without adding graph or validation value
- custom profiles are sufficient when a workspace truly needs a special spec-writing class

Use the existing profiles unless a workspace-local convention proves that a new one is worth maintaining.
