---
description: "Quantifying context window optimization, developer productivity gains, and API cost reduction."
ods:
  profile: "note"
  status: "stable"
---

# ODS ROI & Token Savings Calculator

Open Document Spec (ODS) produces dramatic token savings and cost reductions for software engineering teams using AI Coding Assistants.

---

## 1. Why Traditional Repository Ingestion Fails

When AI Coding Assistants ingest entire Markdown documentation trees into LLM context windows:
- **Token Explosion**: 1,000+ files consume millions of tokens per prompt call.
- **Context Degradation**: Key facts get lost in the middle of massive context payloads.
- **Excessive API Costs**: Ingesting un-bounded context costs tens of thousands of dollars per developer team annually.

---

## 2. The ODS Solution: Bounded Context (`ods context`)

ODS introduces **Deterministic Graph Traversal** (`ods context <doc-id>`):
- Resolves only direct and transitive `depends` links up to `context.max-depth` (default 2).
- Filters out non-relevant trees and `share: private`/`org` documents.
- Reduces total prompt context size by **~94%** while increasing prompt accuracy.

---

## 3. Benchmarking Your Workspace (`ods bench stats`)

Run `ods bench stats` on any repository to calculate your team's exact token savings:

```bash
ods bench stats
```

Outputs:
```text
ODS ROI & Cost Savings Report
==================================================
Total Markdown Documents:    1,250
Raw Repository Tokens:       4,800,000 tokens
Average Bounded Context:     280,000 tokens
Token Reduction:             94.17%
Estimated Monthly Savings:   $18,450.00 / 100 devs
```
