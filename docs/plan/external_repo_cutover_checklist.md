---
profile: plan
status: stable
share: public
description: Manual cutover checklist for the few existing ODS workspaces moving to OpenDocify (odc) CLI.
---

# External repo cutover checklist (≈3 ODS workspaces)

Do this **per external repository** that already uses ODS root markers. This monorepo is already migrated.

## 0. Publish OpenDocify release (once, this monorepo)

After merging multi-spec work to `main`:

1. Ensure CI (`pr.yml`) is green on `main`.
2. Trigger **`release.yml`** (`workflow_dispatch` or auto after CI success on main).
3. Confirm release assets include **`odc-v*-linux-x86_64.tar.gz`** (and platform peers) plus optional legacy **`ods-v*-…`**.
4. Smoke: `gh release view <tag> --json assets --jq '.assets[].name'`

Local dry-run without publishing:

```bash
./src/scripts/package-local-release.sh
# → dist-local/odc-vX.Y.Z-linux-x86_64.tar.gz
```

## 1. Install CLI

```bash
# After a GitHub Release publishes odc-* assets:
export GH_TOKEN="$(gh auth token)"   # private repo
curl -fsSL -H "Authorization: Bearer ${GH_TOKEN}" \
  https://raw.githubusercontent.com/StaytunedLLP/open-document-spec/main/src/scripts/install.sh | bash
odc --version
```

## 2. CI

Replace:

```yaml
- run: ods lint
- run: ods index --check
```

With:

```yaml
- run: odc lint            # auto-detect (preferred)
- run: odc index --check
# or explicit: odc ods lint / odc ods index --check
# legacy argv0: ods lint (ODS only) if the `ods` symlink is on PATH
```

GitHub Action consumers: continue using the composite action; it installs **`odc`** and runs lint (ODS namespace when needed).

## 3. Root `index.md`

- Keep **`ods:`** (spec). CLI pin is **`odc: ">=x.y.z"`** — replace any legacy **`ods-cli:`** (`odc upgrade --write` can rewrite).
- Bump `odc:` if your policy requires a minimum CLI with multi-spec support.
- OKF roots use **`okf_version: "0.2"`** only (do not invent `okf:`).

## 4. Validate

```bash
odc lint .
odc doctor .
odc audit --write-report   # → .odc/odc-errors.md
odc upgrade --write        # ods-cli→odc, ~/.ods→~/.odc hints
```

## 5. Agents (optional)

```bash
odc agents sync .
# commit AGENTS.md if desired
```

## 6. OKF (only if that repo holds knowledge bundles)

```bash
odc okf init .    # only for new OKF roots — do not mix blindly
odc okf lint .
```

## Owners

List the three repos and owners here when known:

| Repo | Owner | Done |
|---|---|---|
| _(fill in)_ | | |
| | | |
| | | |
