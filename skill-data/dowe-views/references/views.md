# Views reference

## Named declarations

Place route graphs in `views/routes`, layouts in `views/layouts`, pages in `views/pages`, reusable
components in `views/components`, shared View Stores in `views/store`, and frontend-only types in
`views/types`. Imports remain authoritative, but new frontend source should not use flat root
folders.

| Declaration | Binding role |
| --- | --- |
| `views viewRoutes` | Importable route graph connected from `main.dowe` |
| `layout AppLayout` | Importable shell with exactly one normal visual root and optional direct Splash |
| `page BlogsPage` | Importable routed screen with one or more normal visual roots and optional direct Splash |
| `component AppBrand` | Importable reusable static view tree |
| `store session` | Shared application View Store exported from an imported module |
| `signal blogs` | Reactive value owned by a page or layout instance |
| `const plans` | Immutable value owned by a page or layout |
| `fn loadBlogs` | Ordered view workflow owned by its page or layout |
| `init` | Unnamed ordered workflow that runs once when its page or layout mounts |
| `meta name:"..." content:"..."` | Direct static layout or page metadata for the web document head |

## Routes

```text
import AppLayout from "@/views/layouts/app-layout"
import BlogsPage from "@/views/pages/blogs-page"

views viewRoutes
  group path:"/" layout:AppLayout
    route path:"" page:BlogsPage
```

`group` uses `path`, `layout`, and optional `platform`. `route` uses `path`, `page`, and optional
`platform`. A group contains direct routes only; do not nest groups. Route graphs compose imported
bindings; they do not embed page or layout declarations.

## Layouts and pages

Every layout has one normal `Scaffold` root and inserts routed content with `children` inside `main`.

```text
layout AppLayout
  Scaffold
    appBar
      AppBar
        start
          Text
            "Dowe Journal"
    main
      children
```

Every page starts with `Section`. Add sibling `Section` declarations for distinct hero, content,
form, catalog, pricing, testimonial, or call-to-action bands.

```text
page BlogsPage
  Section
    Grid columns:1 gap:4
      Title
        "Journal"
      Text
        "Recent writing from the team."
  Section
    Grid columns:{ xs:1 md:2 } gap:4
      Card
        Text
          "First article"
```

### Web metadata

A layout or page may declare direct static `meta name:"..." content:"..."` entries. Layout values
are defaults; the page overrides matching names. Supported names are
`title`, `description`, `keywords`, `robots`, `canonical`, `og:title`, `og:description`,
`og:image`, `og:image:alt`, `og:type`, `og:url`, `og:site_name`, `twitter:card`, `twitter:title`,
`twitter:description`, `twitter:image`, `twitter:image:alt`, `twitter:site`, and
`twitter:creator`.

```text
layout SiteLayout
  meta name:"title" content:"Acme"
  meta name:"og:image" content:"https://acme.dev/og.png"
  Scaffold
    main
      children
```

`meta` is not visual and accepts no children or dynamic values. It affects web SSR and browser
routing only; desktop, Android, and iOS accept the syntax without emitting native metadata.

### Init and Splash

A layout or page accepts one direct `init` and one direct `Splash`. `init` has no name, params, or
return prop. It uses the same ordered statements as `fn` and runs once per mounted scope. Outer
layouts initialize before the page, and a preserved web layout does not initialize again during
page-only navigation. Startup data loading belongs in `init`; there is no other startup request
mechanism.

`Splash bind:<path>` requires a boolean Signal or View Store. `true` shows only the Splash children;
`false` shows the normal layout root or ordered page roots. Always set the binding explicitly in
both success and error paths. Splash does not provide default presentation and cannot contain the
layout `children` boundary.

```text
page BlogsPage
  signal loading value:true
  signal blogs value:[]
  init
    request result method:"GET" route:"/api/blogs"
    if result.ok
      set blogs value:result.data
      set loading value:false
    else
      set loading value:false
  Section
    Text
      "Recent writing"
  Splash bind:loading
    Section
      Skeleton w:"full" h:8
```

## Container decisions

| Component | Use it for |
| --- | --- |
| `Scaffold` | One complete layout shell with `appBar`, `start`, `main`, `end`, `bottomBar`, or `overlays` regions |
| `Splash` | Direct layout or page replacement boundary bound to a boolean Signal or View Store |
| `Section` | Major ordered page bands and page-level vertical rhythm |
| `Grid` | Exact tracks, responsive columns, repeated cards, dashboards, catalogs, and structural stacks |
| `Flex` | One-axis rows or columns, alignment, navigation bars, toolbars, and compact groups |
| `Card` | One related semantic unit such as a form, metric, article, or profile |
| `Box` | Advanced relative/absolute or fixed layer geometry that Section, Grid, Flex, Card, media, controls, and content cannot express in normal flow |
| `RailNav` | Narrow vertical navigation with required Solar icons, accessible tooltips, and optional labels below icons |
| `Tabs` | Related content panels selected from a compact intrinsic-width control list |
| `Svg` | Portable vector paths; use direct `Path` children for static geometry or `data:<reference>` for one normalized runtime record |

Avoid Card inside Card. Use `Grid` or `Flex` inside a Card. Begin with no Box nodes; never use Box
for ordinary styling, spacing, sizing, Grid gutters, or control wrappers. When choosing between
Section, Grid, Flex, Card, and Box, or
when decomposing a reference design into a layout and pages, follow the ordered decision tree and
anti-pattern table in `references/composition.md`.

`RailNav` accepts direct `item` and `divider` entries. Every item requires a quoted static `label`
and `icon`. Icon-only mode is the default and reveals the label on web and desktop through hover or
focus tooltip behavior; set `showLabels:true` to place labels below icons on every target. Use
`SideNav` for headers, descriptions, status text, submenus, or custom SVG icons.

For shell navigation, keep the component orientations explicit. `NavMenu` is horizontal and belongs
directly in AppBar `center` or `end`; it is not the navigation child of `Drawer`, `Sidebar`, or
`Scaffold start/end`. `SideNav` is vertical and is the navigation child for `Sidebar body` and
`Drawer body`. On responsive shells, hide the AppBar `NavMenu`, show an `IconButton` that opens the
Drawer, and reuse one static `SideNav` component in the desktop Sidebar and mobile Drawer.

AppBar regions are already horizontal flex containers. Keep `Brand`/Logo, `NavMenu`, `Button`, and
`IconButton` as direct region children; never add a wrapper `Flex` just for `align`, `gap`, or
sibling placement. For a mobile Drawer trigger, put `IconButton` before Brand in `start` or directly
in `end`. Treat an imported Logo component as one semantic Brand child.

`Tabs` accepts direct `tab` children with unique quoted `id` and `label` props plus panel content.
Its control list wraps its labels instead of filling the panel width. Horizontal lists scroll only
when their intrinsic width exceeds the available space; use `position:"start"` or
`position:"end"` for a compact vertical list. The `line` variant marks the active control with a
bottom line for horizontal lists or a leading or trailing line for vertical lists; it does not
outline the complete control.

Common structural props include `Grid columns`, `rows`, and `gap`; `Flex direction`, `gap`, `align`,
`justify`, and `wrap`; and responsive values such as `columns:{ xs:1 md:2 }`. Read
`references/styles.md` for the complete color, variant, spacing, sizing, typography, `show`,
`animation`, cover, anchor, and navigation prop contract. Static visible text for `Text`, `Title`,
and `Button` is one direct quoted child. Dynamic visible text uses one complete braced binding path.

## Repeated views

`each in:<collection> as:<item> key:<stable-path>` repeats view nodes. All three props are
required bare references; `in` names the array, `as` introduces the scoped item, and `key`
identifies the item across updates.

Declare immutable data with `const` directly in a page or layout, or directly inside a visual block
above the `each` that uses it. Dowe hoists the nested declaration into the owning page or layout
scope without adding a visual wrapper, preserving the container's direct children. Do not declare
constants inside reusable `component` exports.

This is mandatory for two or more same-shape sibling units, including rows composed from `Flex`,
`Icon`, and text. Put the collection declaration before the visual tree and make the `each` wrap the
complete repeated unit. Copying the unit and changing only its content is invalid authoring even when
the rendered screenshot is correct.

```text
each in:blogs as:blog key:blog.id
  Card
    Title
      "{blog.title}"
    Text
      "{blog.content}"
```

Inside the loop, item paths stay scoped for visible text and reactive props such as
`scheme:blog.scheme`. `Select` also accepts a structural `each` over an immutable `const` catalog
producing `Option value:option.value label:option.label` entries.

Static-only props do not become dynamic because they are inside `each`. `Icon.name` remains a quoted,
compiler-validated name; do not use `name:<item.icon>`. Use the supported runtime `Svg data:<reference>`
contract for a genuinely runtime vector catalog, or keep the component's static contract intact.

`"blog.title"` is literal text. A braced binding must resolve to a string. Mixed text such as
`"By {blog.author}"` is not interpolated and remains literal. Braces apply to direct visible-text
children only; props continue to use bare references such as `bind:form.title`, `show:ready`, and
`onClick:save`. Static `Text` and `Title` copy remains verbatim across targets; email- and URL-shaped
strings do not implicitly become links on iOS.

## State

Use `const` for immutable data, `signal` for page or layout state, and an imported `store` module for
state shared across routes. Signals and Stores accept an optional `type:<Name>` or `type:<Name>[]`
referencing an imported `type` declaration; typed state validates initial values, binding paths,
each-item fields, and request bodies.

```text
import Blog from "@/views/types/blog"

page BlogsPage
  signal blogs type:Blog[] value:[]
``` Store modules may live anywhere in the project. Add `persistent:true`
only when target-local storage should survive an application restart.

For a short Store, the inline form is valid. When its props or initial value make a long line, use a
property suite: end the Store declaration with `:` and put one prop on each following line.

```text
store session:
  type:SessionState
  persistent:true
  value:{ authorization:"" token:"" user:{ id:"" name:"" email:"" } }
```

Do not mix inline props with the `store session:` header. Props appear before any structural children.

Import Stores with `@/` or a relative path from their actual project location. Do not put secrets in
a View Store; target-local persistence is not a credential vault. Store declarations remain
Views-only regardless of their folder. Persistent hydration falls back to the declared initial value
when stored data is malformed or structurally incompatible with that initial shape.

## View-to-server contracts

An internal `request` crosses the Views and Server authoring boundary. The View remains the owner of
interaction and presentation state, but the task must load the companion `dowe-server` skill when
the project-owned route needs to be added or changed. Do this even when the prompt begins with a
page, screenshot, table, form, search field, pagination control, or other UI reference.

Record one row per affected request before editing:

| Field | Required decision |
| --- | --- |
| Caller | Page or layout plus its named `fn` or `init` |
| Request | HTTP method and resolved `route` or `path` |
| Client input | Body fields, headers, route params, and values that remain untrusted |
| Client output | Minimal serializable response shape consumed by the View |
| View states | Loading, success, empty, error, unauthorized, and retry behavior when applicable |
| Server route | Endpoint entry and matching method |
| Server layers | Handler, middleware, service, repository, provider, and config owners |
| Data impact | Entity, migration, Database, Cache, Vector, Queue, or file changes required |
| Security | Authentication, authorization, tenant/owner scope, validation, and safe error behavior |

Inspect the route implementation and its full imports instead of assuming that an `/api` string
already has a compatible endpoint. A missing route, wrong method, incompatible body or response,
missing authorization, or missing persistence behavior is part of the same fullstack task. Use the
Server skill to modify only the layers required by the requested behavior, then re-check the matrix
against the actual source.

Do not solve a missing Server capability by moving Database, Cache, Vector, Queue, outbound
server-only HTTP, secrets, filesystem, crypto, or spawn into a View. Do not copy an authorization or
business rule into a Signal or View Store. The Server must treat every client value as untrusted and
return only the fields the View needs. Views may hide an action, but the Server remains the
authorization authority.

For a request with an explicitly external, independently owned `base`, verify the external client
contract without inventing a project Server route. The cross-skill gate applies to project-owned
backend behavior and to any local adapter required by the requested feature.

## View function utilities

View functions contain ordered, target-neutral statements.

| Utility | Binding | Props |
| --- | --- | --- |
| `request result` | Function-local result with `ok` and `data` | `method`, exactly one of `route` or `path`; optional `base`, `body`, `headers` |
| `validate signal` | Marks all registered fields touched and stops the sequence when invalid | Signal root with validated controls |
| `set target` | none | `value`, or `source:<standard-library function>` with its props |
| `reset target` | none | Restores a Signal or View Store to its initial value |
| `toast` | none | `value:{ type title message visible duration? }`; optional `duration`, Card-equivalent `variant` (`solid`, `soft`, `outlined`, `ghost`), design `scheme`, and corner `position`; `variant` also resolves from `design Toast` |
| `redirect` | none | Required static absolute `path` to a declared internal route; replaces history and terminates the function |

When controls with `validate` children bind to fields of the same Signal, Dowe derives read-only
`<signal>.isValid`, `<signal>.isInvalid`, `<signal>.errors.<field>`, and `<signal>.touched.<field>`
paths. Use those paths for reactive `Button disabled:` state; they never become part of the Signal
value or a request body. Put `validate <signal>` before `request` in a submit function to touch all
registered fields and stop the sequence while any rule fails.

```text
fn createBlog
  request result method:"POST" route:"/api/blogs" body:form
  if result.ok
    set blogs value:result.data
    reset form
    toast value:{ type:"success" title:"Published" message:"Blog created." visible:true }
  else
    redirect path:"/login"
```

Use `Button onClick:createBlog` to dispatch the named function. `fn` accepts optional
`params:{ name:Type }` naming the reactive Signals or imported Stores it depends on, and an
optional `return` contract of `"boolean"`, `"number"`, `"string"`, or a declared type used for
validation. Requests use `GET`, `POST`, `PUT`, `PATCH`, or `DELETE` and can only call client-safe
routes or client-visible environment bases; a `/api` route without `base` uses `env.BACKEND_URL`
implicitly. Views cannot access Database, KV, server HTTP, crypto, spawn, filesystem, or
server-only environment values.

Use `redirect path:"/login"` in either `fn` or `init` for route guards and completed workflows.
The path must exist in the effective route graph. Redirect is terminal, uses replace navigation on
web, desktop, Android, and iOS, and does not accept external or dynamic destinations.

`set target value:<value>` accepts a reactive reference, its boolean negation such as
`value:!openMenu`, a boolean literal, or a quoted static string, and writes Signals, nested Signal
paths, or imported Store paths.

### Inline click updates

For one small local update, `onClick` accepts an inline object instead of a named function. It
requires exactly one `set` target and one operation: `value`, `add`, or `append`.

```text
IconButton label:"menu" variant:"ghost" icon:"menu-dots" onClick:{ set:openDrawer value:!openDrawer }
Button onClick:{ set:counter add:1 }
  "Increment"
Button onClick:{ set:name append:"!" }
  "Append punctuation"
```

`add` requires a numeric target and numeric literal; `append` requires a string target and quoted
string. Targets are mutable Signals or imported Store paths; inline updates never touch server
state.

## Localized content

`Text`, `Title`, `Button`, navigation entry labels, and `Tabs` tab labels accept an `i18n` prop
with a quoted dot-separated translation key. The direct text child or required `label` remains the
static fallback. Navigation descriptions use `descriptionI18n` and `SideNav` status copy uses
`statusI18n`; each secondary key requires its corresponding fallback prop.

```text
Title i18n:"home.hero.title"
  "AI GENERATES CODE. DOWE BUILDS SYSTEMS."

NavMenu
  item label:"Views" i18n:"navigation.views" href:"/docs/views"
```

Catalogs live in root `i18n/<locale>.dowe`; read the dowe-core `main.md` reference for the
catalog shape. Every referenced key must exist in every locale catalog, and `i18n` outside a
supported surface fails before target generation.

Portable standard-library calls use `set target source:namespace.function` inside `fn`. Convert SVG
XML text with `set output source:parse.svg value:input fallback:""`. Add
`colors:"original" format:"source"` to preserve hexadecimal fills in copyable Dowe source. Use a
second assignment with `colors:"original" format:"data"` when a normalized JSON result must feed
`Svg data:<reference>` for a safe preview. `parse.svg` does not mount or execute the XML.

The portable view catalog also includes `str`, `math`, `parse`, `url`, `csv`, `sort`, `list`,
`json`, and `date`. Use their namespaced functions for text and numeric values, bounded CSV and
JSON transforms, stable collection operations, URL query construction, and ISO date arithmetic.
The catalog is lowered once and implemented by web, Android, and iOS generators; `id.ulid` and
server capabilities remain unavailable in views. Invalid parsing returns the declared fallback or
`null`, and collection operations return new values rather than mutating their inputs. See the
portable standard-library documentation for the complete function and argument catalog.

`Svg` is for portable vector marks such as logos and simple geometry. Original photographs,
illustrations, textures, and authentic screenshots explicitly supplied or requested by the user are
`Image` assets and must never be redrawn as `Svg` paths or `Canvas` commands. A design reference or
any crop derived from it is not an application asset; rebuild the UI it depicts with Dowe
components and use named missing paths for unavailable original media.

### Public image paths

Project media belongs in the root `assets/` tree. It is not imported through `@/` and it is not
addressed with a machine filesystem path. Dowe maps the path relative to `assets/` to the same
root-relative URL under `/assets/` in the development web server and web packages:

| Source file | View URL |
| --- | --- |
| `assets/img/hero.webp` | `/assets/img/hero.webp` |
| `assets/social/home.png` | `/assets/social/home.png` |

Use the view URL in static `Image src` and in `cover` props. A dynamic `Image src` may read a
string field from a constant, Signal, or current `each` item. `assets/img/hero.webp`,
`/img/hero.webp`, and `/Users/name/project/assets/img/hero.webp` are not equivalents and must not be
substituted for `/assets/img/hero.webp`. The built-in runtime only resolves project media under
`/assets/**`; a 404 at `/img/**` means the public prefix was dropped. Check the file's spelling and
case on disk, then request the exact URL from the running `dowe dev` server before debugging layout,
`show`, `objectFit`, or responsive props.

The same source URL is kept in target-neutral Dowe source. Web and desktop serve `/assets/**`, while
native target packaging copies the local project file into the platform bundle and resolves it from
the packaged asset store.

Static `Svg` requires a quoted `viewBox` and one or more direct `Path` children. Runtime `Svg`
instead uses `data:<reference>` and cannot declare `viewBox` or `Path`. The runtime reference resolves
to one normalized Dowe vector record or JSON string; it does not accept SVG markup. `Path` accepts
quoted `d`, a `fill` using `currentColor`, `none`, a design color token, or a hexadecimal color, and
optional `transform:"matrix(a b c d e f)"`. Keep `Path` documented
with `Svg` rather than treating it as a standalone component.

For static `Svg`, authoring only `w` or only `h` leaves the other axis automatic and preserves the
`viewBox` ratio. If neither is authored, the default is `6` by `6`. `Brand` follows the same
single-axis rule using the intrinsic measure of its children.

Canvas is a built-in View component, not a separate application surface. Use it only when semantic
components cannot express a drawing, chart, diagram, game-like scene, or custom pointer
interaction, and read `references/canvas.md` for its complete command, input, and animation
contract.
