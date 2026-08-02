# Cross-Repo Services & Internal Infrastructure Secrets

## Public API Specification: Inventory Lookup
API Endpoint: `GET /api/v1/inventory/{sku}`
Response: `{"sku": "SKU-101", "quantity": 42}`

## Internal Infrastructure Connection Strings (DO NOT SHARE)
Production Postgres: `postgres://admin:SuperSecretPass2026@prod-db.internal:5432/commerce`
Redis Cache Cluster: `redis://:RedisAuthPass99@redis-cluster.internal:6379/0`
Vault Enterprise Key: `s.v38a7d9f8a7s9d8a7f9d`
profile md files?