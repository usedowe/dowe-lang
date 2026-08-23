# Style and design-system reference

Dowe validates every design token and style prop while lowering source; invalid tokens, props,
breakpoints, and dynamic design expressions fail before target generation. Configure repeated
visual props once in `theme.dowe` `design`; write a local visual prop only when one instance
intentionally differs. A local prop always overrides the matching theme default, and omitted props
keep project or built-in defaults.

## Semantic color tokens

| Family                                                                                              | Base tokens                                       | Soft tokens                                             |
| --------------------------------------------------------------------------------------------------- | ------------------------------------------------- | ------------------------------------------------------- |
| Action families `primary`, `secondary`, `accent`, `muted`, `success`, `info`, `warning`, `danger` | `<family>`, `<family>Text`, `<family>Title`       | `soft<Family>`, `soft<Family>Text`, `soft<Family>Title` |
| Structural `background`                                                                             | `background`, `backgroundText`, `backgroundTitle` | none                                                    |
| Structural `surface`                                                                                | `surface`, `surfaceText`, `surfaceTitle`          | none                                                    |

`bg` and `color` accept color tokens. On a child-bearing container, `color:<token>` sets the
inherited foreground for descendant text and current-color paint until a descendant declares its
own `color`. This includes `Box`, `Section`, `Flex`, `Grid`, `Brand`, `Banner`, `Marquee`, and
`Scaffold`. `Box bg:<token>` paints the container after padding and size and before radius and
border. On `Text` and `Title`, `bg:<token>` paints the content-sized text surface; combine it with
`rounded` and padding when the text should read as a pill or badge.

`scheme` on `Button`, `ToggleTheme`, `Fab`, `fabAction`, `Slider`, `Input`, `Select`, `SideNav`,
and `RailNav` accepts action families only. `scheme` on `Accordion` accepts action families plus
`background` and `surface`. `scheme` on `SelectTheme`, `Card`, `Video`, the chart
components, `Table`, `Dropzone`, `NavMenu`, `Sidebar`, `Tabs`, `Drawer`, `AppBar`, `Footer`,
`Modal`, `Dropdown`, and `Tooltip` also accepts
`background` and `surface`. Structural schemes have no soft pair; soft variants degrade to the
structural tokens.

## Variants

`Card`, `Video`, `Candlestick`, `ArcChart`, `AreaChart`, `BarChart`, `LineChart`, `PieChart`,
`Table`, `Button`, `ToggleTheme`, `SelectTheme`, `Dropzone`, `Input`, `Select`, `NavMenu`,
`SideNav`, `RailNav`, `Sidebar`, `Drawer`, `Toast`, `Modal`, `Dropdown`, `Tooltip`, and
`Accordion` support
`solid`, `soft`, `outlined`, and `ghost`.
`Fab` supports `solid` and `soft`. `Tabs` supports `solid`, `outlined`, `line`, `ghost`, and
`pills`. Defaults are `variant:"solid"` and `scheme:"primary"` unless a component declares
otherwise: `SelectTheme` defaults to `outlined` plus `surface`, and `RailNav` defaults to `ghost`
plus `muted`. The normalized variant name is `outlined`.

`solid` maps the scheme family to its base, text, and title roles; `soft` maps to the matching
`soft*`, `soft*Text`, and `soft*Title` roles. Author `muted` as a lighter tonal counterpart of
`primary`, with `mutedText` and `mutedTitle` chosen for clear contrast against that lighter fill.
Use `muted` for lower-emphasis solid controls such as `Input` when a solid primary surface feels
too heavy, rather than treating muted as an unrelated neutral.
`outlined` uses a structural surface with a family-colored border, and `ghost` is transparent with
family-colored content. Child-bearing variant surfaces pass their resolved foreground token to all
of their content regions unless the descendant declares `color`; for example, a soft muted Card
supplies `softMutedText` to ordinary content and `softMutedTitle` to `Title`. AppBar or Footer
supplies its content roles to `top`, `start`, `center`, `end`, and `bottom`. Button labels use the
text role. Transparent `SideNav` headers use the visible base color of their `scheme`; an explicit
icon color remains a local override.
Native iOS rows explicitly restore the background foreground for inactive labels and descriptions
so a muted scheme cannot make them disappear against the page background.

`Accordion` keeps `variant` and `scheme` orthogonal across targets: `ghost` is a flat row treatment
with a 22% bottom separator, `soft` uses a quiet family surface with neutral item panels and a 16%
item border, `outlined` uses a structural panel with a family-colored outer and item border, and
`solid` uses the family base with a 24% paired-text item border. Structural schemes remain readable
in every treatment; `soft` falls back to structural roles when no soft token exists. The default
`Accordion` variant is `ghost`, and its item state plus bundled `SideNav` disclosure arrow are
generated from the same normalized model for web, Android Compose, the Android development launcher,
and iOS.

### Built-in component defaults

If `theme.dowe` omits a component entry, Dowe resolves the following values before web, desktop,
Android, or iOS generation:

| Component                                          | Defaults                                        |
| -------------------------------------------------- | ----------------------------------------------- |
| `Button`, `IconButton`                             | `scheme:"primary" variant:"solid" rounded:"md"` |
| `Card`                                             | `scheme:"surface" variant:"solid" rounded:"md"` |
| `Drawer`                                           | `scheme:"surface" variant:"solid"`              |
| `Toast`                                            | `scheme:"info" variant:"solid" rounded:"md"`    |
| `Section`                                          | `scheme:"background" variant:"solid"`           |
| `Accordion`                                        | `variant:"ghost"`                               |
| `Checkbox`                                         | `scheme:"primary"`                              |
| `Input`, `Date`, `Password`, `Select`, `Pin`       | `variant:"outlined" scheme:"primary"`           |
| `AppBar`, `Footer`, `Modal`, `Dropdown`, `Tooltip` | `scheme:"surface" variant:"solid"`              |
| `Tabs`                                             | `variant:"pills" scheme:"primary"`              |

These built-ins set no `border` or `shadow`. Resolution is per prop: an explicit component prop
wins, then the matching `design` slot, then this built-in value. A partial override such as
`Button scheme:"secondary"` therefore preserves the configured or built-in variant and radius.
See `/specs/features/00149-normalize-view-component-defaults`.

### Minimal source rule

Generated and hand-authored Views should omit resolved defaults. Prefer `Button "Log in"`,
`Button w:"full" "Log in"`, `Input bind:email label:"Email"`, `Tabs position:"top"`, and `Card` over declarations
that repeat `variant`, `scheme`, `size`, or `rounded` with their default values. Keep a visual prop
when it changes the default, is reactive, controls layout or behavior, supplies required content or
accessibility, or is the specific decision being demonstrated. Do not remove props such as
`w`, `p`, `href`, `onClick`, `label`, `bind`, `icon`, `loading`, `show`, or non-default `variant`,
`scheme`, `size`, `rounded`, `border`, or `shadow` values when they pass the admission gate below;
their availability alone is not a reason to keep them.

Admit a local prop only when at least one condition is true:

- The component contract, content, behavior, binding, or accessibility role requires it.
- It owns essential structure that a default cannot infer, such as `boxed:true`, responsive
  `columns`, `direction:"column"`, `cover`, a binding, or an event.
- It expresses a deliberate non-default choice visible in the requested design.
- A render of the default-first tree proves that this one prop fixes a specific mismatch on the
  smallest real owner.

Do not translate a screenshot's measurements into a complete prop list before rendering. Start
from the minimal semantic tree, then add proven exceptions one at a time. If a prop's reason cannot
be stated without saying only that the value is visible, common, or supported, omit it.

### Spacing economy

Dowe Views has no margin contract. Do not emit or invent `m`, `mx`, `my`, `mt`, `mr`, `mb`, or `ml`.
Translate a request for margin into the owning parent's `gap`, responsive flow, alignment, sizing,
or one intentional padding override on the real owner.

Resolve container spacing in this order:

1. Keep the component and theme defaults. An ordinary `Section` already provides
   `px:{ xs:4 md:6 }` and `py:{ xs:10 md:16 }`; a `Card` already provides `p:{ xs:4 lg:5 }`.
2. Render the minimal tree. `Grid` and `Flex` default to zero gap; add one `gap` on the owning
   container only when the child group needs explicit nonzero rhythm. Do not add a gap automatically
   with every Grid or Flex, and do not add padding merely to separate its children.
3. Add one local `p`, `px`, `py`, `pt`, or `pb` override only when a user requirement or reference
   blueprint proves that the default is insufficient. Prefer one axis and the smallest scope;
   do not repeat the same inset on `Section`, `Grid`, and `Card`.
4. Use `Box` only for its documented layer responsibility, never as a padding or margin workaround.

An explicit responsive padding object is an exception, not the normal way to author every band. A
transparent `Card variant:"ghost" p:0` used only as a layout wrapper is also unnecessary; remove it
unless the Card owns a meaningful semantic or behavioral boundary.

## Surface hierarchy and modern depth

Do not style a marketing page as a catalog of equal components. Establish three surface roles and
apply them consistently:

| Role       | Purpose                                                              | Typical treatment                                                                          |
| ---------- | -------------------------------------------------------------------- | ------------------------------------------------------------------------------------------ |
| Focal      | Primary offer, product stage, important metric, or conversion action | High-contrast `solid` or `soft`, largest radius, one strong shadow or glow, optional cover |
| Supporting | Feature, proof, process step, or secondary panel                     | Quiet `soft` or selective `outlined`, medium radius, restrained border or shadow           |
| Ambient    | Shell, background field, logo rail, technical texture, or separator  | Structural token, `ghost`, Section preset, cover plus overlay, Divider, or no Card at all  |

Use borders to describe structure and shadows to establish elevation. Applying both at maximum
strength to every surface destroys hierarchy. Reserve colored `shadowColor` for one or two focal
objects per viewport; use quiet structural contrast elsewhere. In dark themes, distinguish
`background`, `surface`, and at least one soft family so Cards do not disappear into the canvas or
form a wall of identical navy rectangles.

Create depth with supported relationships, not random effects:

- Put a full-bleed `Section cover` or background preset behind a boxed content rail.
- Place a focal Card, Image, chart, or product visual inside a relative Box and add one to three
  absolute proof or status wrappers.
- Use a nearby soft token for broad surfaces and a saturated family for small accents, values,
  active controls, and visual anchors.
- Combine one strong foreground silhouette with quiet background ornament; do not make every layer
  equally bright.
- Keep `translateX` and `translateY` out of ordinary layout. They do not replace AppBar regions,
  Flex `direction`/`justify`/`align`/`gap`, Grid `columns`/`rows`/`justify`/`align`/`gap`, padding,
  responsive direction, `w`, or `maxW`, and must not be used as measured-coordinate corrections
  after visual comparison.
- Never translate `AppBar`, `Brand`, `NavMenu`, `Drawer`, or another compound overlay trigger or
  root. Keep semantic navigation geometry untransformed so its menus and floating surfaces stay
  anchored to the component that owns them.
- Use responsive translation only as an advanced effect on a decorative or floating layer inside
  a documented relative Box scene when deliberate overlap cannot be expressed by normal flow and
  absolute offsets after trying Flex and Grid composition. Keep the effect inside compact bounds
  and preserve source reading order.

Natural visual detail comes from relationships: a label aligned to a divider, an oversized value
paired with small supporting copy, a Card that overlaps a media field, a repeated number system,
or a logo rail that changes the section rhythm. More borders and more Cards are not substitutes.

## Typography and editorial rhythm

Use typography to create composition before adding containers.

- Give the hero one dominant Title and constrain its measure through the containing Grid track or
  `maxW`; do not let every section title use the hero scale.
- Build a clear ladder: compact uppercase or wide-spaced eyebrow, display promise, readable body,
  and small proof or legal copy. Reuse the ladder rather than choosing unrelated sizes per band.
- Mix alignment intentionally across sections. Centering can suit a constellation or CTA; product,
  trust, and process sections often feel more natural with left-aligned editorial tracks.
- Use `RichText` marks only when the reference has a meaningful highlighted phrase. One marked
  phrase is a focal device; many marks become decoration noise.
- Keep body copy short enough to preserve the visual silhouette. Do not solve weak composition by
  adding explanatory paragraphs.

## Motion discipline

Motion is a finishing layer, not the source of visual quality. Use one entrance character per band
and gestures only on interactive or clearly actionable surfaces. A common restrained system is
`fadeIn` for text, `scaleIn` for one focal visual, and `lift` for clickable Cards. Do not animate
every child separately, mix all entrance presets on one screen, or use gesture props on static legal
and informational copy. Reduced-motion behavior remains owned by Dowe.

## Style props and responsive values

Style props accept a scalar or a responsive object keyed by `xs`, `sm`, `md`, `lg`, and `xl`, such
as `p:{ xs:4 md:8 }`. A responsive object without `xs` does not apply below its first declared
breakpoint.

Prop availability does not determine container choice. Apply style props to the real Section,
Grid, Flex, Card, media, control, or content owner. Do not introduce Box only to gain padding,
sizing, background, border, visibility, animation, or responsive values; reserve Box for an
exceptional layer plane that normal flow cannot express.

| Group              | Props                                               | Values                                                                                                                                                                               |
| ------------------ | --------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Padding            | `p`, `px`, `py`, `pl`, `pr`, `pt`, `pb`             | Dowe numeric scale                                                                                                                                                                   |
| Size               | `w`, `h`, `minW`, `minH`, `maxW`, `maxH`            | `w` and `minW` accept scale values, `full`, container `sm` through `7xl`, or quoted `10%` through `100%` by tens; `maxW` excludes percentages; height props accept scale values, `auto`, `full`, or `vh-<scale>` |
| Border             | `rounded`, `border`                                 | `xs`, `sm`, `md`, `lg`, `xl`, `full`; integers `1` to `4`                                                                                                                            |
| Layout             | `justify`, `align`, `gap`, `columns`, `rows`, `flex` | Layout keywords, numeric grid counts, scale values, validated pixel gaps; `flex` accepts `"initial"`, `"auto"`, `"none"`, or `1`, including responsive values |
| Grid item          | `colSpan`, `rowSpan`                                | Positive integers on direct `Box`, `Section`, or `Card` children of `Grid`                                                                                                           |
| Box position       | `position`, `top`, `right`, `bottom`, `left`        | Static position mode; responsive Dowe-scale offsets on absolute or fixed Box                                                                                                         |
| Media background   | `cover`, `overlay`                                  | Static asset path or `https://` URL; boolean, opacity number, RGBA, or linear gradient                                                                                               |
| Section background | `background`                                        | `soft`, `aurora`, `sunrise`, `ocean`, `meadow`, `slate` on `Section`                                                                                                                 |
| Boxed width        | `boxed`                                             | Static boolean on `Section`, `Scaffold`, `AppBar`, `Footer`, `BottomBar`                                                                                                             |
| Elevation          | `shadow`, `shadowColor`                             | `xs` to `xl`; semantic color family                                                                                                                                                  |
| Text               | `size`, `align`, `color`, `bg`, `weight`, `spacing` | `xs` to `9xl`; `start`, `center`, `end`, or `justify`; color tokens; typography overrides                                                                                            |

Breakpoints are `xs:0`, `sm:640`, `md:768`, `lg:1024`, and `xl:1280` logical units on every
target. The numeric scale is `0` to `4` in `0.5` steps, `4` to `12` in `1` steps, then `12`, `14`,
`16`, `20`, `24`, `28`, `32`, `36`, `40`, `44`, `48`, `52`, `56`, `60`, `64`, `72`, `80`, and
`96`. One scale unit is `0.25rem` on web and `4` points or dp on native targets. `auto` keeps the
height contribution content-driven at its active breakpoint. `full` uses a definite height made
available by the immediate parent; it does not create viewport height, so a child remains
content-sized when its normal-flow parent has no height bound. `maxW` and `maxH` set upper bounds
without forcing the component to occupy the limit. `h:"vh-16"`, `minH:"vh-16"`, and
`maxH:"vh-16"` resolve against the viewport height minus the scale value. Responsive height
values use the highest matching `xs` through `xl` breakpoint on every target, for example
`Grid columns:{ xs:1 md:2 } h:{ xs:"auto" md:"full" }`.

The width-only container values are `sm`, `md`, `lg`, `xl`, `2xl`, `3xl`, `4xl`, `5xl`, `6xl`, and
`7xl`. They represent `24rem` through `80rem` on web and the equivalent `384` through `1280`
points/dp on native targets. Use `w:"sm"`, `minW:"md"`, or `maxW:{ xs:"full" md:"lg" }`.
They are defined by `/specs/features/00175-add-container-width-values` and are not valid for
`h`, `minH`, or `maxH`.

`w` and `minW` also accept `"10%"`, `"20%"`, `"30%"`, `"40%"`, `"50%"`, `"60%"`, `"70%"`,
`"80%"`, `"90%"`, and `"100%"`. Percentages are relative to the immediate container's available
width and may be responsive, such as `w:{ xs:"100%" md:"50%" }`. Do not use percentages with
`maxW` or any height prop, and do not invent intermediate values such as `"15%"`.

For a height-bounded `Section`, the generated content body consumes the Section's inner height
after its effective padding. A direct child can use `h:"full"` or `minH:"full"` to fill that body;
the same rule works with explicit or responsive Section padding and remains content-sized when the
Section has no `h` or `minH`.

`flex` is a flex-item prop on `Section`, `Box`, `Flex`, `Grid`, and `Card`. It is effective only
when the direct parent is a flex parent: `Section`, `Box`, `Flex`, or `Card`. `Grid` accepts
`flex` when the Grid itself is that direct child, but Grid children remain grid items and ignore
the prop. Use `flex:{ xs:1 md:"none" }` for breakpoint changes; `1` fills the available main-axis
space, `auto` grows from its intrinsic basis, `initial` uses the intrinsic shrinkable basis, and
`none` remains intrinsic without growth or shrinkage.

`Brand` and `Svg` preserve intrinsic proportions when only one of `w` or `h` is authored. `Svg`
uses its `viewBox` for the automatic axis; `Brand` uses its children. With neither dimension,
`Svg` uses its default `6` by `6` size.

### Sizing, border, radius, and shadow compatibility

The following built-in components use the full shared style surface and therefore accept `w`,
`minW`, `maxW`, `h`, `minH`, `maxH`, `border`, `borderColor`, `rounded`, `shadow`, and
`shadowColor`:

| Group               | Components                                                                                                                                                                                                                                                                                                                  |
| ------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Layout and shell    | `Box`, `Section`, `Flex`, `Grid`, `AppBar`, `Footer`, `BottomBar`, `Sidebar`, `Scaffold`, `Drawer`                                                                                                                                                                                                                          |
| Navigation          | `NavMenu`, `SideNav`, `RailNav`, `Tabs`, `Stepper`                                                                                                                                                                                                                                                                          |
| Content and display | `Text`, `Title`, `Code`, `Video`, `Iframe`, `Device`, `Canvas`, `Table`, `Divider`, `Brand`, `Banner`, `Alert`, `Icon`, `Avatar`, `AvatarGroup`, `Badge`, `Chip`, `Skeleton`, `Card`, `Empty`, `Marquee`, `TypeWriter`, `RichText`, `Collapsible`, `Countdown`, `Map`, `Audio`, `Image`, `Accordion`, `Carousel`, `ChatBox` |
| Charts              | `Candlestick`, `ArcChart`, `AreaChart`, `BarChart`, `LineChart`, `PieChart`                                                                                                                                                                                                                                                 |
| Controls and forms  | `Input`, `Select`, `Button`, `IconButton`, `ToggleTheme`, `SelectTheme`, `Fab`, `Slider`, `Dropzone`, `ComboBox`, `CsvField`, `DragDrop`, `Editor`, `ImageCropper`, `Password`, `Phone`, `Pin`, `Textarea`, `Record`, `ToggleGroup`, `Checkbox`, `Color`, `Date`, `DateRange`, `RadioGroup`, `Toggle`                       |
| Overlays            | `Modal`, `AlertDialog`, `Tooltip`, `Toast`, `Dropdown`, `Command`                                                                                                                                                                                                                                                           |

`Svg` is the sizing exception: it accepts `w` and `h`, but not minimum or maximum sizing,
`border`, `borderColor`, `rounded`, `shadow`, or `shadowColor`. Context-only entries such as
`Option`, `fabAction`, `comboOption`, `csvColumn`, `dragGroup`, `dragItem`, `tab`, `step`, region
entries, `Splash`, and `Path` do not accept these shared style props.

## Typography

`Text` and `Title` use fluid typography driven by `size`, defaulting to `md`. `Title` defaults to
weight `600` with tight tracking; `Text` defaults to weight `400`. Both accept:

| Prop               | Values                                                                                       |
| ------------------ | -------------------------------------------------------------------------------------------- |
| `align`            | `start`, `center`, `end`, `justify`, or responsive object                                    |
| `size`             | `xs` through `9xl`                                                                           |
| `color`, `bg`      | Design color tokens                                                                          |
| `weight`           | `thin`, `extralight`, `light`, `regular`, `medium`, `semibold`, `bold`, `extrabold`, `black` |
| `spacing`          | `tightest`, `tighter`, `tight`, `normal`, `wide`, `wider`, `widest`                          |
| `font`             | Dowe font token, overriding the `theme.dowe` `Text` or `Title` default                       |
| Common style props | `p*`, `w`, `h`, `minW`, `minH`, `maxW`, `maxH`, `rounded`, `border`                          |

`size` is fluid/responsive when written as a scalar, so `size:"lg"` is the preferred form and
does not need a breakpoint object. Use `size:{ xs:"md" lg:"xl" }` only for an intentional
breakpoint override. `align` is independent from `Flex.align` and `Grid.align`; it controls the
text lines themselves and uses logical edges so `start` and `end` remain portable in RTL layouts.
Use a multiline string child when a line boundary must be deterministic; use `maxW` when natural
wrapping is acceptable. Both forms remain one semantic `Text` or `Title` node across targets.

## Button metrics and navigation

`Button` defaults to `variant:"solid"`, `scheme:"primary"`, and `size:"md"`. `size` values `xs`
through `xl` set padding and minimum height from the numeric scale; explicit padding or `h`/`minH`
props override the size-derived metrics. `rounded` overrides the control radius.

A navigable Button uses static navigation props instead of `onClick`:

| Prop           | Behavior                                                                            |
| -------------- | ----------------------------------------------------------------------------------- |
| `href`         | Internal route, `#fragment` or `/route#fragment` anchor, or `https://` external URL |
| `navigate`     | `push` by default, or `replace`                                                     |
| `history`      | `back` action without `href`                                                        |
| `target`       | Web external URL target: `self` or `blank`                                          |
| `externalMode` | Desktop and mobile external URL mode: `system` or `webview`                         |

Internal `href` values must resolve to connected routes, and fragments must resolve to validated
section ids. The same navigation props apply to navigable `NavMenu`, `RailNav`, and `SideNav`
entries. `javascript:`, `data:`, and `file:` schemes are rejected.

## Containers

`Box` has no default padding, border, radius, background, shadow, or flex behavior. This makes it an
advanced neutral layer primitive, not the default styling wrapper. A Box should normally be a
relative plane with direct absolute children or a fixed viewport layer. `Section` owns
the outer band (background, cover, overlay, sizing, radius, border, anchor) and generates an inner
content body with responsive defaults `px:{ xs:4 md:6 }` and `py:{ xs:10 md:16 }`; the larger
vertical inset separates ordinary page bands without repeated props. `boxed:true` caps and centers
only that body at `96rem` web or `1536` native. `Section background:<preset>` cannot combine with
`cover` because both are base layers. `Card` defaults to `variant:"solid"`, `scheme:"surface"`,
`rounded:"md"`, and inner padding `p:{ xs:4 lg:5 }`.

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

Resolve the direction before choosing centering props:

| Direction        | Main axis controlled by `justify` | Cross axis controlled by `align` |
| ---------------- | --------------------------------- | -------------------------------- |
| `row` or omitted | Horizontal                        | Vertical                         |
| `column`         | Vertical                          | Horizontal                       |

A default row Flex therefore needs `justify:"center"` to center a bounded child horizontally;
`align:"center"` only centers it vertically. A column Flex uses `align:"center"` for horizontal
centering and `justify:"center"` for vertical centering. A child with `w:"full" maxW:96` fills only
up to its maximum measure and does not center itself; the parent still owns its placement.

`Grid` accepts an integer `columns` value from `1` through `12`, `rows:auto` or a positive integer,
responsive count objects such as `columns:{ xs:1 md:3 }`, and `gap:"10px 20px"` as row gap then
column gap. Track templates such as `fr`, `px`, percentages, or `auto` strings are rejected for
`columns` and `rows` so the generated structure stays identical across web, iOS, and Android.
`colSpan` and `rowSpan` are valid only on direct `Box`, `Section`, or `Card` grid children, and a
span wider than a statically known column count fails compilation.

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

Use `cover` for media that belongs to a container's background and crops with that container. Use
`Image` for foreground media with independent bounds, aspect behavior, and alternative text. A
split media panel may use a neutral `Box cover:...` when it is genuinely a media stage; use Card
only when the region has grouped-surface semantics, not merely to obtain a background image.

| Prop                                   | Behavior                                   |
| -------------------------------------- | ------------------------------------------ |
| `cover:"/images/hero.jpg"`             | Static asset path cover image              |
| `cover:"https://example.com/hero.jpg"` | Validated HTTPS cover image                |
| `overlay:true`                         | Black overlay at `0.4` opacity             |
| `overlay:0.6`                          | Black overlay at the given opacity         |
| `overlay:"rgba(0,0,0,0.5)"`            | Validated RGBA overlay                     |
| `overlay:"linear-gradient(...)"`       | Portable validated linear gradient overlay |

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
and `backgroundText`, even for `ghost` or `outlined` menus. Popovers float without changing layout
and close after their content is activated. iOS uses the same Dowe-owned anchored overlay strategy
as `Dropdown` instead of a system popover. Navigation dispatches before dismissal so fragment links
can animate to their validated `Section` destination.

## Visibility with show

| Form               | Example                         | Behavior                                     |
| ------------------ | ------------------------------- | -------------------------------------------- |
| Boolean            | `show:false`                    | Hides on every target                        |
| Responsive boolean | `show:{ xs:false md:true }`     | Hidden below `md`, visible from `md`         |
| Signal bool        | `show:isReady`                  | Visible while the Signal is `true`           |
| Signal bool field  | `show:state.ready`              | Visible while the referenced field is `true` |
| Numeric comparison | `show:{ when:itemCount gt:10 }` | Visible while the comparison holds           |

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

All styled view components accept portable transforms, entrance animation, transition, and gesture
props. This includes layout and shell roots, navigation roots, content components, charts, controls,
forms, and overlays. Context-only entries such as `Option`, `fabAction`, `comboOption`, `csvColumn`,
`dragGroup`, `dragItem`, `tab`, `step`, region entries, `Svg`, and `Path` do not.

Availability is not a layout recommendation. The default authored value for every transform is its
identity, and most screens should author no translation props. Use `Title size` for typography,
container `w` or `maxW` for measure, AppBar regions for shell distribution, and Grid/Flex props for
geometry. Do not add `translateX` or `translateY` to close a gap, center content, compensate for a
parent width, reproduce a screenshot coordinate, or align one breakpoint. Translation belongs only
to an intentional advanced layer scene documented in the composition blueprint. Compound
navigation and overlay-owning roots remain untransformed even in those scenes; translate a purely
visual wrapper or ornament instead.

| Prop                       | Values                                                                         | Default                                               | Responsive |
| -------------------------- | ------------------------------------------------------------------------------ | ----------------------------------------------------- | ---------- |
| `rotate`                   | whole degrees `-180` through `180`                                             | `0`                                                   | Yes        |
| `scale`                    | decimal factor `0.5` through `2`                                               | `1`                                                   | Yes        |
| `translateX`, `translateY` | signed Dowe scale `-96` through `96` in half steps                             | `0`                                                   | Yes        |
| `animation`                | `none`, `fadeIn`, `slideUp`, `slideDown`, `slideLeft`, `slideRight`, `scaleIn` | `none`                                                | No         |
| `transition`               | `none`, `quick`, `smooth`, `spring`                                            | target feedback timing                                | No         |
| `gesture`                  | `none`, `lift`, `press`, `grow`, `tilt`                                        | `none`; `press` for `Button`, `IconButton`, and `Fab` | No         |

`animation` accepts:

| Value                                             | Behavior                                             |
| ------------------------------------------------- | ---------------------------------------------------- |
| `none`                                            | No visual animation                                  |
| `fadeIn`                                          | Enters from transparent to opaque                    |
| `slideUp`, `slideDown`, `slideLeft`, `slideRight` | Enters from the named offset while fading in         |
| `scaleIn`                                         | Enters from a slightly smaller scale while fading in |

Animations run once when the component appears. Transforms compose with entrance and gesture
motion instead of replacing it. Web and desktop use hover plus active feedback; touch targets use
press feedback. `Button`, `IconButton`, and the primary `Fab` trigger default to `press`, scaling
the complete surface to `0.94` during every press and restoring it afterward, including on Android
controls with a Dowe shadow. Android observes consecutive pointer sequences independently, so every
tap restarts the visual cycle; `gesture:"none"` opts out and an explicit preset replaces the default.
Every target disables non-essential motion when the operating system requests reduced motion.

Use an internal Button that replaces its current route when a page needs an explicit portable
replay control. Same-route replacement remounts the page without adding a history entry:

```text
Button href:"/motion" navigate:"replace"
  "Replay animations"
```

Whole-surface `onClick` is a separate action contract supported by `Button`, `IconButton`, `Avatar`,
`Fab`, `Empty`, `Box`, `Card`, and `Chip`. Its value is a visible Dowe `fn` reference or supported
inline `set` action, and the same source dispatches on web, desktop, Android, and iOS. Other compound
entries can expose their own separately documented action props.

```text
page chipMotionPage
  signal selected value:""

  fn selectMobile
    set selected value:"mobile"

  Flex direction:"column" align:"center" gap:3 animation:"fadeIn"
    Chip variant:"solid" scheme:"warning" size:"sm" rotate:-7 transition:"spring" gesture:"lift" onClick:selectMobile
      "Mobile Apps"
    Chip variant:"soft" scheme:"muted" size:"sm" rotate:4 transition:"smooth" gesture:"press"
      "Web Sites"
    Chip variant:"solid" scheme:"success" size:"sm" rotate:-4 transition:"quick" gesture:"grow"
      "Software"
    Chip variant:"solid" scheme:"muted" size:"sm" rotate:8 gesture:"tilt"
      "UI/UX Design"
```

Unsupported presets and out-of-range transform values fail before target generation.
