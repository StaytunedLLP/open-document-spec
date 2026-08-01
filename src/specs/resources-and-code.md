---
description: "How non-Markdown files (resources) and source code implementations (code references) are mapped in ODS."
status: "stable"
order: 5
ods:
  profile: "note"
  status: "stable"
---

# ODS Resources and Code References

This document defines how non-Markdown files (resources) and source code implementations (code references) are mapped in ODS.

---

## 1. Resources

A resource refers to any non-Markdown file. ODS aims to describe and document these resources without replacing their native formats (e.g., CSV, OpenAPI specifications, images, and PDFs remain unchanged).

A document serving as the canonical definition of a resource lists it inside `ods.resources` in its frontmatter:

```yaml
ods:
  resources:
    - path: ../resources/report.pdf
    - path: ../resources/user-flow.jpg
    - path: ../resources/users-sample.csv
```

- Each entry MUST include `path`: a file path relative to the document. Format is implied by the native file extension (or content type of the file on disk); tools MAY derive a format hint from the extension when needed. ODS does not require a separate `type` field on resource entries.
- In Level-3 workspaces, resource paths MUST resolve to actual files on the filesystem.

---

## 2. Code References

The `ods.code` field maps an ODS document to the source files, infrastructure files, tests, schemas, and automation that implement or operationalize the document. Code references exist to help humans and AI agents move from "what and why" in Markdown to "where and how" in the repository without broad searches.

Code files are not ODS documents. They do not join the document graph, do not require frontmatter, and MUST NOT be indexed as Markdown children. The Markdown document remains the canonical source of truth for the mapping.

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
        - PricingSchema
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

- `path`: a file path relative to the document. This answers "where is the relevant code?" In Level-3 workspaces the path MUST resolve to an actual file. **Line number suffixes (e.g., `:L45`) are strictly prohibited** in `path` strings (to prevent stale metadata rot on code shifts).
- `role`: one of the fixed standard roles below. This answers "why should an agent load this file?" Unknown roles MUST be errors; projects MUST NOT define custom code roles.
- `symbol` (Optional): Implementation symbol name(s). Accepts a single string OR a multi-line YAML array up to $N$ symbols. Symbol resolution is resilient to line shifts during refactoring.

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

Tools that implement `ods ods context` SHOULD include declared code paths for the loaded document context. Tools that implement rename automation (`ods ods mv`, `ods sync`, `ods ods watch`, or equivalent) SHOULD update `code` item `path` when a referenced file or containing folder moves. Tools MUST NOT silently rewrite `symbol` values unless they have language-aware proof of a symbol rename.
