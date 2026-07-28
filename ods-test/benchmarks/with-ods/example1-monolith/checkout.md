---
profile: feature
status: stable
description: Checkout flow & API endpoint specification for placing orders
depends:
  - auth
---

# Checkout Flow Specification

## Endpoint: POST /api/v1/orders
Payload format:
```json
{
  "cart_id": "cart_123",
  "payment_method": "stripe",
  "shipping_address": {
    "line1": "123 Main St",
    "city": "Austin",
    "state": "TX",
    "zip": "78701"
  }
}
```
