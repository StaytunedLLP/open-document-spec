# E-Commerce Platform Monolithic Documentation (Legacy Dump)

## Section 1: Overview & History
The E-Commerce platform was founded in 2021 to serve online retail customers. It consists of multiple subsystems including order processing, payments, inventory, user authentication, cart management, shipping, notifications, and analytics. Over the past 4 years, many developers have worked on this system.

## Section 2: User Authentication System
User authentication uses JWT tokens with RSA-256 signatures. Passwords are hashed using bcrypt with cost factor 12. Refresh tokens are stored in Redis with a 7-day TTL.
Database credentials for staging:
DB_USER=staging_user
DB_PASS=secret_pass_12345!
DB_HOST=staging-db.internal

## Section 3: Payment Gateway Integrations
Stripe and PayPal are used for processing payments. Webhooks handle asynchronous events like charge.succeeded and payment_intent.payment_failed.
Internal API keys:
STRIPE_SECRET_KEY=sk_test_51Mz...
PAYPAL_SECRET=secret_pp_key_98765

## Section 4: Checkout Flow & API Details
To create an order, send a POST request to `/api/v1/orders` with cart items.
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

## Section 5: Inventory Management Rules
Inventory is reserved for 15 minutes when an order item is added to cart. If checkout is not completed, background worker releases the reservation back to available stock.

## Section 6: Analytics & Tracking Events
Every cart action triggers segment events: `Cart Viewed`, `Item Added`, `Checkout Started`, `Payment Information Submitted`, `Order Completed`.

## Section 7: Deployment & Operations SOP
To deploy services to AWS ECS, run `helm upgrade --install ecommerce ./helm`. Ensure Kubernetes cluster context is set to `prod-cluster-us-east-1`.
