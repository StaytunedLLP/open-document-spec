---
profile: index
---

# Specs

Normative and author-facing specifications for dialects supported by the **`ods`** CLI.

This is the **only** specs tree in the repository (repo-root `specs/`). Implementation code lives under `src/`.

| Dialect | Folder | CLI | Best for |
| :--- | :--- | :--- | :--- |
| **ODS** (default) | [ods/](ods/index.ods.md) | bare `ods …` | Document graphs, profiles, code bindings, indexes |
| **OKF v0.2** | [okf/](okf/index.ods.md) | `ods … --okf` | Knowledge bundles: trust, provenance, freshness |
| **Agent Skills** | [skills/](skills/index.ods.md) | `ods … --skills` | `SKILL.md` packages for AI agents |

There is **no** `--ods` flag and **no** `ods okf` / `ods skills` namespaces.

Operational CLI notes: `docs/other-specs/cli-multi-spec.md`.  
Key comparison (ODS vs OKF): `docs/other-specs/frontmatter-keys-ods-vs-okf.md`.

## Start here

1. New to ODS? → [ods/intro.md](ods/intro.md)
2. Authoring frontmatter? → [ods/keys.md](ods/keys.md)
3. OKF bundles? → [okf/intro.md](okf/intro.md)
4. Skill packages? → [skills/intro.md](skills/intro.md)

- [ods/](ods/index.ods.md)
- [okf/](okf/index.ods.md)
- [skills/](skills/index.ods.md)
