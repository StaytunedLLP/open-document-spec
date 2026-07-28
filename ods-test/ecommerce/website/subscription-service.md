---
profile: feature
id: docs/guide/07-examples/ecommerce/website/subscription-service
status: stable
description: Subscription service engine managing customer reorders and schedules.
---

# Subscription Service Engine

## Goal

Automate recurring customer orders for skincare replenishments on 30/60/90 day cycles.

## Scope

- Customer subscription preference management.
- Integration with Stripe recurring payments.
- Automatic inventory allocation.

## Requirements

- Send email notification 3 days prior to renewal charge.
- Provide a customer dashboard to pause/modify cycles.
- Offer a 10% discount on subscription items.

## Acceptance Criteria

- Renewal cron job triggers payment processing successfully.
- Failed payment triggers standard dunning sequence (4 retries).

## Risks

- High churn due to billing errors — mitigated by early warning emails and grace periods.
