# CLI surface (locked)

- Binary: `ods`
- ODS is default — bare `ods <cmd>` (there is **no** `--ods` flag)
- Extra specs: `--okf`, `--skills` only
- Never use namespaces `ods okf` / `ods ods` / `ods skills`
- Editors: `ods lsp` (JSON-RPC). Watcher: `ods serve` / `ods start` (not LSP)
- Do not invent `odc:` or `ods-cli:` frontmatter keys
- Subcommand args: `ods <cmd> <sub> <name…>` — name is **after** the sub verb (e.g. `ods profile init rfc` → name at argv[3])
- Legacy name `odc`: do not reintroduce product teaching or fixture pins; run `./src/scripts/check-odc-residue.sh`
