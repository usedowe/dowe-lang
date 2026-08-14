# Customer relationship management blueprint

Use for contact management, lead capture, sales pipelines, account management, support follow-up,
or customer success. Keep pipeline state separate from immutable activity history.

## Modules

| Module | Responsibility | Baseline |
| --- | --- | --- |
| Organization and access | Tenant, users, teams, roles, permissions | Required |
| Parties | Companies, contacts, addresses, relationships | Required |
| Lead capture | Leads, sources, qualification, ownership | Required for sales CRM |
| Pipeline | Pipelines, stages, deals, assignments | Required for sales CRM |
| Activities | Calls, meetings, tasks, notes, reminders | Required |
| Communication | Messages, templates, provider references | Optional |
| Reporting and audit | Stage history, attribution, operational evidence | Required for controlled sales |

## Baseline entity plan

| Entity | Important fields | Key rules |
| --- | --- | --- |
| `Tenant` | `id`, `name`, `active`, `createdAt` | Root access boundary |
| `User` | `id`, `tenantId`, `email`, `active` | Email uniqueness is tenant-scoped |
| `Team` | `id`, `tenantId`, `name`, `active` | Team belongs to one tenant |
| `UserTeam` | `id`, `tenantId`, `userId`, `teamId` | Explicit parent and duplicate checks |
| `Company` | `id`, `tenantId`, `name`, `ownerId`, `status` | Ownership and visibility are explicit |
| `Contact` | `id`, `tenantId`, `companyId`, `name`, `email`, `ownerId` | Contact may belong to a company but remains tenant-scoped |
| `Lead` | `id`, `tenantId`, `contactId`, `source`, `status`, `ownerId` | Lead conversion is a controlled workflow |
| `Pipeline` | `id`, `tenantId`, `name`, `active` | Pipeline is tenant-scoped |
| `Stage` | `id`, `tenantId`, `pipelineId`, `name`, `position`, `closedKind` | Stage belongs to one pipeline |
| `Deal` | `id`, `tenantId`, `pipelineId`, `stageId`, `companyId`, `amount`, `currency`, `ownerId`, `status` | Stage and pipeline must agree; amount is decimal |
| `DealContact` | `id`, `tenantId`, `dealId`, `contactId`, `kind` | Join membership is checked explicitly |
| `Activity` | `id`, `tenantId`, `ownerId`, `subjectType`, `subjectId`, `kind`, `status`, `dueAt` | Activity subject must be authorized |
| `Note` | `id`, `tenantId`, `authorId`, `subjectType`, `subjectId`, `body` | Sensitive notes require scoped access |
| `Task` | `id`, `tenantId`, `assigneeId`, `subjectType`, `subjectId`, `status`, `dueAt` | Assignment and completion are server-owned |
| `Tag` | `id`, `tenantId`, `name` | Tag name is tenant-scoped |
| `ContactTag` | `id`, `tenantId`, `contactId`, `tagId` | Duplicate membership is checked in service logic |
| `StageEvent` | `id`, `tenantId`, `dealId`, `fromStageId`, `toStageId`, `actorId`, `createdAt` | Append-only pipeline history |
| `AuditEvent` | `id`, `tenantId`, `actorId`, `event`, `entityType`, `entityId`, `data`, `createdAt` | Operational evidence |

## Relations and invariants

- Every record is filtered by authenticated tenant and then by team, owner, or explicit sharing
  scope. A hidden view action is never an authorization boundary.
- A deal's `stageId` must belong to its `pipelineId`; a closed stage requires a close reason and
  close timestamp according to the chosen contract.
- Lead conversion must be idempotent and must not create duplicate company, contact, or deal data
  when a retry follows a successful write.
- Activities, notes, and tasks use scalar subject identifiers. The service checks subject type,
  existence, tenant ownership, and actor permission.
- Composite uniqueness such as `(tenantId, email)` or `(dealId, contactId)` requires an explicit
  repository or service check.
- Amounts, forecasts, stage changes, owner changes, and exports are server-authoritative and audit-
  visible where the business requires it.

## Roles and permissions

| Role | Typical capabilities |
| --- | --- |
| Admin | Users, teams, roles, pipelines, all tenant records |
| Sales manager | Pipeline design, assignment, approvals, team records, reports |
| Sales representative | Owned leads, contacts, deals, activities, notes |
| Support agent | Assigned contacts, tasks, notes, customer history |
| Analyst | Read-only reports and permitted exports |
| Auditor | Read-only operational and audit history |

Define whether managers can see all team records, whether reps can share records, and whether notes
or exports have stricter scopes.

## Workflows

1. Capture lead: validate source and consent fields, assign owner, create a lead, and audit origin.
2. Qualify lead: record qualification outcome, next action, and owner; reject or disqualify with a
   reason.
3. Convert lead: create or link company and contact, create an optional deal, preserve the lead
   history, and use an idempotency key.
4. Move deal: verify pipeline and stage transition, calculate any required forecast fields, append
   `StageEvent`, and audit the actor.
5. Complete activity: verify assignment and subject visibility, update task or activity status,
   and schedule the next action when requested.

## Endpoints

| Method | Path | Owner |
| --- | --- | --- |
| `GET` | `/api/contacts` | Contacts handler |
| `POST` | `/api/leads` | Leads handler |
| `POST` | `/api/leads/:id/convert` | Lead conversion handler |
| `GET` | `/api/pipelines/:id/deals` | Pipeline handler |
| `POST` | `/api/deals` | Deals handler |
| `POST` | `/api/deals/:id/stage` | Deal workflow handler |
| `POST` | `/api/activities` | Activities handler |
| `GET` | `/api/reports/pipeline` | Reporting handler |

Use typed inputs, safe error codes, server-side owner and tenant checks, and explicit pagination.

## Seeders and views

Seed roles, permissions, one pipeline, ordered stages, safe activity kinds, and small demo records.
Do not seed real contacts, provider credentials, or private notes.

Recommended views:

- `/crm/dashboard`: pipeline metrics, tasks, activity feed, loading and permission states;
- `/crm/contacts`: searchable list, profile, company relationship, notes, and empty state;
- `/crm/leads`: qualification queue, assignment, conversion confirmation, and failure state;
- `/crm/pipeline`: stage columns, deal details, movement action, and responsive fallback;
- `/crm/activities`: agenda, filters, create/edit, completion, and overdue states;
- `/crm/settings`: teams, roles, pipelines, stages, and permission-aware administration.

## Optional extensions

Add email sync, call logging, automation rules, custom fields, scoring, territory management,
consent history, or customer support cases only after their provider, privacy, retry, and audit
contracts are explicit.
