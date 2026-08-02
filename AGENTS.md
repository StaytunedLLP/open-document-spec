# AGENTS.md

Rules for coding agents working in this Open Document Spec repository.

## CLI product surface (locked)

- Binary: **`ods`**
- **ODS is default** — bare `ods <cmd>` (there is **no** `--ods` flag)
- Extra specs use flags only:
  - `--okf` — Google OKF v0.2
  - `--skills` — Agent Skills (`SKILL.md` packages)
- Do **not** use namespaces `ods okf` / `ods ods` / `ods skills` (deprecated if present)
- Editors: **`ods lsp`** (JSON-RPC). Background watcher: `ods serve` / `ods start` — not an LSP
- Do not invent `odc:` or `ods-cli:` frontmatter keys

## After document / skill edits

```bash
ods lint
ods lint --okf      # if OKF trees changed
ods lint --skills   # if SKILL.md packages changed
```

## Code style

Prefer functional style in `ods-core` / CLI: data + free functions, pipelines, effects at the edge.
See `docs/maintainer/functional-style.md`.

## Useful commands

| Goal | Command |
|---|---|
| Validate ODS | `ods lint` |
| OKF | `ods lint --okf` / `ods init --okf` |
| Skills | `ods lint --skills` / `ods init --skills` |
| Context | `ods context <id>` |
| Editor LSP | `ods lsp` |
| Install skill into host | `ods skill install --agent <name>` |
| Refresh this file | `ods agents sync` |
