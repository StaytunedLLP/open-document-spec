---
title: "Secrets Isolation Use Case"
description: "Isolating confidential internal information using ODS 3-tier visibility control and share-cascading."
status: "stable"
order: 15
ods:
  profile: "note"
  status: "stable"
---

# Secrets & Data Isolation Use Case

Enterprise documentation repositories frequently contain mixed content: public feature specifications alongside internal DB connection setups, staging environment setups, and proprietary IP.

---

## 3-Tier Visibility Control (`share`)

- **`public`** (default): Included in `odc ods context`, `odc ods export`, and `odc ods share`.
- **`org`**: Excluded from `odc ods context`, `odc ods export`, and `odc ods share` by default.
- **`private`**: Strictly confidential IP & secrets. Excluded by default across all export surfaces.

```yaml
---
profile: index
share: private
description: Internal infrastructure & database cluster setups
---
```
