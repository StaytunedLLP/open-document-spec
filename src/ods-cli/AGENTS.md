# AGENTS.md — `src/ods-cli`

- Binary name: **`ods`**
- ODS default commands; extra engines via `--okf` / `--skills` only
- Reject `--ods` and legacy namespaces in help / arg parsing
- LSP: `ods lsp`; service: `ods serve` / `ods start` (not LSP)
- When adding a command: tests under `tests/`, help strings, CHANGELOG, skill matrix if user-facing
- **Errors:** use catalog builders from `ods_core::error` / re-exports + `fail_msg` / `usage_msg` / `fail_load` — never invent long stderr prose in the command file

## Argv / subcommands

- Layout is `include!` of `main/cli/*` + `main/commands/*` (not clap derive)
- For `ods <cmd> <sub> <arg>`, after dispatch on `<sub>`, the user arg is typically **index 3**
  - Correct: `ods profile init rfc` → name `rfc`
  - Bug class: treating `init` itself as the name
- Prefer `ods schema` from `ods_core::generate_ods_json_schema()` — do not re-hardcode key JSON

## Gates

```bash
SKIP_RELEASE_BUILD=true ./src/scripts/check-local.sh
./src/scripts/check-odc-residue.sh
```
