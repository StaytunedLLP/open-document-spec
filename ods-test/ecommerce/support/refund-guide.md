---
profile: guide
id: docs/guide/07-examples/ecommerce/support/refund-guide
status: stable
description: Customer support representative's workflow for issuing product refunds.
related:
  - ../website/cart-checkout.md
---

# Customer Refund Workflow Guide

## Overview

Process customer refund requests safely in our ecommerce dashboard.

## Prerequisites

- Access to the Stripe dashboard.
- Access to the customer support Zendesk console.

## Steps

1. Find the customer order in the admin portal by email.
2. Verify that the order is within the 30-day refund window.
3. Select "Issue Refund" and choose the reason (e.g., "Unhappy with product").
4. Confirm in Stripe that the transaction is marked as refunded.

## Troubleshooting

- **Refund fails in Stripe** — check if the original charge is older than 180 days (which requires manual bank transfer support).
