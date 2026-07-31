# Style and design-system reference

Dowe validates every design token and style prop while lowering source; invalid tokens, props,
breakpoints, and dynamic design expressions fail before target generation. Configure repeated
visual props once in `theme.dowe` `design`; write a local visual prop only when one instance
intentionally differs. A local prop always overrides the matching theme default, and omitted props
keep project or built-in defaults.

## Semantic color tokens

| Family | Base tokens | Soft tokens |
| --- | --- | --- |
| Action families `primary`, `secondary`, `tertiary`, `muted`, `success`, `info`, `warning`, `danger` | `<family>`, `on<Family>` | `soft<Family>`, `onSoft<Family>` |
| Structural `background` | `background`, `onBackground` | none |
| Structural `surface` | `surface`, `onSurface` | none |

`bg` and `color` accept color tokens. On a child-bearing container, `color:<token>` sets the
inherited foreground for descendant text and current-color paint until a descendant declares its
own `color`. This includes `Box`, `Section`, `Flex`, `Grid`, `Brand`, `Banner`, `Marquee`, and
`Scaffold`. `Box bg:<token>` paints the container after padding and size and before radius and
border.

`scheme` on `Button`, `ToggleTheme`, `Fab`, `fabAction`, `Slider`, `Input`, `Select`, `SideNav`,
and `RailNav` accepts action families only. `scheme` on `SelectTheme`, `Card`, `Video`, the chart
components, `Table`, `Dropzone`, `NavMenu`, `Sidebar`, `Tabs`, and `Drawer` also accepts
`background` and `surface`. Structural schemes have no soft pair; soft variants degrade to the
structural tokens.

## Variants

`Card`, `Video`, `Candlestick`, `ArcChart`, `AreaChart`, `BarChart`, `LineChart`, `PieChart`,
`Table`, `Button`, `ToggleTheme`, `SelectTheme`, `Dropzone`, `Input`, `Select`, `NavMenu`,
`SideNav`, `RailNav`, `Sidebar`, and `Drawer` support `solid`, `soft`, `outlined`, and `ghost`.
`Fab` supports `solid` and `soft`. `Tabs` supports `solid`, `outlined`, `line`, `ghost`, and
`pills`. Defaults are `variant:"solid"` and `scheme:"primary"` unless a component declares
otherwise: `SelectTheme` defaults to `outlined` plus `surface`, and `RailNav` defaults to `ghost`
plus `muted`. The normalized variant name is `outlined`.

`solid` maps the scheme family to its base and `on*` tokens, `soft` maps to `soft*` and `onSoft*`,
`outlined` uses a structural surface with a family-colored border, and `ghost` is transparent with
family-colored content. Child-bearing variant surfaces pass their resolved foreground token to all
of their content regions unless the descendant declares `color`; for example, a soft muted Card
supplies `onSoftMuted`, and AppBar or Footer supplies its content color to `top`, `start`, `center`,
`end`, and `bottom`.

## Style props and responsive values

Style props accept a scalar or a responsive object keyed by `xs`, `sm`, `md`, `lg`, and `xl`, such
as `p:{ xs:4 md:8 }`. A responsive object without `xs` does not apply below its first declared
breakpoint.

| Group | Props | Values |
| --- | --- | --- |
| Padding | `p`, `px`, `py`, `pl`, `pr`, `pt`, `pb` | Dowe numeric scale |
| Size | `w`, `h`, `minW`, `minH`, `maxW`, `maxH` | Scale values, `full`; `h`, `minH`, and `maxH` also accept `vh-<scale>` |
| Border | `rounded`, `border` | `xs`, `sm`, `md`, `lg`, `xl`, `full`; integers `1` to `4` |
| Layout | `justify`, `align`, `gap`, `columns`, `rows` | Layout keywords, grid tracks, scale values, validated pixel gaps |
| Grid item | `colSpan`, `rowSpan` | Positive integers on direct `Box`, `Section`, or `Card` children of `Grid` |
| Box position | `position`, `top`, `right`, `bottom`, `left` | Static position mode; responsive Dowe-scale offsets on absolute or fixed Box |
| Media background | `cover`, `overlay` | Static asset path or `https://` URL; boolean, opacity number, RGBA, or linear gradient |
| Section background | `background` | `soft`, `aurora`, `sunrise`, `ocean`, `meadow`, `slate` on `Section` |
| Boxed width | `boxed` | Static boolean on `Section`, `Scaffold`, `AppBar`, `Footer`, `BottomBar` |
| Elevation | `shadow`, `shadowColor` | `xs` to `xl`; semantic color family |
| Text | `size`, `color`, `bg`, `weight`, `spacing` | `xs` to `9xl`; color tokens; typography overrides |

Breakpoints are `xs:0`, `sm:640`, `md:768`, `lg:1024`, and `xl:1280` logical units on every
target. The numeric scale is `0` to `4` in `0.5` steps, `4` to `12` in `1` steps, then `12`, `14`,
`16`, `20`, `24`, `28`, `32`, `36`, `40`, `44`, `48`, `52`, `56`, `60`, `64`, `72`, `80`, and
`96`. One scale unit is `0.25rem` on web and `4` points or dp on native targets. `maxW` and `maxH`
set upper bounds without forcing the component to occupy the limit. `h:"vh-16"`, `minH:"vh-16"`,
and `maxH:"vh-16"` resolve against the viewport height minus the scale value.

## Typography

`Text` and `Title` use fluid typography driven by `size`, defaulting to `md`. `Title` defaults to
weight `600` with tight tracking; `Text` defaults to weight `400`. Both accept:

| Prop | Values |
| --- | --- |
| `size` | `xs` through `9xl` |
| `color`, `bg` | Design color tokens |
| `weight` | `thin`, `extralight`, `light`, `regular`, `medium`, `semibold`, `bold`, `extrabold`, `black` |
| `spacing` | `tightest`, `tighter`, `tight`, `normal`, `wide`, `wider`, `widest` |
| `font` | Dowe font token, overriding the `theme.dowe` `Text` or `Title` default |
| Common style props | `p*`, `w`, `h`, `minW`, `minH`, `maxW`, `maxH`, `rounded`, `border` |

`Text` and `Title` have no text-alignment prop; `align` is rejected. Center or align text through
the parent container: `Flex direction:"column" align:"center"` for centered stacks, or
`Grid justify:"center"` for centered grid content.

## Button metrics and navigation

`Button` defaults to `variant:"solid"`, `scheme:"primary"`, and `size:"md"`. `size` values `xs`
through `xl` set padding and minimum height from the numeric scale; explicit padding or `h`/`minH`
props override the size-derived metrics. `rounded` overrides the control radius.

A navigable Button uses static navigation props instead of `onClick`:

| Prop | Behavior |
| --- | --- |
| `href` | Internal route, `#fragment` or `/route#fragment` anchor, or `https://` external URL |
| `navigate` | `push` by default, or `replace` |
| `history` | `back` action without `href` |
| `target` | Web external URL target: `self` or `blank` |
| `externalMode` | Desktop and mobile external URL mode: `system` or `webview` |

Internal `href` values must resolve to connected routes, and fragments must resolve to validated
section ids. The same navigation props apply to navigable `NavMenu`, `RailNav`, and `SideNav`
entries. `javascript:`, `data:`, and `file:` schemes are rejected.

## Containers

`Box` has no default padding, border, radius, background, shadow, or flex behavior. `Section` owns
the outer band (background, cover, overlay, sizing, radius, border, anchor) and generates an inner
content body with responsive defaults `px:{ xs:4 md:6 }` and `py:{ xs:10 md:16 }`; the larger
vertical inset separates ordinary page bands without repeated props. `boxed:true` caps and centers
only that body at `96rem` web or `1536` native. `Section background:<preset>` cannot combine with
`cover` because both are base layers. `Card` defaults to `variant:"solid"`, `scheme:"primary"`,
theme radius, and inner padding `p:{ xs:4 lg:5 }`.

Padding overrides follow scope: `p` replaces all sides, `px`/`py` replace one axis, and
`pl`/`pr`/`pt`/`pb` replace one side while unspecified sides keep the default.

`Box`, `Section`, and `Card` add no implicit gaps between children. Use
`Flex direction:"column" gap:<scale>` for vertical rhythm, `Flex gap:<scale>` for horizontal
groups, and `Grid columns:<n> gap:<scale>` for structural tracks. Containers fill the available
width unless `w` declares another dimension.

`Flex` defaults are `direction:"row"`, `justify:"start"`, `align:"stretch"`, `gap:0`, and
`wrap:false`. `direction` accepts `row`, `column`, or a responsive object such as
`direction:{ xs:"column" md:"row" }`. `wrap:true` lets a resolved row continue on additional
lines.

`Grid` accepts `columns:3`, validated templates such as `columns:"200px 1fr 200px"` and
`rows:"100px auto"`, `gap:"10px 20px"` as row gap then column gap, plus `justify` and `align` for
cell alignment. `colSpan` and `rowSpan` are valid only on direct `Box`, `Section`, or `Card` grid
children, and a span wider than a statically known column count fails compilation.

## Box positioning

`Box position:"relative"` stays in normal flow and creates a portable layer plane. A direct
`Box position:"absolute"` child leaves flow and anchors to that plane. Wrap the actual Card, Chip,
Icon, Svg, or other content in the positioned Box; positioning props do not apply directly to
those semantic components.

```text
Box position:"relative" cover:"/assets/images/hero.jpg" minH:"vh-32" rounded:"xl"
  Box position:"absolute" top:{ xs:4 md:6 } right:{ xs:4 md:6 }
    Card variant:"solid" scheme:"surface" p:4
      Text weight:"bold"
        "Audience proof"
```

`position` is a static scalar accepting `static`, `relative`, `absolute`, or `fixed`. The four
offsets use the Dowe scale and may be responsive. Use only one edge on each axis. Missing axes
default to `top:0` and `left:0`; negative offsets, `zIndex`, and opposite-edge stretching are not
portable. A fixed Box anchors to the safe route viewport and cannot appear inside `each` or
`Splash`. Later positioned siblings render above earlier siblings.

## Cover and overlay

| Prop | Behavior |
| --- | --- |
| `cover:"/images/hero.jpg"` | Static asset path cover image |
| `cover:"https://example.com/hero.jpg"` | Validated HTTPS cover image |
| `overlay:true` | Black overlay at `0.4` opacity |
| `overlay:0.6` | Black overlay at the given opacity |
| `overlay:"rgba(0,0,0,0.5)"` | Validated RGBA overlay |
| `overlay:"linear-gradient(...)"` | Portable validated linear gradient overlay |

The stack renders image, then overlay, then content. `overlay` without `cover`, dynamic `cover`
values, and unsafe URL schemes fail compilation. `Box`, `Section`, and `Card` support both props.

## Section anchors

A visible component can declare a static quoted `id` that becomes a target-neutral anchor.
`Section id:"hero"` is the canonical band anchor; `Button href:"#hero"` and `href:"/page#hero"`
navigate to it. `NavMenu item`, `RailNav item`, and navigable `SideNav header` or `item` entries can
use the same fragment destinations. Fragment navigation scrolls smoothly unless reduced motion is
requested. On web, a fixed or sticky AppBar in the same Scaffold is measured so the destination
begins below the bar. Empty, dynamic, duplicated, or non-portable ids fail compilation.

`NavMenu scheme` styles trigger, open, and active-entry states consistently across web, Android,
and iOS. Its submenu and megamenu popovers remain visible structural surfaces using `background`
and `onBackground`, even for `ghost` or `outlined` menus. Popovers float without changing layout
and close after their content is activated. iOS uses the same Dowe-owned anchored overlay strategy
as `Dropdown` instead of a system popover. Navigation dispatches before dismissal so fragment links
can animate to their validated `Section` destination.

## Visibility with show

| Form | Example | Behavior |
| --- | --- | --- |
| Boolean | `show:false` | Hides on every target |
| Responsive boolean | `show:{ xs:false md:true }` | Hidden below `md`, visible from `md` |
| Signal bool | `show:isReady` | Visible while the Signal is `true` |
| Signal bool field | `show:state.ready` | Visible while the referenced field is `true` |
| Numeric comparison | `show:{ when:itemCount gt:10 }` | Visible while the comparison holds |

A hidden component does not occupy layout space. Responsive objects accept only booleans; a
missing `xs` keeps the component visible below the first declared breakpoint. Conditional objects
take a number Signal path in `when` and exactly one of `gt`, `gte`, `lt`, or `lte` with a numeric
literal. Direct paths must resolve to bool and compared paths to number.

`show` is available on `Box`, `Flex`, `Grid`, `Card`, `Code`, `Video`, `Canvas`, the chart
components, `Table`, `Divider`, `Button`, `ToggleTheme`, `SelectTheme`, `Fab`, `Slider`,
`Dropzone`, `Input`, `Select`, `Alert`, `AppBar`, `Footer`, `BottomBar`, `NavMenu`, `SideNav`,
`Sidebar`, `Scaffold`, `Tabs`, `Drawer`, `Title`, `Text`, and `Svg`. It is not available on
context-only entries such as `Option`, `fabAction`, `Path`, `column`, `tab`, or region blocks. An
`Alert` with both `visible` and `show` requires both to be true.

## View motion

`Box` and `Card` accept the static `animation` prop; it is invalid on other components.

| Value | Behavior |
| --- | --- |
| `none` | No visual animation |
| `fadeIn` | Enters from transparent to opaque |
| `slideUp`, `slideDown`, `slideLeft`, `slideRight` | Enters from the named offset while fading in |
| `scaleIn` | Enters from a slightly smaller scale while fading in |

Animations run once when the component appears, and every target generates the equivalent native
motion. Unsupported values fail before target generation.
