---
description: "How non-Markdown resources and source code bindings are mapped in ODS."
ods:
  profile: "note"
  status: "stable"
  depends:
    - keys.md
  related:
    - context.md
    - validation.md
---

# ODS · Assets

**Assets** are attachments on a document: non-Markdown **resources** and source **code** bindings. They map “what and why” in Markdown to “where and how” on disk.

Keys: `ods.resources`, `ods.code` — see [keys.md](keys.md).

---

## 1. Resources

A resource is any non-Markdown file. ODS describes it without replacing native formats (CSV, OpenAPI, images, PDFs stay as-is).

```yaml
ods:
  resources:
    - path: ../resources/report.pdf
    - path: ../resources/user-flow.jpg
    - path: ../resources/users-sample.csv
```

- Each entry MUST include `path` relative to the document. Format is implied by extension; no separate `type` field is required.
- Level-3: paths MUST resolve to real files.
- Do not put implementation source under `resources`—use `code`.

---

## 2. Code references

`ods.code` maps a document to source, infra, tests, schemas, and automation that implement it. Code files are **not** ODS documents: no frontmatter requirement, not graph nodes, not index children. The Markdown document remains the mapping source of truth.

```yaml
ods:
  code:
    - path: apps/web/src/routes/checkout.tsx
      symbol: CheckoutRoute
      role: entrypoint
    - path: apps/web/src/features/checkout/pricing.ts
      symbol:
        - calculateCheckoutTotal
        - applyDiscountCode
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

Each entry MUST include:

- **`path`** — relative file path. Level-3: MUST exist. **Line suffixes (`:L45`) are prohibited.**
- **`role`** — one of the fixed roles below. Unknown roles MUST be errors; projects MUST NOT invent custom roles.
- **`symbol`** (optional) — string or list of symbols. Prefer symbols over line numbers so refactors do not stale metadata.

| Role | Meaning |
| :--- | :--- |
| `entrypoint` | Where execution, user flow, API flow, CLI flow, or route begins. |
| `implementation` | Main behavior or domain logic. |
| `test` | Tests and fixtures. |
| `schema` | Contracts, types, OpenAPI/GraphQL/Zod/protobuf, table models. |
| `migration` | Persistent state changes (SQL, Prisma, Diesel, …). |
| `config` | Runtime/build settings, flags, framework config. |
| `infrastructure` | Terraform, Pulumi, CDK, Kubernetes, cloud resources. |
| `pipeline` | CI/CD and release automation. |

`code` is separate from `resources` because source often needs symbol precision and role-based selection.

Tools implementing `ods context` SHOULD include declared code paths. Rename tooling (`ods mv`, `ods sync`, watch) SHOULD update `code[].path` when files move. Tools MUST NOT silently rewrite `symbol` without language-aware proof.
