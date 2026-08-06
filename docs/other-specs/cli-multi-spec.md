---
profile: note
status: stable
---

# CLI multi-spec surface (current)

Authoritative product rules for the `ods` binary.

| Rule | Value |
|---|---|
| Default | Bare `ods <cmd>` runs **ODS** (auto-enables `okf`/`skills` if declared in `ods.toml` `[specs.*]`) |
| Declarative root config | `ods.toml` `[specs.okf]` / `[specs.skills]` configures activation (`enabled = true`) and key linting (`check_keys`, `ignore_keys`) |
| Force ODS flag | **Does not exist** (`--ods` is rejected) |
| OKF | `ods <cmd> --okf` |
| Agent Skills | `ods <cmd> --skills` |
| Key suppression | `--skip-frontmatter-keys` / `--ignore-keys key1,key2` |
| Namespaces | **Removed** (`ods okf`, `ods ods` are errors) |
| Machine config | `~/.ods/odsconfig.toml` (legacy `odcconfig.toml` readable if present) |
| Editor LSP | `ods lsp` / `ods setup --editor zed\|vscode\|nvim\|cursor` |
| Crates | `src/ods-core`, `src/ods-cli`, `src/ods-test-support` |
| Error shape | `error:`/`usage:` + `Next:` from `src/ods-core/src/error/messages.rs` |
| Reject `--ods` | `usage: unknown flag: --ods` → bare `ods <cmd>` |
| Not a workspace | `error: not an ODS workspace…` → `ods init` (Hint: `--okf` / `--skills` when markers exist) |

```bash
ods init .
ods lint
ods init --okf ./bundle && ods lint --okf ./bundle
ods init --skills ./my-skill && ods lint --skills ./my-skill
ods setup --editor zed
ods lsp
```

See also: [frontmatter-keys-ods-vs-okf.md](./frontmatter-keys-ods-vs-okf.md), [agentskills.md](./agentskills.md), [../guide/04-tooling.md](../guide/04-tooling.md), normative trees [`specs/ods/`](../../specs/ods/), [`specs/okf/`](../../specs/okf/), [`specs/skills/`](../../specs/skills/).
