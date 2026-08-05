# AGENTS.md — `specs/ods` (repo-root specs tree)

When editing this folder:

- Start readers at **intro.md**; keys belong only in **keys.md**
- **core.md** is format model + levels + lifecycle — not a second key encyclopedia
- One-word filenames; keep content owned by the file name (`assets` = resources+code, `scope` = non-goals)
- Relative links between siblings; site URLs are `/spec/ods/<name>`
- Root marker is `ods.toml` (`spec`); discovery via CLI (no nested indexes)
- Custom profiles: document **`custom-profiles:`** (legacy `profiles:` allowed)
- After changes: sync `skills/ods/references/{intro,keys,core,scope}.md` and site nav
- If **keys.md** changes: update engine registry `src/ods-core/src/spec/schema.rs` in the same change set
