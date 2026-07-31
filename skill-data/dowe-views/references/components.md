# Built-in view component catalog

Dowe lowers this catalog from one target-neutral component tree to web, desktop, Android, and iOS.
Use the semantic component that owns the behavior instead of rebuilding it from generic containers.
Compiler diagnostics remain the final authority for exact props, values, binding types, and target
support.

Static string props use double quotes. Reactive props use bare Signal, Store, constant, scoped item,
or function bindings. Event props reference a named view `fn`. Common visual props such as spacing,
size, visibility, semantic color, border, radius, and shadow are accepted only where the component
contract declares them.

## Layout and text

| Component | Use and essential contract |
| --- | --- |
| `Box` | Neutral wrapper for exceptional background, overlay, cover, local styling, or portable relative/absolute/fixed positioning. Prefer a semantic container when one exists. |
| `Section` | Ordered page band and page-level vertical rhythm. A page begins with one or more sibling Sections; `boxed:true` constrains and centers only its generated inner body at `96rem` web or `1536` native. |
| `Flex` | One-axis row or column using `direction`, `gap`, `align`, `justify`, and optional wrapping. |
| `Grid` | Explicit or responsive tracks using `columns`, optional `rows`, `gap`, and alignment. |
| `Card` | One related semantic unit such as a form, metric, article, or profile. Avoid nesting Card inside Card. |
| `Title` | One direct quoted visible-text child or one complete braced string binding. |
| `Text` | One direct quoted visible-text child or one complete braced string binding. |
| `Divider` | Horizontal or vertical separator; choose `orientation` instead of drawing a border-only Box. |

`Section boxed:true` keeps the outer band, background, cover, overlay, border, and anchor full width while limiting the generated content body to `96rem` on web and `1536` logical units on Android and iOS. It defaults to `false` and accepts a static boolean.

## Application shells and navigation

| Component | Use and essential contract |
| --- | --- |
| `AppBar` | Top application bar with optional full-width `top` and `bottom` regions around `start`, `center`, and `end`; `boxed:true` centers its inner content at `96rem` web or `1536` native while preserving the full-width surface. It stays visually flat across targets unless `border`, `bordered:true`, or `floating:true` requests separation. |
| `Footer` | Page or shell footer with optional full-width `top` and `bottom` regions around `start`, `center`, and `end`; it includes horizontal padding `4` from `xs` and `6` from `md`, overridable with `p`, `px`, `pl`, or `pr`. `boxed:true` centers its central row at `96rem` web or `1536` native. Put responsive `show` on children inside a region, not on the structural region block. |
| `BottomBar` | Bottom navigation containing one or more direct `tab` entries; each entry owns one Icon and navigation metadata; `boxed:true` centers the tab row at `96rem` web or `1536` native. |
| `NavMenu` | Horizontal navigation composed from direct `item`, `submenu`, or `megamenu` entries. Submenu and megamenu content opens in a Dowe-owned floating overlay on web, Android, and iOS, uses the structural background surface, preserves `scheme` for trigger and active states, dispatches fragment or route navigation before closing, and uses the same anchored overlay strategy as `Dropdown` on iOS. |
| `SideNav` | Detailed vertical navigation with optional `header`, direct `item`, `divider`, and `submenu` entries. |
| `RailNav` | Narrow icon navigation with direct `item` and `divider` entries; each item requires quoted `label` and Solar `icon`. |
| `Sidebar` | Shell side surface with optional `header`, required `body`, and optional `footer` regions. |
| `Scaffold` | The single normal layout root. It accepts optional `appBar`, `start`, `end`, `bottomBar`, and `overlays` regions plus required `main`; `boxed:true` centers only the `start`/`main`/`end` body at `96rem` web or `1536` native. |
| `Splash` | Direct layout or page boundary with required `bind` to a boolean Signal or View Store. Its children replace every normal root while the binding is true; it has no default spinner or style. |
| `Drawer` | Openable side surface with optional `header`, required `body`, and optional `footer`; direct view children also form body content. |
| `Tabs` | Related panels selected through one or more direct `tab` entries with unique quoted `id` and `label`. |
| `tab` | Context-only child of Tabs or BottomBar. A Tabs entry owns panel children; a BottomBar entry owns navigation metadata and one Icon. |
| `Stepper` | Ordered numbered workflow selected through direct `step` entries; use `scheme` and `horizontal` or `vertical` orientation. |
| `step` | Context-only child of Stepper with unique quoted `id`, quoted `label`, and panel children. |

`Scaffold boxed:true` centers and limits only the `start`, `main`, and `end` body while leaving the outer shell, bars, and overlays full width.

Section, Scaffold, AppBar, Footer, and BottomBar use the wide boxed content cap of `96rem` on web and `1536` logical units on Android and iOS. Their outer bands, shells, and bar surfaces remain full width.

## Controls and theme selection

| Component | Use and essential contract |
| --- | --- |
| `Button` | Text action or navigation control. Use one direct quoted or complete braced text child, reference a view function with `onClick`, and bind `loading` to a boolean Signal or View Store path when the action is pending. Loading reuses the bundled `svg-spinners:3-dots-move` Icon and blocks duplicate actions. |
| `IconButton` | Accessible icon-only action. Supply quoted `label` and Solar `icon`; use `onClick` or supported navigation props. |
| `ToggleTheme` | Control that switches between configured themes without duplicating theme state in page source. |
| `SelectTheme` | Theme selector for the configured named theme catalog. |
| `Fab` | Primary floating action with optional direct `fabAction` secondary actions. Place shell-level floating behavior in Scaffold overlays. |
| `fabAction` | Context-only secondary action inside Fab with an icon, label, and function or navigation target. |
| `Record` | Recording control driven by named start, pause, resume, cancel, and confirm functions where supported. |
| `ToggleGroup` | One-of-many or multi-choice control with direct `item` entries, a state value, and a named change function. |
| `Pagination` | Binds the current page with `bind`, accepts a static count or numeric Signal in `total`, and uses `pageSize` plus optional `onChange`; the portable subset supports at most 25 pages. |

## Forms

| Component | Use and essential contract |
| --- | --- |
| `Input` | Single-line value bound through `bind`; add a quoted label and the input type accepted by diagnostics. |
| `Select` | Bound choice control containing one or more direct `Option` entries. |
| `Option` | Context-only Select entry with quoted `value`, `label`, and optional description. |
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
| `ImageCropper` | Bound image-selection and crop result with portable aspect and file limits. |
| `PasswordField` | Bound password input with Dowe-owned reveal, strength, and validation behavior. |
| `PhoneField` | Bound digit-only local phone input with separate dial-code storage and the same Dowe-owned searchable country popover, flags, and ordering on web, Android, and iOS. |
| `PinField` | Bound fixed-length PIN or verification-code input with Input-sized cells, automatic focus movement, distributed paste, and text, password, or numeric modes. |
| `Textarea` | Bound multiline text with row and length limits. |
| `Checkbox` | Bound boolean choice with a quoted accessible label. |
| `Color` | Bound color value using the implemented picker formats and optional displayed representations. |
| `Date` | Input-like bound date with a Dowe-owned calendar dropdown, month navigation, selected/today states, and optional minimum and maximum values. |
| `DateRange` | Input-like bound start/end range with a Dowe-owned calendar dropdown, range highlighting, automatic ordering, and optional limits. |
| `RadioGroup` | Bound single choice composed from one or more direct `item` entries. |
| `Toggle` | Bound boolean control with a quoted accessible label. |

## Media, code, icons, and custom drawing

| Component | Use and essential contract |
| --- | --- |
| `Code` | Displays source without executing it. Set `language`, `content`, and optional template or copy behavior. |
| `Video` | Portable HTTPS video or HLS playback with optional poster, aspect, autoplay preference, and Dowe-owned controls. |
| `Iframe` | Embeds one quoted HTTPS URL or root-relative internal route. Quoted `src` and accessible `title` are required. During native `dowe dev`, an internal route uses the active Views origin instead of the API `BACKEND_URL`. |
| `Device` | Responsive preview frame that contains exactly one Iframe and selects a supported device profile. |
| `Canvas` | Custom drawing or pointer surface for visuals that semantic components cannot express; keep its commands and data target-neutral. |
| `Audio` | Portable audio playback for a supported static source with Dowe-owned playback behavior. |
| `Image` | Portable original media whose quoted `src` is a project asset path such as `/assets/images/hero.jpg` or an HTTPS URL, with `alt` text (empty marks it decorative), `aspect` (`horizontal`, `vertical`, `square`, `auto`), `objectFit`, `scheme`, and `rounded`. An unavailable source keeps the styled frame as a placeholder without crashing, so authoring the final path first and adding the file later is the canonical placeholder workflow. Never rebuild a photograph with `Svg` or `Canvas`, and never use the design reference or a crop from it to flatten UI into an image asset. |
| `Icon` | Bundled vector selected by quoted `name`: Solar names, `country-flags:<ISO code>`, animated `svg-spinners:<name>`, or brand-colored `svg-logos:<name>`. Solar supports six styles; namespaced catalogs use `linear`. |
| `Svg` | Portable vector using either quoted `viewBox` plus direct `Path` children, or runtime `data:<reference>` with no static paths. |
| `Path` | Context-only Svg path with quoted `d`, paint, and optional matrix transform. |

`Icon name` is always a static quoted value; it cannot bind an `each` item or Signal path, so a
collection with distinct icons uses explicit sibling declarations. Names must exist in the bundled
Solar catalog for the selected style and diagnostics reject unknown names: for example
`magnifier` is valid but `search` is not. Standalone icons accept `fill:<color token>`, `w`, `h`,
and optional `style`.

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
| `Table` | Tabular Signal data with one or more direct `column` definitions using field and label metadata. |

Charts consume portable Signal data rather than a target-specific chart library. Category charts
(`ArcChart`, `BarChart`, `PieChart`) read items with `label` and `value`. Point charts
(`AreaChart`, `LineChart`) read items with numeric `x` and `y`, or a `series` Signal whose items
contain `label` and `data:[{ x, y }]`. Optional item `color` fields must be Dowe color tokens.
Every chart accepts `variant`, `scheme`, `size` (`sm` to `xl`), `palette` (`default`, `rainbow`,
`ocean`, `sunset`, `forest`, `neon`), `legendPosition` (`top`, `right`, `bottom`, `left`, `none`),
`emptyLabel`, `loading`, and `hideLegend`. Diagnostics reject missing, incompatible, or invalid
negative data where the chart contract requires non-negative values.

Each `Candlestick` item provides `time` plus numeric `open`, `high`, `low`, and `close`. Optional
props include `stream` for an SSE feed upserted by `time`, `upColor` and `downColor` tokens, and
`maxPoints`. Validation rejects OHLC values where `high` or `low` contradicts the body.

`Table data:<signal>` requires at least one direct `column` with quoted `field` and `label`;
`field` is a relative row path such as `profile.email`, optional `align` accepts `start`,
`center`, or `end`, and optional `width` accepts static portable hints such as `160px`, `25%`, or
`1fr`. Table props include `size` (`sm`, `md`, `lg`), `striped`, `bordered`, `dividers`,
`emptyTitle`, and `emptyDescription`. Cells render strings, numbers, and booleans; objects,
arrays, and missing fields render empty. Sorting, pagination, selection, and custom cell renderers
are outside the Table contract.

## Display, feedback, and rich content

| Component | Use and essential contract |
| --- | --- |
| `Alert` | Inline status feedback using a supported type, quoted message, variant, and semantic scheme. |
| `Avatar` | Person or entity image, initials, or optional `icon` region with supported action and navigation behavior. |
| `AvatarGroup` | Static `item` children or bound `items`, with visible-count and overflow behavior. |
| `Badge` | Compact status or count surface containing one or more view children. |
| `Brand` | Logo or identity container with one or more arbitrary view children, optional quoted `href` navigation, optional accessible `label`, and Box-compatible `w` and `h`; it adds no Button chrome. |
| `Banner` | Full-width external surface with one or more arbitrary view children, required quoted HTTPS `href`, optional accessible `label`, and common background, cover, spacing, sizing, border, radius, shadow, and visibility props; web opens a protected new tab and native targets use the system browser. |
| `Chip` | Compact labeled token with optional `start` and `end` icon regions and supported close behavior. |
| `Skeleton` | Loading placeholder sized to the content surface it represents. |
| `ChatBox` | Bound message list with named send and pagination functions plus loading, sending, and streaming state. |
| `Empty` | Empty-state icon, title, description, and optional action or navigation target. |
| `Marquee` | Repeating overflow presentation for one or more view children. |
| `TypeWriter` | Sequential text presentation composed from one or more direct `item` entries. |
| `RichText` | Portable styled text composed from one or more direct `mark` runs. |
| `mark` | Context-only RichText run with quoted text and one supported style. |
| `Collapsible` | Expandable content with a quoted label and one or more view children. |
| `Countdown` | Time-based display with an optional named completion function; large values expand, while narrow containers compact `lg` and `xl` before bounded horizontal scrolling. |
| `Map` | Portable map with direct `marker` and optional route `waypoint` entries plus named location or route functions. |
| `marker` | Context-only Map marker with stable id, latitude, longitude, and optional named click function. |
| `waypoint` | Context-only Map route point with latitude and longitude. |
| `Accordion` | Expandable collection composed from one or more direct `item` entries. |
| `Carousel` | Slide collection composed from one or more direct `slide` entries and portable navigation behavior. |

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
