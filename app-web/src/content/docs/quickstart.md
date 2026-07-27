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

ODS is one native CLI binary: **`ods`**. The normal flow is install, run `ods setup`, initialize or adopt Markdown, keep the user service running, then validate with `ods lint` and `ods index --check`.

---

## 1. Setup & Installation

### Option 1: Primary Preference — Skill-First Setup for AI Assistants

The recommended, zero-friction entry point for ODS is via the **ODS Skill** in your AI Coding Assistant (Claude Code, Antigravity, Cursor, Codex, Windsurf, etc.).

When an AI assistant activates the ODS skill, the bundled cross-platform bootstrap script automatically:
1. Detects your host Operating System (macOS, Linux, Windows) and architecture (`x86_64`, `arm64`).
2. Downloads and verifies the matching native `ods` release binary.
3. Registers and starts the persistent background OS user service (`systemd` user unit / `launchd` agent / Windows Scheduled Task).
4. Validates workspace health and prints status:

```text
==> ODS is installed and running now in your machine!
==> Version: ods v0.1.x
```

---

### Option 2: Direct CLI Installation

If you are operating directly in a terminal without an AI coding skill, you can manually install the CLI binary:

**macOS / Linux**:

```bash
curl -fsSL https://opendocify.com/install.sh | bash
ods --version
```

**Windows (PowerShell)**:

```powershell
irm https://opendocify.com/install.ps1 | iex
ods --version
```

---

## 2. Initialize a Workspace

New documentation folder:

```bash
mkdir my-docs
cd my-docs
ods init .
```

Existing Markdown tree:

```bash
cd existing-docs
ods init . --adopt
```

`ods init` makes the folder ODS-compliant by creating a root `index.md` with `ods: 0.1` and `ods-cli: ">=0.0.1"` metadata and generating child index files.

---

## 3. Run Setup & Start Background Service

```bash
ods setup
```

`ods setup` checks release freshness, verifies the root spec header, starts/registers the background OS user service (`systemd` / `launchd` / `schtasks`), and runs `ods doctor`.

Direct service commands:

```bash
ods start .
ods start --status
ods stop .
ods stop --unregister .
```

Foreground alternative:

```bash
ods watch .
```

While `ods start` or `ods watch` runs, rename/move Markdown normally. ODS keeps path-shaped `id`, `depends`, `related`, body links, resource paths, context path entries, and generated `index.md` child lists current.

---

## 4. Validate Trust

```bash
ods lint
ods index --check
```

Clean lint output:

```text
Everything is fine — graph and links are consistent. No update required.
```

---

## 5. Use AI Context

Preferred bounded reading list:

```bash
ods context <doc-id>
```

Optional full graph file:

```bash
ods export
ods export --out ai/graph.md
```

Publishing a filtered subset for external hand-off:

```bash
ods share . --out ../shared-docs
```

---

## 6. Keep Current

```bash
ods update --check
ods update
```

`ods update` downloads the latest binary release from GitHub Releases and automatically restarts the background service so it runs with the updated binary.

---

## Next

- Existing repos: [Adopting ODS](/docs/adoption)
- CLI, service, CI, updates: [Tooling Reference](/docs/tooling)
- Profiles: [Profiles & Catalogs](/docs/profiles)
- FAQ: [FAQ & Troubleshooting](/docs/faq)
