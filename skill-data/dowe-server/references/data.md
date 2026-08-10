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

Cache providers are `kv` for Cloudflare KV, `redis` for Redis, and `dowe` for Dowe Cache. Vector
initially supports only `dowe`. Config modules may export `database`, `cache`, and `vector`
bindings; import those bindings into repository functions instead of opening the same handle
repeatedly.

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
assigns each seeder a deterministic fingerprint, so it runs once per database.

```text
seeder Bootstrap
  insert entity:Users value:{ id:"01ARZ3NDEKTSV4RRFFQ69G5FAV" name:"Admin" email:"admin@example.com" active:true createdAt:"2026-01-01T00:00:00Z" }
```

Entity declarations can live in separate modules and be imported by the config module:

```text
import Blog from "@/server/entities/blog-entity"

database appDb provider:"dowe" host:env.DATABASE_HOST port:env.DATABASE_PORT account:env.DATABASE_ACCOUNT secret:env.DATABASE_SECRET name:env.DATABASE_NAME entities:[Blog] seeders:[]
```

## Database queries

Every Database operation uses `query <binding> db:<handle>.<operation>`.

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
  query created db:appDb.insert table:"blogs" value:{ title:args.title content:args.content }
  return value:created
```

An `insert` result exposes the fields of its `value` object plus the generated `id`; `update` and
`delete` results expose `changed`. A known invalid field reference fails during compilation.

Use Cloudflare's account and Database identifiers for D1:

```text
database appDb provider:"d1" account:env.ACCOUNT_ID secret:env.CLOUDFLARE_API_TOKEN name:env.DATABASE_ID entities:[Blog] seeders:[Bootstrap]
```

D1 supports compound equality filters but not `db:<handle>.tx`. Keep account, token, and Database ID
in server-only environment variables. Bind request values separately from SQL when a query needs
custom filtering or pagination:

```text
query rows db:appDb.query sql:"SELECT id, name FROM icons WHERE category = ?1 LIMIT 60 OFFSET ((CAST(?2 AS INTEGER) - 1) * 60)" params:[req.params.category, req.params.page]
```

Do not interpolate a request reference into `sql`. The runtime binds query parameters using the
selected provider's native placeholder rules.

`db:<handle>.tx` is supported by local development storage and remote `provider:"dowe"` handles.
It stages literal `insert` children and ends with `commit` or `rollback`:

```text
query result db:appDb.tx
  query delivery db:appDb.insert table:"sms_deliveries" value:{ recipient:args.recipient status:"queued" }
  query outbox db:appDb.insert table:"sms_outbox" value:{ recipient:args.recipient status:"pending" }
  commit value:delivery
```

Commit is atomic and durable: Dowe validates every staged insert, records one checksummed WAL frame,
syncs the group to disk, and only then publishes the records. A conflict rejects the complete
transaction. PostgreSQL and D1 handles reject `tx` during compilation. Write requests are not
automatically retried after a transport failure; use a stable ULID or business idempotency key when
the caller may retry.

During `dowe dev`, Dowe uses its embedded persistent Database under `.dowe/db/<name>` for every
provider and resolves only `name`; it does not start Wrangler or contact the authored provider.
`dowe deploy` generates SQL migration artifacts for Postgres and D1, and production applies pending
migrations and seeders before the server starts listening.

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
  str sessionKey source:"join" values:["session", session] delimiter:":"
  kv cached conn:appCache.set key:sessionKey value:{ id:session userId:args.userId }
  return value:{ id:session userId:args.userId }
```

During `dowe dev`, every provider uses persistent local data under `.dowe/kv/<name>`. Only `name`
is resolved; Dowe does not validate the effective remote credentials, start Wrangler, or connect to
the authored provider. Production resolves the full connection.

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
