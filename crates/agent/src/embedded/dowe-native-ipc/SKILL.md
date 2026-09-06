---
name: dowe-native-ipc
description: Use for Dowe native IPC contracts: registering imported functions in main.ipc, invoking them from Views, designing desktop Android and iOS host bridges, and validating native-only behavior. Pair with dowe-views for UI callers and dowe-server for the imported function implementation.
---

# Dowe native IPC authoring

Native IPC lets a View call an allowlisted Dowe function through the native host without opening a loopback HTTP server. The contract is shared by Desktop, Android, and iOS. Web does not execute native IPC.

## Workflow

1. Read `main.dowe`, the imported View routes, and the imported server function.
2. Read `references/ipc.md` before changing the registration, invocation, or host contract.
3. Define the function under `server/` with explicit params and return shape. Load `dowe-server` for validation, authorization, persistence, or provider behavior.
4. Register the imported function exactly once in the root `main.ipc` block.
5. Invoke it from a View function with `invoke result fn:<name> args:{...}`. Load `dowe-views` for the caller and its loading, success, cancellation, and error states.
6. Validate the project with the compiler and run native target tests. Do not validate native IPC through a Web build.

## Registration

Use one shared registry instead of platform-specific blocks:

```text
import readSettings from "@/server/settings"
import saveSettings from "@/server/settings"

main
  app name:"Settings" bundle:"com.example.settings"
  views:viewRoutes
  ipc functions:[readSettings saveSettings]
```

Only imported Dowe `fn` declarations may be registered. Duplicate names are invalid. Keep `main.desktop` available for future Desktop-only capabilities; do not create `main.desktop.ipc`, `main.android.ipc`, or `main.ios.ipc`. `main.desktop.server` is a separate loopback HTTP/WebSocket contract and is not required for IPC.

## Invocation

Invoke only inside a View `fn`:

```text
fn load
  invoke result fn:readSettings args:{ userId:session.userId } update:settings
  if result.ok
    toast type:"success" title:"Settings" message:"Loaded"
  else
    toast type:"error" title:"Settings" message:"Unable to load settings"
```

The result has the shape `{ ok, data }`. Arguments must be JSON-compatible and should use declared bindings, Signals, constants, arrays, or objects. Keep the UI explicit about loading, success, empty, cancellation, and failure states. Never put secrets or authoritative authorization decisions in View arguments.

## Host contract

The generated native runtime sends a request containing a request id, function name, target, and JSON arguments to the host bridge. The host must verify the registered function through the compiled project, execute it through the shared runtime API, and return a matching `{ ok, data }` response. Unknown functions, malformed requests, unavailable bridges, and execution failures fail closed. A picker cancellation is a normal failed result and must be handled by the View.

The host implementation must remain target-specific while the registry, function action model, runtime invocation API, and response contract remain shared. Do not duplicate server function logic in Swift, Kotlin, Java, Rust host adapters, or JavaScript.

## Validation

Check the source compiler first, then the owning runtime and generators. Assert registration diagnostics, argument and return validation, request/response encoding, unknown-function rejection, and cancellation behavior. Inspect generated native output for all applicable targets. Web output must not claim native IPC support.
