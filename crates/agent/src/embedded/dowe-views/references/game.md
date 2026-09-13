# Game reference

`Game` is the semantic component for a high-frequency game surface. It shares the portable Canvas
command model and normalized input for retained 2D, and adds a deterministic software `raycast3d`
renderer for Doom-like first-person worlds. It also owns an optional WebSocket lifecycle for
server-authoritative state. Use `Canvas` when the surface is a static visualization, drawing tool,
or custom illustration; use `Game` when the screen has a game loop or realtime transport.

```text
page gamePage
  signal scene value:[{ type:"circle" x:160 y:90 radius:24 fill:"primary" }]
  signal outbound value:{ type:"input" action:"idle" }
  signal connection value:"closed"
  signal message value:""

  fn captureMessage
    set message source:item.data

  Game scene:scene label:"Realtime game" socket:"/game" send:outbound status:connection onMessage:captureMessage w:"full" h:80
```

## Props

`label`, `viewWidth`, `viewHeight`, `fit`, `fps`, `autoplay`, `background`, `pixelated`, `onPointer`,
and `onKey` follow the Canvas contract. `onMotion` and `motionRate` are available for `canvas2d`
only; `raycast3d` rejects them until motion controls are part of that profile. `Game` has no child
content and does not accept Draw layer props.

| Prop | Contract | Default |
| --- | --- | --- |
| `renderer` | `canvas2d` or `raycast3d` | `canvas2d` |
| `scene` | Retained Canvas command Signal array; required by `canvas2d` | None |
| `world` | Object Signal with `map`, colors, and optional `sprites`; required by `raycast3d` | None |
| `camera` | Object Signal with `x`, `y`, `angle`, `fov`, and optional `pitch`; required by `raycast3d` | None |
| `controls` | `none` or built-in `doom` keyboard/touch controls | `none` |
| `moveSpeed` | Raycast movement speed, integer from `1` through `20` | `2` |
| `turnSpeed` | Raycast keyboard turn speed in degrees per second, integer from `1` through `360` | `120` |
| `onFire` | Action receiving position, angle, target, and hit for a raycast shot | None |

For `raycast3d`, `map` uses `0` for walkable cells and other characters for walls. The built-in
`doom` controls move with WASD/arrows, turn with pointer drag, walk/turn with touch drag, and fire
with Space, Enter, click, or tap. This is a bounded 2.5D renderer with grid walls and billboard
sprites, not a full Doom engine; arbitrary meshes, textures, physics, audio, AI, shaders, and
GPU/WebGL/Metal backends remain future work.

| Prop | Contract | Default |
| --- | --- | --- |
| `socket` | Static `ws://`/`wss://` URL, root-relative backend path, or string Signal path | None |
| `send` | Signal path whose current value is sent after open; strings are raw and other values are JSON | None |
| `status` | Writable string Signal receiving `connecting`, `open`, `closed`, or `error` | None |
| `onOpen` | Action receiving `item:{ source:"game", kind:"open", data:null }` | None |
| `onMessage` | Action receiving `item:{ source:"game", kind:"message", data:<raw text> }` | None |
| `onClose` | Action receiving `item:{ source:"game", kind:"close", data:{ code, reason } }` | None |
| `onError` | Action receiving `item:{ source:"game", kind:"error", data:<text or null> }` | None |
| `reconnect` | Reconnect after an unexpected close or failure | `true` |
| `reconnectDelay` | Delay in milliseconds, bounded from `100` through `60000` | `1000` |

The client sends only the latest distinct `send` value. It does not parse inbound messages, invoke
an action for every animation frame, or persist game state implicitly. Parse a JSON message in a
normal view function when the server protocol requires it.

For a raycast world, the handler can replace the object Signals with an authoritative snapshot:

```text
signal snapshot value:{}

fn captureSnapshot
  set snapshot source:parse.json value:item.data fallback:{}
  set world source:snapshot.world
  set camera source:snapshot.camera
```

## Backend boundary

The component is a client transport adapter; the Dowe server owns authentication, rooms,
authoritative state, validation, and message fan-out. Declare a Dowe `websocket` route at the same
root-relative path, or use an absolute secure URL for a separate backend. Browser credentials belong
in the server's upgrade policy and short-lived query parameters; do not put secrets in a Signal or
source literal.

Relative paths use the same origin in the browser. Native generated applications resolve them
against the client-visible backend endpoint. The route must use `wss://` outside local loopback.

## Performance boundary

Keep retained scene/world/camera snapshots small and replace them at simulation ticks rather than
writing state from every frame. `fps` controls redraw cadence, while command motion and raycast
movement are bounded and independent of device refresh rate. Reconnect, socket listeners, frame
loops, and input listeners stop when the route is removed.

Web and Desktop use the device-pixel-ratio-aware HTML Canvas runtime. Android Compose and the
development launcher use native software Canvas raycasting. iOS uses SwiftUI Canvas raycasting with
touch gestures. All modes reuse the same Game lifecycle and server boundary; a future GPU renderer
can consume the same contract.
