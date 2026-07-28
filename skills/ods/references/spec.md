---
profile: note
status: draft
---

# ODS Specification

**Open Document Spec - Working Draft 1**

The key words **MUST**, **MUST NOT**, **SHOULD**, **SHOULD NOT**, and **MAY** are to be interpreted as described in RFC 2119.

Design principles, in priority order:

1. **Human first**: readable in any text editor, forever.
2. **Free at level zero**: standard Markdown is already fully conformant; adopting ODS means enriching existing documents with metadata, never migrating or rewriting them.
3. **Token efficient (DRY)**: every fact has exactly one canonical location to minimize redundancy and optimize AI context consumption.
4. **Graph native**: relationships between documents are declared explicitly as frontmatter metadata, rather than inferred from text links or prose.
5. **Trust from validation**: metadata is only as reliable as the checks that enforce it; the specification does not require any rules that cannot be automatically validated or linted.

---

## 1. Core model

ODS defines four core concepts:

| Concept | Description |
| :--- | :--- |
| **Workspace** | The top-level repository unit; a directory tree of documents marked by a root `index.md`. |
| **Document** | A Markdown file (`.md`), optionally containing YAML frontmatter. |
| **Index** | A generated `index.md` file listing a directory's immediate children. |
| **Resource** | Any non-Markdown file referenced by a document (e.g., CSV, PNG, OpenAPI spec, PDF). |

There are no other core concepts. A workspace may contain any number of subdirectories and files. Concepts like bundles, shared folders, archives, and local groups are represented as ordinary directories—the filesystem tree naturally expresses these relationships. `collection` is not a core concept; if a workspace uses the term, it must define distinct, documented semantics outside of this specification.

## 2. Documents and conformance levels

Each conformance level described below is fully valid under ODS.

### Level 0 - plain Markdown

Any `.md` file, with or without frontmatter, is a valid ODS document. Tools MUST accept frontmatter-less documents and MUST NOT require any fields upon document creation.

### Level 1 - typed

A Level-1 document introduces identity frontmatter (such as `profile` and `status`):

```yaml
---
profile: guide
status: draft
---
```

### Level 2 - linked

A Level-2 document establishes graph relationships (see [The Graph](#4-the-graph-depends-and-related)) and resides within an indexed workspace (see [Indexes](#5-indexes-progressive-disclosure)).

### Level 3 - validated

A Level-3 workspace enforces structural integrity and lint rules both locally and in Continuous Integration (CI). These checks ensure there are no dangling references, no duplicate IDs, all profiles resolve correctly, and indexes remain up to date. Documents marked as `status: stable` or listed in an index SHOULD be held to Level-3 validation ("creation is free; promotion has rules").

## 3. Frontmatter

Frontmatter is a single YAML block delimited by `---` at the very beginning of the document. **All fields are optional.** To allow for custom extensions, tools MUST ignore unknown fields.

| Field | Type | Meaning |
| :--- | :--- | :--- |
| `profile` | string | The category of the document; see [Profiles and Catalogs](#9-profiles-and-catalogs). Defaults to `note`. There is no separate document `type` field. |
| `status` | string | Lifecycle state: `draft`, `stable`, `deprecated`, or `archived`. Defaults to `draft`. |
| `id` | string | An explicit identifier to override the path-derived default; see [Identity: IDs are Paths](#31-identity-ids-are-paths). Typically omitted. |
| `description` | string | A brief, single-line summary of the document. Used to populate index files. |
| `depends` | list of refs | List of IDs representing documents that are prerequisites to understanding or implementing this document; see [The Graph](#4-the-graph-depends-and-related). |
| `related` | list of refs | List of IDs for associated documents; see [The Graph](#4-the-graph-depends-and-related). |
| `resources` | list | List of non-Markdown files referenced or described by this document; see [Resources](#6-resources). |
| `code` | list | List of source or code-adjacent files that implement or operationalize this document; see [Code References](#7-code-references). |
| `context` | map | Rules defining scope and boundaries for AI agents loading context; see [Context](#8-context-deterministic-ai-loading). |
| `owner` | string | The individual or team responsible for maintaining the document. |
| `tags` | list of strings | Free-form strings for searching and grouping; never used to define structure. Tools SHOULD normalize tags to lowercase on ingest. Tools MAY ship a small built-in suggestion set for completions; project tags are the observed set of values used in the workspace. Tags are never required and unknown tags MUST NOT be errors. |
| `ods` | string | Root `index.md` only: the ODS spec/schema compatibility version. Individual documents SHOULD omit it. |
| `ods-cli` | string | Root `index.md` only: the compatible ODS CLI version or minimum version range. Individual documents SHOULD omit it. |
| `profiles` | list of paths | Root `index.md` only: catalog roots for custom profile definitions. |
| `ignore` | list of path prefixes | Root `index.md` only: workspace-relative trees excluded from document scan and generated indexes (for example `src`). Not the same as document `context.ignore`. |
| `aliases` | map | Root `index.md` only: workspace section heading aliases for profiles. |

Recommended default key order for ordinary documents: **`profile`**, **`status`**, **`id`**, **`description`**, then relationships and the rest. Tools that draft frontmatter SHOULD write `profile` and `status` first; `id` is usually omitted (path-derived).

All enum values (such as `status`) MUST be lowercase. Tools SHOULD normalize inputs to lowercase instead of rejecting them.

### Recommended Field Order & Complete Example

For human-readability, frontmatter fields SHOULD be organized in a logical sequence from core metadata to relationship edges and AI context:

1. **Classification & Lifecycle**: `profile` and `status` define what the document is and how current it is.
2. **Identification & Description**: `id`, `description`, `owner`, and `tags` provide descriptive metadata.
3. **Graph Relationships**: `depends` and `related` list connection edges.
4. **Physical Assets**: `resources` lists attached files.
5. **Code References**: `code` maps documents to implementation files and symbols.
6. **AI Rules**: `context` sets retrieval boundaries.

Here is a complete frontmatter example showing all supported keys in this sequence:

```yaml
---
profile: guide
status: stable

id: support/refund-guide
description: How to process customer refunds in the dashboard.
owner: support-team
tags:
  - customer-care
  - billing

depends:
  - website/checkout
related:
  - policies/returns

resources:
  - path: docs/refund-flow.pdf

code:
  - path: apps/web/src/routes/refund.tsx
    symbol: RefundRoute
    role: entrypoint
  - path: apps/web/src/features/refunds/process.ts
    symbol: processRefund
    role: implementation
  - path: apps/web/src/features/refunds/process.test.ts
    symbol: processRefund
    role: test

context:
  max-depth: 2
  load:
    - website/checkout
  ignore:
    - marketing/
---
```

### 3.1 Identity: IDs are paths

A document's ID **is its workspace-relative path, without the `.md` extension**, using `/` separators:

```
features/login.md        ->  features/login
guides/setup/oauth.md    ->  guides/setup/oauth
```

Instead of manually generating IDs, the filesystem path implicitly defines the document's identity.

Rules:

- IDs MUST be unique within a workspace. This uniqueness is guaranteed automatically when using path-derived IDs.
- An explicit `id:` field overrides the path-derived ID. Its primary purpose is **stability across renames**: when a document needs to be moved or renamed, you can preserve the existing ID using the `id:` field to avoid breaking external references, or use tooling (e.g., `ods mv`) to update references.
- References to non-existent IDs are considered dangling references, which trigger lint errors in Level-3 workspaces.
- IDs are case-insensitive and normalized to lowercase alphanumeric characters, hyphens, and slashes (`a-z`, `0-9`, `-`, `/`). Tools MUST resolve all references case-insensitively and normalize them to lowercase.
- Path separators MUST be normalized to forward slashes (`/`) across all operating systems.

### 3.2 Single Source of Truth (Token Efficiency Rules)

- A document's title MUST only exist once: as the first `# H1` header in the markdown body. Frontmatter MUST NOT define a duplicate title field.
- Machine-readable relationships MUST reside exclusively in the frontmatter. The document body MUST NOT contain redundant lists of dependencies or related links. Prose within the body may explain the *why* behind a relationship, but the edge declaration belongs solely in the frontmatter.
- Metadata fields (such as `status` and `owner`) exist only in the frontmatter and MUST NOT be restated in the document body.
- Information should have a single canonical source. Instead of duplicating knowledge across documents, reference or link to the authoritative document.

## 4. The graph: `depends` and `related`

ODS defines exactly **two** edge types. Richer vocabularies (e.g., `implements`, `extends`, and `replaces`) are deliberately excluded from the core; use `related` until a future extension standardizes them from observed need.

```yaml
depends:
  - auth/sessions # ID = path without .md
  - api/tokens
related:
  - guides/login-setup
```

Semantics:

- **`depends`**: Indicates that this document cannot be fully understood, implemented, or acted upon without first reading the target document. This is a directional relationship. When an AI parses this document, it SHOULD load its dependencies transitively (up to `context.max-depth` if specified).
- **`related`**: Points to content that serves as useful reference or further reading. This is a non-binding relationship; an AI agent MAY load it optionally.

Rules:

- All references in `depends` or `related` MUST use document IDs (see [Identity: IDs are Paths](#31-identity-ids-are-paths)). Tools MUST attempt to resolve references against explicit `id:` overrides before falling back to path-derived IDs.
- The dependency graph MUST NOT contain cyclic dependencies in Level-3 workspaces (resulting in a validation error).
- Relationships are declared only on the dependent side. Backlinks and reverse lookups ("Which documents depend on this one?") are computed dynamically by tooling; they MUST NOT be written manually, as hardcoded backlinks are prone to becoming outdated.

## 5. Indexes: progressive disclosure

Index files (`index.md`) provide a clear, hierarchical navigation path for both humans and AI: start at the root index, descend one directory level at a time, and retrieve only the documents along the path to the target.

### 5.1 Placement and scope

- An `index.md` in the root directory designates the boundary of a workspace. This file serves as the sole workspace marker; there is no separate manifest file.
- Any directory MAY have its own `index.md`.
- This structure applies recursively: any subdirectory within the workspace can hold folders, files, and its own local `index.md` to guide navigation.
- An `index.md` file MUST only list the immediate children (documents and subdirectories) of its own directory. It MUST NOT recursively list deeper descendants; nested navigation is handled by child `index.md` files.

### 5.2 Content

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
ods-cli: ">=0.0.1"
---
```

An ODS-compliant workspace is identified by this root `ods` field or by entry in the global workspace registry (`~/.ods/odsconfig.toml`). `ods` changes only when the ODS schema semantics change. `ods-cli` records the CLI compatibility requirement, such as the minimum CLI version that understands the workspace. If the root index lacks either field and the path is not in the global registry, workspace commands MUST treat the workspace as unversioned or non-compliant and exit without modifying files. `ods setup` is the repair path: it checks release freshness first, then creates or updates `ods` to the current spec version and `ods-cli` to the running CLI requirement before service and doctor checks.

Ordinary documents MUST NOT carry `ods:` or `ods-cli:`. Nested navigation indexes SHOULD omit both when they are part of the same workspace. A nested `index.md` MAY carry both only when that directory is intentionally usable as its own workspace root, such as an example workspace or fixture.

## 6. Resources

A resource refers to any non-Markdown file. ODS aims to describe and document these resources without replacing their native formats (e.g., CSV, OpenAPI specifications, images, and PDFs remain unchanged).

A document serving as the canonical definition of a resource lists it in its frontmatter:

```yaml
resources:
  - path: ../resources/report.pdf
  - path: ../resources/user-flow.jpg
  - path: ../resources/users-sample.csv
```

- Each entry MUST include `path`: a file path relative to the document. Format is implied by the native file extension (or content type of the file on disk); tools MAY derive a format hint from the extension when needed. ODS does not require a separate `type` field on resource entries.
- In Level-3 workspaces, resource paths MUST resolve to actual files on the filesystem.

## 7. Code References

The `code` field maps an ODS document to the source files, infrastructure files, tests, schemas, and automation that implement or operationalize the document. Code references exist to help humans and AI agents move from "what and why" in Markdown to "where and how" in the repository without broad searches.

Code files are not ODS documents. They do not join the document graph, do not require frontmatter, and MUST NOT be indexed as Markdown children. The Markdown document remains the canonical source of truth for the mapping.

```yaml
code:
  - path: apps/web/src/routes/checkout.tsx
    symbol: CheckoutRoute
    role: entrypoint
  - path: apps/web/src/features/checkout/pricing.ts
    symbol: calculateCheckoutTotal
    role: implementation
  - path: apps/web/src/features/checkout/pricing.test.ts
    symbol: calculateCheckoutTotal
    role: test
  - path: apps/web/src/flags.ts
    symbol: checkoutV2Enabled
    role: config
  - path: infra/terraform/checkout_queue.tf
    role: infrastructure
  - path: .github/workflows/deploy.yml
    role: pipeline
```

Each `code` entry MUST include:

- `path`: a file path relative to the document. This answers "where is the relevant code?" In Level-3 workspaces the path MUST resolve to an actual file.
- `role`: one of the fixed standard roles below. This answers "why should an agent load this file?" Unknown roles MUST be errors; projects MUST NOT define custom code roles.

Each `code` entry MAY include:

- `symbol`: a function, type, component, module, command, schema name, or other implementation symbol within the file. This answers "what exact implementation unit matters?" Symbol validation is optional tooling behavior; path validation is the portable conformance rule.

Standard code roles:

| Role | Meaning |
| :--- | :--- |
| `entrypoint` | Where execution, user flow, API flow, CLI flow, or route begins. |
| `implementation` | Main behavior, domain logic, or implementation module. |
| `test` | Verification code, fixtures, unit tests, integration tests, or end-to-end tests. |
| `schema` | Contracts, data shapes, types, OpenAPI/GraphQL/Zod/protobuf definitions, or table models. |
| `migration` | Persistent state changes such as SQL, Prisma, Diesel, or data migrations. |
| `config` | Runtime/build settings, environment wiring, feature flags, or framework configuration. |
| `infrastructure` | Cloud/runtime infrastructure definitions such as Terraform, Pulumi, CDK, Kubernetes, or deployment resources. |
| `pipeline` | CI/CD, release, build, and deployment automation such as GitHub Actions or release scripts. |

`code` is separate from `resources` because source code often needs symbol-level precision and role-based context selection. `resources` remains a path-only mechanism for attached artifacts whose format is implied by extension.

Tools that implement `ods context` SHOULD include declared code paths for the loaded document context. Tools that implement rename automation (`ods mv`, `ods sync`, `ods watch`, or equivalent) SHOULD update `code[].path` when a referenced file or containing folder moves. Tools MUST NOT silently rewrite `symbol` values unless they have language-aware proof of a symbol rename.

## 8. Context: deterministic AI loading

The `context` block allows authors to define a precise reading list for AI agents working on the document's topic, replacing expensive repository-wide searches with a bounded, predictable context scope.

```yaml
context:
  load:
    - auth/sessions # documents, by ID
    - ../resources/users-sample.csv # resources, by relative path
  ignore:
    - archive/
  max-depth: 2 # bound for transitive `depends` resolution
```

Semantics:

- **`load`**: A list of document IDs or resource paths that MUST be loaded alongside this document and its dependencies.
- **`ignore`**: Path prefixes (workspace-relative or document-relative) that tooling and AI agents MUST skip when expanding `depends` / `load` from this document. Matching is by path prefix (directory or document id path).
- **`max-depth`**: The maximum number of hops to traverse when transitively loading dependencies (default is tool-dependent, but a depth of 2 is recommended).

`context` is optional at every level. When present, an ODS-aware agent SHOULD treat it as the authoritative reading list for the topic. Tools that implement `ods context` MUST honor `ignore` when resolving the reading list.

Detailed guidelines on how tooling and agents should interpret the `context` field can be found in [04-tooling.md](../docs/guide/04-tooling.md).

## 9. Profiles and catalogs

Profiles define the nature of a document (e.g., a guide, decision record, or system architecture) and specify the H2 section headings tools should expect. They act as validation schemas, not copy-paste templates.

ODS uses the same **default vs project** layering for profiles as for free-form `tags` (search facets), with different enforcement:

| Layer | Profiles | Tags |
| :--- | :--- | :--- |
| **Default ODS** | Standard profiles built into the specification (always available) | Optional built-in suggestion set for tools (never required) |
| **Project** | Catalog Markdown under `profiles:` / `ods-profiles/` | Observed `tags:` values on documents |
| Unknown values | SHOULD warn; treat as `note` for section checks | MUST NOT error |

Rules:

- The `profile` field is optional. If omitted, the document defaults to the `note` profile.
- **Default ODS (standard) profiles** are built into the specification and are always available.
- **Project profiles:** workspaces can declare custom profile catalogs in the root `index.md` frontmatter:

```yaml
---
profile: index
ods: 0.1
ods-cli: ">=0.0.1"
profiles:
  - ods-profiles
  - docs/profiles
---
```

- If no custom profiles are listed in the root `index.md`, tools SHOULD look for a local `ods-profiles/` directory in the workspace root by default.
- Profile definitions SHOULD resolve in the following order of precedence: default ODS (standard) profiles, catalog roots listed in the root index, and finally the fallback `ods-profiles/` directory.
- If a profile name is defined in multiple catalogs, tools SHOULD use the first definition loaded and issue a conflict warning. Level-3 workspaces SHOULD treat duplicate profile definitions as validation errors.
- Profile catalogs are workspace-local utilities. They do not participate in the document graph and SHOULD be excluded from ordinary index listings.
- Each profile is defined in a Markdown file. The profile's identifier is derived from the file name (excluding the `.md` extension). If a catalog contains an `index.md` file, the profile name is taken from its parent directory name.
- Second-level headings (`## H2`) in a profile document declare the expected sections. A heading can define multiple alternative names separated by pipes, for example: `## Goal | Objective | Purpose`.
- A profile catalog file can also specify section-level aliases in its frontmatter using an `aliases:` list.
- Encountering an unrecognized profile SHOULD produce a warning, and tools SHOULD treat the document as a `note` for validation purposes until a definition is provided.
- Custom profiles are purely additive; they define structural requirements for specific document types without altering standard profiles or establishing complex inheritance hierarchies.

Default ODS (standard) profiles:

| Profile | Meaning | Expected sections |
| :--- | :--- | :--- |
| `note` | Free-form prose with no fixed structure. | none |
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

Aliases are normalized, not rejected:

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

## 10. Validation

Conformance for metadata is defined by **validation, not intention**. The normative lint rules:

| Rule | Level | Severity |
| :--- | :---: | :---: |
| Frontmatter parses as YAML | 1+ | error |
| `status` is one of `draft`, `stable`, `deprecated`, or `archived` | 1+ | error |
| `profile` resolves to a known profile | 1+ | warning |
| No dangling `depends`/`related` refs | 3 | error |
| No duplicate IDs | 3 | error |
| No `depends` cycles | 3 | error |
| Resource and `context` paths exist | 3 | error |
| `code[].path` exists and `code[].role` is a standard role | 1+ role / 3 path | error |
| Index lists exactly its directory's current children | 3 | error |
| Markdown links in the document body do not dangle | 3 | error |
| Profile's expected sections present (see [Profiles and Catalogs](#9-profiles-and-catalogs)) | 3 | warning |

Tools SHOULD support selecting lint level (for example `--level 1` vs `--level 3`) so teams can adopt gradually: Level 1 checks frontmatter shape; Level 3 checks graph integrity, indexes, resources, and body links.

Tools MUST normalize enum-like field values (`status`, `profile`) to lowercase on ingest rather than rejecting mixed case when the lowercased form is valid.

A workspace claims Level 3 compliance by enforcing these validation rules in CI. To maintain spec integrity, stable documents that fail Level 3 validation checks SHOULD be demoted to draft status until the violations are resolved.

## 11. Tooling workspace scan (ignore defaults)

Tools that walk a workspace MUST NOT treat the following directory names as documentation content (case-insensitive base names): `target`, `node_modules`, `dist`, `build`, `.artifacts`, `.git`, `.hg`, `.svn`, `.jj`, `__pycache__`, `.venv`, `venv`, `vendor`, and any name that starts with `.`.

In addition, the **root** `index.md` MAY declare workspace-level excludes:

```yaml
---
profile: index
ods: 0.1
ods-cli: ">=0.0.1"
ignore:
  - src
  - apps/web
---
```

Each `ignore` entry is a **workspace-relative path prefix** (with or without a trailing `/`). Tools MUST NOT collect Markdown under those prefixes, MUST NOT require `index.md` files there, and MUST NOT list ignored trees as children of parent indexes. Implementation code and package READMEs typically live outside the ODS document graph this way.

Tools MAY also respect the repository's existing `.gitignore` when scanning (for build and dependency noise). ODS does **not** define a separate ignore or config file format such as `.odsignore`; workspace policy stays on the root `index.md` (`ods`, `profiles`, `ignore`, `aliases`). Documents remain plain `.md` only.

Index child validation SHOULD consider only top-level Markdown list links (unordered list items with a single link), not every link in prose or tables. Indexes SHOULD be generated by tooling (`ods index` and equivalent LSP structure sync) and treated like lockfiles.

## 12. Adoption tooling contract

Optional tools MAY implement workspace adoption helpers:

- **Report / dry-run**: scan Markdown, report missing frontmatter, unknown profiles, and alias suggestions without writing files.
- **Write mode**: optionally draft minimal frontmatter (`profile`, `status: draft`) for documents that have no frontmatter, inferring `profile` from headings when possible.

Adoption never rewrites prose body content. Plain Markdown without frontmatter remains valid Level 0.

## 13. Compatibility (CLI-versioned workspaces)

Tools that implement tag discovery SHOULD treat the **project tag set** as the observed union of document `tags` values after normalization. A built-in suggestion list, when present, is for completions and documentation only and MUST NOT make documents invalid.

For the `ods: 0.1` core field set (`profile`, `status`, `description`, `id`, `depends`, `related`, `resources`, `code`, `context`, `owner`, `tags`, and root `ods` / `ods-cli` / `profiles` / `ignore` / `aliases`):

- A workspace root `ods:` value SHOULD equal the current ODS spec version.
- A workspace root `ods-cli:` value SHOULD be an exact CLI version or minimum range such as `>=0.0.1`.
- Workspace discovery SHOULD remain tolerant of older `ods:` values so setup can upgrade them in place.
- `ods lint` and `ods doctor` MUST report missing or stale root `ods:` values and missing, invalid, or unsatisfied `ods-cli:` values.
- `ods init` and `ods setup` MUST write the current spec version to `ods:` and the current CLI minimum requirement to `ods-cli:`.
- Unknown frontmatter keys MUST continue to be ignored by core tools.
- Breaking changes to core field meaning require a new `ods` version string on the root index.

## 14. What ODS deliberately does not define

See [non-goals.md](non-goals.md) for the full list of excluded concepts and the reasoning behind each one.
