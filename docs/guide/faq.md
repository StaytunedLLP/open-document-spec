---
description: "Frequently asked questions about file extensions, adoption, profiles vs tags, renames, and background services."
ods:
  profile: "note"
  status: "stable"
---

# FAQ and Troubleshooting

Feature catalog: [Features](/docs/features).

## Do I need a special file extension?

No. Documents are plain `.md` only.

## Does installing ODS auto-make my Markdown compliant?

**No.** Without root `ods:`, tools do not rewrite your tree. Plain Markdown is plain Markdown.

- Opt in: `ods init .` or `ods init . --adopt`, then `ods lint`.
- Opt out: `ods disable .` then `ods disable . --write`.
- Automation (`ods start` / `ods watch`) only rewrites paths when the workspace is enabled.

## How do I remove ODS completely?

```bash
ods stop --unregister .   # if a service was registered
ods disable . --write
```

## Profile vs Tags

| | **Profile** | **Tags** |
| :--- | :--- | :--- |
| Means | Document **kind** (structure / expected H2s) | Cross-cutting **topics** |
| Unknown | Warning | Always allowed |
| CLI | `ods profiles` | `ods tags`, `ods tag list`/`show`, `ods find --tag` / `--key` |

## Should `depends` / `related` include `.md`?

Yes. Canonical Document references use editor-jumpable `.md` paths:

```yaml
depends:
  - website/subscription-service.md
```

## `ods lint` says everything is fine

That is success: the graph and links are consistent. No `.ods/ods-errors.md` (or legacy `ods-error.md`) should remain.

## How much RAM does `ods serve` use?

Local measurements on macOS:
- Empty ODS workspace: ~7.5 MB
- 1000-document workspace: ~17 MB

For low-memory environments, use polling mode:

```bash
ODS_LOW_MEMORY=1 ods serve --mode poll --memory-report --poll-secs 30 --root .
```
