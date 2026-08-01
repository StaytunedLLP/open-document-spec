# ODS Context & Token Reduction Benchmarks

This directory contains test cases and an automated benchmark script comparing **unstructured full-folder documentation dumps (`without-ods/`)** against **targeted ODS graph context resolution (`with-ods/`)**.

---

## Directory Layout

```text
ods-test/benchmarks/
├── README.md                        # Benchmark guide (this file)
├── without-ods/                     # Flat, un-indexed documentation dumps
│   ├── example1-monolith/           # Legacy e-commerce mega doc
│   ├── example2-packs/              # Mixed secrets and specs
│   └── example3-compliance/         # Unstructured compliance handbook
└── with-ods/                        # Graph-native ODS workspaces
    ├── example1-monolith/           # Decomposed checkout & auth docs
    ├── example2-packs/              # Public API vs share: private secrets
    └── example3-compliance/         # Target onboarding SOP & policies
```

---

## Generating "Without ODS" Baselines Automatically

You can convert any ODS workspace into a clean, un-biased "Without ODS" baseline for benchmarking by running:

```bash
ods ods bench strip --write --full
```

This command backs up all frontmatters, index lockfiles (`index.md`), and custom profile schemas into `$HOME/.ods/backups/<repo-hash>/`, deletes non-root `index.md` files, removes `ods:` keys, and strips frontmatters. To restore the workspace back to 100% compliant ODS state, run `ods ods bench restore`.

Run the native Rust benchmark evaluation tool or CLI benchmark command:

```bash
ods ods bench stats
# Or run the Rust benchmark evaluation binary:
cargo run --example benchmark_eval
```

### Sample Output

```text
=========================================================================https://github.com/StaytunedLLP/open-document-spec
               ODS CONTEXT & TOKEN SAVINGS BENCHMARK REPORT             
=========================================================================

Example Scenario                 | Without ODS  | With ODS   | Token Saving | Reduction %
----------------------------------------------------------------------------------------
Monolith E-Commerce Setup        |      412 tok |     104 tok |      308 tok |      74.8%
Cross-Repo Pack Sharing & Secrets|       62 tok |      28 tok |       34 tok |      54.8%
SOP & Onboarding Compliance      |      154 tok |      91 tok |       63 tok |      40.9%
----------------------------------------------------------------------------------------
TOTAL (Cumulative Across Queries)|      628 tok |     223 tok |      405 tok |      64.5%
=========================================================================================

Enterprise Daily API Cost (100 devs @ 20 queries/day):
  - Without ODS:  $6.28 / day
  - With ODS:     $2.23 / day
  - Daily Saving: $4.05 / day
  - Annual Savings (250 days): $1,012.50 / year
```

---

## How to Interpret the Metrics

- **Without ODS**: AI agents must swallow entire documentation directories or mega-files (wasting tokens on irrelevant sections and risking secret exposure).
- **With ODS**: `ods ods context` traverses only the target document and its explicit `depends:` prerequisites up to `max-depth`. Documents marked `share: private` are automatically skipped.
- **Reduction %**: Shows the percentage reduction in token volume achieved by loading only the necessary bounded context graph.
