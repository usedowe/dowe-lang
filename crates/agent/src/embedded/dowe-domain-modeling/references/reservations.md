# Reservations and resource scheduling blueprint

Use for appointments, rooms, tables, equipment, vehicles, classes, staff schedules, or any product
where a resource is held for a time interval or capacity window.

## Modules

| Module | Responsibility | Baseline |
| --- | --- | --- |
| Organization and access | Tenant, branches, staff, roles, permissions | Required |
| Resources | Resource types, individual units, capacity, location | Required |
| Availability | Rules, opening hours, blackout periods, exceptions | Required |
| Customers | Customer identity, guests, contact preferences | Required |
| Reservations | Holds, bookings, participants, status transitions | Required |
| Payments and policy | Deposits, payments, cancellation rules, refunds | Required when money is collected |
| Operations | Check-in, check-out, no-show, assignments | Optional |
| Communication and audit | Reminders, provider references, immutable events | Required for controlled booking |

## Baseline entity plan

| Entity | Important fields | Key rules |
| --- | --- | --- |
| `Tenant` | `id`, `name`, `active`, `createdAt` | Root scheduling boundary |
| `Branch` | `id`, `tenantId`, `name`, `timezone`, `active` | Timezone is explicit and stable |
| `User` | `id`, `tenantId`, `email`, `active` | Staff identity and authorization |
| `ResourceType` | `id`, `tenantId`, `name`, `capacity`, `active` | Defines bookable class and policy |
| `Resource` | `id`, `tenantId`, `branchId`, `resourceTypeId`, `name`, `status` | Resource belongs to one branch and type |
| `AvailabilityRule` | `id`, `tenantId`, `resourceTypeId`, `weekday`, `startTime`, `endTime`, `active` | Rule timezone and boundary are explicit |
| `Blackout` | `id`, `tenantId`, `resourceId`, `startsAt`, `endsAt`, `reason` | Blackouts block or explain capacity |
| `Customer` | `id`, `tenantId`, `name`, `email`, `active` | Consent and privacy policy required |
| `Reservation` | `id`, `tenantId`, `branchId`, `customerId`, `status`, `startsAt`, `endsAt`, `expiresAt`, `total` | Server validates interval, policy, and total |
| `ReservationItem` | `id`, `tenantId`, `reservationId`, `resourceId`, `quantity`, `unitPrice` | Item links reservation to resource |
| `ReservationGuest` | `id`, `tenantId`, `reservationId`, `name`, `data` | Only collect necessary guest data |
| `Payment` | `id`, `tenantId`, `reservationId`, `amount`, `status`, `providerRef` | Never store raw credentials |
| `CancellationPolicy` | `id`, `tenantId`, `name`, `cutoffMinutes`, `refundKind`, `refundValue` | Policy selection is server-owned |
| `ReservationEvent` | `id`, `tenantId`, `reservationId`, `actorId`, `event`, `data`, `createdAt` | Append-only lifecycle history |
| `AuditEvent` | `id`, `tenantId`, `actorId`, `event`, `entityType`, `entityId`, `data`, `createdAt` | Operational evidence |

## Relations and invariants

- All reads and writes are scoped by tenant and branch where applicable. Resource and customer
  identifiers from a request are checked before use.
- `startsAt` must precede `endsAt`; both are normalized to a documented timezone policy before the
  server evaluates availability.
- A confirmed reservation cannot overlap another confirmed reservation for an exclusive resource.
  Capacity-based resources need a sum-of-quantity rule rather than a simple overlap check.
- Holds expire and cannot be confirmed after expiration. Confirmation, cancellation, no-show, and
  check-in are explicit status transitions.
- Availability rules and blackouts are inputs to a server-owned availability calculation; a client
  cannot reserve by sending a guessed availability result.
- Payment totals, deposits, cancellation fees, and refunds are calculated by the server using the
  selected policy and currency precision.
- Overlap, capacity, and duplicate guest or participant checks require explicit service logic and a
  concurrency policy. They are not inferred from entity names or relations.

## Roles and permissions

| Role | Typical capabilities |
| --- | --- |
| Owner | Tenant settings, resources, availability, policies, reports |
| Scheduler manager | Staff schedules, resource assignment, overrides, cancellations |
| Booking agent | Create and edit customer reservations within assigned scope |
| Front-desk staff | Check-in, check-out, no-show, operational notes |
| Finance operator | Deposits, payments, refunds, reconciliation |
| Customer | Search availability, hold, confirm, view, cancel within policy |

Decide whether customers may reschedule, whether staff may override blackouts, and who can waive a
cancellation fee.

## Workflows

1. Search availability: validate interval and scope, calculate available resources, and return only
   authorized, non-sensitive results.
2. Create hold: re-check availability, create an expiring hold with an idempotency key, and expose
   the expiry timestamp.
3. Confirm reservation: re-check hold ownership and expiry, recalculate price and policy, confirm
   payment when required, and append a lifecycle event.
4. Reschedule: validate new interval, preserve history, recompute policy and price, and avoid a gap
   or duplicate allocation during the transition.
5. Cancel: authorize actor, calculate refund or fee, transition status, release capacity, and audit
   the reason.
6. Operate: check in, check out, or mark no-show only from allowed states and with staff scope.

Overlap and capacity workflows need a provider-supported concurrency plan. If the current Database
contract cannot guarantee the required atomic check-and-write, keep a pending state and document the
required runtime or provider capability rather than claiming race-free booking.

## Endpoints

| Method | Path | Owner |
| --- | --- | --- |
| `GET` | `/api/availability` | Availability handler |
| `POST` | `/api/reservation-holds` | Reservation handler |
| `POST` | `/api/reservations` | Reservation handler |
| `GET` | `/api/reservations/:id` | Reservation handler |
| `POST` | `/api/reservations/:id/reschedule` | Reservation workflow handler |
| `POST` | `/api/reservations/:id/cancel` | Cancellation handler |
| `POST` | `/api/reservations/:id/check-in` | Operations handler |
| `POST` | `/api/resources/:id/blackouts` | Availability administration handler |

Use typed inputs, safe errors for unavailable capacity, and never return internal provider details.

## Seeders and views

Seed roles, permissions, one branch with an explicit timezone, resource types, a small resource set,
availability rules, and a safe cancellation policy. Keep real customer data and payment references
out of seeders.

Recommended views:

- `/book`: date/time and resource search, filters, loading, empty, and unavailable states;
- `/book/checkout`: customer data, hold countdown, policy, payment, pending, and failure states;
- `/account/reservations`: upcoming and past bookings, detail, reschedule, and cancellation;
- `/operations/calendar`: staff calendar, resource occupancy, check-in, check-out, and no-show;
- `/admin/resources`: resources, availability rules, blackouts, policy editing, and permissions;
- `/admin/reports`: utilization, cancellations, revenue, filters, and empty states.

## Optional extensions

Add recurring reservations, waitlists, deposits, memberships, staff calendars, group bookings,
capacity pools, external calendar sync, reminders, or dynamic pricing only after their overlap,
retry, privacy, and provider contracts are explicit.
