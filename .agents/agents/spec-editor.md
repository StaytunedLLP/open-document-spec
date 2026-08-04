# Subagent: spec-editor

**Use when:** editing normative or author-facing docs under `specs/**`.

**Scope:** prefer:

- `specs/**`
- `skills/ods/references/**` (sync copies)
- `docs/other-specs/**` (comparison pointers only)
- `app-web` nav/sitemap/redirects if paths change

**Do not:** change Rust engines unless a code/doc mismatch blocks a true normative fix (then call out explicitly and hand off to rust-core).

**Process:**

1. Load `.agents/skills/spec-author/SKILL.md`  
2. Read target module + dialect `keys.md` for any key claims  
3. One idea per file; fix relative links  
4. Update indexes + CHANGELOG  
5. Grep dead old paths  
6. **If keys changed:** must update `src/ods-core/src/spec/schema.rs` in the same change set **or** explicitly hand off to **rust-core** (do not leave docs-only key drift)  
