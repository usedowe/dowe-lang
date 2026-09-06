# Database, Cache, and Vector reference

## Handles

| Utility | Binding | Props |
| --- | --- | --- |
| `database appDb` | Database connection | Required static `provider`; provider-specific `host`, `port`, `account`, `secret`, and `name`; optional imported `entities` and `seeders` |
| `cache appCache` | Cache connection | Required `provider`, `host`, `port`, `account`, `secret`, and `name` |
| `vector appVector` | Vector connection | Required `provider:"dowe"`, `host`, `port`, `account`, `secret`, and `name` |

Database providers are `postgres`, `d1`, and `dowe`. Postgres and Dowe require `host`, `port`,
`account`, `secret`, and `name`; D1 requires `account`, `secret`, and `name`. Connection values may
be static or server environment references; `provider` must be static and `name` must resolve
during compilation. `entities` and `seeders` contain imported or local bindings.

Cache providers are `kv` for Cloudflare KV, `redis` for Redis, and `dowe` for Dowe Cache. `provider`
may be a static provider or a server-only environment reference such as `env.CACHE_PROVIDER`.
Vector
initially supports only `dowe`. Config modules may export `database`, `cache`, and `vector`
bindings; import those bindings into repository functions instead of opening the same handle
repeatedly.

Database declarations can live in any imported server configuration module. Register handles that
belong to the project-wide migration and seeder catalog from `main.dowe`:

```text
import appDb from "@/server/config/database"

main
  server port:8080
    databases:[appDb]
```

`databases` accepts only imported Database handles. It does not copy connection props or expose
credentials. A registered handle may be unused by a route and is still available to `dowe database
migrate` and `dowe database seeders`; normal runtime handles remain available when discovered by
compiled server operations.

## Entities and seeders

Entity names become lower-snake-case table names by default. Field types are `string`, `bool`,
`int`, `number`, `decimal`, `timestamp`, and `json`. Constraints are `primary`, `required`,
`unique`, and `index`. When no field declares `primary:true`, an `id` field becomes primary
automatically.

```text
entity Users
  id:string primary:true
  name:string required:true
  email:string required:true unique:true
  active:bool required:true index:true
  createdAt:timestamp required:true
```

Seeders contain static entity inserts. The compiler validates entity references and field names and
assigns each seeder a deterministic fingerprint, so it runs once per database. Normal `dowe dev`
does not load or apply seeder modules; run `dowe database seeders` to compile the complete seeder
set and populate the local embedded databases under `.dowe/db`. Production applies each pending
seeder and its ledger record in one provider transaction.

```text
seeder Bootstrap
  insert entity:Users value:{ id:"01ARZ3NDEKTSV4RRFFQ69G5FAV" name:"Admin" email:"admin@example.com" active:true createdAt:"2026-01-01T00:00:00Z" }
```

Group related declarations into a focused bounded-domain module and import its bindings together.
Do not generate a separate file for every entity by default, and do not create an unrelated
application-wide catch-all. Keep an isolated entity in its own file or split a module when ownership,
lifecycle, authorization, or module size requires a clearer boundary:

```text
entity Users
  id:string primary:true
  name:string required:true
  email:string required:true unique:true

entity UserRoles
  id:string primary:true
  userId:string required:true index:true
  roleId:string required:true index:true
```

The declarations above belong together in `server/entities/user-entities.dowe`. Register both
named bindings through one module import:

```text
import Users, UserRoles from "@/server/entities/user-entities"

database appDb provider:"dowe" host:env.DATABASE_HOST port:env.DATABASE_PORT account:env.DATABASE_ACCOUNT secret:env.DATABASE_SECRET name:env.DATABASE_NAME entities:[Users UserRoles] seeders:[]
```

## Relations

Dowe models relations explicitly with identifier fields and server queries. There is no automatic
ORM relationship graph, `belongsTo` or `hasMany` helper, `references` prop, foreign-key declaration,
or cascade action in the current entity contract.

Use a scalar field containing the related record id, and index it when the field is used for
filtering or joining:

```text
entity Users
  id:string primary:true
  name:string required:true

entity Posts
  id:string primary:true
  authorId:string required:true index:true
  title:string required:true

entity Profiles
  id:string primary:true
  userId:string required:true unique:true index:true
  bio:string
```

These shapes represent one-to-many (`Posts.authorId` to `Users.id`) and one-to-one
(`Profiles.userId` to `Users.id`). The `unique:true` constraint limits one profile per user, but
the server still owns the check that the related user exists.

For many-to-many data, create a join entity with one indexed field for each side:

```text
entity UserRoles
  id:string primary:true
  userId:string required:true index:true
  roleId:string required:true index:true
```

Query related records with the supported SQL-like `query` operation and bound parameters:

```text
query rows conn:appDb.query sql:"SELECT posts.id, posts.title, users.name AS authorName FROM posts JOIN users ON posts.authorId = users.id WHERE posts.authorId = ?1" params:[req.params.userId]
```

`JOIN` results are ordinary server data. Validate parent existence, authorization, cleanup, and
duplicate join rows in server repositories or services; Database does not infer those rules from
field names. Composite unique constraints are not part of the current entity contract, so a
many-to-many membership needs an explicit server-side duplicate check.

## Database queries

Every Database operation uses `query <binding> conn:<handle>.<operation>`.

| Operation | Props |
| --- | --- |
| `list` | `table`; optional `where` |
| `read` | `table`; optional `where`, `required` |
| `insert` | `table`, `value` |
| `update` | `table`, `where`, `value`; optional `required` |
| `delete` | `table`, `where`; optional `required` |
| `query` | `sql`; optional scalar `params` bound as prepared statements |
| `tx` | Indented query children followed by `commit` or `rollback` |

```text
import appDb from "@/server/config/database"

fn createBlogRepository params:{ title:string content:string }
  query created conn:appDb.insert table:"blogs" value:{ title:args.title content:args.content }
  return value:created
```

An `insert` result exposes the fields of its `value` object plus the generated `id`; `update` and
`delete` results expose `changed`. A known invalid field reference fails during compilation.

Use Cloudflare's account and Database identifiers for D1:

```text
database appDb provider:"d1" account:env.ACCOUNT_ID secret:env.CLOUDFLARE_API_TOKEN name:env.DATABASE_ID entities:[Blog] seeders:[Bootstrap]
```

D1 supports the same compound equality filters and insert transaction form as the other providers.
Keep account, token, and Database ID in server-only environment variables. Bind request values
separately from SQL when a query needs custom filtering or pagination:

```text
query rows conn:appDb.query sql:"SELECT id, name FROM icons WHERE category = ?1 ORDER BY name LIMIT 60 OFFSET 0" params:[req.params.category]
```

Do not interpolate a request reference into `sql`. The runtime binds query parameters using the
selected provider's native placeholder rules.

`conn:<handle>.tx` is supported by PostgreSQL, D1, local development storage, and remote
`provider:"dowe"` handles. It stages literal `insert` children and ends with exactly one final
`commit` or `rollback`:

```text
query result conn:appDb.tx
  query delivery conn:appDb.insert table:"sms_deliveries" value:{ recipient:args.recipient status:"queued" }
  query outbox conn:appDb.insert table:"sms_outbox" value:{ recipient:args.recipient status:"pending" }
  commit value:delivery
```

Commit is atomic: Dowe records one checksummed WAL frame, PostgreSQL uses a native transaction, and
D1 uses one atomic batch. A conflict or invalid insert rejects the complete transaction. `rollback`
discards all staged inserts and returns `null`. Write requests are not automatically retried after a
transport failure; use a stable ULID or business idempotency key when the caller may retry.

During `dowe dev`, Dowe uses its embedded persistent Database under `.dowe/db/<name>` for every
provider and resolves only `name`; it does not start Wrangler or contact the authored provider.
In an interactive terminal, `dowe database` shows the complete Database command menu, including
`migrate` and `seeders`; commands with required arguments ask for them before dispatching through
the same CLI implementation as a typed command. Without a TTY, provide the subcommand explicitly.
`dowe database seeders` is the explicit local data bootstrap command. Run
`dowe database migrate` after entity changes; it maintains
`migrations/database.graph.json` and immutable SQL nodes for PostgreSQL and D1 while recording a
dynamic no-SQL head for Dowe. Deployment validates the graph instead of regenerating history, and
production applies pending migrations and seeders before the server starts listening.

The portable `query` read grammar supports projections, `AS` aliases, multiple equality joins,
equality filters joined by `AND`, `ORDER BY`, `LIMIT`, `OFFSET`, and `?N` parameters. Dowe renders
provider-safe identifiers and native placeholders internally. Provider-specific functions, casts,
subqueries, and arithmetic are not part of this operation.

## Cache KV operations

Every Cache operation uses `kv <binding> conn:<connection>.<operation>`. Do not use Database
`query` for Cache.

| Operation | Props |
| --- | --- |
| `get` | `key`; optional `required` |
| `set` | `key`, `value` |
| `delete` | `key` |
| `keys` | optional `prefix` |
| `clear` | no operation props |

`key` accepts a quoted literal or a server reference that resolves to text. Runtime validation rejects
empty keys, path separators, control characters, `.` and `..`.

```text
import appCache from "@/server/config/data"

fn createSessionRepository params:{ userId:string }
  id session source:"ulid"
  str sessionKey source:"join" values:["session" session] delimiter:":"
  kv cached conn:appCache.set key:sessionKey value:{ id:session userId:args.userId }
  return value:{ id:session userId:args.userId }
```

During `dowe dev`, every provider uses persistent local data under `.dowe/kv/<name>`. Only `name`
is resolved; the effective provider and remote credentials are ignored, and Dowe does not start
Wrangler or connect to the authored provider. Production resolves the provider and full connection.

## Vector embedding operations

Every Vector operation uses `emb <binding> conn:<connection>.<operation>`.

| Operation | Props |
| --- | --- |
| `upsert` | `id`, `vector`; optional `metadata` |
| `search` | `vector`; optional `limit`, `minScore`, `where` |
| `read` | `id`; optional `required` |
| `delete` | `id` |
| `list` | optional `limit`, `where` |

```text
import appVector from "@/server/config/data"

fn findRelated params:{ vector:unknown }
  emb matches conn:appVector.search vector:args.vector limit:10 minScore:0.7
  return value:matches
```

Development resolves only `name` and stores data under `.dowe/vector/<name>`. In production,
`host:"local"` keeps the embedded engine; any other host uses Dowe Vector over an authenticated
persistent WebSocket.
