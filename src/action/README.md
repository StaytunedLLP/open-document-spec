# OpenDocify (ODC) GitHub Action

[![GitHub Marketplace](https://img.shields.io/badge/Marketplace-OpenDocify-blue.svg?logo=github)](https://github.com/marketplace)
[![Compliance](https://img.shields.io/badge/ODS-Level--3-green.svg)](https://github.com/StaytunedLLP/open-document-spec)

Official GitHub Action for **OpenDocify (`odc`)**. Installs the CLI and validates ODS documentation graph integrity, frontmatter compliance, index freshness, and health diagnostics across any repository stack (Rust, Node, Python, Go, C++, or pure Markdown). OKF bundles can use setup-only mode then run `odc okf lint`.

---

## Usage

### Quick Start (2 Lines)

Add this step to `.github/workflows/ci.yml` or `.github/workflows/docs.yml`:

```yaml
- name: Verify Documentation Graph
  uses: StaytunedLLP/open-document-spec@v1
```

### Options & Inputs Reference

```yaml
- name: Run OpenDocify Lint Gate
  uses: StaytunedLLP/open-document-spec@v1
  with:
    # Target release version tag ('latest', 'v0.1.24', '0.1.24')
    version: 'latest'

    # Command ('lint', 'index-check', 'doctor', 'fmt-check', 'bench', 'export', or '' for setup-only)
    command: 'lint'

    # Path to workspace root directory
    path: '.'

    # Compliance level ('1' or '3')
    level: '3'

    # Output format ('text' or 'json')
    format: 'text'

    # GitHub Token (required for downloading release binaries in private repos)
    token: ${{ secrets.GITHUB_TOKEN }}

    # Enable inline PR code annotations for lint errors
    annotate: 'true'

    # Extra arguments passed to the CLI
    extra-args: '--canonical-refs'
```

---

## Examples

### 1. Setup Only Mode

Installs `odc` into `PATH` so subsequent workflow steps can run custom subcommands:

```yaml
- name: Setup OpenDocify CLI
  uses: StaytunedLLP/open-document-spec@v1
  with:
    command: '' # Setup mode

- name: Verify Index Freshness
  run: odc index --check
  # or explicit: odc ods index --check

- name: Workspace Health Doctor
  run: odc doctor
```

### 2. Multi-OS Matrix Testing

```yaml
jobs:
  validate-docs:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v7
      - uses: StaytunedLLP/open-document-spec@v1
        with:
          version: 'latest'
          command: 'lint'
```

---

## Outputs

| Output | Description |
|---|---|
| `odc-path` / `ods-path` | Absolute path to installed CLI (`ods-path` is a deprecated alias) |
| `odc-version` / `ods-version` | Installed version string |

---

## Root markers

ODS workspaces use root `index.md` keys `ods:` (spec) and `odc:` (CLI pin). OKF bundles use `okf_version: "0.2"`. See [frontmatter keys](../docs/specs/frontmatter-keys-ods-vs-okf.md).
