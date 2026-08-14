# Server runtime reference

## Native TLS

Put `tls` directly inside the main server. ACME mode issues and renews a multi-domain Let's Encrypt
certificate in the Rust runtime and caches it below `.dowe`; local mode creates a self-signed
certificate for loopback development.

```text
main
  server port:443
    tls:
      mode:"acme"
      domains:["example.com", "www.example.com"]
      email:"admin@example.com"
      staging:false
```

`staging` defaults to `true`. ACME domains must be public DNS names and cannot be wildcard, IP, or
localhost values. For local HTTPS use `mode:"local"` with `localhost` or a `.localhost` subdomain.
Large catalogs are served through SNI and deterministic certificate groups of at most 100 names.

An authored domain manager may add local KV domains with
`domainsFrom:{ kv:"domains" key:"tls" }` or Database records with
`domainsFrom:{ db:"control" table:"domains" field:"hostname" }`. `refreshSeconds` defaults to 60
and must be between 30 and 86400. A remote control plane uses
`domainsFrom:{ endpoint:env.CONTROL_PLANE_URL path:"/v1/domains" bearer:env.CONTROL_PLANE_TOKEN timeoutMs:5000 }`.
The endpoint must be HTTPS and return domain strings or records with `host`/`hostname`.
`httpPort:80` adds an HTTP listener that redirects only effective catalog hosts to HTTPS. Keep TLS
caches, endpoint credentials, account state, and private keys server-only.

## Outbound HTTP

`http <binding>` sends a server-only outbound request from the Rust runtime.

```text
http upstream method:"get" base:env.CATALOG_BASE_URL path:"/products" headers:[{ name:"Accept" value:"application/json" }] redirect:"manual" timeoutMs:5000 mode:"json"
```

| Prop | Behavior |
| --- | --- |
| `headers:[{ name:"..." value:"..." }]` | Validated static headers; values may use server-only `env.NAME` |
| `bearer:env.NAME` | Adds a bearer token without exposing it to clients |
| `json:<binding>` | Sends a JSON body |
| `redirect` | `"follow"` up to `maxRedirects`, `"manual"` exposing `status` and `location`, or `"error"` |
| `timeoutMs` | Per-request timeout returning `http_timeout` when exceeded |
| `mode:"json"` | Buffers the body; the binding exposes `status`, `ok`, `url`, `redirected`, `contentType`, `headers`, `location`, and `json` |
| `mode:"proxy"` | Preserves upstream status, content type, bytes, and SSE stream for `return proxy:` or a WebSocket bridge |
| `mode:"bytes"` | Buffers the body as bytes for `return bytes:` or server crypto |

Outbound HTTP is a server integration surface, not a browser automation surface.

## Responses

Handlers and middleware return only explicit responses directly with `return <props>`; static
routes use `response <props>`. Never write `return response ...`; remove `response` after
`return`.

| Form | Behavior |
| --- | --- |
| `return status:201 json:<value>` | JSON response; `status` defaults to 200 |
| `return text:"OK"` | Text response |
| `return bytes:<binding>` | Byte response from an HTTP, crypto, or function byte binding |
| `return proxy:<binding>` | Sends the upstream `mode:"proxy"` response without exposing secrets |
| `return agent:<binding> request:<binding>` | Dowe Agent envelope with request identity and provider payload |
| `contentType:"video/mp4"` | Sets the response content type |
| `headers:[{ name:"Cache-Control" value:"no-store" }]` | Validated response headers from literals or bindings |
| `cookies:[{ name:"session" value:token path:"/" httpOnly:true sameSite:"Lax" maxAge:3600 }]` | Appends `Set-Cookie` with optional `path`, `httpOnly`, `secure`, `sameSite`, `maxAge` |

## Standard-library capabilities

Server standard-library calls are capability-first declarations. The binding is available to later
statements, and `source` selects one implemented function:

```text
<namespace> <binding> source:"<function>" <props>
```

Reusable server functions can chain these values before returning `value`; handlers call the
function and own the HTTP response:

```text
fn summarizeScores params:{ payload:string }
  parse parsed source:"json" value:args.payload fallback:[]
  sort sorted source:"asc" values:parsed
  math total source:"sum" values:sorted
  return value:{ total:total sorted:sorted }
```

```text
handler summarize
  const body value:req.json
  summarizeScores result args:{ payload:body.payload }
  return json:result
```

These calls are pure Rust-owned runtime operations. They do not open the network, read files,
launch processes, or access secrets. A server `fn` and handler are server-only; views cannot use
server bindings or server environment values. Each operation returns a new logical value, so sort
and list operations do not mutate request bodies or other bindings.

### Implemented catalog

| Namespace | Functions and arguments |
| --- | --- |
| `str` | `trim(value)`, `lower(value)`, `upper(value)`, `length(value)`, `contains(value, needle)`, `startsWith(value, prefix)`, `endsWith(value, suffix)`, `replace(value, from, to)`, `split(value, delimiter, limit?)`, `join(values, delimiter?)` |
| `math` | `add(left, right)`, `sub(left, right)`, `mul(left, right)`, `div(left, right)`, `round(value)`, `floor(value)`, `ceil(value)`, `abs(value)`, `min(values)`, `max(values)`, `sum(values)`, `average(values)` |
| `parse` | `int(value, fallback?)`, `float(value, fallback?)`, `bool(value, fallback?)`, `json(value, fallback?)`, `string(value, fallback?)`, `svg(value, fallback?)` |
| `url` | `encode(value)`, `decode(value, fallback?)`, `parse(value)`, `queryGet(value, name)`, `querySet(value, name, param)` |
| `csv` | `parse(value, delimiter?, header?, maxRows?, maxColumns?)`, `stringify(rows, delimiter?)` |
| `sort` | `asc(values)`, `desc(values)`, `by(values, field, direction?, nulls?)` |
| `list` | `take(values, count)`, `skip(values, count)`, `first(values)`, `last(values)`, `count(values)`, `filterEquals(values, field, value)`, `filterContains(values, field, value)`, `mapField(values, field)`, `sumBy(values, field)`, `averageBy(values, field)` |
| `json` | `get(value, path, fallback?)`, `set(value, path, next)`, `pick(value, fields)`, `omit(value, fields)`, `merge(left, right)`, `stringify(value, pretty?)`, `parse(value, fallback?)` |
| `date` | `now()`, `formatIso(value)`, `addDays(value, days)`, `diffDays(start, end)` |
| `id` | `ulid()` (server-only) |

The names in the table are the exact `source` values and prop names. Unknown functions or props
are rejected before the server starts.

### Math, parse, and sort behavior

`math` accepts finite numbers. `sum` of an empty array is `0`; `average`, `min`, and `max` return
`null` for an empty array. Division by zero and non-finite results are controlled runtime results,
not panics.

`parse` never throws an input exception into a handler. Invalid `int`, `float`, `bool`, `json`, or
`svg` input returns the declared `fallback`, or `null` when no fallback is provided. `parse.string`
converts a JSON-compatible value to text. `parse.svg` only accepts the bounded portable SVG subset.

`sort.asc`, `sort.desc`, and `sort.by` return stable, non-mutating arrays. `sort.by` reads a dotted
`field`; missing or null fields follow `nulls` (`first` or `last`, default `last`), and `direction`
accepts `asc` or `desc`. No locale-dependent comparator or in-place mutation is used.

## Byte requests and file storage

```text
request payload source:"bytes"
file stored source:"write" root:env.STORAGE_ROOT path:req.params.hash data:payload sha256:req.params.hash
file artifact source:"read" root:env.STORAGE_ROOT path:req.params.hash
return bytes:artifact contentType:"application/octet-stream"
```

`request ... source:"bytes"` preserves the HTTP body without JSON conversion. `file` is
server-only and supports `write`, `read`, `exists`, and `delete`. Every operation requires an
explicit `root` and relative `path`; traversal, absolute paths, and symlinks are rejected. Write is
atomic and accepts optional `sha256` verification before publishing the destination.

## Passwords

Use `password passwordHash source:"hash" value:body.password` during registration and store only
the returned Argon2id PHC string. During login, use
`password verified source:"verify" value:body.password hash:user.passwordHash required:true`.
Verification returns `{ valid:boolean }`; `required:true` terminates with Unauthorized on mismatch.
Password values and hashes remain server-only and must not be returned, logged, or copied to Views.

## Crypto and spawn

```text
spawn ffmpeg command:"ffmpeg" args:["-version"] timeoutMs:5000 maxOutputBytes:65536
http upstream method:"get" base:env.MEDIA_BASE_URL path:"/segment.m4s" mode:"bytes"
crypto encrypted encryption:"cencAesCtr" data:upstream key:env.MEDIA_KEY iv:env.MEDIA_IV
```

`crypto` applies AES-128-CTR (`aesCtr`) or CENC subsample AES-CTR (`cencAesCtr`) to a byte
binding; keys and IVs may be hex or base64 and must decode to 16 bytes. `spawn` runs a server-side
process and binds stdout, stderr, and metadata; `background:true` returns `spawnId` and
`systemPid` without waiting. If a function returns a byte-producing binding with
`return value:<binding>`, the caller receives the JSON metadata and the byte binding under the
call name, so the handler can respond with `return bytes:<binding>`.

## JWT

`jwt <binding>` binds each result directly. A JWS operation uses `secret` and
`algorithm:"HS256"`; a JWE operation uses `key`, `algorithm:"dir"`, and `encryption:"A256GCM"`.
`claims` creates a token; `token` verifies or decrypts one.

| Capability | Runtime behavior |
| --- | --- |
| `jwt token secret:env.JWT_SECRET algorithm:"HS256" claims:{ sub:"user-1" }` | Produces compact JWS |
| `jwt verified secret:env.JWT_SECRET algorithm:"HS256" token:token` | Verifies JWS and registered claims; exposes `verified.valid` and `verified.claims` |
| `jwt token key:env.JWT_KEY algorithm:"dir" encryption:"A256GCM" claims:{ sub:"user-1" }` | Produces compact JWE |
| `jwt decrypted key:env.JWT_KEY algorithm:"dir" encryption:"A256GCM" token:token` | Decrypts JWE and validates registered claims |

`alg:"none"` is rejected. Trust claims only inside branches guarded by a valid verification
result; expired tokens and future `nbf` values fail closed. JWT secrets live in dotenv keys
referenced exclusively by server source.

## WebSockets

Declare WebSockets inside an endpoint group or the server block with
`websocket path:"..." middleware:[...]`. Lifecycle children are `open`, `message`, `close`, and
`drain`; each is optional and binds the socket, such as `message ws`. Route middleware runs
against the HTTP upgrade; an invalid result returns its HTTP response and does not upgrade.
Browsers cannot set upgrade headers, so WebSocket middleware reads `req.query.<name>`; treat those
values as credentials, use TLS, and avoid logging complete URLs.

```text
websocket "/api/v1/agent/ws"
  message ws
    ws request source:"json"
    send ws json:{ event:"started" requestId:request.requestId }
    agent chat source:"chat" request:request
    http upstream method:"post" base:env.OPENROUTER_BASE_URL path:"/api/v1/chat/completions" bearer:env.OPENROUTER_API_KEY json:chat mode:"proxy"
    bridge sse:upstream to:ws requestId:request.requestId requestType:request.requestType model:request.model
```

| Utility | Behavior |
| --- | --- |
| `ws event source:"json"` | Parses the inbound message payload |
| `send ws json:{...}` | Sends one JSON event to the socket |
| `agent chat source:"chat" request:event` | Transforms a Dowe Agent envelope for an OpenRouter-compatible provider |
| `bridge sse:<binding> to:ws ...` | Bridges an upstream SSE proxy response into structured socket events |

Streaming agent requests over plain HTTP are rejected; streaming agent clients use a declared
WebSocket path and `bridge sse`.

## Queue service and application publication

`queue service` is valid once as a direct child of `main.server`, accepts no props or children, and
reserves the authenticated WebSocket path `/v1/queues/:name`:

```text
main
  server port:4150
    queue service
```

The maintained Dowe Queue service persists namespaces below `.dowe/queue/<name>`. Each namespace
has an exclusive OS process lock at `.dowe/queue/<name>/.lock`; a live service, second server, and
local CLI cannot directly open it concurrently, and lock acquisition fails closed. The upgrade
requires a bearer secret and `X-Dowe-Queue-Account`; loopback may use `ws://`, while remote Rust
clients use `wss://` with Rustls/WebPKI roots. Queue receipts remain bound to the subscription that
issued them, and pending deliveries are requeued when that session or the Dowe service closes.

The account catalog at `.dowe/queue/_auth` has its own exclusive
`.lock`. Account writers take `.dowe/queue/_auth/.lock` before read-modify-write and atomically
replace the catalog; a second writer fails closed instead of losing an account record.

The shared Rust provider contract supports Dowe and RabbitMQ transports. The root CLI exposes
`dowe queue start`, `create-account`, `init`, `list`, `inspect`, `declare`, `bind`, `publish`, and
`purge`. Dowe returns authoritative created flags, publication destinations, and inspected topology.
RabbitMQ confirms publication but AMQP cannot know whether declare/bind created resources, the exact
destinations, or enumerate topology, so those reports use unknown/`None` rather than fabricated
`true` or empty values. RabbitMQ connections require TLS outside loopback, and selecting the Dowe
provider does not contact RabbitMQ. A future explicit management authority would be required for
RabbitMQ topology enumeration.

Server actions can declare a provider connection and publish directly to an existing queue:

```text
queue appQueue provider:"dowe" host:env.QUEUE_HOST port:env.QUEUE_PORT account:env.QUEUE_USER secret:env.QUEUE_PASSWORD vhost:env.QUEUE_VHOST
msg sent conn:appQueue.publish queue:"notifications" payload:{ userId:"123" event:"user_created" }
return json:{ ok:sent.ok messageId:sent.id }
```

The connection and all credentials are server-only. Add the names used by `env.*` to the root
`.env.example`, keep local values in ignored `.env`, and use `.env.live`, `.env.stage`, or `.env.uat`
for deploy values. During `dowe dev`, Queue resolves only `vhost` and uses persistent local Dowe
storage below `.dowe/queue/<vhost>`; production resolves provider, host, port, account, secret, and
vhost. `provider:"rabbitmq"` uses `vhost` as the AMQP virtual host.

`msg ... publish` is direct work-queue publication. The queue must already exist; the operation does
not declare or bind a queue or interpret the destination as a topic. Its exact result shape is
`{ ok, id }`, and it is not retried after an ambiguous response because the provider may already
have committed. Source-level consume, subscribe, ACK, and NACK remain streaming Rust provider APIs
with session-bound receipts rather than finite Dowe Source Format statements.

## CORS

Configure CORS in `main.dowe` under the `server` that owns the routes. The Rust runtime answers
preflights before handlers run; successful preflights never execute handlers or Database logic.

```text
main
  server port:8080
    cors target:"server" devOrigins:true origins:["https://app.example.com"] methods:["GET","POST","PATCH","DELETE"] headers:["Content-Type"] credentials:false maxAge:600
```

| Prop | Behavior |
| --- | --- |
| `target` | Applies policy to `server`, `desktop`, or `all` |
| `origins` | Exact `http`/`https` origins, or `*` without credentials |
| `devOrigins` | Allows origins started by the same `dowe dev` session |
| `methods`, `headers` | Limits CORS methods and allowed preflight headers |
| `exposeHeaders` | Adds `Access-Control-Expose-Headers` to real responses |
| `credentials` | Adds `Access-Control-Allow-Credentials:true` for exact origins |
| `maxAge` | Adds `Access-Control-Max-Age` to successful preflights |

## Tasks and cron

`task fn:<function> [args:{...}]` starts an imported server function fire-and-forget and discards its
result. Tasks are immediate by default. In HTTP handlers and server functions, named task arguments
may contain typed local bindings; Dowe resolves and serializes those values before launching the
isolated worker.

```text
task fn:writeAudit args:{ orderId:order.id event:"order.created" }
```

`task [args:{...}]` with an indented body starts small local work in an isolated worker scope.
An inline task may use `args` and imported server configuration but cannot capture outer locals,
`req`, environment values, or runtime handles. It cannot return a value, launch another task or
cron registration, or use HTTP or WebSocket response-control statements.

```text
task args:{ orderId:order.id }
  log "queued audit" args.orderId
```

`init`, named task registrations under `init`, and `cron` arguments stay static JSON. Runtime
handles such as Database, Cache, KV, and environment values do not cross the process boundary.
`cron fn:<function> schedule:"0 3 * * *" args:{...}` registers a UTC five-field schedule and is valid
only directly under `server.init` or `desktop.server.init`. Cron accepts no positional target and
creates no result binding. `server init` runs once after the
listener is prepared and before traffic is accepted. Tasks and cron registrations never create a
result binding. `cron` does not accept task-only `after:"headers"` timing.

### Reverse-proxy response-header telemetry

Only a direct HTTP handler whose final response is `return reverse:...` may use
`task fn:<function> args:{ event:{ ... } } after:"headers"`. The `event` object is authored at the
launch site and lets Dowe add measured reverse-proxy data:

```text
request host source:"header" name:"Host"
kv route conn:RouteCache.get key:host required:true
task fn:emitTelemetry args:{ event:{ projectId:route.projectId status:0 method:"" path:"" latencyMs:0 bytesIn:0 bytesOut:0 } } after:"headers"
return reverse:route.url
```

Dowe holds this task until a real upstream response returns headers, then starts it without waiting
for the worker. Upstream `4xx` and `5xx` responses count. Loading and error fallbacks, no ready
upstream, invalid URLs, and client or connection failures do not launch it. Before launch, Dowe
overwrites `event.status`, `event.method`, `event.path`, `event.latencyMs`, `event.bytesIn`, and
`event.bytesOut`, preserving other event fields; `bytesOut` is `0` without upstream
`Content-Length`. `after:"headers"` is rejected in init, reusable functions, middleware,
WebSockets, protocol handlers, non-reverse handlers, and cron.

## Cache-backed reverse proxy

Use `return reverse:route.url` only after the route binding comes from a required Cache read in the
same handler:

```text
request host source:"header" name:"Host"
kv route conn:RouteCache.get key:host required:true
return reverse:route.url
```

Dowe preserves the incoming method, path, query, body and safe headers, then streams the upstream
response. Request JSON cannot select the upstream. Dowe removes hop-by-hop headers and rejects URLs
outside HTTP or HTTPS or containing embedded credentials.

For a Cache-backed Runtime pool, use the Dowe-owned round-robin contract:

```text
request host source:"header" name:"Host"
kv route conn:RouteCache.get key:host required:true
return reverse:route.upstreams strategy:"roundRobin" state:route.state loadingUrl:route.loadingUrl errorUrl:route.errorUrl
```

The array may contain URL strings or records with `url`/`upstreamUrl`. Records with
`enabled:false` or a `status` other than `ready` are excluded. `loading` and `error` route states
produce temporary redirects to their Cache-backed fallback URLs. All reverse response references
must come from the same required Cache binding.

## Protocol transports

Server targets can declare TCP and UDP listeners next to HTTP routes; they are server-only Rust
runtime tasks for protocol gateways such as SIP signaling or telemetry ingestion.

```text
main
  server port:8080
    udp name:"sip-udp" bind:"0.0.0.0" port:5060
      packet pkt
        log "udp packet" pkt.addr pkt.text pkt.bytes
    tcp name:"sip-tcp" bind:"0.0.0.0" port:5060
      connection conn
        log "tcp payload" conn.addr conn.text conn.bytes
    rtp bind:"0.0.0.0" min:40000 max:40100
```

`udp` and `tcp` require unique `name` and `port`; `bind` defaults to `127.0.0.1`. `packet` and
`connection` handlers resolve `<binding>.text`, `<binding>.bytes`, and `<binding>.addr`. `rtp`
declares the local RTP port pool. Higher-level protocol parsing remains authored server logic.

## Local models

Server targets can declare local inference models; they are never available to client targets.

```text
main
  server port:8090
    model name:"voice-vad" kind:"vad.silero" engine:"candle" format:"onnx" source:"assets/silero_vad.onnx" sampleRates:[8000,16000]
```

`kind` initially supports `vad.silero`. `engine` is `candle` for ONNX inference or `energy` for
the built-in fallback, with matching `format` `onnx` or `builtin`. `source` is a project-relative
asset path, and `sampleRates` currently accepts `8000` and `16000`.
