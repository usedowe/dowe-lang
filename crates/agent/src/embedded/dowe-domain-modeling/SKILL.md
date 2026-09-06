---
name: dowe-domain-modeling
description: "Use when turning a business description into a Dowe application architecture: bounded modules, entities, relations, invariants, permissions, workflows, endpoints, seeders, and views. Choose this skill for POS, CRM, ecommerce, reservations, or another domain-heavy product; pair it with dowe-server, dowe-views, dowe-theme, and dowe-core for source authoring."
---

# Dowe domain modeling

Use this skill before generating a domain-heavy Dowe application. It converts business intent into
an explicit model that an agent can implement and validate through Dowe's existing authoring skills.
It supplies four blueprints and a reusable modeling workflow; it does not add compiler syntax or
pretend that unsupported database behavior exists.

## When to use it

Use it when a request includes a business system, operating workflow, database design, roles,
permissions, lifecycle states, financial records, inventory, scheduling, or several related CRUD
surfaces. Use the closest blueprint:

- POS or retail operations: `references/pos.md`
- CRM, sales pipeline, or customer operations: `references/crm.md`
- Online catalog, cart, checkout, or fulfillment: `references/ecommerce.md`
- Appointments, rooms, tables, equipment, or resource booking: `references/reservations.md`

Load `references/workflow.md` for every domain-modeling task. Load one domain blueprint first, then
load another only when the product genuinely combines domains.

## Required transformation

Follow this order and keep the output reviewable at every stage:

`description -> modules -> entities -> relations -> invariants -> permissions -> workflows -> endpoints -> seeders -> views`

Do not jump from a product name directly to tables. A domain name is not enough to decide tenancy,
ownership, money, time, lifecycle, authorization, or concurrency.

## Workflow

1. Read `main.dowe`, existing imports, `theme.dowe`, and the relevant public skill references.
2. Capture the domain brief: actors, tenant boundary, business units, resources, money and currency,
   time zone, lifecycle states, integrations, reporting needs, retention, and success/error states.
3. Select a blueprint and mark which baseline modules are required, optional, or excluded.
4. Define modules with one owner each. Keep catalog, identity, transactions, inventory, scheduling,
   communication, and reporting separate when their invariants differ.
5. Define entities and fields. Choose supported Dowe field types, primary identifiers, required
   fields, indexes, and uniqueness checks. Group related declarations into one bounded
   `server/entities/<domain>-entities.dowe` module and import its named bindings together into the
   Database config. Do not generate one file per entity by default. Keep an entity in its own file
   only when it is genuinely isolated, and split a module when ownership, lifecycle, authorization,
   or file size makes the boundary clearer.
6. Define relations as scalar identifier fields and write the parent-existence, tenant, duplicate,
   and authorization checks that repositories or services must perform.
7. Register invariants and lifecycle transitions. For each rule, name its enforcing service,
   repository, handler, or database operation; never leave a critical rule as prose only.
8. Define roles and permissions before endpoints. Every mutating endpoint needs an authorization
   owner and a safe response shape.
9. Define workflows and idempotency keys. Identify which writes must share a supported transaction
   and which states allow retries, cancellation, refund, expiration, or recovery.
10. Define endpoint groups, typed inputs, responses, errors, and server module ownership. Generate
    explicit handlers, services, repositories, and middleware with `dowe-server`.
11. Define deterministic static seeders for roles, permissions, reference data, and safe demo data.
12. Define view routes and page states with `dowe-views`: loading, empty, error, unauthorized,
    read-only, create, edit, confirmation, and destructive-action states. Use `dowe-theme` for
    repeated visual policy. When creating or explicitly changing `theme.dowe`, use grouped
    `colors:` families with `color`, `text`, and `title`; when the task is page-only, preserve the
    existing theme and consume its semantic tokens.
13. Run compiler validation, review the import graph, and re-check the domain decision list before
    considering the model complete.

## Artifact contract

The modeling pass must produce these artifacts, even if some are kept as a plan before source is
written:

| Artifact | Minimum content |
| --- | --- |
| Domain brief | Actors, tenant boundary, business units, resources, assumptions, unresolved decisions |
| Module map | Module name, owner, dependencies, data ownership, public workflows |
| Entity plan | Fields, supported types, identifiers, indexes, lifecycle, sensitive data |
| Relation matrix | From entity, scalar id, target, cardinality, parent check, tenant check, cleanup rule |
| Invariant register | Rule, trigger, enforcement owner, failure response, test case |
| Permission matrix | Role, permission, scope, read/write/delete/export behavior |
| Workflow map | States, transitions, actor, side effects, transaction, retry and idempotency policy |
| Endpoint plan | Method, path, handler, input type, authorization, response, errors |
| Seed plan | Static reference data, stable ids, ordering, fingerprint-safe values |
| View inventory | Route, page, data owner, actions, states, responsive and accessibility requirements |

## Dowe contract guardrails

- Use only current Dowe Source Format and the installed compiler diagnostics as syntax authority.
- Use `entity`, `seeder`, `database`, `query`, typed request bodies, and explicit server functions as
  documented by `dowe-server`; do not invent a model generator command or ORM declarations.
- Current entity fields are `string`, `bool`, `int`, `number`, `decimal`, `timestamp`, and `json`,
  with `primary`, `required`, `unique`, and `index` constraints.
- Relations use fields such as `customerId`, `tenantId`, or `productId` plus explicit server checks.
  Do not emit `belongsTo`, `hasMany`, `references`, foreign-key, cascade, or composite-unique syntax.
- Composite uniqueness, overlap checks, totals, status transitions, tenant isolation, and permission
  decisions belong in explicit server logic unless a current Dowe contract says otherwise.
- Use `decimal` for currency and calculate authoritative totals on the server. Never trust client
  totals, price, tax, stock, role, tenant, payment, or status fields.
- Keep secrets, provider credentials, payment data, and persistence handles server-only.
- Do not infer CRUD from entity names or route names. Declare every table, operation, filter, and
  authorization path explicitly.
- Group by cohesive domain ownership, not by a target file count. Keep join entities beside the
  domain they connect, and never create an unrelated catch-all entity module merely to reduce files.
- If a blueprint requires compiler or runtime behavior that is not currently supported, record it
  as an unresolved decision or implementation prerequisite instead of generating fictional source.

## Companion skills

- Use `dowe-core` for project roots, imports, shared types, diagnostics, and validation workflow.
- Use `dowe-server` for entities, Database config, repositories, services, handlers, middleware,
  seeders, routes, and runtime behavior.
- Use `dowe-views` for pages, layouts, route graphs, forms, collections, responsive states, and
  requests.
- Use `dowe-theme` for semantic colors, typography, repeated component defaults, and target-neutral
  visual policy. Its grouped-role contract is mandatory: every theme family uses `color`, `text`,
  and `title` under `colors:`.

## References

- Shared process, artifact contract, and validation gates: `references/workflow.md`
- Point of sale and retail operations: `references/pos.md`
- Customer relationship management: `references/crm.md`
- Ecommerce and fulfillment: `references/ecommerce.md`
- Reservations and resource scheduling: `references/reservations.md`

## Completion gate

Stop before implementation when a critical decision is missing about tenant scope, actor authority,
money, time zone, lifecycle, inventory or capacity, authorization, idempotency, or data retention.
Otherwise hand the approved model to the companion skills and validate the complete Dowe import graph.
