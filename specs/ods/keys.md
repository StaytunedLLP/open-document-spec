---
description: "ODS frontmatter key dictionary: placement rules, purpose, examples, and common mistakes for every key."
ods:
  profile: "note"
  status: "stable"
  depends:
    - intro.md
  related:
    - core.md
    - graph.md
    - assets.md
    - context.md
    - indexes.md
---

# ODS · Keys

Author-facing dictionary of **every ODS frontmatter key**: where it goes, what it means, and a short example.

Normative format rules: [core.md](core.md). Multi-spec comparison (ODS vs OKF): `docs/other-specs/frontmatter-keys-ods-vs-okf.md`.

---

## Author cheat sheet (copy-paste)

```yaml
---
description: One-line summary for indexes and previews.
tags:
  - billing
owner: team:support
ods:
  profile: note
  status: draft
  share: public
  depends:
    - ../auth/sessions.md
  related:
    - ../policy/refunds.md
---

# Document Title Lives Only In The H1
```

Minimal typed document:

```yaml
---
ods:
  profile: note
  status: draft
---
```

---

## 1. Placement rule

| Layer | Placement | Purpose |
| :--- | :--- | :--- |
| **Universal (common)** | **Top-level** only | Any YAML consumer can read it (SSGs, Obsidian, Hugo, Docusaurus, Astro, CMS, agents) |
| **ODS engine** | Nested under **`ods:`** only | Profile, lifecycle, graph, share, assets, context |
| **ods.toml only** | Top-level on **root** `ods.toml` | Workspace boundary, packs, ignore, multi-spec config |

**Why the split:** universal keys stay visible to non-ODS tools; engine keys stay in a private namespace so they do not collide with SSG reserved keys.

### Multi-tool frontmatter interoperability (normative)

ODS enforces a **non-destructive frontmatter policy** so Hugo, Astro, Jekyll, Docusaurus, Next.js, Obsidian, and custom pipelines can coexist with ODS on the same files.

- **Unknown / non-ODS keys** (e.g. Hugo `layout`, Astro `hero_image`, Jekyll `permalink`, custom `author`) MUST be preserved through parse, lint, and CLI mutations (`ods status`, `ods tag`, `ods fmt`, `ods adopt`, `ods fmt --migrate`).
- Mutations MUST change **only** native keys for the active dialect(s): ODS by default; OKF / Skills only when those engines are enabled. They MUST NOT delete, alter values of, or invent foreign keys.
- Surgical mutators (status, tags) preserve relative order of other keys. `ods fmt --migrate` may reorder **engine** keys under `ods:` and normalize blank lines between frontmatter and body; it MUST still re-emit every non-engine top-level block and any unknown nested keys under `ods:`.

### Where does this key go?

| Keys | Level |
| :--- | :--- |
| `description`, `tags`, `owner`, `created`, `updated` | **Top-level (common)** |
| `profile`, `status`, `id`, `share`, `depends`, `related`, `resources`, `code`, `context` | **Under `ods:`** |
| `spec`, `custom_profiles`, `packs`, `ignore`, `aliases`, `specs`, `service` | **`ods.toml` only** |

### Tags placement (normative)

- MUST declare `tags` at the **top level** when present.
- MUST NOT nest `tags` under `ods:`.
- Tools SHOULD warn on nested `tags`; `ods fmt --migrate` SHOULD hoist them without dropping values.

```yaml
# WRONG
ods:
  profile: note
  tags: [billing]

# RIGHT
tags:
  - billing
ods:
  profile: note
  status: draft
```

### Forbidden

| Key / pattern | Why |
| :--- | :--- |
| Frontmatter `title:` | Title is only the first `# H1` in the body (single source of truth). |
| Universal keys under `ods:` | Hides them from non-ODS tools. |
| Engine keys at top level (new docs) | Accepted for migration only; emit under `ods:`. |

Unknown keys MUST be preserved and ignored by core validation.

---

## 2. Universal top-level keys

**Placement: top-level only** (never under `ods:`).

### `description`

| | |
| :--- | :--- |
| **Type** | string |
| **Purpose** | One-line summary. Feeds index listings and SSG/social previews. |
| **Example** | `description: How to process customer refunds.` |
| **Mistake** | Putting multi-paragraph prose here (keep it short). |

### `tags`

| | |
| :--- | :--- |
| **Type** | list of strings |
| **Purpose** | Free-form search facets. Normalized to lowercase. No closed registry. |
| **Example** | `tags: [billing, customer-care]` |
| **Mistake** | Nesting under `ods:`; using status words as tags without care. |

### `owner`

| | |
| :--- | :--- |
| **Type** | string or list of strings |
| **Purpose** | Responsible person or team. |
| **Example** | `owner: team:support` or `owner: [alice, bob]` |
| **Mistake** | Duplicating ownership only in body prose without frontmatter. |

### `created` / `updated`

| | |
| :--- | :--- |
| **Type** | string (`YYYY-MM-DD` or ISO-8601) |
| **Purpose** | Optional timestamps for non-git workflows. `last_updated` is an alias of `updated`. |
| **Example** | `created: 2026-01-15` |
| **Mistake** | Hand-maintaining timestamps that git history already covers. |

---

## 3. Engine keys (nested under `ods:`)

**Placement: under `ods:` only.** Do not put `tags`, `description`, or `owner` here.

### Canonical emit sequence

When scaffolding (`ods new`), adopting (`ods adopt`), or migrating (`ods fmt --migrate`), tools MUST emit keys inside `ods:` in this order:

`profile` → `status` → `id` → `share` → `depends` → `related` → `resources` → `code` → `context`

Parsers MUST NOT error if order differs; only emit tooling enforces sequence.

### `profile`

| | |
| :--- | :--- |
| **Type** | string |
| **Purpose** | Document shape / expected sections. Default: `note`. |
| **Values** | See [profiles.md](profiles.md) (`guide`, `feature`, `decision`, …). |
| **Example** | `profile: note` |
| **Mistake** | Inventing a parallel `type` field. |

### `status`

| | |
| :--- | :--- |
| **Type** | string |
| **Purpose** | Lifecycle maturity. Default: `draft`. |
| **Values** | `draft` \| `stable` \| `deprecated` \| `archived` |
| **Example** | `status: stable` |
| **Mistake** | Adding a second `lifecycle` field. |

### `id`

| | |
| :--- | :--- |
| **Type** | string |
| **Purpose** | Explicit graph ID override for rename stability. Default: workspace-relative path without `.md`. |
| **Example** | `id: docs/support/refund-guide` |
| **Mistake** | Hand-writing IDs when path-derived IDs are enough. See [graph.md](graph.md). |

### `share`

| | |
| :--- | :--- |
| **Type** | string |
| **Purpose** | Visibility for export/context filtering. |
| **Values** | `public` \| `org` \| `private` |
| **Example** | `share: private` |
| **Mistake** | Putting secrets in public docs without `share: private`. |

### `depends`

| | |
| :--- | :--- |
| **Type** | list of document refs (prefer `.md` paths) |
| **Purpose** | Hard prerequisites—reader/agent SHOULD load these first (directional). |
| **Example** | `depends: [../auth/sessions.md]` |
| **Mistake** | Cycles (Level-3 error); putting edges only in body prose. See [graph.md](graph.md). |

### `related`

| | |
| :--- | :--- |
| **Type** | list of document refs |
| **Purpose** | Soft references; optional further reading. |
| **Example** | `related: [../policy/refunds.md]` |
| **Mistake** | Using `related` for hard prerequisites (use `depends`). |

### `resources`

| | |
| :--- | :--- |
| **Type** | list of maps with `path` |
| **Purpose** | Non-Markdown attachments (PDF, CSV, image, OpenAPI file, …). |
| **Example** | `resources: [{ path: ../files/refund-flow.pdf }]` |
| **Mistake** | Putting source code here (use `code`). See [assets.md](assets.md). |

### `code`

| | |
| :--- | :--- |
| **Type** | list of maps: `path` (required), `role` (required), `symbol` (optional string or list) |
| **Purpose** | Bind the document to implementation files. |
| **Roles** | `entrypoint`, `implementation`, `test`, `schema`, `migration`, `config`, `infrastructure`, `pipeline` |
| **Example** | see complete example below |
| **Mistake** | Line numbers in `path` (e.g. `:L45`)—forbidden. |

### `context`

| | |
| :--- | :--- |
| **Type** | map: `max-depth`, `load`, `ignore` |
| **Purpose** | Bounded AI reading list for this document. |
| **Example** | see complete example below |
| **Mistake** | Using context instead of `depends` for structural prerequisites. See [context.md](context.md). |

---

## 4. ods.toml keys only

Appear at the **top level of the workspace ods.toml** (`ods.toml` preferred). Ordinary documents MUST NOT use them as workspace markers.

### `ods` (spec version)

| | |
| :--- | :--- |
| **Type** | string |
| **Purpose** | Workspace boundary + ODS spec version (currently `0.1`). |
| **Example** | `ods: 0.1` |
| **Mistake** | Putting `ods: 0.1` on nested documents. |

> **Note:** Older docs mentioned a second scalar for CLI version constraints. The engine model today treats the root **`ods:`** marker as the workspace/spec version. Preserve unknown keys (including legacy `odc:` constraints in older trees) without inventing a dual `ods:` YAML key pair.

### `custom-profiles`

| | |
| :--- | :--- |
| **Type** | list of paths |
| **Purpose** | Explicit custom profile definition files. Canonical registration key. |
| **Example** | `custom-profiles: [docs/profiles/rfc.md]` |
| **Mistake** | Expecting auto-scan of `ods-profiles/` without listing paths. |

### `profiles` (legacy)

| | |
| :--- | :--- |
| **Type** | list of paths |
| **Purpose** | Legacy profile catalog roots; prefer `custom-profiles`. |
| **Example** | `profiles: [vendor/pack/ods-profiles]` |

### `packs`

| | |
| :--- | :--- |
| **Type** | list of paths or pack refs |
| **Purpose** | Import reusable ODS Packs (profiles, skills, templates). |
| **Example** | `packs: [vendor/engineering-pack]` |

### `ignore`

| | |
| :--- | :--- |
| **Type** | list of path prefixes |
| **Purpose** | Exclude trees from scan/index/lint document graph. |
| **Example** | `ignore: [src, node_modules, app-web]` |

### `aliases`

| | |
| :--- | :--- |
| **Type** | map |
| **Purpose** | Workspace section-heading aliases for profile section matching. |
| **Example** | `aliases: { Goal: [Objective, Purpose] }` |

### `specs`

| | |
| :--- | :--- |
| **Type** | map (`okf`, `skills` → `enabled`, `lint.check_keys`, `lint.ignore_keys`) |
| **Purpose** | Multi-spec auto-activation and key-lint suppression. |
| **Example** | |

```yaml
specs:
  okf:
    enabled: true
    lint:
      check_keys: false
      ignore_keys: [runtime, sources]
  skills:
    enabled: true
```

---

## 5. Complete example

```markdown
---
description: How to process customer refunds in the dashboard.
tags:
  - customer-care
  - billing
owner: team:support
ods:
  profile: note
  status: stable
  share: public
  depends:
    - ../website/cart-checkout.md
    - ../auth/sessions.md
  related:
    - ../products/glow-serum.md
  resources:
    - path: ../files/refund-flow.pdf
  code:
    - path: apps/web/src/routes/refund.tsx
      role: entrypoint
      symbol: RefundRoute
    - path: apps/web/src/features/refunds/process.ts
      role: implementation
      symbol:
        - processRefund
        - validateRefund
  context:
    max-depth: 2
    load:
      - ../website/cart-checkout.md
    ignore:
      - marketing/
---

# Refund Processing

## Overview

Use this guide when processing customer refunds.
```

---

## 6. ods.toml example

```toml
# ods.toml — repository root only (workspace marker)
spec = "0.1"

ignore = ["src", "app-web"]
custom_profiles = ["docs/profiles/rfc.md"]
packs = ["vendor/engineering-pack"]

[specs.okf]
enabled = false

[specs.skills]
enabled = false
```

See [indexes.md](indexes.md) for workspace config fields and progressive CLI discovery (indexes are not generated).

