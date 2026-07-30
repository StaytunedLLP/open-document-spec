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
- run: odc ods lint
- run: odc ods index --check
# or keep `ods lint` if the `ods` symlink is on PATH
```

GitHub Action consumers: continue using the composite action; it now installs `odc` and runs `odc ods …`.

## 3. Root `index.md`

- Keep **`ods:`** and **`ods-cli:`** (do not rename keys).
- Bump `ods-cli: ">=x.y.z"` if your policy requires a minimum CLI with multi-spec support.

## 4. Validate

```bash
odc ods lint .
odc ods doctor .
odc ods audit --write-report   # optional inventory
odc upgrade --check            # optional machine readiness
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
