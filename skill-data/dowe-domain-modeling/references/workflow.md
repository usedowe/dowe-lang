# Domain modeling workflow reference

Use this reference with every domain request. The four domain blueprints provide defaults; this
workflow decides what is actually safe and needed for the product.

## Domain brief

Capture these decisions before naming entities:

| Decision | Questions |
| --- | --- |
| Actors | Who operates the system, who receives output, and which actions are privileged? |
| Tenant | Is data isolated by account, organization, branch, workspace, or project? Can an actor belong to many? |
| Ownership | Which actor or business unit owns each record? Can ownership change? |
| Time | Which timezone defines dates, cutoffs, opening hours, expiration, and reporting? |
| Money | Which currency, precision, tax model, rounding rule, and refund policy apply? |
| Lifecycle | Which statuses exist, who may transition them, and which transitions are irreversible? |
| Concurrency | Which records may be edited or claimed concurrently? What prevents duplicate or overlapping work? |
| Idempotency | Which retries may repeat a charge, booking, sale, stock movement, or notification? |
| Audit | Which changes need actor, timestamp, reason, before/after data, or immutable history? |
| Retention | Which data can be deleted, anonymized, archived, or legally retained? |
| Integrations | Which providers are authoritative for identity, payment, tax, messaging, or fulfillment? |

Unknown answers become explicit decisions, not hidden defaults.

## Module boundaries

Give every module one data owner and one reason to change. A typical application separates:

- organization and access;
- catalog or resources;
- customers and parties;
- transactions or bookings;
- inventory, fulfillment, or capacity;
- money and settlement;
- communication and tasks;
- reporting and audit.

Do not create a module only because a table is large. Split when ownership, authorization, lifecycle,
or consistency rules differ.

## Entity design

For each entity record:

1. Stable id strategy and whether the compiler-generated id is acceptable.
2. Tenant and owner identifiers.
3. Required, optional, sensitive, immutable, and derived fields.
4. Supported Dowe field type and index needs.
5. Lifecycle fields such as `status`, `createdAt`, `updatedAt`, or `archivedAt` when the contract
   supports them.
6. Uniqueness rules, including the service-level check required for composite uniqueness.
7. Read and write authorities.

Use lower-snake-case table names generated from entity names. Keep derived totals, current stock,
and permission results server-owned. Use snapshots on historical transaction lines when a later
catalog change must not rewrite history.

### Entity module boundaries

Default to one source file per cohesive bounded module, not one source file per table. Put related
entities, their line or event records, and their join entities together under a plural file such as
`server/entities/user-entities.dowe`, `server/entities/kitchen-entities.dowe`, or
`server/entities/inventory-entities.dowe`. Import every named binding from that module in one
statement and register those bindings explicitly in the Database `entities` list.

Keep a dedicated file when an entity is genuinely isolated. Split a grouped module when declarations
have different owners, authorization or lifecycle rules, when they belong to different bounded
domains, or when the file would stop being focused. Do not make a single application-wide catch-all
file, and do not split every declaration automatically just because each entity maps to one table.

## Relation matrix

Represent each relation with a scalar identifier and an index when it is filtered or joined.

| Relation concern | Required output |
| --- | --- |
| Parent existence | Repository or service check before insert or update |
| Tenant boundary | `tenantId` check on both records and every query path |
| Cardinality | Explicit one-to-one, one-to-many, or join entity decision |
| Duplicate membership | Service check because composite unique constraints are not current entity syntax |
| Cleanup | Explicit archive, delete, or retention workflow; no implicit cascade |
| Authorization | Actor permission and record ownership check |

## Invariant register

Write each invariant in this form before source generation:

`When <trigger>, <actor> may/must <action> only if <condition>; otherwise return <safe error>.`

Attach an owner and a test scenario. Examples include:

- a sale cannot be finalized without at least one line and an authorized payment;
- a reservation cannot overlap another confirmed reservation for the same resource;
- a deal cannot move to a closed stage without a close reason;
- a client cannot choose another tenant, price, stock count, or role through request input.

## Permissions

Start with capabilities, then assign them to roles. Keep record scope explicit:

| Scope | Example |
| --- | --- |
| Tenant | Read all records in one organization |
| Branch or team | Read and write records for assigned units |
| Own records | Read or update records assigned to the actor |
| Record-specific | Approve a refund or close a cash session |
| Platform | Cross-tenant operations reserved for a separate trusted surface |

Enforce permissions in middleware for coarse access and services for record-level rules. Views may
hide actions, but server authorization remains authoritative.

## Workflows and consistency

Describe every workflow as states and transitions. For each transition specify actor, validation,
writes, side effects, retry key, and recovery path. Use the current Database transaction contract
where it covers the writes. If a workflow needs unsupported cross-table update/delete atomicity,
record that as an implementation prerequisite and use explicit pending or failed states rather than
claiming atomic behavior.

## Dowe artifact mapping

| Model artifact | Dowe location or surface |
| --- | --- |
| Entity schema | Cohesive `server/entities/*-entities.dowe` modules and `database ... entities:[...]` |
| Database connection | `server/config/*.dowe` |
| Static reference data | `server/seeders/*.dowe` and `database ... seeders:[...]` |
| Reusable data access | `server/repositories/*.dowe` |
| Business rules | `server/services/*.dowe` |
| HTTP boundary | `server/handlers/*.dowe`, `server/middlewares/*.dowe`, `server/endpoints.dowe` |
| Shared request or view shapes | Shared `type` modules according to the current project layout |
| Route and page inventory | `views/routes`, `views/layouts`, `views/pages`, `views/store`, `views/types` |
| Repeated visual policy | Root `theme.dowe` |

Keep data access out of views and keep server secrets out of view state.

## Validation gates

- Re-read the domain brief after creating the entity plan.
- Check every relation for tenant, parent, duplicate, and cleanup behavior.
- Check every mutating endpoint for authorization, input validation, safe errors, and idempotency.
- Check every money or quantity field for server authority and precision.
- Check every status transition for valid predecessors and forbidden skips.
- Check every view for loading, empty, error, unauthorized, and destructive-action states.
- Run the compiler against the complete import graph and use `dowe-server` and `dowe-views` for
  focused syntax and target validation.
