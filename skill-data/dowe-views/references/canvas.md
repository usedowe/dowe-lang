# Canvas reference

`Canvas` is the portable retained 2D drawing surface. A page owns a Signal array of ordered
commands, and Dowe renders the same scene through browser canvas, Jetpack Compose, and SwiftUI.
Use it only when semantic components cannot express a drawing, diagram, infographic, game-like
scene, or custom pointer interaction. Scene Signal updates redraw the surface; source never calls
target drawing APIs.

```text
page canvasPage
  signal scene value:[{ type:"circle" x:160 y:90 radius:24 fill:"primary" }]

  Canvas scene:scene label:"Primary circle" w:"full" h:48
```

## Component props

| Prop | Values | Default |
| --- | --- | --- |
| `scene` | Signal array path | Required |
| `label` | Non-empty static accessibility label | Required |
| `viewWidth`, `viewHeight` | Positive integers for the logical viewport | `320` by `180` |
| `fit` | `contain`, `cover`, `stretch` | `contain` |
| `fps` | Integer `1` to `120` | `60` |
| `autoplay` | Boolean; `false` freezes motion at time zero | `true` |
| `background` | Dowe color token or `transparent` | `transparent` |
| `pixelated` | Boolean; disables image-command smoothing | `false` |
| `onPointer`, `onKey`, `onMotion` | Named view functions receiving the event as `item` | None |
| `motionRate` | Sensor samples per second, `1` to `60` | `30` |
| Style | `id`, `show`, spacing, sizing, `rounded`, `border`, `borderColor`, shadow | Common style behavior |

Canvas borders draw above the scene, so an opaque `background` cannot cover them.

## Drawing commands

Commands draw in array order; later commands appear above earlier commands. Color fields accept
semantic Dowe tokens. Coordinates and numeric values must be finite. Malformed or unknown runtime
commands are skipped instead of terminating the view.

| Type | Required fields | Common optional fields |
| --- | --- | --- |
| `rect` | `x`, `y`, `width`, `height` | `fill`, `stroke`, `strokeWidth`, `radius`, `opacity`, `rotation`, `motion` |
| `circle` | `x`, `y`, `radius` | `fill`, `stroke`, `strokeWidth`, `opacity`, `motion` |
| `line` | `x1`, `y1`, `x2`, `y2` | `stroke`, `strokeWidth`, `opacity`, `motion` |
| `polyline` | `points` | `closed`, `fill`, `stroke`, `strokeWidth`, `opacity`, `motion` |
| `text` | `x`, `y`, `text` | `fill`, `size`, `align`, `opacity`, `rotation`, `motion` |
| `image` | `src`, `x`, `y`, `width`, `height` | `fit`, `opacity`, `rotation`, `motion` |

## Interactive input

Input functions receive one normalized event object as `item`. Store it with a normal view
function, then drive command geometry from the Signal through command `bind` paths.

```text
page inputPage
  signal pointer value:{ x:160 y:90 }
  signal scene value:[{ type:"circle" x:160 y:90 radius:20 fill:"primary" bind:{ x:"pointer.x" y:"pointer.y" } }]
  fn capturePointer
    set pointer value:item

  Canvas scene:scene onPointer:capturePointer label:"Pointer playground" w:"full" h:48
```

Pointer events contain `kind`, `pointerType`, `id`, logical `x` and `y`, `dx`, `dy`, `inside`,
`buttons`, `pressure`, `primary`, and `timestamp`; active pointers stay captured through `up` or
`cancel`, and touch identifiers are independent. Keyboard events contain `kind`, `key`, `code`,
`repeat`, modifier booleans, and `timestamp`; Canvas becomes focusable when `onKey` is configured.
Motion events contain screen-oriented `acceleration:{ x y z }`, `rotation:{ alpha beta gamma }`,
`interval`, and `timestamp`; missing or denied sensors produce no callback.

Command `bind` values are shallow quoted Signal paths evaluated before every draw; a missing path
preserves the static command value. Listeners, captures, and sensors stop when the Canvas route is
removed.

## Animation

Attach `motion` to a command for time-based movement independent of frame rate:

```text
signal scene value:[{ type:"circle" x:24 y:90 radius:12 fill:"secondary" motion:{ vx:40 vy:12 pulse:0.8 wrap:true } }]
```

| Field | Behavior |
| --- | --- |
| `vx`, `vy` | Logical units per second on each axis |
| `rotation` | Degrees per second |
| `pulse` | Opacity oscillations per second |
| `wrap` | Wraps the translated origin at scene bounds |

Reduced-motion preferences and `autoplay:false` freeze the scene at time zero.

## Dynamic scenes and limits

A request or function may replace the scene Signal array and every target redraws from the new
commands. Keep an equivalent textual summary beside data graphics.

Canvas supports pointer, multitouch, physical keyboard, command bindings, accelerometer, and
device-orientation input. Physics, collision detection, gamepads, WebGL/3D, shaders, pixel
filters, video-frame processing, and JavaScript chart or game libraries are outside the contract.
