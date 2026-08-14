# Point-of-sale blueprint

Use for retail stores, restaurants, kiosks, service counters, or branch-based selling. Start with
the smallest operating model and add optional modules only when the brief requires them.

## Modules

| Module | Responsibility | Baseline |
| --- | --- | --- |
| Organization and access | Tenant, branches, users, roles, permissions | Required |
| Catalog and pricing | Products, categories, prices, taxes, discounts | Required |
| Customers | Customer identity, contact, loyalty reference | Optional |
| Inventory | Warehouses, stock, movements, suppliers | Required when stock is tracked |
| Sales | Sale header, lines, payments, refunds | Required |
| Cash | Registers, sessions, opening, closing, movements | Required for physical tills |
| Reporting and audit | Immutable operational events and read models | Required for controlled operations |

## Baseline entity plan

All tenant-owned records carry `tenantId`; add `branchId` or `warehouseId` where the scope requires
it. Use stable scalar ids and indexes for filtering.

| Entity | Important fields | Key rules |
| --- | --- | --- |
| `Tenant` | `id`, `name`, `active`, `createdAt` | Tenant is the root authorization boundary |
| `Branch` | `id`, `tenantId`, `name`, `active` | Name or code uniqueness is checked per tenant |
| `User` | `id`, `tenantId`, `email`, `active` | Email uniqueness is checked per tenant |
| `Role` | `id`, `tenantId`, `name` | Role names are scoped to a tenant |
| `Permission` | `id`, `key`, `description` | Permission keys are stable reference data |
| `UserRole` | `id`, `tenantId`, `userId`, `roleId` | Parent and duplicate membership checks are explicit |
| `Product` | `id`, `tenantId`, `sku`, `name`, `active`, `taxRate` | SKU uniqueness is checked per tenant |
| `ProductPrice` | `id`, `tenantId`, `productId`, `currency`, `amount`, `active` | Price authority stays on the server |
| `Warehouse` | `id`, `tenantId`, `branchId`, `name` | Warehouse must belong to the same tenant and branch |
| `StockItem` | `id`, `tenantId`, `warehouseId`, `productId`, `quantity` | Current quantity is never accepted from a client |
| `StockMovement` | `id`, `tenantId`, `warehouseId`, `productId`, `kind`, `quantity`, `reason` | Append movement history; validate sign and authority |
| `Customer` | `id`, `tenantId`, `name`, `email`, `active` | Customer identity is tenant-scoped and privacy-aware |
| `Sale` | `id`, `tenantId`, `branchId`, `cashSessionId`, `customerId`, `status`, `subtotal`, `tax`, `total` | Totals and status are server-derived |
| `SaleLine` | `id`, `tenantId`, `saleId`, `productId`, `quantity`, `unitPrice`, `tax`, `lineTotal` | Store a price snapshot for history |
| `Payment` | `id`, `tenantId`, `saleId`, `method`, `amount`, `status`, `providerRef` | Never store raw payment credentials |
| `Refund` | `id`, `tenantId`, `saleId`, `status`, `amount`, `reason` | Refund amount cannot exceed refundable balance |
| `CashRegister` | `id`, `tenantId`, `branchId`, `name`, `active` | Register belongs to one branch |
| `CashSession` | `id`, `tenantId`, `registerId`, `userId`, `status`, `openingAmount`, `closingAmount` | One open session per register is enforced by service logic |
| `CashMovement` | `id`, `tenantId`, `cashSessionId`, `kind`, `amount`, `reason` | Every adjustment has actor and reason |
| `AuditEvent` | `id`, `tenantId`, `actorId`, `event`, `entityType`, `entityId`, `data`, `createdAt` | Append-only operational evidence |

## Relations and invariants

- Every query includes the authenticated `tenantId`; branch and warehouse scope are checked again
  in the service.
- `SaleLine.saleId`, `Payment.saleId`, and `Refund.saleId` are scalar identifiers with indexes.
- A sale is finalized only when it has lines, server-calculated totals, an authorized payment, and
  available stock according to the chosen stock policy.
- Stock changes are represented by explicit movements. If current stock is materialized, update it
  through a server-owned workflow and document the consistency boundary.
- A paid sale is immutable except through a refund workflow. Do not let a client update totals,
  payment status, stock, or tenant ownership.
- Composite uniqueness such as `(tenantId, sku)` and `(registerId, open status)` requires explicit
  checks because the entity contract has no composite unique declaration.

## Roles and permissions

| Role | Typical capabilities |
| --- | --- |
| Owner | Tenant settings, users, roles, reports, all operations |
| Manager | Catalog, inventory, refunds, cash close, reports |
| Cashier | Open register, create sales, accept payments, view assigned customer data |
| Inventory clerk | Products, suppliers, stock counts, adjustments with reason |
| Auditor | Read-only sales, cash, inventory, and audit history |

Keep refund approval, stock adjustment, and cash closing separate when separation of duties matters.

## Workflows

1. Open cash session: authenticate actor, verify register scope and no existing open session, record
   opening amount, and audit the event.
2. Complete sale: validate product and price authority, calculate totals, create sale lines and
   payment, record stock movements, and make the operation retry-safe with a client idempotency key.
3. Refund: authorize actor, calculate refundable balance, create refund and reversal movements,
   then audit the decision.
4. Close cash session: stop new sales, calculate expected balance, record counted amount and
   variance, require manager approval when policy requires it.

Check the current Database transaction contract before promising atomicity across sale, payment,
and stock writes. Use explicit pending or failed states when a provider cannot cover the complete
workflow.

## Endpoints

| Method | Path | Owner |
| --- | --- | --- |
| `GET` | `/api/catalog/products` | Catalog handler |
| `POST` | `/api/sales` | Sales handler |
| `GET` | `/api/sales/:id` | Sales handler |
| `POST` | `/api/sales/:id/refunds` | Refund handler |
| `POST` | `/api/registers/:id/sessions` | Cash handler |
| `POST` | `/api/cash-sessions/:id/close` | Cash handler |
| `GET` | `/api/reports/sales` | Reporting handler |

Use typed request bodies, safe error codes, and server-owned totals and statuses.

## Seeders and views

Seed stable roles, permissions, a default tax configuration, one demo branch, one register, and
small demo products. Keep real credentials, payment references, and tenant secrets out of seeders.

Recommended views:

- `/pos/sell`: product search, cart, customer lookup, payment, success, and failure states;
- `/pos/products`: catalog list, create/edit, inactive state, and permission-aware actions;
- `/pos/inventory`: stock summary, movement history, adjustment confirmation, and empty state;
- `/pos/cash`: register sessions, close review, variance, and approval state;
- `/pos/reports`: date and branch filters, metrics, table, loading, empty, and export permission.

## Optional extensions

Add loyalty, bundles, serial numbers, kitchen tickets, purchase orders, multi-currency, or
promotions only after their ownership, pricing, tax, refund, and audit rules are specified.
