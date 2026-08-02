---
description: "AI agent context scope loading configurations and visibility filtering rules."
status: "stable"
order: 3
ods:
  profile: "note"
  status: "stable"
---

# ODS Context Specification

This document defines AI agent context scope loading configurations and visibility filtering rules.

---

## 1. Context: deterministic AI loading

The `ods.context` block allows authors to define a precise reading list for AI agents working on the document's topic, replacing expensive repository-wide searches with a bounded, predictable context scope.

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

Semantics:

- **`load`**: A list of Document `.md` paths or resource file paths that MUST be loaded alongside this document.
- **`ignore`**: Path prefixes that tooling and AI agents MUST skip when expanding context.
- **`max-depth`**: Maximum number of graph hops to traverse along `depends` edges.
- **`share` Filtering**: Tooling MUST skip target nodes where `ods.share: private` is set to prevent sensitive internal notes from leaking into context exports.
