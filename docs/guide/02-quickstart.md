---
title: "Quickstart Guide"
description: "Installation options, workspace initialization, service background daemon, validation, and AI context commands."
status: "stable"
order: 2
ods:
  profile: "note"
  status: "stable"
---

# Quickstart

OpenDocify is one native CLI binary: **`odc`** (optional legacy argv0 **`ods`** = ODS engine only). The normal flow is install, run `odc setup`, initialize or adopt Markdown, keep the user service running, then validate with bare **`odc lint`** / **`odc index --check`** (or explicit `odc ods …` in CI).

---

## 1. Setup & Installation

### Option 1: Primary Preference — Skill-First Setup for AI Assistants

The recommended, zero-friction entry point for ODS is via the **ODS Skill** in your AI Coding Assistant (Claude Code, Antigravity, Cursor, Codex, Windsurf, etc.).

When an AI assistant activates the ODS skill, the bundled cross-platform bootstrap script automatically:
1. Detects your host Operating System (macOS, Linux, Windows) and architecture (`x86_64`, `arm64`).
2. Downloads and verifies the matching native `odc` (legacy `ods`) release binary.
3. Registers and starts the persistent background OS user service (`systemd` user unit / `launchd` agent / Windows Scheduled Task).
4. Validates workspace health and prints status:

```text
==> OpenDocify is installed and running on your machine!
==> Version: odc v0.0.x
```

---

### Option 2: Direct CLI Installation

If you are operating directly in a terminal without an AI coding skill, you can manually install the CLI binary:

**macOS / Linux**:

```bash
curl -fsSL https://opendocify.com/install.sh | bash
odc --version
```

**Windows (PowerShell)**:

```powershell
irm https://opendocify.com/install.ps1 | iex
odc --version
```

---

## 2. Initialize a Workspace

New documentation folder:

```bash
mkdir my-docs
cd my-docs
odc init .          # ODS default (writes ods: + odc:); or: odc ods init .
```

Existing Markdown tree:

```bash
cd existing-docs
odc init . --adopt  # or: odc ods init . --adopt
```

OKF knowledge bundle:

```bash
odc init --okf .    # or: odc okf init .
```

`odc init` (ODS) makes the folder ODS-compliant by creating a root `index.md` with `ods: 0.1` and `odc: ">=0.0.1"` metadata and generating child index files.

---

## 3. Run Setup & Start Background Service

```bash
odc setup
```

`odc setup` checks release freshness, verifies the root spec header, starts/registers the background OS user service (`systemd` / `launchd` / `schtasks`), and runs `odc ods doctor`.

Direct service commands:

```bash
odc ods start .
odc ods start --status
odc ods stop .
odc ods stop --unregister .
```

Foreground alternative:

```bash
odc ods watch .
```

While `odc ods start` or `odc ods watch` runs, rename/move Markdown normally. ODS keeps path-shaped `id`, `depends`, `related`, body links, resource paths, context path entries, and generated `index.md` child lists current.

---

## 4. Validate Trust

```bash
odc ods lint
odc ods index --check
```

Clean lint output:

```text
Everything is fine — graph and links are consistent. No update required.
```

---

## 5. Use AI Context

Preferred bounded reading list:

```bash
odc ods context <doc-id>
```

Optional full graph file:

```bash
odc ods export
odc ods export --out ai/graph.md
```

Publishing a filtered subset for external hand-off:

```bash
odc ods share . --out ../shared-docs
```

---

## 6. Keep Current

```bash
odc update --check
odc update
```

`odc update` downloads the latest binary release from GitHub Releases and automatically restarts the background service so it runs with the updated binary.

---

## Next

- Existing repos: [Adopting ODS](/docs/adoption)
- CLI, service, CI, updates: [Tooling Reference](/docs/tooling)
- Profiles: [Profiles & Catalogs](/docs/profiles)
- FAQ: [FAQ & Troubleshooting](/docs/faq)
