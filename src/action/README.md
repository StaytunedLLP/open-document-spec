# ODS (Open Document Specs) GitHub Action

[![GitHub Marketplace](https://img.shields.io/badge/Marketplace-ODS%20Action-blue.svg?logo=github)](https://github.com/marketplace)
[![Compliance](https://img.shields.io/badge/ODS-Level--3-green.svg)](https://github.com/StaytunedLLP/open-document-spec)

Official GitHub Action for **ODS (Open Document Specs)**. Install `ods` CLI and validate documentation graph integrity, frontmatter compliance, index freshness, and health diagnostics across any repository stack (Rust, Node, Python, Go, C++, or pure Markdown).

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
- name: Run ODS Lint Gate
  uses: StaytunedLLP/open-document-spec@v1
  with:
    # Target release version tag ('latest', 'v0.1.24', '0.1.24')
    version: 'latest'

    # ODS command ('lint', 'index-check', 'doctor', 'fmt-check', 'bench', 'export', or '' for setup-only)
    command: 'lint'

    # Path to ODS workspace root directory
    path: '.'

    # Compliance level ('1' or '3')
    level: '3'

    # Output format ('text' or 'json')
    format: 'text'

    # GitHub Token (required for downloading release binaries in private repos)
    token: ${{ secrets.GITHUB_TOKEN }}

    # Enable inline PR code annotations for lint errors
    annotate: 'true'

    # Extra arguments passed to ODS CLI
    extra-args: '--canonical-refs'
```

---

## Examples

### 1. Setup Only Mode
Installs `ods` into `PATH` so subsequent workflow steps can execute custom scripts or subcommands:

```yaml
- name: Setup ODS CLI
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
      - uses: actions/checkout@v4
      - uses: StaytunedLLP/open-document-spec@v1
        with:
          version: 'latest'
          command: 'lint'
```

---

## Features

- ⚡ **Zero Dependencies**: Prebuilt multi-OS binary installation (Linux `x86_64`/`arm64`, macOS `x86_64`/`arm64`, Windows `x86_64`).
- 🎯 **Inline PR Annotations**: GitHub Problem Matcher highlights broken references directly on PR diff lines.
- 🔒 **Secure**: Automatic SHA256 checksum verification for downloaded release binaries.
- 🛠️ **Private & Public Repos**: Authenticates via `GITHUB_TOKEN` to fetch release assets safely.

---

## License

Proprietary — Copyright (c) Staytuned LLP.
