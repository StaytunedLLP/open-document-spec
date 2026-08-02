---
description: "Complete reference for frontmatter keys, root index configuration, profiles vs packs, AI context, and CLI commands."
status: "stable"
order: 9
ods:
  profile: "note"
  status: "stable"
---

# ODS Features & Architecture Overview

Open Document Specs (ODS) is a lightweight, graph-native Markdown convention layer. This document provides a complete guide to all frontmatter keys, workspace mechanics, profiles, tags, code references, AI context rules, and ODS Packs.

---

## 1. Core Frontmatter Features

| Field | Type | Level | Purpose & Usage |
| :--- | :--- | :---: | :--- |
| **`profile`** | string | 1+ | **Document Classification**: Defines document kind (`guide`, `decision`, `feature`, `sop`, etc.). Defaults to `note`. |
| **`status`** | string | 1+ | **Document Lifecycle**: Maturity state (`draft`, `stable`, `deprecated`, `archived`). Defaults to `draft`. |
| **`share`** | string | 1+ | **Visibility Control**: `public` (default), `org`, or `private`. |
| **`id`** | string | 2+ | **Explicit Identity**: Override the path-derived default ID (`docs/setup`). |
| **`description`** | string | 1+ | **Summary**: One-line summary used to populate generated `index.md` lockfiles. |
| **`depends`** | list of refs | 2+ | **Prerequisite Edges**: Graph prerequisite links to other documents (`- ../auth/sessions.md`). |
| **`related`** | list of refs | 2+ | **Association Edges**: Non-binding reference links (`- ../products/glow.md`). |
| **`resources`** | list | 2+ | **Native Asset Links**: Non-Markdown files attached to the document (`- path: docs/diagram.pdf`). |
| **`code`** | list | 2+ | **Implementation Map**: Fixed-role source code mappings (`path`, `symbol`, `role: entrypoint\|implementation\|test\|...`). |
| **`context`** | map | 2+ | **Bounded AI Reading Scope**: Directives for AI agents (`load: [...]`, `ignore: [...]`, `max-depth: 2`). |
| **`owner`** | string | 1+ | **Maintainer**: Responsible individual or team (`support-team`). |
| **`tags`** | list of strings | 1+ | **Search Facets**: Free-form lowercase facets (`- customer-care`). |

---

## 2. Root `index.md` Configuration Keys

| Field | Type | Scope | Purpose |
| :--- | :--- | :--- | :--- |
| **`ods`** | string | Root `index.md` | **Workspace Spec Marker**: Declares ODS workspace boundary and version (`ods: 0.1`). Not the CLI binary name. |
| **`ods`** | string | Root `index.md` | **CLI Requirement**: Minimum Open Document Spec CLI version (`ods: ">=0.0.1"`). Replaces legacy `ods-cli:`. |
| **`profiles`** | list of paths | Root `index.md` | **Custom Profile Catalogs**: Workspace paths to custom profile schemas (`[".ods-profiles"]`). |
| **`packs`** | list of paths | Root `index.md` | **Imported ODS Packs**: Reusable workspace bundles containing profiles, skills, and SOPs (`["vendor/engineering-pack"]`). |
| **`ignore`** | list of paths | Root `index.md` | **Workspace Excludes**: Workspace-relative path prefixes excluded from scan (`["src/", "dist/"]`). |
| **`aliases`** | map | Root `index.md` | **Section Heading Aliases**: Workspace-wide H2 section aliases (`Goal: [Objective, Purpose]`). |
