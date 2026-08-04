# CLI surface (locked)

- Binary: `ods`
- ODS is default — bare `ods <cmd>` (there is **no** `--ods` flag)
- Extra specs: `--okf`, `--skills` only
- Never use namespaces `ods okf` / `ods ods` / `ods skills`
- Editors: `ods lsp` (JSON-RPC). Watcher: `ods serve` / `ods start` (not LSP)
- Do not invent `odc:` or `ods-cli:` frontmatter keys
- Subcommand args: `ods <cmd> <sub> <name…>` — name is **after** the sub verb (e.g. `ods profile init rfc` → name at argv[3])
- Before writing CLI tests: confirm usage via `ods <cmd>` error string / help text
- New subcommand: tests for missing arg, wrong order, and one success path
- Legacy name `odc`: do not reintroduce product teaching or fixture pins; run `./src/scripts/check-odc-residue.sh`
## User-facing errors (locked)

- **SoT:** `src/ods-core/src/error/messages.rs`
- First-call stderr: `error: <short>` or `usage: <short>`, then `Next: <command>`
- Do not add new bare `failure(e.to_string())` / free-form multi-paragraph usage dumps
- Prefer catalog builders + CLI `fail_msg` / `usage_msg` / `fail_load` / `fail_io`
- Update/install internal `Result<_, String>` details: `ods_core::error::detail` / `update_*` helpers; CLI boundary uses `update_failed`
- When adding a `UserMsg` builder with a stable id, append it to `CATALOG_MESSAGE_IDS` in `messages.rs`
- Guide catalog: `docs/guide/07-troubleshooting-and-diagnostics.md` must match live strings
