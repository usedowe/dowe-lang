# Native IPC reference

## Source contract

The root project imports reusable server functions and registers them once:

```text
import pickDirectory from "@/server/desktop"
import createFolder from "@/server/desktop"

main
  views:viewRoutes
  ipc functions:[pickDirectory createFolder]
```

A function must be an imported `fn`. The registry is an allowlist shared by Desktop, Android, and iOS. It does not expose an HTTP endpoint and does not apply to Web.

The Desktop-only namespace remains reserved for future capabilities:

```text
main
  desktop
    server port:8080
```

Do not use platform-specific IPC registries. `main.desktop.server` is optional and independent from `main.ipc`.

## Function declarations

Keep contracts explicit in the imported module:

```text
fn createFolder params:{ parent:string name:string } return:"string"
  return value:args.name
```

The shared runtime may provide a native implementation for a registered function name, but the source function remains the compiler-visible action and return contract. Native implementations must preserve the declared result shape and security rules.

## View invocation

The canonical syntax is sequential:

```text
fn create
  invoke result fn:createFolder args:{ parent:parentPath name:folderName } update:createdPath
  if result.ok
    toast type:"success" title:"Created" message:"Folder created"
  else
    toast type:"error" title:"Error" message:"Folder could not be created"
```

`result` is scoped to the action and contains `ok` and `data`. `update` stores successful data in the named binding according to normal View action semantics. The invocation must name a function registered by `main.ipc`; otherwise compilation or runtime validation rejects it.

## Request and response behavior

A native host bridge should carry:

| Field | Meaning |
| --- | --- |
| `id` | Request correlation id |
| `function` | Registered function name |
| `target` | Desktop, Android, or iOS target |
| `args` | JSON-compatible arguments |

The response correlates with `id` and contains:

```json
{"ok":true,"data":{}}
```

or:

```json
{"ok":false,"data":null}
```

Malformed messages, unknown functions, missing bridges, invalid arguments, host failures, and user cancellation must not fall back to HTTP or execute arbitrary functions. Return a failed result and let the View render its error or cancellation state.

## Security and portability

Validate authorization, filesystem paths, and business rules in the shared Rust/runtime or server-owned implementation. Do not trust a path, tenant id, role, price, or permission supplied by the View. For filesystem operations, reject absolute paths, parent traversal, separators, and empty names when only one folder name is intended.

## Native project-file helpers

The portable View standard library never reads the filesystem. The server-only `file` capability is
for bounded byte storage below a server-owned root and cannot be used from Views.

Native editor workflows may register the runtime-provided `listProjectFiles`, `readProjectFile`,
and `writeProjectFile` functions:

```text
fn listProjectFiles params:{ root:string } return:ProjectFileTree
  return value:{ files:[] folders:[] }

fn readProjectFile params:{ root:string path:string } return:"string"
  return value:""

fn writeProjectFile params:{ root:string path:string content:string } return:"boolean"
  return value:false

main
  views:appRoutes
  ipc functions:[listProjectFiles readProjectFile writeProjectFile]
```

`listProjectFiles` returns sorted paths for visible regular files below `root`, excluding hidden
entries and `AGENTS.md` or `CLAUDE.md`. `readProjectFile` returns UTF-8 text for one relative path
up to 2 MiB. `writeProjectFile` accepts UTF-8 content up to 2 MiB and replaces an existing regular
file after applying the same path checks. The runtime rejects absolute paths, traversal, and
symlinks. These helpers are intended for native editor tooling; they are not available through Web
or the portable standard library.

Use the same `main.ipc` registry and View action across Desktop, Android, and iOS. Keep host adapters thin. Web uses `request` against an explicit HTTP service and must not invoke native IPC.

## Validation checklist

- The imported function has explicit params and a return contract.
- `main.ipc` contains each function once.
- Every View invocation names a registered function.
- Arguments and bindings match declared types.
- Unknown, malformed, failed, and cancelled calls are covered.
- Generated Desktop, Android, and iOS bridges use the shared request/response contract.
- No platform-specific IPC registry or unnecessary loopback server was added.
