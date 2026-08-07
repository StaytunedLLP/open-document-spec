---
description: "AI agent context scope: load, ignore, max-depth, and share filtering."
ods:
  profile: "note"
  status: "stable"
  depends:
    - keys.md
    - graph.md
---

# ODS · Context

Deterministic AI loading: authors declare a **bounded reading list** so agents do not need repository-wide search.

Key shape: [keys.md](keys.md) (`ods.context`). Graph hops follow [graph.md](graph.md).

---

## When to use context vs depends

| Need | Use |
| :--- | :--- |
| Structural prerequisite for understanding this doc | `ods.depends` |
| Soft related reading | `ods.related` |
| Extra files to load for an AI task (including non-md) | `ods.context.load` |
| Cap how deep depends chains expand | `ods.context.max-depth` |
| Skip noisy trees during expansion | `ods.context.ignore` |

`depends` describes the **document graph**. `context` describes **what to put in an AI prompt window** for work on this topic.

---

## Semantics

```yaml
---
description: Auth session setup guide.
ods:
  profile: guide
  status: stable
  share: public
  context:
    max-depth: 2
    load:
      - ../auth/sessions.md
      - ../resources/users-sample.csv
    ignore:
      - archive/
      - legacy/
---
```

| Field | Meaning |
| :--- | :--- |
| **`load`** | Document `.md` paths or resource paths that MUST be loaded alongside this document. |
| **`ignore`** | Path prefixes tooling and agents MUST skip when expanding context. |
| **`max-depth`** | Maximum hops along `depends` edges. |
| **Share filter** | Tooling MUST skip targets with `ods.share: private` when assembling context exports (unless the operation explicitly includes private). |

CLI: `ods context <id-or-path>` resolves the bounded list for agents and humans.
