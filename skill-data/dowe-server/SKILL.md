---
name: dowe-server
description: Use for Dowe source under server or server blocks: routes, handlers, middleware, functions, persistence, tasks, protocols, security, and responses; skip for view- or theme-only edits.
---

# Dowe server authoring

Server source is compiled into Rust-owned runtime behavior. Keep routes, input validation, data access and responses explicit.
Keep every new backend module under `server/`; only root `main.dowe` connects it to the application.

## Workflow

1. Inspect the `server` block and imported `endpoints` binding.
2. Keep HTTP boundaries in handlers and middleware. Write `handler <name>` without `async`; request
   context and asynchronous execution are implicit.
3. Put reusable `fn` declarations under the matching `server` responsibility folder and use folder
   names to express provider, service, repository, task, or utility ownership. When the same
   domain spans layers, include the responsibility in the filename, such as
   `blogs-handler.dowe`, `blogs-service.dowe`, and `blogs-repository.dowe`.
4. Put Database, Cache, and Vector work in repository functions and reuse imported config connection declarations.
5. Keep external providers, secrets, process handles, and persistence server-only.
6. Use portable standard-library capabilities as `<namespace> <binding> source:"<function>" <props>`;
   read `references/runtime.md` for the implemented catalog and runtime rules.
7. Keep physical files behind `file` with an explicit storage root and relative path; use
   `request ... source:"bytes"` for byte-exact uploads and `sha256` for immutable artifacts.
8. Prefer opaque ULID sessions with Cache-aside validation and Database fallback when the application needs immediate revocation; Bearer does not imply JWT.
9. Use the canonical HTTP return forms exactly:
   - handlers and middleware: `return text:"..."`, `return json:<value>`,
     `return status:201 json:<value>`, `return bytes:<binding>`, or `return proxy:<binding>`;
   - static routes: `response text:"..."` or another `response <props>` form;
   - reusable server functions: `return value:<value>`.
   Never write `return response ...`: `response` after `return` is rejected by the compiler.
10. Keep endpoint groups one level; put middleware on the group, HTTP method, or WebSocket instead
   of nesting a group.
11. Validate the complete import chain with the compiler.

## Reference routing

| Task | Read only |
| --- | --- |
| Declarations, binding rules, functions, routing, sessions, or layer boundaries | `references/server.md` |
| Database, Cache, Vector, entities, seeders, or persistence operations | `references/data.md` |
| Standard-library namespaces, signatures, fallbacks, sorting, or server-only execution | `references/runtime.md` |
| TLS, HTTP, responses, crypto, spawn, JWT, WebSockets, CORS, jobs, transports, or models | `references/runtime.md` |
