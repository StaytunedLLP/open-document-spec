---
profile: index
---

# odc (CLI crate)

Primary binary **`odc`** and legacy argv0 **`ods`**.

## Source layout

```
src/
  main.rs              # odc entry
  ods_main.rs          # ods entry (same includes)
  main/
    cli/               # entry, dispatch, argv, exit codes
    commands/          # user-facing commands
      okf/             # OKF command surface
      pack/            # pack add/sync/list/…
      workspaces/      # global workspace registry
    support/           # formatters, loaders, helpers
  service/             # OS user service
  update/              # GitHub release self-update
tests/                 # integration tests (*.test.rs)
```

When adding a command: put it under `main/commands/`, `include!` it from `main.rs` and `ods_main.rs`, add a CLI test under `tests/`.
