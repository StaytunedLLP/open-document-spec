# Open Document Spec (ODS) GitHub Action

[![GitHub Marketplace](https://img.shields.io/badge/Marketplace-Open--Document--Spec-blue.svg?logo=github)](https://github.com/marketplace)
[![Compliance](https://img.shields.io/badge/ODS-Level--3-green.svg)](https://github.com/StaytunedLLP/open-document-spec)

Official GitHub Action for **Open Document Spec (`ods`)**. Installs the native `ods` CLI and validates ODS documentation graph integrity, frontmatter compliance, profile schemas, index freshness, and health diagnostics across any repository stack (Rust, Node, Python, Go, C++, or pure Markdown). Google OKF v0.2 knowledge bundles can be validated via `ods lint --okf`.

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
- name: Run Open Document Spec Lint Gate
  uses: StaytunedLLP/open-document-spec@v1
  with:
    # Target release version tag ('latest', 'v0.0.13', '0.0.13')
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
    extra-args: ''
```

---

## Examples

### 1. Setup Only Mode

Installs `ods` into `PATH` so subsequent workflow steps can run custom subcommands:

```yaml
- name: Setup Open Document Spec CLI
  uses: StaytunedLLP/open-document-spec@v1
  with:
    command: '' # Setup mode

- name: Verify Index Freshness
  run: ods index --check

- name: Workspace Health Doctor
  run: ods doctor
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
| `ods-path` | Absolute path to installed `ods` binary |
| `ods-version` | Installed `ods` version string |
| `ods-path` | Deprecated alias of `ods-path` |
| `ods-version` | Deprecated alias of `ods-version` |

---

## Root markers

ODS workspaces use root `index.md` key `ods:` (spec version). Google OKF bundles use `okf_version: "0.2"`. See [frontmatter keys](../../docs/specs/frontmatter-keys-ods-vs-okf.md).
