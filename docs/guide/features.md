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

Frontmatter is split into **universal top-level** keys (any tool can read them) and **engine keys under `ods:`** (ODS-only). See the key dictionary [ods/keys.md](../../specs/ods/keys.md).

| Field | Type | Placement | Level | Purpose & Usage |
| :--- | :--- | :--- | :---: | :--- |
| **`description`** | string | **Top-level** | 1+ | **Summary**: One-line summary used for indexes and SSG meta. |
| **`tags`** | list of strings | **Top-level only** | 1+ | **Search Facets**: Free-form lowercase facets (`- customer-care`). Never under `ods:`. |
| **`owner`** | string \| list | **Top-level** | 1+ | **Maintainer**: Responsible individual or team (`support-team`). |
| **`profile`** | string | **Under `ods:`** | 1+ | **Document Classification**: Document kind (`guide`, `decision`, `feature`, `sop`, etc.). Defaults to `note`. |
| **`status`** | string | **Under `ods:`** | 1+ | **Document Lifecycle**: Maturity (`draft`, `stable`, `deprecated`, `archived`). Defaults to `draft`. |
| **`share`** | string | **Under `ods:`** | 1+ | **Visibility Control**: `public` (default), `org`, or `private`. |
| **`id`** | string | **Under `ods:`** | 2+ | **Explicit Identity**: Override the path-derived default ID. |
| **`depends`** | list of refs | **Under `ods:`** | 2+ | **Prerequisite Edges**: Graph prerequisite links to other documents. |
| **`related`** | list of refs | **Under `ods:`** | 2+ | **Association Edges**: Non-binding reference links. |
| **`resources`** | list | **Under `ods:`** | 2+ | **Native Asset Links**: Non-Markdown files attached to the document. |
| **`code`** | list | **Under `ods:`** | 2+ | **Implementation Map**: Fixed-role source code mappings (`path`, `symbol`, `role`). |
| **`context`** | map | **Under `ods:`** | 2+ | **Bounded AI Reading Scope**: `load`, `ignore`, `max-depth`. |

Misplaced nested `tags` under `ods:`: `ods lint` warns; repair with `ods fmt --migrate`.

---

## 2. Root `ods.toml` Configuration Keys

| Field | Type | Scope | Purpose |
| :--- | :--- | :--- | :--- |
| **`ods`** | string | Root `ods.toml` | **Workspace Spec Marker**: Declares ODS workspace boundary and version (`spec = "0.1"`). Not the CLI binary name. |
| **`ods`** | string | Root `ods.toml` | **CLI Requirement**: Minimum Open Document Spec CLI version (`ods: ">=0.0.1"`). Replaces legacy `ods-cli:`. |
| **`custom-profiles`** | list of paths | Root `ods.toml` | **Custom Profile Catalogs**: Workspace paths to custom profile schemas (`[".ods/profiles/rfc.md"]`). |
| **`packs`** | list of paths | Root `ods.toml` | **Imported ODS Packs**: Reusable workspace bundles containing profiles, skills, and SOPs (`["vendor/engineering-pack"]`). |
| **`ignore`** | list of paths | Root `ods.toml` | **Workspace Excludes**: Workspace-relative path prefixes excluded from scan (`["src/", "dist/"]`). |
| **`aliases`** | map | Root `ods.toml` | **Section Heading Aliases**: Workspace-wide H2 section aliases (`Goal: [Objective, Purpose]`). |
