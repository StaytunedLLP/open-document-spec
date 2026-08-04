# Quality gates (before claiming done)

## Do not claim CI-safe without green gates

Minimum for any engine/script change:

```bash
SKIP_RELEASE_BUILD=true ./src/scripts/check-local.sh
```

Includes: `fmt --check`, `clippy -D warnings`, `cargo test --workspace`, fixtures, **naming**, **odc residue**.

When Rust tests or coverage-sensitive code changed:

```bash
ODS_COVERAGE_FAIL_UNDER_LINES=90 ./src/scripts/coverage.sh
```

## Coverage policy

- **Gate:** workspace **≥90% lines** with T3 excludes
- Per-file ≥90% is stretch, not the commit bar
- T3 list: `docs/maintainer/coverage-excludes.md`
- Expanding T3 requires **same PR** update of: `coverage.sh`, `.github/workflows/pr.yml`, `coverage-excludes.md` (and CONTRIBUTING if listed)
- Ratchet CI `--fail-under-lines` **up only**

## Sequence for multi-step engine work

Use skill `.agents/skills/quality-gate/SKILL.md`:

1. Inventory / baseline  
2. Naming + odc residue  
3. Schema + docs alignment  
4. Edge-case tests  
5. Coverage  
6. release-docs if user-visible  

## Definition of Done

- [ ] `check-local.sh` exit 0  
- [ ] Coverage ≥90% if code changed (T3-aware)  
- [ ] No Class-A `odc:` fixtures reintroduced  
- [ ] Schema ↔ keys.md aligned if keys touched  
- [ ] CHANGELOG / docs touchpoints if user-visible  
