# Ecommerce blueprint

Use for online catalogs, carts, checkout, orders, payments, fulfillment, subscriptions, or
marketplace-style selling. Treat an order as a historical business record, not a live product view.

## Modules

| Module | Responsibility | Baseline |
| --- | --- | --- |
| Store and access | Tenant, staff, roles, permissions, channels | Required |
| Catalog | Products, variants, categories, media references | Required |
| Pricing and promotions | Prices, currencies, coupons, tax policy | Required when pricing varies |
| Customer | Accounts, addresses, consent, preferences | Required |
| Cart and checkout | Cart state, checkout intent, idempotency | Required |
| Orders and payments | Historical order, lines, payment attempts, refunds | Required |
| Fulfillment | Shipments, delivery status, returns | Required for physical goods |
| Reporting and audit | Operational evidence and business reporting | Required |

## Baseline entity plan

| Entity | Important fields | Key rules |
| --- | --- | --- |
| `Tenant` | `id`, `name`, `active`, `createdAt` | Root store boundary |
| `User` | `id`, `tenantId`, `email`, `active` | Staff identity and authorization |
| `Product` | `id`, `tenantId`, `name`, `description`, `active` | Product is not the purchasable stock unit when variants exist |
| `ProductVariant` | `id`, `tenantId`, `productId`, `sku`, `name`, `active` | SKU uniqueness is tenant-scoped |
| `Category` | `id`, `tenantId`, `name`, `active` | Category visibility is explicit |
| `ProductCategory` | `id`, `tenantId`, `productId`, `categoryId` | Join duplicate checks are explicit |
| `Price` | `id`, `tenantId`, `variantId`, `currency`, `amount`, `active`, `startsAt`, `endsAt` | Server chooses applicable price |
| `InventoryItem` | `id`, `tenantId`, `variantId`, `locationId`, `available`, `reserved` | Client cannot set availability |
| `Cart` | `id`, `tenantId`, `customerId`, `status`, `currency`, `expiresAt` | Cart ownership and expiry are server-controlled |
| `CartLine` | `id`, `tenantId`, `cartId`, `variantId`, `quantity`, `unitPrice` | Reprice before checkout |
| `Customer` | `id`, `tenantId`, `email`, `name`, `active` | Privacy and identity merge policy required |
| `Address` | `id`, `tenantId`, `customerId`, `kind`, `data` | Validate allowed fields and protect sensitive data |
| `Order` | `id`, `tenantId`, `customerId`, `status`, `currency`, `subtotal`, `tax`, `shipping`, `total` | Immutable totals after confirmation policy |
| `OrderLine` | `id`, `tenantId`, `orderId`, `variantId`, `skuSnapshot`, `nameSnapshot`, `quantity`, `unitPrice`, `lineTotal` | Snapshot historical catalog values |
| `PaymentAttempt` | `id`, `tenantId`, `orderId`, `provider`, `providerRef`, `amount`, `status` | Never store raw card data |
| `Refund` | `id`, `tenantId`, `orderId`, `paymentAttemptId`, `amount`, `status`, `reason` | Cannot exceed captured refundable amount |
| `Shipment` | `id`, `tenantId`, `orderId`, `status`, `trackingRef` | Status transitions are explicit |
| `Coupon` | `id`, `tenantId`, `code`, `kind`, `value`, `active`, `startsAt`, `endsAt` | Code uniqueness and eligibility are server-checked |
| `CouponRedemption` | `id`, `tenantId`, `couponId`, `orderId`, `customerId` | Enforce usage limits in service logic |
| `AuditEvent` | `id`, `tenantId`, `actorId`, `event`, `entityType`, `entityId`, `data`, `createdAt` | Append-only evidence |

## Relations and invariants

- Product, price, cart, order, payment, shipment, and inventory queries are tenant-scoped.
- Checkout recalculates prices, taxes, shipping, discounts, totals, and availability on the server.
  The client may propose a coupon or quantity but cannot authorize the result.
- Order lines snapshot product name, SKU, price, and tax inputs so later catalog edits do not alter
  the order history.
- Inventory reservations, payment attempts, order status, and idempotency keys need an explicit
  concurrency policy. Do not promise exactly-once external payment behavior from a local retry.
- Coupon eligibility and usage limits require explicit checks; composite unique constraints are not
  available in the entity declaration.
- Store provider references, never payment credentials or secrets. Record only the minimum data
  required by the chosen payment contract.

## Roles and permissions

| Role | Typical capabilities |
| --- | --- |
| Store owner | Full tenant configuration, staff, catalog, pricing, orders, reports |
| Catalog manager | Products, variants, categories, prices, media references |
| Fulfillment staff | Inventory, shipments, returns, operational order details |
| Support agent | Customer records, orders, permitted refunds, shipment status |
| Finance operator | Payments, refunds, reconciliation, reports |
| Customer | Own cart, checkout, orders, addresses, permitted returns |

Never expose staff or finance actions merely because a view route is hidden.

## Workflows

1. Browse and cart: expose active catalog data, validate cart ownership, and expire stale carts.
2. Checkout intent: reprice cart, validate address and stock, create an idempotent checkout intent,
   and show a safe pending state.
3. Payment confirmation: verify provider result server-side, transition the order, reserve or
   decrement inventory according to policy, and avoid duplicate fulfillment on retries.
4. Fulfillment: create shipment, advance status, record tracking reference, and notify through an
   explicit provider workflow.
5. Return or refund: validate policy and refundable balance, record return state, then create the
   refund and inventory disposition explicitly.

## Endpoints

| Method | Path | Owner |
| --- | --- | --- |
| `GET` | `/api/catalog` | Catalog handler |
| `GET` | `/api/products/:id` | Catalog handler |
| `GET` | `/api/cart` | Cart handler |
| `POST` | `/api/cart/lines` | Cart handler |
| `POST` | `/api/checkout` | Checkout handler |
| `POST` | `/api/payments/:id/confirm` | Payment handler |
| `GET` | `/api/orders/:id` | Orders handler |
| `POST` | `/api/orders/:id/returns` | Returns handler |
| `POST` | `/api/shipments/:id/status` | Fulfillment handler |

Use typed bodies, safe payment responses, explicit status errors, and server-derived totals.

## Seeders and views

Seed staff roles, permissions, one catalog taxonomy, safe demo products, prices, and a non-sensitive
demo coupon. Do not seed payment credentials, production customer data, or provider references.

Recommended views:

- `/shop`: catalog filters, product cards, loading, empty, and unavailable states;
- `/shop/products/:id`: variant selection, price, stock message, add-to-cart, and error state;
- `/cart`: line editing, repricing feedback, empty cart, and checkout action;
- `/checkout`: address, delivery, payment handoff, pending, success, and failure states;
- `/account/orders`: order history, detail, shipment, return request, and permission states;
- `/admin/orders`: queue, filters, fulfillment actions, refund confirmation, and audit details.

## Optional extensions

Add subscriptions, marketplace sellers, tax provider integration, internationalization, search,
wishlists, reviews, digital delivery, or returns logistics only after their ownership and provider
contracts are decided.
