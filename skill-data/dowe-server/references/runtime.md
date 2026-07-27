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
and must be between 30 and 86400. Keep TLS caches, account state, and private keys server-only.

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

## Background jobs

`go <function> args:{...}` runs an imported function fire-and-forget and discards its result.
`cron <function> schedule:"0 3 * * *" args:{...}` registers a UTC five-field schedule and is valid
only directly under `server.init` or `desktop.server.init`. `server init` runs once after the
listener is prepared and before traffic is accepted. Neither creates a result binding.

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
