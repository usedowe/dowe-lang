---
name: dowe-server
description: "Use for Dowe source under server or server blocks: routes, handlers, middleware, functions, persistence, tasks, protocols, security, responses, and project-owned backend behavior required by a View request. Pair with dowe-views when a route has an affected View consumer; skip work that is entirely visual or theme-only."
---

# Dowe server authoring

Server source is compiled into Rust-owned runtime behavior. Keep routes, input validation, data access and responses explicit.
Keep every new backend module under `server/`; only root `main.dowe` connects it to the application.

## View-consumer gate

When a project route is called by a View, or a Server change alters a View's method, input, output,
authorization, loading, empty, error, or unauthorized behavior, inspect the caller before editing
the Server. In those cases, load the companion `dowe-views` skill. A route is not complete for that
task merely because its handler compiles.

Create the same request-to-route matrix used by Views: caller page or layout and function or `init`;
HTTP method and resolved path; body, headers, and route parameters; safe response shape and status;
UI states; endpoint, handler, middleware, service, and repository owners; data impact; and
authorization boundary. Reuse the current route and layers when they already own the capability.

Implement the smallest complete Server change required by the request. This may include endpoints,
handlers, middleware, services, repositories, entities, migrations, Database, Cache, Vector, Queue,
configuration, or server-only environment names. Update the View consumer in the same task when its
request contract or states must change. Never make client-provided ids, tenant, roles, prices,
totals, permissions, or lifecycle state authoritative, and never return connections, secrets,
provider credentials, or unrelated record fields.

For server-backed catalogs, expose a dedicated metadata endpoint that returns stable keys, display
labels, ordering, and authoritative counts. The View should load that metadata into a Signal and
render it with one keyed collection, while record endpoints own filtering, pagination, and totals.
Do not require clients to duplicate catalog records or create one endpoint/function per category;
category and style remain validated request parameters and the Server remains the source of truth.

## Workflow

1. Inspect the `server` block and imported `endpoints` binding.
2. Keep HTTP boundaries in handlers and middleware. Write `handler <name>` without `async`; request
   context and asynchronous execution are implicit.
3. Put reusable `fn` declarations under the matching `server` responsibility folder and use folder
   names to express provider, service, repository, task, or utility ownership. When the same
   domain spans layers, include the responsibility in the filename, such as
   `blogs-handler.dowe`, `blogs-service.dowe`, and `blogs-repository.dowe`.
4. Put every third-party HTTP integration in `server/providers`, with one provider integration per
   file. Use names such as `server/providers/openai.dowe`, `server/providers/cloudflare.dowe`, and
   `server/providers/other.dowe`; declare the integration operations as separate `fn` functions in
   that file. Provider functions own `http` calls, provider URLs, authentication headers, request
   payloads, response normalization, timeouts, and safe errors. Services call provider functions;
   handlers must not call third-party HTTP APIs directly. Keep provider credentials in server-only
   `env.*` references and never return or log them.
5. Group related `entity` declarations in one focused plural module under `server/entities`, such as
   `user-entities.dowe` or `kitchen-entities.dowe`, and import its bindings together into Database
   config. Do not generate one file per entity by default or mix unrelated domains into a catch-all.
6. Put Database, Cache, Vector, and Queue work in repository functions and reuse imported config connection declarations. Queue connections are server-only; `msg ... publish` targets an already-provisioned queue and returns `{ ok, id }`.
7. Keep external providers, secrets, process handles, and persistence server-only.
8. Use portable standard-library capabilities as `<namespace> <binding> source:"<function>" <props>`;
   read `references/runtime.md` for the implemented catalog and runtime rules.
9. Keep physical files behind `file` with an explicit storage root and relative path; use
   `request ... source:"bytes"` for byte-exact uploads and `sha256` for immutable artifacts.
10. Prefer opaque ULID sessions with Cache-aside validation and Database fallback when the application needs immediate revocation; Bearer does not imply JWT.
11. Use the canonical HTTP return forms exactly:
   - handlers and middleware: `return text:"..."`, `return json:<value>`,
     `return status:201 json:<value>`, `return bytes:<binding>`, or `return proxy:<binding>`;
   - static routes: `response text:"..."` or another `response <props>` form;
   - reusable server functions: `return value:<value>`.
   Never write `return response ...`: `response` after `return` is rejected by the compiler.
12. Keep endpoint groups one level; put middleware on the group, HTTP method, or WebSocket instead
   of nesting a group.
13. Validate the complete import chain with the compiler. For a View-consumed route, also reconcile
    the request-to-route matrix with the actual caller and its loading, success, empty, error, and
    unauthorized branches.

Queue providers are `dowe`, `rabbitmq`, `cloudflare`, and `vercel`. `dowe dev` always uses the local Dowe provider, regardless of the authored provider. Deploys resolve the selected environment values; Cloudflare uses a Worker Queue binding and Vercel uses its regional Queue API.

## Reference routing

| Task | Read only |
| --- | --- |
| Declarations, binding rules, functions, routing, sessions, or layer boundaries | `references/server.md` |
| Database, Cache, Vector, entities, seeders, or persistence operations | `references/data.md` |
| Standard-library namespaces, signatures, fallbacks, sorting, or server-only execution | `references/runtime.md` |
| TLS, HTTP, responses, crypto, spawn, JWT, WebSockets, CORS, jobs, transports, or models | `references/runtime.md` |
