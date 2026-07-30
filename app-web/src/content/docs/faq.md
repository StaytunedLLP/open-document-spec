---
title: "FAQ and Troubleshooting"
description: "Frequently asked questions about file extensions, adoption, profiles vs tags, renames, and background services."
status: "stable"
order: 11
ods:
  profile: "note"
  status: "stable"
---

# FAQ and Troubleshooting

Feature catalog: [Features](/docs/features).

## Do I need a special file extension?

No. Documents are plain `.md` only.

## Does installing ODS auto-make my Markdown compliant?

**No.** Without root `ods:`, tools do not rewrite your tree. Plain Markdown is Level 0.

- Opt in: `odc ods init .` or `odc ods init . --adopt`, then `odc ods lint`.
- Opt out: `odc ods disable .` then `odc ods disable . --write`.
- Automation (`odc ods start` / `odc ods watch`) only rewrites paths when the workspace is enabled.

## How do I remove ODS completely?

```bash
odc ods stop --unregister .   # if a service was registered
odc ods disable . --write
```

## Profile vs Tags

| | **Profile** | **Tags** |
| :--- | :--- | :--- |
| Means | Document **kind** (structure / expected H2s) | Cross-cutting **topics** |
| Unknown | Warning | Always allowed |
| CLI | `odc ods profiles` | `odc ods tags`, `odc ods find --tag` |

## Should `depends` / `related` include `.md`?

Yes. Canonical Document references use editor-jumpable `.md` paths:

```yaml
depends:
  - website/subscription-service.md
```

## `odc ods lint` says everything is fine

That is success: the graph and links are consistent. No `.odc/odc-errors.md` (or legacy `ods-error.md`) should remain.

## How much RAM does `odc ods serve` use?

Local measurements on macOS:
- Empty ODS workspace: ~7.5 MB
- 1000-document workspace: ~17 MB

For low-memory environments, use polling mode:

```bash
ODS_LOW_MEMORY=1 odc ods serve --mode poll --memory-report --poll-secs 30 --root .
```
