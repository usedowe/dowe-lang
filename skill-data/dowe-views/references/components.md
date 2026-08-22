# Built-in view component catalog

Dowe lowers this catalog from one target-neutral component tree to web, desktop, Android, and iOS.
Use the semantic component that owns the behavior instead of rebuilding it from generic containers.
Compiler diagnostics remain the final authority for exact props, values, binding types, and target
support.

Static string props use double quotes. Reactive props use bare Signal, Store, constant, scoped item,
or function bindings. Event props reference a named view `fn`. Common visual props such as spacing,
size, visibility, semantic color, border, radius, and shadow are accepted only where the component
contract declares them.

## Minimal component props

Start every component with no visual props. The compiler supplies built-in defaults, then applies
the matching `design` slot from `theme.dowe`; generated source should not repeat either layer.
Keep only props that change the default, bind reactively, provide required content or accessibility,
control layout or behavior, or are needed to demonstrate a deliberate variant.

Use this admission gate for every local prop:

1. Is the prop required by the component contract, content, binding, event, or accessibility role?
2. Does it define essential structure that the component default cannot infer, such as responsive
   Grid columns, a column Flex, a boxed Section, or a real cover asset?
3. Does it express a visible and intentional non-default choice, such as the outlined secondary
   action beside a default primary action?
4. After rendering the default-first tree, does it fix one specific comparison mismatch on the
   smallest real owner?

If every answer is no, omit the prop. A prop must not be added merely because diagnostics accept it,
because generators commonly emit it, or because the reference contains a measurable value for it.

```text
Button w:"full"
  "Log in"

Input bind:email label:"Email"

Card
  Title
    "Starter"
```

These declarations are equivalent to spelling out the default `Button` solid/primary/md visual
contract, the default outlined/primary control contract, and the default surface/solid/md Card
contract. The minimal Tabs form is `Tabs` (or `Tabs position:"top"` when documenting orientation),
which resolves to pills/primary. A non-default decision remains explicit, for example
`Button variant:"outlined"`, `Tabs variant:"line"`, or `Card shadow:"lg"`. Prefer one intentional
prop over a complete restatement of the component's default style.

Apply the same minimal-prop rule to container spacing. `Section` and `Card` already own responsive
insets. `Grid` and `Flex` have zero gap by default; add one `gap` only when a rendered sibling group
needs explicit nonzero rhythm, never as an automatic companion to `columns` or `direction`. Do not
add `px`, `py`, `pt`, `pb`, or `p` to every container, and do not invent unsupported margin aliases
such as `mt`. Add a local padding override only when the real owner has a documented exception that
the defaults cannot express. A `Card variant:"ghost" p:0` that only wraps another layout is not a
meaningful Card; remove the wrapper and let the owning layout component carry the tree.

## Layout and text

| Component | Use and essential contract |
| --- | --- |
| `Box` | Advanced neutral layer plane when normal flow cannot express the composition: normally a relative stage with direct absolute wrappers or a fixed viewport layer. Its normal-flow children are a vertical flex parent, so direct layout children can use `flex`. Do not use it for ordinary spacing, sizing, centering, backgrounds, borders, visibility, Grid gutters, or control wrappers; put those props on the semantic, flow, media, or control owner. |
| `Section` | Ordered page band and page-level vertical rhythm. A page begins with one or more sibling Sections; `boxed:true` constrains and centers only its generated inner body at `96rem` web or `1536` native, `center:true` or `center:{ xs:false md:true }` centers direct children responsively, `gap` controls vertical spacing between them with a default of `0`, and `h`/`minH` make that body available to direct `h:"full"` or `minH:"full"` children after padding. It establishes a vertical flex parent for direct children. |
| `Flex` | One-axis flex parent using `direction`, `gap`, `align`, `justify`, and optional wrapping. |
| `Grid` | Equal-width numeric column counts (1–12), optional numeric or `auto` rows, `gap`, and alignment. Grid itself may use `flex` as a flex child, but it does not establish a flex parent for its own children. |
| `Card` | One visibly independent surface with a contained background, border, radius, elevation, or inset treatment, such as a pricing offer or raised form. It establishes a vertical flex parent for direct children. A semantic grouping that remains visually flat uses Grid or Flex instead. Avoid nesting Card inside Card. |
| `Title` | One direct quoted or multiline visible-text child or one complete braced string binding; accepts logical `align` (`start`, `center`, `end`, or `justify`) and fluid `size`. |
| `Text` | One direct quoted or multiline visible-text child or one complete braced string binding; accepts logical `align` (`start`, `center`, `end`, or `justify`) and fluid `size`. |
| `Divider` | Horizontal or vertical separator; choose `orientation` instead of drawing a border-only Box. |

`Section boxed:true` keeps the outer band, background, cover, overlay, border, and anchor full width while limiting the generated content body to `96rem` on web and `1536` logical units on Android and iOS. It defaults to `false` and accepts a static boolean.

`Section center:true` centers direct children horizontally inside the generated content body. It defaults to `false` and accepts a boolean or responsive boolean object such as `center:{ xs:false md:true }`; the same value lowers to web, Android, and iOS alignment behavior.

`Section gap:3` adds vertical spacing between direct children. It defaults to `0`, accepts a Dowe scale or pixel value such as `gap:"8px"`, and supports responsive values such as `gap:{ xs:2 md:4 }`; the same value lowers to web, Android, and iOS spacing behavior.

`Section`, `Box`, `Flex`, `Grid`, and `Card` accept `flex:"initial"`, `flex:"auto"`,
`flex:"none"`, or `flex:1`, including responsive values such as `flex:{ xs:1 md:"none" }`.
The prop is effective only for a direct child of `Section`, `Box`, `Flex`, or `Card`. A direct Grid
child can therefore fill a height-bounded Section with `flex:1`; a Grid child never receives flex
item behavior because Grid owns tracks rather than a flex axis.

When a Section declares `h` or `minH`, its generated content body fills the remaining inner height
after effective padding. Use a direct `Grid`, `Flex`, or `Box` with `h:"full"` or `minH:"full"`
when a split panel, form, or media region must reach the padded Section edges. A Section without
`h` or `minH` remains content-sized; explicit or responsive Section padding stays on the generated
body.

`Text` and `Title` alignment is a text-node concern, not a container concern. Use `align:"start"`, `align:"center"`, `align:"end"`, or `align:"justify"`; the same logical value lowers to web, iOS, and Android. `RichText` remains a separate marked-text contract and does not accept `align`.

Use one multiline string child for an intentional hard line break:

```text
Title size:"7xl" align:"center" maxW:"6xl"
  """
  Full-stack development,
  from one codebase
  """
```

Use `maxW` for natural wrapping. Do not duplicate `Text` or `Title` nodes or add `Flex` only to
force a line boundary.

## Application shells and navigation

| Component | Use and essential contract |
| --- | --- |
| `AppBar` | Top application bar with optional full-width `top` and `bottom` regions around `start`, `center`, and `end`; `boxed:true` centers its inner content at `96rem` web or `1536` native while preserving the full-width surface. It stays visually flat across targets unless `border`, `bordered:true`, or `floating:true` requests separation. A direct `Scaffold appBar` with `position:"sticky" floating:true` overlays the web and desktop body so `main` remains visible beneath the floating surface. `dockOnScroll:true` requires `position:"fixed" floating:true` and makes web, desktop, Android, and iOS dock the floating surface at the viewport top after `100` logical scroll units. |
| `Footer` | Page or shell footer with optional full-width `top` and `bottom` regions around `start`, `center`, and `end`; it includes horizontal padding `4` from `xs` and `6` from `md`, top padding `10` from `xs` and `16` from `md`, and bottom padding `4` from `xs` and `6` from `md`, overridable with `p`, `px`, `py`, `pl`, `pr`, `pt`, or `pb`. `boxed:true` centers one shared inner container holding `top`, the central row, and `bottom` at `96rem` web or `1536` native while the surface remains full width. Put responsive `show` on children inside a region, not on the structural region block. |
| `BottomBar` | Bottom navigation containing one or more direct `tab` entries; each entry owns one Icon and navigation metadata; `boxed:true` centers the tab row at `96rem` web or `1536` native. |
| `NavMenu` | Horizontal navigation composed from direct `item`, `submenu`, or `megamenu` entries. For shell navigation, place it only as a direct child of AppBar `center` or `end`; never use it as the body of a `Drawer`, `Sidebar`, or other vertical surface. Submenu and megamenu content opens in a Dowe-owned floating overlay on web, Android, and iOS; activating the same trigger again closes it. The overlay uses the structural background surface, preserves `scheme` for trigger and active states, dispatches fragment or route navigation before closing, and uses the same anchored overlay strategy as `Dropdown` on iOS. |
| `SideNav` | Detailed vertical navigation with optional `header`, direct `item`, `divider`, and `submenu` entries. Use it directly or through a static reusable component in `Sidebar body` and `Drawer body`. `submenu open` is initial state; the runtime retains later toggles in session memory across unmount and remount. Use distinct `id` values for structurally identical SideNav instances that need independent memory. |
| `RailNav` | Narrow icon navigation with direct `item` and `divider` entries; each item requires quoted `label` and Solar `icon`. |
| `Sidebar` | Shell side surface with optional `header`, required `body`, and optional `footer` regions. |
| `Scaffold` | The single normal layout root. It accepts optional `appBar`, `start`, `end`, `bottomBar`, and `overlays` regions plus required `main`; `boxed:true` centers only the `start`/`main`/`end` body at `96rem` web or `1536` native. |
| `Splash` | Direct layout or page boundary with required `bind` to a boolean Signal or View Store. Its children replace every normal root while the binding is true; it has no default spinner or style. |
| `Drawer` | Openable side surface with optional `header`, required `body`, and optional `footer`; direct view children also form body content. When the surface contains navigation, use `SideNav` in `body`; `NavMenu` remains the horizontal AppBar navigation component. |
| `Tabs` | Related panels selected through one or more direct `tab` entries with unique quoted `id` and `label`. |
| `tab` | Context-only child of Tabs or BottomBar. A Tabs entry owns panel children; a BottomBar entry owns navigation metadata and one Icon. |
| `Stepper` | Ordered numbered workflow selected through direct `step` entries; use `scheme` and `horizontal` or `vertical` orientation. |
| `step` | Context-only child of Stepper with unique quoted `id`, quoted `label`, and panel children. |

`Scaffold boxed:true` centers and limits only the `start`, `main`, and `end` body while leaving the outer shell, bars, and overlays full width.

Section, Scaffold, AppBar, Footer, and BottomBar use the wide boxed content cap of `96rem` on web and `1536` logical units on Android and iOS. Their outer bands, shells, and bar surfaces remain full width.

### AppBar composition rules

`AppBar` owns the shell surface and its explicit regions. Use one AppBar directly under
`Scaffold appBar`, then place the brand in `start`, the primary horizontal `NavMenu` directly in
`center` (or a compact/secondary `NavMenu` directly in `end`), and actions directly in `end`.
`center` is the flexible region; `start` and `end` size to their content. Put responsive `show` on
the built-in node that changes visibility:

AppBar regions already provide a horizontal flex row, vertical centering, and component-owned gap.
Keep direct children flat: place `Brand`/logo and controls directly in `start`, and place the mobile
Drawer trigger either before the brand in `start` or directly in `end`. Do not use `Flex` just to
align or space those siblings; reserve it for a real nested axis, wrapping, or independently
structured group.

```text
layout SiteLayout
  signal openNavigation value:false
  Scaffold
    appBar
      AppBar position:"fixed" floating:true boxed:true
        start
          Brand href:"/" label:"Site home"
            Text weight:"black"
              "SITE"
        center
          NavMenu show:{ xs:false md:true }
            item label:"Overview" href:"/"
            item label:"Contact" href:"/#contact"
        end
          Button show:{ xs:false md:true } href:"/#contact"
            "Get started"
          IconButton show:{ xs:true md:false } icon:"menu-dots" label:"Open navigation" onClick:{ set:openNavigation value:!openNavigation }
    main
      children
    overlays
      Drawer open:openNavigation show:{ xs:true md:false }
        body
          SiteNavigation
```

Do not use `center > Box show:{ ... } > NavMenu` as a visibility or alignment shim, and do not
place a second desktop or mobile AppBar inside `overlays`. A `Box` is appropriate in `top` or
`bottom` only when that full-width region is itself a styled band, or inside a slot when it is a
real positioned visual layer. Imported reusable components do not accept props; use the direct
built-in `NavMenu` only in the AppBar and keep the prop-free vertical `SideNav` component in both
the `Sidebar body` and `Drawer body`.

## Controls and theme selection

| Component | Use and essential contract |
| --- | --- |
| `Button` | Text action or navigation control. Use one direct quoted or complete braced text child, reference a view function with `onClick`, and bind `loading` or `disabled` to boolean Signal paths when the action is pending or invalid. Loading reuses the bundled `svg-spinners:3-dots-move` Icon; both states block duplicate actions, and `disabled:true` preserves the authored scheme and variant at `0.5` opacity across web, desktop, Android, and iOS. Labels and icons are not selectable. The full control defaults to press feedback at scale `0.94`; `gesture:"none"` opts out. |
| `IconButton` | Accessible icon-only action. Supply quoted `label` and Solar `icon`; use `onClick` or supported navigation props. Its full square surface defaults to press feedback at scale `0.94`; `gesture:"none"` opts out. |
| `ToggleTheme` | Control that switches between configured themes without duplicating theme state in page source. |
| `SelectTheme` | Theme selector for the configured named theme catalog. |
| `Fab` | Primary floating action with optional direct `fabAction` secondary actions. Place shell-level floating behavior in Scaffold overlays. Its primary trigger defaults to press feedback at scale `0.94`; `gesture:"none"` opts out. |
| `fabAction` | Context-only secondary action inside Fab with an icon, label, and function or navigation target. |
| `Record` | Recording control driven by named start, pause, resume, cancel, and confirm functions where supported. |
| `ToggleGroup` | One-of-many or multi-choice control with direct `item` entries, a state value, and a named change function. |
| `Pagination` | Binds the current page with `bind`, accepts a static count or numeric Signal in `total`, and uses `pageSize` plus optional `onChange`; the portable subset supports at most 25 pages. |

## Forms

| Component | Use and essential contract |
| --- | --- |
| `Input` | Single-line value bound through `bind`; add a quoted label and the input type accepted by diagnostics. It accepts `helpText`, `errorText`, and direct `validate` children. `size` accepts `sm`, `md`, or `lg`. |
| `Select` | Bound choice control containing direct `Option` and `validate` entries. It accepts `helpText` and `errorText`; `size` accepts `sm`, `md`, or `lg`. |
| `Option` | Context-only Select entry with quoted `value`, `label`, and optional description. |
| `validate` | Context-only validation rule for `Input`, `Date`, `Pin`, `Phone`, `Select`, or `Checkbox`. Supply quoted, non-empty `rule` and `message` props and no children. |
| `Slider` | Bound numeric value constrained by its minimum, maximum, and step. |
| `Dropzone` | File-drop and picker surface with accepted file and size limits; web uses drag-and-drop, while iOS and Android open their native document selectors and show selected file summaries. |
| `ComboBox` | Searchable bound choice control containing one or more direct `comboOption` entries. |
| `comboOption` | Context-only ComboBox entry describing one selectable value. |
| `CsvField` | Bound CSV input whose schema is declared by one or more direct `csvColumn` entries. |
| `csvColumn` | Context-only CsvField column contract with field, label, and accepted column metadata. |
| `DragDrop` | Reorder or transfer surface containing direct `dragItem` entries or `dragGroup` collections. |
| `dragGroup` | Context-only DragDrop group containing one or more `dragItem` entries. |
| `dragItem` | Context-only draggable entry with stable identity and display data. |
| `Editor` | Bound rich text or source editor using the supported language, limits, and named change workflow. |
| `ImageCropper` | Bound local image selection and crop editor with shared preview, Reset/Cancel/Apply/Remove transitions, portable aspect and file limits, and a `data:image/*;base64,...` result. |
| `Password` | Bound password input with Dowe-owned strength and validation behavior plus a shared `Icon` reveal action using `eye` and `eye-closed`; visible Show/Hide text is not rendered. |
| `Phone` | Bound digit-only local phone input with separate dial-code storage and direct `validate` children. A floating label stays over the local-number input after the country selector. It uses the same Dowe-owned anchored searchable country popover, 12-unit trigger inset, compact search, horizontal flag/name/dial rows, selected state, and ordering on web, Android Compose, the Android launcher, and iOS. |
| `Pin` | Bound fixed-length PIN or verification-code input with direct `validate` children, Input-scaled `sm`, `md`, and `lg` cells, automatic focus movement after accepted characters, distributed paste, and text, password, or numeric modes. Android reduces cell widths evenly when a narrow parent cannot fit their nominal widths. `PinField` is rejected. |
| `Textarea` | Bound multiline text with row and length limits. |
| `Checkbox` | Bound boolean choice with a quoted accessible label. It accepts `helpText`, `errorText`, and direct `validate` children; `required` means the value must be true. |
| `Color` | Bound canonical `#RRGGBB` value using the portable saturation/brightness plane, hue slider, preview, contrast foreground, and optional Hex, RGB, CMYK, and OKLCH rows. |
| `Date` | Input-like bound date with direct `validate` children, a Dowe-owned calendar dropdown, month navigation, selected/today states, and optional minimum and maximum values. |
| `DateRange` | Input-like bound start/end range with a Dowe-owned calendar dropdown, range highlighting, automatic ordering, and optional limits. |
| `RadioGroup` | Bound single choice composed from one or more direct `item` entries. |
| `Toggle` | Bound boolean control with a quoted accessible label. |

For authentication pages, keep `hideStrength` disabled on login forms by using
`hideStrength:false` or omitting the prop. Enable `hideStrength:true` only on registration forms.

`Input`, `Select`, `ComboBox`, `Password`, `Phone`, `Color`, `Date`, and `DateRange`
share one single-line height contract. `sm`, `md`, and `lg` are 32, 40, and 48 logical units;
`labelFloating:true` adds 8 units, producing 40, 48, and 56. Their value and placeholder use the
matching body `sm`, `md`, or `lg` typography on web, Android, and iOS. `Textarea` remains
rows-driven, but its text follows the same typography scale.

### Portable form validation

Declare validation as direct structural children of the control. Keep rules ordered because Dowe
shows the first failing message. Validation becomes visible after blur, selection/close, or checkbox
activation, then updates on every value change. An explicit `errorText` takes priority over rule
errors, and rule errors take priority over `helpText`.

```text
signal form value:{ email:"" role:"" accepted:false }

Input bind:form.email label:"Email" helpText:"Use your work address."
  validate rule:"required" message:"Email is required."
  validate rule:"email" message:"Enter a valid email address."

Select bind:form.role label:"Role"
  validate rule:"required" message:"Choose a role."
  Option value:"admin" label:"Administrator"

Checkbox bind:form.accepted label:"Accept the terms"
  validate rule:"required" message:"You must accept the terms."
```

The portable rule set is `required`, `email`, `min:N`, `max:N`, `url`, `phone`,
`pattern:EXPRESSION`, `alphanumeric`, `numeric`, `alpha`, `matches:PATH`, `strongPassword`,
`creditCard`, `date`, `minWords:N`, and `maxWords:N`. `N` must be a positive integer.
`min`, `max`, and the minimum length in `strongPassword` count UTF-16 units to preserve exact
reference behavior. `date` accepts `YYYY-MM-DD` with month `01` through `12` and day `01` through
`31`.
`matches:PATH` must identify a compatible Signal or View Store value in the current reactive scope.
`pattern` accepts only the compiler-validated portable regular-expression subset. Do not implement
validation with target-specific JavaScript, Swift, or Kotlin.

### Derived Signal form state

Controls with `validate` children are grouped by the root of their `bind` path. The root Signal
exposes read-only `isValid`, `isInvalid`, `errors.<field>`, and `touched.<field>` paths. These virtual
properties never change the stored Signal value or a request body. Use `Button disabled:<path>` to
disable an action, and call `validate <signal>` as the first statement in a submit function to mark
all registered fields touched and stop the sequence before its request when invalid.

```text
signal formLogin value:{ email:"" accepted:false }

Input bind:formLogin.email label:"Email"
  validate rule:"required" message:"Email is required."

Checkbox bind:formLogin.accepted label:"Accept terms"
  validate rule:"required" message:"Accept the terms."

Button disabled:formLogin.isInvalid onClick:submit
  "Log in"

fn submit
  validate formLogin
  request result method:"POST" route:"/api/auth/login" body:formLogin
```

## Media, code, icons, and custom drawing

| Component | Use and essential contract |
| --- | --- |
| `Code` | Displays source without executing it. Set `language`, `content`, and optional template or copy behavior. |
| `Video` | Portable HTTPS video or HLS playback with optional poster, aspect, autoplay preference, and Dowe-owned controls. |
| `Iframe` | Embeds one quoted HTTPS URL or root-relative internal route. Quoted `src` and accessible `title` are required. During native `dowe dev`, an internal route uses the active Views origin instead of the API `BACKEND_URL`. |
| `Device` | Responsive preview frame that contains exactly one Iframe and selects a supported device profile. |
| `Canvas` | Custom drawing or pointer surface for visuals that semantic components cannot express; keep its commands and data target-neutral. |
| `Audio` | Portable audio playback for a supported static source. Use `src`, optional `subtitle` and `avatarSrc`, and `variant`/`scheme`; Dowe owns the play/pause control, 50-bar seekable waveform, remaining-time footer, and web/Android/iOS interaction parity. |
| `Camera` | Portable still-photo capture. Use `facing`, `label`, `disabled`, and named lifecycle functions; capture results include a target-local `url`, `mimeType`, dimensions, and `facing`. |
| `Microphone` | Portable audio recording. Use `label`, optional positive `maxDuration`, `disabled`, and named lifecycle functions; stop results include a target-local `url`, `mimeType`, and `durationMs`. |
| `Image` | Portable original media whose quoted `src` is a project asset path such as `/assets/images/hero.jpg` or an HTTPS URL, or whose bare `src` resolves to a typed string constant, Signal, or current `each` item field. Use `alt` text (empty marks it decorative), `aspect` (`horizontal`, `vertical`, `square`, `auto`), `objectFit`, `scheme`, and `rounded`. Web download and fullscreen actions are hidden by default; set `hideControls:false` to enable both. An unavailable source keeps the styled frame as a placeholder without crashing, so authoring the final path first and adding the file later is the canonical placeholder workflow. Never rebuild a photograph with `Svg` or `Canvas`, and never use the design reference or a crop from it to flatten UI into an image asset. |
| `Icon` | Bundled vector selected by quoted `name` or a string Signal, constant, or current `each` item path. Names use Solar variants, `country-flags:<ISO code>`, animated `svg-spinners:<name>`, or `svg-logos:<name>`. A plain Solar name is linear; append `-broken`, `-outline`, `-bold`, `-line-duotone`, or `-bold-duotone` for another variant. Web, native targets, and the Android development launcher update from the shared catalog; invalid runtime values fall back to the validated initial icon. |
| `Svg` | Portable vector using either quoted `viewBox` plus direct `Path` children, or runtime `data:<reference>` with no static paths. If only `w` or only `h` is authored, the other axis stays automatic and preserves the vector ratio; with neither, the default is `6` by `6`. |
| `Path` | Context-only Svg path with quoted `d`, paint, optional `fillRule:"nonzero|evenodd"`, and optional matrix transform. Use `evenodd` to preserve holes in compound paths. |

`Icon name` accepts a quoted catalog value or a readable string path. A collection can bind its
current item, and a Signal can change the selected catalog entry without rebuilding the view tree.
Names must exist in the bundled catalog at their initial value and diagnostics reject unknown names:
for example `magnifier` and
`magnifier-bold-duotone` are valid but `search` is not. Standalone icons accept
`fill:<color token>`, `stroke:<color token>`, `w`, and `h`; `style` is not an Icon prop.

Iframe and remote media do not bypass server embedding, authentication, cookie, transport, or
platform security policy. They never authorize user-authored JavaScript or native host bridges.

## Charts and tabular data

| Component | Use and essential contract |
| --- | --- |
| `Candlestick` | Financial OHLC data from a compatible Signal, with the implemented stream and visual options. |
| `ArcChart` | Non-negative arc or radial values from `data`, with portable palette and label behavior. |
| `AreaChart` | Series data rendered as filled lines; provide compatible `data` and series metadata. |
| `BarChart` | Categorical values rendered as bars from compatible `data` and optional series metadata. |
| `LineChart` | Series data rendered as lines from compatible `data` and series metadata. |
| `PieChart` | Non-negative categorical values rendered as slices from compatible `data`. |
| `Table` | Semantic portable table for a Signal or compatible immutable array of scalar rows. Use one or more direct `column` definitions with quoted field and label metadata; use `references/table.md` for advanced table composition. |

Charts consume portable Signal data rather than a target-specific chart library. Category charts
(`ArcChart`, `BarChart`, `PieChart`) read items with `label` and `value`. Point charts
(`AreaChart`, `LineChart`) read items with numeric `x` and `y`, or a `series` Signal whose items
contain `label` and `data:[{ x y }]`. Optional item `color` fields must be Dowe color tokens.
Every chart accepts `variant`, `scheme`, `size` (`sm` to `xl`), `palette` (`default`, `rainbow`,
`ocean`, `sunset`, `forest`, `neon`), `legendPosition` (`top`, `right`, `bottom`, `left`, `none`),
`emptyLabel`, `loading`, and `hideLegend`. Diagnostics reject missing, incompatible, or invalid
negative data where the chart contract requires non-negative values.

`ArcChart` optionally reads a positive `max` per category for independent progress arcs. It keeps
the plot centered in a square responsive viewport on web, Android, and iOS; `right` and `left`
legends stack below the plot when space is narrow. Its portable options are `centerText`,
`centerValue`, `thickness`, `gap`, `startAngle`, `endAngle`, `showInlineLabels`, `hideValues`,
and `showGlow`, and the same Signal update refreshes arcs, labels, center content, and legend.

`PieChart` keeps its plot centered in a square responsive viewport. A `right` or `left` legend
stacks below the plot when the available width is narrow, and web, Android, and iOS observe the
same `data` Signal so slice geometry, totals, donut center content, and legend entries update
together. Its portable options include `donut`, `donutWidth`, `centerLabel`, `centerValue`,
`startAngle`, `padAngle`, `hideLabels`, `hideValues`, `hidePercentages`, and `showGlow`.

Each `Candlestick` item provides `time` plus numeric `open`, `high`, `low`, and `close`. Optional
props include `stream` for an SSE feed upserted by `time`, `upColor` and `downColor` tokens, and
`maxPoints`. Validation rejects OHLC values where `high` or `low` contradicts the body.

`Table data:<array-path>` requires at least one direct `column` with quoted `field` and `label`;
`field` is a relative row path such as `profile.email`, optional `align` accepts `start`,
`center`, or `end`, and optional `width` accepts static portable hints such as `160px`, `25%`,
`1fr`, `auto`, `min-content`, or `max-content`. Table props include `size` (`sm`, `md`, `lg`),
`striped`, `bordered`, `dividers`, `emptyTitle`, and `emptyDescription`, plus applicable common
style props. Cells render strings, numbers, and booleans; objects, arrays, null, and missing fields
render empty. Web and desktop use a real table inside horizontal overflow; Android Compose and iOS
use native readable rows with the same headers and empty copy. Sorting, pagination, selection,
search, toolbars, and custom cell renderers are outside the Table contract; compose them around the
component or use the responsive Grid pattern in `references/table.md` when cells need rich content.

## Display, feedback, and rich content

| Component | Use and essential contract |
| --- | --- |
| `Alert` | Inline status feedback using a supported type, quoted message, variant, and semantic scheme. |
| `Avatar` | Person or entity image, initials, or optional `icon` region with supported action and navigation behavior. |
| `AvatarGroup` | Static `item` children or bound `items`, with visible-count and overflow behavior. |
| `Badge` | Compact status or count surface containing one or more view children. |
| `Brand` | Logo or identity container with one or more arbitrary view children, optional quoted `href` navigation, optional accessible `label`, and Box-compatible `w` and `h`; if only one dimension is authored, the other stays automatic. A native static `Svg w:"full" h:"full"` child preserves its `viewBox` ratio against that outer constraint. It adds no Button chrome. When used as an AppBar child, mount `Brand` or an imported Logo component directly in a region; do not wrap it with `Flex` just to place it beside sibling controls. |
| `Banner` | Full-width external surface with one or more arbitrary view children, required quoted HTTPS `href`, optional accessible `label`, and common background, cover, spacing, sizing, border, radius, shadow, and visibility props; web opens a protected new tab and native targets use the system browser. |
| `Chip` | Compact labeled token with optional static Solar `startIcon` and `endIcon` props or custom `start` and `end` icon regions, supported close behavior, portable motion props, and an optional whole-chip `onClick` action. Icon props scale with the Chip `size` across web, Android, and iOS. |
| `Skeleton` | Loading placeholder sized to the content surface it represents. |
| `ChatBox` | Bound message list with named send and pagination functions plus loading, sending, and streaming state. |
| `Empty` | Empty-state title, description, and optional action or navigation target. `type` selects the shared Solar `bold-duotone` icon: `playlist`, `result`, `data`, or `template`; do not add a custom icon prop. |
| `Marquee` | Repeating overflow presentation for one or more view children. |
| `TypeWriter` | Sequential text presentation composed from one or more direct `item` entries. |
| `RichText` | Portable wrapping styled text composed from one or more direct `mark` runs. Use `title:true` for the Title scale or leave it false for the Text scale. Across web, Android, and iOS, mark backgrounds remain content-sized and oversized marks wrap on whole-word boundaries with centered lines inside the available container. |
| `mark` | Context-only RichText run with quoted text, a semantic scheme, and one of `mark`, `grad`, `pill`, `slant`, `glow`, `under`, `strike`, `box`, `wave`, `neon`, `pop`, or `tag`. |
| `Collapsible` | Expandable content with a quoted label and one or more view children. |
| `Countdown` | Time-based display with an optional named completion function; large values expand, while narrow containers compact `lg` and `xl` before bounded horizontal scrolling. |
| `Map` | Portable map with direct `marker` and optional route `waypoint` entries plus named location or route functions. |
| `marker` | Context-only Map marker with stable id, latitude, longitude, and optional named click function. |
| `waypoint` | Context-only Map route point with latitude and longitude. |
| `Accordion` | Expandable collection composed from one or more direct `item` entries. Each item requires a quoted `id` and `label`, accepts optional `disabled` and `defaultOpen`, and owns normal view children as its body. `multiple:false` keeps at most one item open; `multiple:true` permits independent items. It accepts `variant` (`solid`, `soft`, `outlined`, `ghost`), `scheme` (action or structural family), and common style props. The built-in treatment is `ghost`; `variant` controls surface geometry while `scheme` supplies the semantic color roles. Web, Android Compose, the Android development launcher, and iOS share the same state, SideNav disclosure arrow, metrics, and motion contract. |
| `Carousel` | Slide collection composed from one or more direct `slide` entries. `variant` selects the scroll/effect preset; effect variants derive their transform from the current slide distance on web, Android, and iOS. `showNavigation`, `hideControls`, `hideIndicators`, `showCounter`, `indicatorType`, `disableLoop`, `slideWidth`, `slideHeight`, `slidesPerView`, and `gap` share one active-index and native-scroll contract across targets. Hide flags control generic rows while `controls`, `dots`, and `thumbnails` retain their required affordance. Use several slides when validating responsive behavior; omit `slideWidth` for a viewport-filling track. |

## Overlays and transient surfaces

| Component | Use and essential contract |
| --- | --- |
| `Modal` | Open state, named close function, optional `header` and `footer`, required body content, Card-equivalent `variant` and `scheme`, and a generated Drawer-style close control unless hidden. |
| `AlertDialog` | Open confirmation surface with named confirm and cancel functions; `variant` styles the neutral Card-equivalent panel, while `scheme` styles the generated solid confirm Button and cancel remains outlined muted. |
| `Tooltip` | Accessible contextual label around one or more trigger view children. |
| `Toast` | Renders static or Signal-backed feedback with Card-equivalent `solid`, `soft`, `outlined`, and `ghost` variants, a design `scheme`, one of four corner positions, and the generated Drawer-style close control. It is distinct from the recommended lowercase `toast` statement that updates the global feedback presenter inside a view function. |
| `Dropdown` | Anchored surface with required `trigger`, optional `header` and `footer`, and direct `item` or `divider` entries. |
| `Command` | Searchable command surface with direct `item` entries or `group` collections and named item functions. |

Open overlays bind to view state and use named functions for close, confirm, cancel, or selection.
Place shell-wide overlays in the Scaffold `overlays` region so layouts keep one visual root.

## Context-only structural entries

These lowercase entries are valid only inside their owning component. They are not reusable imported
components and cannot be used as independent page roots.

| Entries | Owner |
| --- | --- |
| `appBar`, `top`, `start`, `center`, `end`, `bottom`, `main`, `bottomBar`, `overlays` | Scaffold, AppBar, Footer, or Chip as described above |
| `header`, `body`, `footer`, `trigger` | Sidebar, Drawer, Modal, Dropdown, or another owner that explicitly accepts the region |
| `item`, `divider`, `submenu`, `megamenu`, `group` | Navigation, selection, accordion, command, and menu owners |
| `column` | Table |
| `icon` | Avatar or another component that explicitly accepts an icon region |
| `mark` | RichText |
| `marker`, `waypoint` | Map |
| `slide` | Carousel |
| `step` | Stepper |

Do not infer a contextual child from a similarly named component. Use the exact owner-child shape and
let compiler diagnostics reject children, props, or bindings outside that context.
