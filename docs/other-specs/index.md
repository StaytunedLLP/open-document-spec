---
profile: index
---

# Other specs (extra dialects in the `ods` CLI)

ODS is the **default** product of the `ods` CLI. Other dialects are enabled **only with flags** (never namespaces, never `--ods`):

| Spec | Flag | Notes |
|---|---|---|
| **ODS** | *(none — default)* | Document graphs, profiles, indexes |
| **OKF v0.2** | `--okf` | Knowledge bundles (`okf_version`) |
| **Agent Skills** | `--skills` | `SKILL.md` packages |

- [agentskills.md](agentskills.md)
- [cli-multi-spec.md](cli-multi-spec.md)
- [frontmatter-keys-ods-vs-okf.md](frontmatter-keys-ods-vs-okf.md) - Comparison of ODS vs Google OKF v0.2 frontmatter keys — which keys exist in which spec, purpose of each family, root markers, and seamless ods CLI.
- [okf-0.2.md](okf-0.2.md)
