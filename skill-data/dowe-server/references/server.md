# Server reference

## Named server declarations

| Declaration | Binding role | Responsibility |
| --- | --- | --- |
| `endpoints apiRoutes` | Importable route graph | Connects handlers and middleware to paths |
| `handler registerUser` | Importable HTTP boundary | Parses requests and returns responses; `req` and asynchronous execution are implicit |
| `middleware requireBearer` | Importable request boundary | Validates or enriches request context |
| `fn sendMail params:{...}` | Importable reusable function | Owns provider, service, repository, task, or utility behavior |
| `database primaryDb provider:"dowe" ...` | Reusable Database connection | Imported config binding or server action connection |
| `entity User` | Importable Database schema | Declares fields and constraints for SQL migrations |
| `seeder Bootstrap` | Importable seed data | Declares static inserts applied once by fingerprint |
| `cache appCache provider:"dowe" ...` | Reusable Cache connection | Imported config binding or server action connection |
| `vector appVector provider:"dowe" ...` | Reusable Vector connection | Imported config binding or server action connection |
| `queue appQueue provider:"dowe|rabbitmq" ...` | Reusable Queue connection | Server-only direct publication to an existing queue |

Handlers call service functions. `req` is implicit inside every HTTP handler, so route parameters,
request metadata, middleware context, and the typed JSON declaration can use `req` directly.
Service functions may call providers, repositories, tasks, or utilities. Repository functions own Database, Cache, and Vector logic. `service` and `repository` are not
declaration keywords. Keep new backend modules under `server/handlers`, `server/middlewares`,
`server/config`, `server/services`, `server/repositories`, `server/providers`, `server/tasks`,
`server/utils`, or `server/types`; connect routes through `server/endpoints.dowe`.

For a CRUD feature, keep the layers explicit: `server/handlers` parses request input and returns
HTTP responses, `server/services` coordinates the use case, `server/repositories` performs
Database operations, `server/entities` exports entity declarations, and `server/config` imports
those entities into the Database handle. A handler should not open a Database handle or contain a
`query` statement. For editor-friendly tabs, name generated modules with the responsibility suffix:
`blogs-handler.dowe`, `blogs-service.dowe`, `blogs-repository.dowe`, and `blogs-types.dowe`. Entity
files use a plural bounded-domain name such as `blog-entities.dowe` and keep the related entity,
revision, membership, or event declarations together. Use a dedicated singular entity file only
when that schema is genuinely isolated.

Read `references/data.md` for Database, Cache, and Vector handles, entities, seeders, and their
operation utilities. Read `references/runtime.md` for TLS, outbound HTTP, responses, crypto,
spawn, JWT, WebSockets, CORS, background jobs, protocol transports, and local models.

## Third-party HTTP providers

Use `server/providers` as the single ownership boundary for integrations with external APIs. Create
one file per third party, such as `server/providers/openai.dowe`,
`server/providers/cloudflare.dowe`, or `server/providers/other.dowe`. A provider file may export
multiple related operations as separate `fn` declarations, but it must not mix unrelated providers.

Each provider function owns the complete outbound contract: endpoint path, HTTP method, request
headers, authentication, JSON payload, timeout, response normalization, and safe error shape. Put
provider credentials and base URLs in server-only environment variables. Do not pass secrets from a
View or request body, log authorization headers, or return the raw upstream response when a smaller
application-owned shape is sufficient.

Services import provider functions and coordinate business behavior. Handlers parse requests and
return HTTP responses; they do not contain `http` statements for third-party APIs.

```text
fn createChatCompletion params:{ prompt:string }
  http upstream method:"post" base:env.OPENAI_BASE_URL path:"/v1/chat/completions" bearer:env.OPENAI_API_KEY json:{ model:"gpt-4o-mini" messages:[{ role:"user" content:args.prompt }] } timeoutMs:30000 mode:"json"
  if upstream.ok
    return value:{ ok:true data:upstream.json }
  return value:{ ok:false error:"provider_unavailable" status:upstream.status }
```

```text
fn sendCloudflareEmail params:{ recipient:string subject:string body:string }
  http upstream method:"post" base:env.CLOUDFLARE_EMAILS_BASE_URL path:"/send" bearer:env.CLOUDFLARE_EMAILS_API_TOKEN json:{ to:args.recipient subject:args.subject text:args.body } timeoutMs:10000 mode:"json"
  if upstream.ok
    return value:{ ok:true id:upstream.json.id }
  return value:{ ok:false error:"email_provider_unavailable" status:upstream.status }
```

Import these functions from services using their provider module path, for example
`@/server/providers/openai` and `@/server/providers/cloudflare`. Add every server-only `env.*`
name to the root `.env.example` without placing real credentials there.

## View request consumers

A route consumed by a Dowe View has one coordinated contract even though source ownership remains
separate. Load the companion `dowe-views` skill and inspect `references/views.md` whenever a Server
change affects an existing View caller or the requested View behavior requires a new project route.
Do not treat a compiling handler as proof that the fullstack behavior is complete.

Build a request-to-route matrix before changing either side:

| Field | Required decision |
| --- | --- |
| Caller | View page or layout and its `fn` or `init` |
| Method and path | Exact request method and resolved project route |
| Input | Body, headers, route params, validation, and client-controlled fields |
| Output | Status and minimal serializable JSON shape consumed by the View |
| UI behavior | Loading, success, empty, error, unauthorized, and retry states |
| Route owner | Endpoint declaration and matching method |
| Logic owners | Handler, middleware, service, repository, provider, and config modules |
| Data owners | Entities, migrations, Database, Cache, Vector, Queue, files, or external provider |
| Security | Authentication, permission, tenant/owner, invariant, and safe-error boundaries |

Trace method and path from the request to the endpoint, then trace the endpoint through every
imported Server layer. Reuse existing owners when they satisfy the capability. When they do not,
change the smallest complete set of Server modules required by the request and update the View in
the same task if its request or presentation states must change.

Client input never becomes authority over generated ids, tenant or owner scope, roles, permissions,
prices, totals, inventory, lifecycle state, table names, provider names, secrets, or storage paths.
Handlers parse and return HTTP values, services enforce use cases, repositories own persistence,
middleware enforces shared request boundaries, and entities plus migrations own schema evolution.
Return only serializable client-safe data; never expose connections, credentials, authorization
headers, provider URLs containing secrets, process handles, or unrelated record fields.

## Capability-first statement shape

Read server statements from left to right:

```text
<capability> <binding> <props>
<capability> <props>
```

Use the first form when the capability produces a named value, such as
`query blogs conn:appDb.list table:"blogs"` or
`createBlogService result args:{ title:body.title }`. Use the second when the capability performs
an action without creating a value, such as `next context:{ auth:verified }`.

The binding is a new server-local name. Props are named `name:value` inputs. An imported function
name is the capability at its call site. Control capabilities can select a target instead of
creating a binding, for example `task fn:refreshIndex args:{ force:true }`; the target is not a result
binding. Server source never uses assignment syntax.

Standard-library operations use
`<namespace> <binding> source:"<function>" <props>`, such as
`str authorization source:"join" values:["Bearer" session.id] delimiter:" "`.
Handlers and middleware use direct HTTP returns such as `return status:201 json:result` or
`return text:"OK"`; reusable `fn` declarations use `return value:<value>`. Static route responses
use `response <props>` without `return`. Never write `return response ...`, because the compiler
rejects that legacy form and asks you to remove `response`.

```text
import createBlogService from "@/server/services/blogs-service"

handler createBlog
  const body value:req.json
  createBlogService result args:{ title:body.title content:body.content }
  return status:201 json:result
```

Imported functions start their call statement and bind the result in the next position. A declared
`params:{ name:type }` object makes every `args.field` type-checked at the call site and inside the
function; an optional quoted `return` contract validates `return value:<value>`. Functions return
JSON-compatible values, never HTTP responses; the handler decides status and shape. A function
without `params` accepts omitted `args` or `args:{}`. Do not write `let result = ...` or any other
server assignment.

`log` writes runtime logs in statement order. Quoted values stay literal text; unquoted values such
as `created.title` or `req.params.id` are restricted references resolved from the handler context.

```text
log "blog created" created.id created.title
```

Request metadata also declares its binding without an assignment:

```text
request query source:"query"
request range source:"header" name:"Range"
request sessionCookie source:"cookie" name:"session"
request payload source:"bytes"
```

## General function utilities

| Utility | Binding | Required props | Optional props and limits |
| --- | --- | --- | --- |
| `function result` | `result` | Imported function name | `args:{...}` when params exist |
| `namespace result source:"function"` | `result` | `source` and function-specific named props | Portable standard library; `id result source:"ulid"` is server-only |
| `spawn process` | `process` | `command:string` | `args:string[]`, `cwd:string`, `timeoutMs:number`, `maxOutputBytes:number`, `background:boolean` |
| `file artifact` | `artifact` | `source:"write|read|exists|delete"`, `root`, `path` | `data` and `sha256` for writes; server-only confined storage |
| `http upstream` | `upstream` | `method:string`, `base:string or env`, `path:string` | `bearer`, `headers`, `json`, `mode:"json|proxy|bytes"`, `redirect`, `maxRedirects`, `timeoutMs` |
| `crypto output` | `output` | `encryption:"aesCtr|cencAesCtr"`, `data`, `key`, `iv` | `subsamples:[{ clear:number encrypted:number }]` |
| `jwt token` | `token` | `secret` or `key`, plus `claims` or `token` | `algorithm`, `encryption`; see `references/runtime.md` |
| `task fn:function` | none | Imported function through `fn` or an indented inline body | `args:{...}`; immediate fire-and-forget from server actions or functions. `after:"headers"` requires `args:{ event:{...} }` and only a direct handler ending `return reverse:...` |
| `cron fn:function` | none | Imported function through `fn`, `schedule:string` | `args:{...}`; valid only directly under `server.init` or `desktop.server.init`; `after` is invalid |

## Routes in `main.dowe`

The `server` block accepts imported `endpoints` graphs plus direct routes. Route paths are
slash-prefixed strings. `:name` captures one segment as `req.params.name`; a final `*name` splat
captures the remaining nested path and must match at least one segment.

```text
main
  server port:8080
    route "/dash/:name/*segment"
      handler
        return json:{ channel:req.params.name segment:req.params.segment }
    route "/api/status"
      response text:"OK"
    route "/api/blogs"
      method GET handler:listBlogs
      method POST handler:createBlog
```

A route declares exactly one of: an inline `handler`, a static `response <props>`, or `method`
entries mapping HTTP methods to imported handlers. A path whose requested method is not registered
returns method-not-allowed. Handlers never declare `async`, `await`, or a request parameter.

## Endpoint routing

```text
import { listBlogs, createBlog } from "@/server/handlers/blogs-handler"
import requireBearer from "@/server/middlewares/auth"

endpoints apiRoutes
  group path:"/api/blogs"
    get path:"" handler:listBlogs
    post path:"" handler:createBlog middleware:[requireBearer]
```

Endpoint groups are one level: a `group path:<string> middleware:[...]` contains direct lowercase
`get`, `post`, `put`, `patch`, and `delete` utilities, plus WebSockets. HTTP methods and WebSockets
also accept optional `middleware`. Do not nest `group` nodes. WebSockets use
`websocket path:"..." middleware:[...]` and the lifecycle children described in
`references/runtime.md`. Middleware runs after CORS preflight and route matching but before the
handler; it is asynchronous by default, has implicit `req` and `next`, and never advances unless it
explicitly calls `next`.

### Opaque Bearer sessions

Bearer identifies the HTTP authorization transport; it does not require JWT. When the application
already has Database and Cache, prefer an opaque ULID session for immediate revocation and explicit
per-device state. Generate it with `id session source:"ulid"`, persist the session record, and cache a
minimal server-owned projection under a namespaced key such as `session:<ulid>`.

```text
import { appDb, appCache } from "@/server/config/database"

middleware requireBearer
  bearer token value:req.header.Authorization
  session verified cache:appCache database:appDb token:token maxAge:2592000
  if verified.valid
    next context:{ auth:{ subject:verified.userId session:verified.id token:token } }
  return status:401 json:{ ok:false error:"Unauthorized" }
```

`session verified ...` declares the verification result, checks the ULID age, reads Cache first,
falls back to `sessions` in Database, and rehydrates Cache on a valid miss. Do not write
`let verified = session.verify ...`; host capabilities bind their result directly. Delete both the
Cache key and Database record on logout or ban. Keep JWT for stateless signed assertions or
interoperability with services that cannot share a session store; do not put sensitive claims in an
opaque token or trust client-provided Cache values.

Connections and operation results stay server-only. Return serializable values, never connections,
secrets, complete provider URLs, authorization headers, encryption keys, or process metadata that
the client does not need.

### Queue publication

Queue connections accept `host`, `port`, `account`, `secret`, and `vhost` as literals or server-only
environment references. Add the names to `.env.example`, keep development values in ignored `.env`,
and keep deploy values in `.env.live`, `.env.stage`, or `.env.uat`.

```text
queue appQueue provider:"dowe" host:env.QUEUE_HOST port:env.QUEUE_PORT account:env.QUEUE_USER secret:env.QUEUE_PASSWORD vhost:env.QUEUE_VHOST
msg sent conn:appQueue.publish queue:"notifications" payload:{ userId:"123" event:"user_created" }
return json:{ ok:sent.ok messageId:sent.id }
```

During `dowe dev`, only `vhost` selects persistent local Dowe storage. Production resolves the
authored provider and full connection; RabbitMQ uses `vhost` as its AMQP virtual host. The target
queue must already be provisioned by the CLI or Rust provider API. Direct `msg ... publish` does not
declare or bind topology and does not retry after an ambiguous response. The result is exactly
`{ ok, id }`. Consume, subscribe, ACK, and NACK remain streaming Rust provider APIs with
session-bound receipts, not finite source statements.
