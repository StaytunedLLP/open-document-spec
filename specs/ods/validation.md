---
description: "ODS lint rules, unknown-content behavior, and tooling contract."
ods:
  profile: "note"
  status: "stable"
  depends:
    - keys.md
    - core.md
  related:
    - graph.md
    - indexes.md
    - assets.md
---

# ODS · Validation

Conformance for metadata is defined by **validation, not intention**. Key placement details: [keys.md](keys.md). Levels: [core.md](core.md).

### CLI / diagnostic presentation

The `ods` CLI surfaces failures as short, directive messages (`error:` or `usage:` plus a `Next:` line). Lint diagnostic text is catalogued in the engine (`src/ods-core/src/error/messages.rs`) so tooling and the end-user guide stay aligned. Authors should fix issues listed by `ods lint` (and `.ods/ods-errors.md` when present), then re-run lint.
---

## 1. Lint rules

| Rule | Level | Severity |
| :--- | :---: | :---: |
| Frontmatter parses as valid YAML | 1+ | error |
| Frontmatter MUST NOT contain a `title:` key | 1+ | error |
| `tags` MUST be top-level when present; nested under `ods:` is misplaced | 1+ | warning |
| `ods.status` (or legacy flat `status`) is `draft`, `stable`, `deprecated`, or `archived` | 1+ | error |
| `ods.share` (when present) is `public`, `org`, or `private` | 1+ | error |
| `ods.profile` resolves to a known profile | 1+ | warning |
| `ods.code[].path` MUST NOT contain line number suffixes | 1+ | error |
| No dangling `ods.depends` / `ods.related` refs | 3 | error |
| No duplicate document IDs | 3 | error |
| No `ods.depends` cycles | 3 | error |
| Resource, `context.load`, and root `packs:` paths resolve | 3 | error |
| `ods.code` path exists (3) and `role` is standard (1+) | 1+ / 3 | error |
| ~~Index child lists~~ (removed — no nested indexes) | — | — |
| Body Markdown links do not dangle | 3 | error |
| Profile expected sections present | 3 | warning |

Lint is binary: **compliant** (exit 0) or **non-compliant** (exit 1). There are no Level 1/3 modes.

Tools MUST normalize enum-like values (`status`, `profile`, `share`) to lowercase. For migration, tools MUST fall back to legacy flat engine keys when the nested `ods:` map is absent.

A workspace is **compliant** when `ods lint` reports no errors in CI. Stable documents that fail lint SHOULD be demoted to draft until fixed.

### 1.1 Unknown content

| Scenario | Behavior |
| :--- | :--- |
| Unknown frontmatter key | Preserve; ignore for core validation |
| Unknown `profile` | Warn; treat as `note` for section checks |
| Top-level `tags` (any string) | Accept; lowercase; include in workspace tag set |
| `tags` under `ods:` | SHOULD warn; migrate SHOULD hoist |
| Legacy flat engine keys without nested `ods:` | Accept for migration; format SHOULD nest under `ods:` |
| Unknown `code` role | Error |
| Invalid `share` | Error |
| Dangling depends/related/resources/code/packs | Level-3 error |
| Duplicate document ID | Level-3 error |

### 1.2 Command examples

```bash
ods lint .
ods overview
ods context support/refund-guide.md
ods pack list
```

---

## 2. Adoption tooling contract

Optional tools MAY implement:

- **Report / dry-run**: scan Markdown; report missing frontmatter, unknown profiles, alias suggestions; no writes.
- **Write mode**: draft minimal frontmatter (`profile`, `status: draft`) for docs without frontmatter; MAY infer profile from headings.

Adoption never rewrites body prose. Plain Markdown remains valid without frontmatter.

---

## 3. Compatibility (versioned workspaces)

For key placement summary see [keys.md](keys.md).

- Root `ods:` SHOULD equal the current ODS spec version (e.g. `0.1`).
- Workspace discovery SHOULD tolerate older markers so setup can upgrade in place.
- `ods lint` / `ods doctor` MUST report missing or stale root markers.
- `ods init` / `ods setup` MUST write the current spec version to root `ods:`.
- Unknown frontmatter keys MUST continue to be ignored by core tools.
- Breaking changes to core field meaning require a new root `ods` version string.
