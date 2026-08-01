---
name: ods
description: >-
  Install, run, update, and author with OpenDocify (`odc`) for ODS workspaces.
  Prefer `odc <cmd>` or `odc ods <cmd>`; legacy binary `ods <cmd>` is equivalent.
  Also documents sibling OKF via `odc okf` and platform `odc update`/`upgrade`.
  Use when working in an ODS workspace (root index.md has `ods:`), OKF bundle
  (`okf_version`), or any odc/ods CLI workflow.

---

# ODS — Open Document Specs

ODS is plain Markdown with **optional** YAML frontmatter, kept consistent by one
Rust binary named `odc` (legacy: `ods`). A **workspace** is any directory tree whose **root
`index.md` carries an `ods:` key** (e.g. `ods: 0.1`) and `odc:` requirement.
The document **graph**
has exactly two edge types — `depends` and `related` — declared in frontmatter.
Nothing here requires a new file format, a manifest, or a config file.

## 1. Setup — install, update, keep the service running

The **Skill-First approach** is the primary, zero-friction method to install and run ODS. AI Coding Assistants automatically trigger the skill's bootstrap script to set up ODS on any machine.

Run the bundled cross-platform bootstrap script instead of manually downloading releases:

- `scripts/bootstrap.sh` (macOS / Linux) or `scripts/bootstrap.ps1` (Windows PowerShell) — default flow: auto-detect OS & architecture → install release binary → verify workspace → register & start background OS service → validate health → print confirmation message (`ODS is installed and running now in your machine!` with `odc --version`).
- `bootstrap.sh install` / `bootstrap.ps1 -Command install` — install latest release binary.
- `bootstrap.sh update` / `bootstrap.ps1 -Command update` — self-update (`odc update`), reinstall on failure.
- `bootstrap.sh ensure <path>` / `bootstrap.ps1 -Command ensure -Path <path>` — guarantee the watch service for a workspace.
- `bootstrap.sh status|doctor <path>` / `bootstrap.ps1 -Command doctor` — version + service state / health checks.
- `odc setup [path]` — check release freshness, detect workspace boundary, ensure background service, and run health doctor.

Requires GitHub CLI auth (while repo is private): `gh auth login` or `GH_TOKEN`.
Unauthenticated downloads fail; installer script fetch and asset downloads both require `GH_TOKEN` / `GITHUB_TOKEN`.
Env knobs: `ODS_PREFIX` (default `~/.local/bin` on Unix, `%LOCALAPPDATA%\Programs\ods` on Windows), `ODS_VERSION` (pin a tag), `ODS_AUTO_UPDATE=0` (disable ~daily auto-update check).

Most commands silently check for and apply updates in the background unless
`ODS_AUTO_UPDATE=0` is set. A mutating `odc update` also **restarts the
background service**, so avoid triggering it mid-task or while a long-running
`odc ods watch`/`odc ods serve` session needs to stay on a specific binary version;
prefer the read-only `odc update --check` when you just need to know if an
update is available.

**The "service" is a background OS user service running `odc ods serve` (systemd
user unit / launchd agent / scheduled task). There is NO network server, port,
or HTTP health check — never invent one.**

## 2. The spec (operational)

Full normative text: references/spec.md and references/non-goals.md.

**Core model:** Workspace (tree marked by root `index.md`), Document (`.md`,
optional frontmatter), Index (generated `index.md`), Resource (any non-Markdown
file). No other core concepts.

**Conformance levels (all valid):** 0 plain Markdown · 1 typed
(`profile`/`status`/`share`) · 2 linked (graph + indexes) · 3 validated (CI lint).
Creation is free; promoting a doc to `status: stable` earns Level-3 checks.

**Frontmatter fields (all optional; unknown keys are ignored, never errors):**

| Field | Meaning |
| --- | --- |
| `profile` | Document kind; defaults to `note` (Default Profile). There is no separate `type`. |
| `status` | `draft` \| `stable` \| `deprecated` \| `archived` (lowercase). Default `draft`. |
| `created` | Document creation timestamp (`YYYY-MM-DD` or ISO-8601). Optional; for non-Git authors. |
| `updated` | Document last update timestamp (`YYYY-MM-DD` or ISO-8601). Optional; accepts `last_updated` alias. |
| `share` | `public` (default, open) \| `org` \| `private` (excluded from `odc context`/`odc export` by default). |
| `id` | Override the path-derived id (use for rename stability). Usually omitted. |
| `description` | One-line summary; feeds index listings. |
| `depends` | IDs this doc requires first (directional; loaded transitively). |
| `related` | IDs for optional further reading (non-binding). |
| `resources` | Non-Markdown files: list of `{ path: ... }` (path-only, no `type`). |
| `code` | Implementation map: list of `{ path, role, symbol? }` with fixed roles. |
| `context` | AI reading scope: `load` (ids/paths), `ignore` (path prefixes), `max-depth`. |
| `owner` | Responsible person/team. |
| `tags` | Free-form lowercase facets; never structural; never required. |

**Root `index.md` only:** `odc` (legacy: `ods`) (spec/schema version), `odc` (CLI
compatibility requirement), `profiles` (custom
catalog roots), `packs` (imported ODS Packs list), `ignore` (workspace-relative path prefixes excluded from scan),
`aliases` (section-heading aliases).

Recommended key order: `profile`, `status`, `share`, `id`, `description`, `owner`, `tags`,
then `depends`, `related`, `resources`, `code`, `context`.

**IDs are paths:** a doc's id is its workspace-relative path minus `.md`,
lowercase, `/`-separated (`guides/setup/oauth.md` → `guides/setup/oauth`). IDs
are unique and case-insensitive. Use an explicit `id:` only to keep an id stable
across a move.

**The graph:** only `depends` and `related`. Declared on the dependent side
only — backlinks are computed by tooling, never hand-written. No dependency
cycles at Level 3.

**Indexes:** an `index.md` lists **only its directory's immediate children**
(never deeper). Generated by `odc ods index` and treated like a lockfile: reviewed
and committed, never hand-edited. Child summaries come from each child's
`description`. No counts, sizes, or stats. The **root** index declares
`ods: 0.1` and `odc: ">=0.0.1"` once — `ods:` is the workspace marker and
`odc:` guards tooling compatibility.

**Resources:** any non-Markdown file, listed as `- path: ...` relative to the
document. Format is implied by extension; no `type` field. At Level 3 the path
must resolve.

**Code references:** `code:` maps a document to implementation or operational
files without making source files ODS documents. Each entry has required `path`
and `role`, optional `symbol`. Fixed roles only: `entrypoint`, `implementation`,
`test`, `schema`, `migration`, `config`, `infrastructure`, `pipeline`. Unknown
roles are errors; projects do not customize roles. `odc ods context` includes code
paths; rename automation rewrites `code[].path` but not `symbol`.

**Frontmatter Layout**: Both flat top-level metadata (`profile: guide`, `status: stable`) and nested `ods:` map metadata (`ods: { profile: guide, status: stable, ... }`) are valid Level 1+ forms recognized by `odc-core`. The nested `ods:` map structure is the **canonical format** promoted by `odc ods fmt --migrate`.

**Context, Export, & Sharing**:
- `odc ods context <id>`: resolved bounded reading list for an AI agent on a specific document topic. Preferred over full export for prompt context.
- `odc ods export [path]`: writes a visual `graph.md` representation of the document graph (`--out PATH`). Excludes `share: private`/`share: org` by default (`--include-private` to include).
- `odc ods share [path] --out <dir>`: publishes a share-filtered copy of documents (filtering by `share: public|org|private`) into a target directory (`--include-org`, `--include-private`). Run `git` manually on the target directory to publish.

**Profiles & Packs:** a profile's `## H2` headings declare expected section headings. Standard Profiles (12 built-in schemas: `note`, `feature`, `guide`, `api`, `architecture`, `decision`, `sop`, `policy`, `meeting`, `faq`, `checklist`, `index`). Custom profiles live under `ods-profiles/`, root `profiles:`, or imported ODS Packs (`packs:`). ODS Packs are reusable workspace bundles managed via `odc pack {add, sync, list, preview, remove, init}` supporting Git URLs and local paths/links.

**Lint rules:**

| Rule | Level | Severity |
| --- | :---: | :---: |
| Frontmatter parses as YAML | 1+ | error |
| `status` is a valid enum | 1+ | error |
| `share` is `public`, `org`, or `private` | 1+ | error |
| `profile` resolves | 1+ | warning |
| No dangling `depends`/`related` | 3 | error |
| No duplicate IDs | 3 | error |
| No `depends` cycles | 3 | error |
| Resource / `context` / `packs` paths exist | 3 | error |
| `code[].role` fixed + `code[].path` exists | 1+ role / 3 path | error |
| Index lists exactly current children | 3 | error |
| Body Markdown links don't dangle | 3 | error |
| Profile's expected sections present | 3 | warning |

**Scan ignores** (case-insensitive base names, never treated as docs): `target`,
`node_modules`, `dist`, `build`, `.artifacts`, `.git`, `.hg`, `.svn`, `.jj`,
`__pycache__`, `.venv`, `venv`, `vendor`, and any name starting with `.`. The
root index may add more via `ignore:`.

## 3. Command reference

Path arg defaults to `.`. Common flags: `--level 1|3` (default 3), `--format text|json`.

- `odc ods init [path]` (alias `enable`) — make folder/repo ODS-compliant (create root `index.md` + `ods:` / `odc:` markers, index). `--adopt` also drafts frontmatter on plain files.
- `odc ods new <path> [--profile <p>] [--title "<t>"]` — scaffold a new document with an inferred or explicit profile (`-p`/`--profile`) and title (`-t`/`--title`).
- `odc ods rm <path-or-id>` — atomically delete a document and scrub graph references (`depends`/`related`) to it workspace-wide.
- `odc ods archive <path-or-id>` — set `status: archived` in frontmatter. This rewrites the file in place; it does **not** move it to an `archive/` folder.
- `odc setup [path]` — check release freshness, find the workspace, start/check the user machine service, then run doctor.
- `odc skill install --agent <name> [--scope project|user]` — install the full ODS skill directory (including scripts and references) for `claude-code`, `antigravity`, `codex`, or `gemini-cli`; install a provider-native rules file for `cursor`, `windsurf`, or `copilot`.
- `odc update` — self-update CLI from GitHub Releases and restart the background user service; `--check` (read-only), `--force`, `--version <tag>`.
- `odc ods disable [path]` (alias `revert`) — dry-run strip of ODS metadata; `--write` to apply. `--keep-frontmatter`, `--remove-indexes`, `--remove-root-index`.
- `odc ods adopt [path]` — report adoption status (dry-run); `--write` drafts `profile` + `status: draft`, inferring profile from headings.
- `odc lint [path]` / `odc ods lint [path]` — validate; `--level 1|3`, `--format text|json`, `--canonical-refs` (warn on extensionless Document refs). Writes/clears `.odc/odc-errors.md` (legacy root `ods-error.md` is removed when present).
- `odc ods index [path]` — generate `index.md` files; `--check` exits 1 if stale.
- `odc ods profiles [path]` — list loaded profiles + conflicts.
- `odc pack add <src>` — add an ODS Pack via Git URL (HTTPS, SSH, Shorthand) or Local Link (`../pack`). This *imports* a pack; ODS has no outbound bundle-building command.
- `odc pack sync` — update installed remote Git packs across machines.
- `odc pack list` — list active installed packs, contributed profiles, and skills.
- `odc pack preview <name>` — inspect profile schemas inside a pack.
- `odc pack remove <name>` — unregister an ODS Pack reference from root `index.md` `packs:`.
- `odc pack init [name]` — scaffold a new reusable ODS Pack repository.
- `odc ods tags [path]` — list project tags with counts; `--all` includes unused defaults.
- `odc ods find [path] --tag <t>` — list doc ids with a tag (repeat `--tag` = OR).
- `odc ods context [path] <id>` — resolved bounded AI reading list for a doc; excludes `share: private` docs by default (`--include-private` to opt in).
- `odc ods graph [path]` — print `depends`/`related` edges as `path -> edge` lines.
- `odc ods doctor [path]` — health: version, doc count, index freshness, profile conflicts, service install/running, pending git renames. Exit 1 on error.
- `odc ods tag rename <old> <new> [path]` — rewrite a tag; dry-run unless `--write`.
- `odc ods mv [path] <from> <to>` — offline move + rewrite refs + indexes.
- `odc ods fmt [path]` — normalize frontmatter/body blank-line spacing.
- `odc ods fmt --refs md-paths [path]` — also rewrite document references to explicit `.md` relative paths.
- `odc ods fmt --migrate [path]` — also rewrite legacy flat/out-of-order `ods:` engine keys into the canonical nested `ods:` block (`profile` → `status` → `id` → `share` → `depends`/`related`/`resources`/`code`/`context`). Opt-in.
- `odc ods sync [path]` — reconcile git-tracked renames (`git status --porcelain`) + rewrite refs.
- `odc ods watch [path]` — foreground live rename mapping + re-lint. Blocks the terminal; use for interactive sessions, not background automation.
- `odc ods logs [path] [-f]` — currently an **alias for `odc ods watch`**; it does not tail the background service's log output despite the name.
- `odc ods start [path]` — register + start the persistent background user service (systemd/launchd/scheduled task running `odc ods serve`). Prefer this over running `odc ods serve` directly for anything long-lived.
- `odc ods start --status [path]` — read-only: print `installed=`/`running=` service state; use this to poll instead of guessing.
- `odc ods stop [path]` — stop the running service (keeps registration).
- `odc ods stop --unregister [path]` — stop and fully remove the service registration.
- `odc ods serve --root <path>` — headless loop the OS service runs; only invoke directly for debugging or when not using OS-managed `start`/`stop`. Flags: `--mode auto|watch|poll`, `--memory-report` (print `rss_kb=` diagnostics), `--poll-secs <n>` (poll interval in poll mode).
- `odc workspaces list|add|remove|path` — manage globally tracked workspace paths in `~/.odc/odcconfig.toml` (legacy `~/.ods/odsconfig.toml` and `workspaces.toml` are read; writes go to `~/.odc`).
- `odc ods bench stats [path]` (alias `odc ods sandbox`) — compute an estimated token reduction % and estimated monthly API cost savings ($).
- `odc ods bench strip [path]` — backup frontmatters to snapshot and strip (dry-run; `--write`, `--full`, `--indexes`/`--strip-indexes`, `--profiles`/`--strip-profiles`, `--path <p>`).
- `odc ods bench restore [path]` — restore frontmatters from JSON snapshot (`--snapshot <id>`).
- `odc ods bench run [path]` — print a **simulated** token/cost comparison (`--prompt "<task>"`, `--llm <provider>`).
- `odc ods share [path] --out <dir>` — publish a share-filtered copy of documents based on `share` metadata (`--include-org`, `--include-private`).
- `odc ods export [path]` — write `graph.md` (`--out PATH`); excludes `share: private`/`share: org` docs by default (`--include-private` to opt in).

Env: `ODS_AUTO_UPDATE=0`, `GH_TOKEN`/`GITHUB_TOKEN`, `ODS_LOW_MEMORY=1`, `ODS_SERVE_MODE=auto|watch|poll`, `ODS_POLL_SECS=<n>`, `ODS_SETUP_NO_START=1`.

## 4. Workflows

**Compliance gate — run first, every time.** Before workspace commands, refresh
or check the local binary first (`odc setup` does this; use `odc update --check`
for read-only checks). Then ensure the root `index.md` carries `ods:` for the
spec version and `odc:` for CLI compatibility. If either marker is missing
or stale, run `odc setup <path>` to repair it before `lint`/`index`/`start`.

**Git vs non-git — branch on the `check` probe:**
- **Git-tracked repo** — after a `git mv` (or a rename made elsewhere), run
  `odc ods sync` to reconcile git-tracked renames and rewrite refs. `.gitignore`-aware
  scanning and CI checks (`odc ods index --check` + `odc ods lint`) apply; `odc ods doctor`
  reports pending git renames.
- **Non-git folder** — `odc ods sync` has nothing to reconcile; do **not** rely on
  it and do not invoke git. Keep the graph true with the live path engine
  (`odc ods start` / `odc ods watch`) or offline moves (`odc ods mv`).

**Common flows:**
- Adopt an existing tree: `odc ods adopt --write` → `odc ods lint --level 1` (tighten toward Level 3 over time).
- New/maintained workspace: `odc ods init` → `odc ods start` → edit & rename freely → `odc ods lint` → `odc ods export` / `odc ods context` → `odc ods doctor`.
- Refactor: with `start`/`watch` on, renames map automatically; otherwise `odc ods mv` (offline) or `odc ods sync` (after a git rename).
- Feed an agent: prefer `odc ods context <id>` (bounded) over full `odc ods export`.

**Workspace Migration Guidelines:**
1. **Document-Relative Code Paths:** `code[].path` in document frontmatter is evaluated relative to the document file location (e.g. `../../src/...` for 1 level down, `../../../src/...` for 2 levels down).
2. **Excluding Legacy Content:** Exclude legacy folders containing unmaintained external links by setting `ignore: [changelog/, plan/, reports/, research/]` in root `index.md` frontmatter.
3. **NPM Integration:** Expose `"docs:lint": "ods lint docs --level 3"`, `"docs:index-check": "ods index docs --check"`, and `"docs:doctor": "ods doctor docs"` in `package.json`.
*(Note: Case-insensitive reference lookups and index prose preservation are handled natively by the ODS CLI binary via PR #14).*

**Troubleshooting & Health Diagnostics:**
- Check if background watch service is running: `odc ods start --status [path]` (returns `installed=true/false running=true/false`).
- Validate complete workspace health: `odc ods doctor [path]` (verifies CLI version, document counts, index freshness, profile conflicts, pending git renames, and background service state; exits 1 on failure).
- Automated repair & setup: `odc setup [path]` (checks CLI freshness, ensures root `index.md` markers, starts user service, and executes `odc ods doctor`).
- Bootstrap diagnostics: `scripts/bootstrap.sh status [path]` or `scripts/bootstrap.sh doctor [path]`.

**Agent reading order** (for AGENTS.md / system prompts):
1. Open root `index.md` (marker, `ods:`, `ignore:`, `profiles:`, `packs:`, `aliases:`).
2. Walk child indexes only as deep as needed (progressive disclosure).
3. Open the target `.md`; read frontmatter before body.
4. Skip nodes where `share: private` is set.
5. Load `depends` transitively up to `context.max-depth` (often 2).
6. Load `context.load` (docs by id, resources by path).
7. Load declared `code[].path` entries when implementation context is useful.
8. Skip `context.ignore` prefixes and workspace `ignore:` trees.
9. Prefer `status: stable` over `draft`; treat `deprecated`/`archived` as last resort.
10. Use `profile`, not an invented `type`; resources are path-only.
11. Relationships live in frontmatter only; never duplicate edge lists in prose.

## 5. Guardrails (MUST / MUST NOT)

- Use only `depends` and `related`. MUST NOT add `implements`, `extends`, `replaces`, or any other edge type.
- Relationships live **only** in frontmatter. MUST NOT restate dependency/related lists in the body; MUST NOT hand-write backlinks.
- The title is the first `# H1` only. MUST NOT add a `title:` frontmatter field.
- Enum values (`status`, `profile`, `share`) are lowercase.
- IDs are paths. MUST NOT invent id schemes; use `id:` only for rename stability.
- Generated `index.md` files are lockfile-like. MUST NOT hand-edit them, add counts/sizes/stats, or list non-immediate children.
- `resources` entries are path-only. MUST NOT add a resource `type`.
- `code` entries use only fixed roles. MUST NOT define project custom roles, add ODS frontmatter to source files, or require inline code backlinks.
- Set `ods:` and `odc:` on workspace root indexes; ordinary docs omit both. Nested indexes carry them only when intentionally usable as separate example/fixture workspaces.
- Unknown frontmatter keys are ignored — MUST NOT treat them as errors.
- The service is a background OS user service (`odc ods serve`). There is NO port/HTTP server — MUST NOT invent one or add a health endpoint.
- Never build `odc` (legacy: `ods`) from source in the skill; install/update via `scripts/bootstrap.sh` (releases).

## 6. Out of scope / non-goals

ODS deliberately excludes (full rationale in references/non-goals.md): new file
extensions (`.od`/`.ods`); any manifest, config, lock, or ignore file (workspace
policy lives on the root `index.md`); `lifecycle`, `updated`, `type`, universal
`url:`, or `assets:` fields; extra edge vocabulary; profile inheritance or a
template engine; a closed tag registry; derived index stats (counts, coverage,
scores); a `collection` concept without distinct semantics; and a dedicated
`specs` profile (spec prose uses `note`, `decision`, or `guide`). The shipped
product is the **single `odc` (legacy: `ods`) binary only** — the former LSP and Zed extension
were removed.

## 7. Evals roadmap

Starter cases live in evals/evals.json (activation ±, authoring correctness,
guardrail/non-goal enforcement, command choice, service-model, output format).
Planned expansion, to keep outcomes consistent across models and effort levels:
grow to ≥10 cases; add before/after fixtures under `evals/fixtures/{good,bad}/`;
run the same suite across models × reasoning-effort levels to catch drift; add a
script-backed validator once the registry publish flow exists.

## 8. Migration Guide — Updating Legacy `ods` to `odc` (OS-Agnostic)

When updating a machine or workspace from legacy `ods` (v0.1.x) to `odc` (OpenDocify), follow these OS-agnostic steps:

### Step 1: Update CLI Binary & Machine Config (OS-Agnostic)
- **Bootstrap script (recommended)**:
  - macOS / Linux: `scripts/bootstrap.sh update`
  - Windows PowerShell: `scripts/bootstrap.ps1 -Command update`
- **CLI direct**:
  - `odc update` (or `ods update`) automatically downloads/installs both `odc` and `ods` binaries to PATH, migrates `~/.ods/` (or `%USERPROFILE%\.ods`) config files to `~/.odc/` (`odcconfig.toml`), rewrites legacy `ods-cli:` -> `odc:` pins on root `index.md`, and cleans up legacy `ods-error.md`.

### Step 2: Upgrade Workspace Root Pins & Machine Registration
- Run `odc upgrade --write [path]` in the workspace root.
- This safely rewrites root `index.md` legacy pins (`ods-cli:` -> `odc: ">=0.0.1"`), verifies config migration, and cleans up stale `ods-error.md` reports.

### Step 3: Canonical Frontmatter Layout Migration (Optional)
- Run `odc ods fmt --migrate [path]` to rewrite legacy flat/out-of-order `ods:` engine keys into the canonical nested `ods:` block (`profile` → `status` → `id` → `share` → `depends`/`related`/`resources`/`code`/`context`).

### Step 4: Verification
- Verify binary: `odc --version` (returns `odc 0.x.y`) and `ods --version` (legacy argv0 alias).
- Verify health: `odc doctor [path]` or `odc lint [path]`. Reports write to `.odc/odc-errors.md`.

### Step 5: Final Update & Service Restart
- Run `odc update` at the end to ensure all binaries, OS service units, and workspace configurations are fully updated and synchronized across processes.

