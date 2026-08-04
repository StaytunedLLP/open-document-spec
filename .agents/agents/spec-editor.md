# Subagent: spec-editor

**Use when:** editing normative or author-facing docs under `specs/**`.

**Scope:** prefer read/write limited to:

- `specs/**`
- `skills/ods/references/**` (sync copies)
- `docs/other-specs/**` (comparison pointers only)
- `app-web` nav/sitemap/redirects if paths change

**Do not:** change Rust engines unless a code/doc mismatch blocks a true normative fix (then call out explicitly).

**Process:**

1. Load `.agents/skills/spec-author/SKILL.md`
2. Read target module + `ods/keys.md` for any key claims
3. Edit for one idea per file; fix relative links
4. Update indexes + CHANGELOG
5. Grep for dead old paths
