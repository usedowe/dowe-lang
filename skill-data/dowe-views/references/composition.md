# UI composition reference

This reference decides structure: what belongs to the layout, what belongs to the page, and which
container owns each region. Combine it with `references/reference-ui.md` when translating a
reference design or screenshot into source.

## Contents

- Layout/page ownership and reusable static components
- Repeated collection ownership and container decisions
- Minimal reference patterns and strict prop admission
- Auth and form composition without generic wrappers
- Visual direction, section richness, and layered scenes
- Hero, landing-page, equal-height, and anti-pattern guidance
- Composition validation checklist

## Layout versus page ownership

| Surface | Root contract | Owns | Never contains |
| --- | --- | --- | --- |
| `layout` | Exactly one `Scaffold`, plus one optional direct `Splash` | AppBar, SideNav or Sidebar, Footer, BottomBar, shell overlays, the `children` boundary, session or shell state | `Section` page bands, page content, routed data loading that belongs to one page |
| `page` | One or more sibling `Section` roots, plus one optional direct `Splash` | Page bands, page state, page functions, page data loading | `Scaffold`, `AppBar`, `Footer`, `BottomBar`, `children` |
| `component` | One reusable static view tree | Identity marks, static navigation trees, social-link groups, and other caller-independent fragments used in multiple places | Props, slots, caller bindings, Signals, Stores, functions, requests, or `each` over caller data |

Anything visible on every route of a group is shell and belongs in the layout Scaffold regions
(`appBar`, `start`, `main`, `end`, `bottomBar`, `overlays`). Anything that changes per route is
page content and lives in Sections. A page never rebuilds shell chrome, and a layout never renders
band content around `children`.

Create or reuse this shell even when the route graph has one page. A marketing site with a top bar
and footer uses a layout-backed group; placing either bar in the home page hides ownership and
encourages duplication when another route is added.

```text
import SiteLayout from "@/views/layouts/site-layout"
import HomePage from "@/views/pages/home"

views siteRoutes
  group path:"/" layout:SiteLayout
    route path:"" page:HomePage
```

```text
layout SiteLayout
  Scaffold
    appBar
      AppBar boxed:true
        start
          Brand href:"/" label:"Home"
            Text weight:"black"
              "SOLTECH"
        end
          NavMenu variant:"ghost" scheme:"surface"
            item label:"About" href:"/#about"
            item label:"Services" href:"/#services"
    main
      children
      Footer boxed:true variant:"solid" scheme:"surface"
        start
          Text weight:"bold"
            "SOLTECH"
        end
          Text size:"sm"
            "Strategy and consulting"
```

```text
layout AppLayout
  Scaffold boxed:true
    appBar
      AppBar boxed:true
        start
          Brand href:"/"
            Text weight:"bold"
              "DOWE JOURNAL"
    main
      children
  Splash bind:sessionLoading
    Section
      Flex direction:"column" align:"center" justify:"center" gap:3 h:"full"
        Icon name:"svg-spinners:ring-resize" w:10 h:10
        Text size:"sm"
          "Validating your session"
```

Use a layout `Splash` for whole-application gates such as session validation, and a page `Splash`
for that page's own loading state. Both bind a boolean set explicitly in every `init` branch.

## AppBar composition

The AppBar is a semantic shell bar, not a generic flex or card wrapper. Supported AppBar patterns
keep the bar directly under `Scaffold appBar` and use its regions as the composition map:

| Region | Owns | Responsive guidance |
| --- | --- | --- |
| `top` / `bottom` | Full-width announcement or secondary band | Use only when the band belongs inside the bar surface. |
| `start` | Brand and leading identity | Put `IconButton` before `Brand`/logo when the mobile Drawer trigger belongs to the leading group; keep both as direct siblings. Use a nested `Flex` only for an independently oriented or wrapped group. |
| `center` | Flexible primary navigation or title | Put the horizontal `NavMenu` or the title directly in the region; use its own `show` prop. |
| `end` | Compact/secondary horizontal navigation, primary action, utilities, and mobile menu trigger | Put `NavMenu`, `Button`, or `IconButton` directly in the region. Prefer direct siblings; use a nested `Flex` only for a distinct axis, wrapping, or grouped behavior. |

AppBar regions already provide the horizontal flex row, vertical centering, and standard spacing
for direct children. Do not write `start > Flex` merely to place `Logo` beside `IconButton`; `Logo`
is already one semantic `Brand` child. For a mobile Drawer trigger, use either:

- `start > IconButton` followed by `Brand`/`Logo`; or
- `end > IconButton`.

For a floating or pill AppBar, express the surface with `floating:true`, `boxed:true`,
`bordered:true` or an intentional border, radius, blur, and shadow on `AppBar`. Do not reconstruct
that surface with nested Boxes in `overlays`. `overlays` is for the mobile `Drawer`, announcements
that intentionally sit outside the bar, or other shell-level surfaces. A Box in `start`, `center`,
or `end` is justified only by an actual positioned/decorative layer; it is not the default way to
apply `show`, center content, or make a slot fill width.

Keep the AppBar tree untransformed. Do not put `translateX` or `translateY` on `AppBar`, `Brand`,
`NavMenu`, its action Buttons, `Drawer`, or wrappers around those nodes to nudge the bar toward
reference coordinates. Use `boxed:true`, AppBar regions, intrinsic slot sizing, `gap`, padding, and
responsive `show` to reproduce the shell rail. `NavMenu` owns anchored submenu and megamenu
surfaces, so its semantic root must not be visually displaced from the geometry used by those
surfaces.

When the same destinations are needed by desktop and mobile, keep the orientations separate:
render a direct built-in `NavMenu show:{ xs:false md:true }` in `AppBar center` (or `end` for a
compact menu), and mount a prop-free reusable `SideNav` in the `Sidebar body` and `Drawer body`.
This keeps responsive behavior attached to the semantic navigation owner instead of placing a
horizontal menu inside a vertical surface or producing a block wrapper that changes the AppBar's
intrinsic row geometry.

## Reusable components

`component <Name>` in `views/components` declares an importable reusable static view tree. A
component owns no signals, functions, stores, or bindings to caller state; it is pure structure
and visual props, and theme defaults apply inside it normally.

Extract a component when the same static fragment appears in two or more places — a brand or logo
(`Brand` plus `Svg`), a vertical `SideNav` tree, a social link block, a footer column — or when a
deep static fragment makes a layout or page hard to read. The canonical responsive navigation
pattern keeps the horizontal AppBar menu and the vertical side-surface tree explicit, while
reusing the same `SideNav` component in both shell surfaces:

```text
import ViewsNavigation from "@/views/components/views-navigation"
import Logo from "@/views/components/logo"

layout DocsLayout
  signal openDrawer value:false
  Scaffold boxed:true
    appBar
      AppBar boxed:true
        start
          IconButton show:{ xs:true md:false } label:"menu" variant:"ghost" icon:"menu-dots" onClick:{ set:openDrawer value:!openDrawer }
          Logo
        center
          NavMenu show:{ xs:false md:true }
            item label:"Overview" href:"/"
            item label:"Reports" href:"/reports"
            item label:"Settings" href:"/settings"
    start
      Sidebar show:{ xs:false md:true } variant:"ghost" scheme:"muted" w:72
        body
          ViewsNavigation
    main
      children
    overlays
      Drawer bind:openDrawer
        body
          ViewsNavigation
```

The `openDrawer` signal and its inline toggle stay in the layout; the `SideNav` component stays stateless.
If a fragment needs bindings to page state, an `each`, or event functions, keep it in the owning
page or layout instead of extracting it. Do not rebuild built-in components as custom components,
and do not extract a fragment used in only one place.

Static reuse means that the complete tree is identical without caller data. A Card template inside
`each` is already authored once for an arbitrary number of records; it is not a reason to invent a
data prop on a reusable component. If the same dynamic pattern appears on multiple pages, keep the
template in each owner until Dowe supports dynamic component inputs, and share only genuine
cross-route state through a Store.

## Repeated collection ownership

Two or more visible same-shape units are a collection even when the reference contains only two or
three items. This rule covers any repeated semantic tree—not only Cards—including a feature row made
of `Flex`, an `Icon`, and a title/description `Grid`. Choose the data owner before writing the Grid:

| Collection behavior | Declaration |
| --- | --- |
| Fixed copy and records visible in the reference | Page or layout `const` with one object per visible unit |
| Data loaded, filtered, paged, appended, or replaced by that page's request or workflow | Typed page or layout `signal` initialized to a valid value, commonly `[]`, then updated with `set` |
| Reactive state genuinely consumed by multiple routes | Imported View Store; add `persistent:true` only when it must survive restart |

Render the collection with one `each in:<collection> as:<item> key:<item.id>` as the direct repeated
unit inside the owning `Grid`. Give every record an explicit stable string `id`, preserve the
reference item count and copy in a `const`, and never author one sibling Card, Flex row, icon/text
group, or list unit per record. The loop boundary must contain the complete repeated subtree; do not
place a small `each` around only the title while copying the surrounding row.

Use this review gate before polishing the layout:

1. Name the collection and its page/layout owner.
2. Declare one record per visible unit before the visual tree.
3. Confirm that one `each` owns every repeated child, including icons, copy, actions, and state.
4. Confirm that all varying supported values resolve from the current item and that the key is stable.
5. Reject any source that still contains copied sibling units with the same shape.

Static-only props remain static inside a loop. `Icon.name` requires a quoted compiler-validated name;
do not write `name:<item.icon>`. Use the supported runtime `Svg data:<reference>` contract only when
the icon source is genuinely runtime data. A limitation in a static component contract is not a reason
to duplicate the complete repeated unit or invent a new binding form.

```text
page ServicesPage
  const services value:[
    { id:"strategy" title:"Strategy" description:"Plan the next durable move." },
    { id:"delivery" title:"Delivery" description:"Ship one coherent system." },
  ]
  Section boxed:true
    Grid columns:{ xs:1 md:2 }
      each in:services as:service key:service.id
        Flex direction:"column"
          Title
            "{service.title}"
          Text
            "{service.description}"
```

For backend data, replace the `const` with a typed `signal`, load it from `init` or a named `fn`,
and `set` the collection from the successful request result. The Grid and `each` template remain
the same; transport changes the data owner, not the visual structure.

## Container decision tree

Start with zero `Box` nodes and decide top-down for every region, in this order:

1. Is it a major horizontal band of the page (hero, features, catalog, form area, pricing,
   testimonials, call to action)? Use a sibling `Section`.
2. Does the reference show the group as an independent surface through a contained fill, border,
   radius, elevation, or inset treatment? Use `Card` and let `theme.dowe` style it. A semantic group
   that remains visually flat is not a Card; continue to Grid or Flex.
3. Do sibling children align to shared tracks — repeated same-shape units, a dashboard, a catalog,
   responsive columns, or a stack with one uniform rhythm? Use `Grid` with the minimum structural
   props, commonly only responsive `columns:{ xs:1 md:3 }`.
4. Do children flow on one axis with individual intrinsic sizes — an icon beside text, a label
   with a trailing action, a toolbar, a chip row, centered content? Use `Flex`; add `direction`,
   `align`, `justify`, `gap`, or `wrap` only for the responsibility the default row does not satisfy.
5. Does the region require children to leave normal flow? Use `Box position:"relative"` as the
   layer plane with direct `Box position:"absolute"` children, or `Box position:"fixed"` for a
   route-viewport layer. Keep the actual content inside semantic Dowe components.

`Grid` and `Flex` arrange; `Card` groups and styles; `Box` owns exceptional layer geometry. A style
prop alone never justifies a Box. Put `p`, `bg`, `rounded`, `border`, `w`, `h`, `maxW`, `show`, and
`animation` on the Section, Grid, Flex, Card, control, media, or content node that already owns the
region. Do not wrap `Input`, `Password`, `Phone`, `Pin`, `Button`, `Image`, or `Svg` only to restyle
or resize it. Do not use empty Boxes as Grid gutters or offset columns; center and constrain the
real Grid, Flex, Card, or control with `align`, `justify`, `w`, and `maxW` instead.

Same-kind layout nesting requires a distinct subgroup; it is not an automatic error and it is not
an excuse for wrapper noise. A column Flex containing a row Flex for two actions is valid because
the inner Flex owns the action group and a different axis. A feature row Flex containing a column
Flex for its title and description is also valid. A column Flex containing another column Flex only
to change spacing, padding, sizing, or visibility must be flattened. Apply the same test to nested
Grids: keep both only when each owns independently verifiable tracks. Do not alternate Grid and Flex
to disguise a wrapper whose only child can own the same props directly.

Do not use `translateX` or `translateY` as a second layout system. Screenshot measurements describe
the target relationships; they do not authorize one transform per node. Express normal geometry
through the decision tree above. Try `Flex` first for rows, columns, centering, distribution, and
intrinsic-size groups through `direction`, `justify`, `align`, `gap`, and `wrap`. Try `Grid` for
shared tracks, repeated units, responsive columns, and structural stacks through `columns`, `rows`,
`justify`, `align`, and `gap`. A translation is justified only inside an advanced relative Box scene
when a decorative or floating layer intentionally overlaps another layer and Flex, Grid, normal
flow, plus absolute offsets cannot express the final effect. Record that responsibility in the
composition blueprint. If the same result can be produced with those structural props, padding,
width, maximum width, responsive direction, AppBar regions, or Box offsets, remove the translation.

`Grid columns:1` and `Flex direction:"column"` are both valid vertical stacks: prefer Grid when the
stack owns structural tracks and Flex when it owns one-axis flow or alignment. Both default to zero
gap. Render the minimal tree before adding a nonzero gap to the one owner that needs it. When the
same content changes between breakpoints, keep one tree and use only the responsive props required
by that change. Never duplicate the complete compact and wide forms just to tune offsets.

A Box should normally declare `position:"relative"`, `position:"absolute"`, or
`position:"fixed"`. A rare non-positioned Box is acceptable only when a neutral media or drawing
stage has no semantic owner; record that reason in the composition blueprint. `cover`, `overlay`,
padding, background, border, or sizing by themselves are not sufficient because ordinary bands and
surfaces already belong to Section, Card, Grid, Flex, media, or controls.

## Minimal reference patterns

Use these source-shaped composition maps as the starting point for common marketing references.
They intentionally omit collection declarations, content props, asset metadata, and interaction
bindings so the structural decision remains visible; the final source must still use one `const` or
typed Signal plus one complete `each` template for every repeated unit.

| Reference family | Minimal composition map |
| --- | --- |
| Hero with lead input | `Section boxed:true > Grid columns:{ xs:1 lg:2 } > [Flex direction:"column" > Chip + Title + Text + Input + Text, Image]` |
| Hero with cover and actions | `Section boxed:true cover:"/assets/images/hero-cover.webp" > Grid columns:{ xs:1 lg:2 } > [Flex direction:"column" > Title + Text + (Flex > Button + Button scheme:"secondary" variant:"outlined") + Text, Image]` |
| Media features | `Section boxed:true > Title align:"center" + Text align:"center" + Grid columns:{ xs:1 lg:3 } > each(Flex direction:"column" > Image + Title + Text)` |
| Icon features | `Section boxed:true > Text align:"center" + Grid columns:{ xs:1 lg:2 } > each(Flex > Icon + (Flex direction:"column" > Title + Text))` |
| FAQ split | `Section boxed:true > Grid columns:{ xs:1 lg:2 } > [Flex direction:"column" > Chip + Title + Text + Button, Accordion]` |
| Pricing | `Section boxed:true > Title + (Flex > Toggle + Text) + Grid columns:{ xs:1 lg:3 } > each(Card > Title + (Flex > Title + Text) + Text + Divider + each(Flex > Icon + Text) + Button)` |

These maps are a prop ceiling for the first pass, not a checklist to expand. Do not add padding,
gap, width, height, maximum width, radius, border, shadow, color, weight, size, alignment, or motion
merely because the screenshot makes that value measurable. First render the minimal semantic tree
with component and theme defaults. Then admit one local prop only when it is required by content,
behavior, accessibility, essential responsive structure, an explicit non-default choice, or a
specific visual mismatch proven by that render.

`Card` is evidence-driven. Pricing offers in the last pattern are Cards because the reference shows
separate contained surfaces. The flat feature items and FAQ copy column are Flex groups because the
reference does not show independent Card surfaces. Never turn every repeated unit into a Card just
because it contains a title, text, icon, or image.

## Auth and form composition

Auth screens use the same container rules as other views. Keep reusable background geometry in the
layout and the form content in the page. A positioned `Box` is appropriate in the layout only when
it creates a real background layer plane; the page should normally need none.

```text
layout AuthLayout
  Scaffold
    main
      Box position:"relative" minH:"vh-0" bg:"background"
        Box position:"absolute" top:0 left:0 w:"full" h:"full"
          Svg viewBox:"0 0 1440 1024" w:"full" h:"full"
            Path d:"M0 0H1440V1024H0Z" fill:"#131031"
        children
```

Use one responsive page tree. Let the form Grid own width, maximum width, and field rhythm; let each
control and Button own its surface and size. Add a Card only when the reference visibly groups the
form on a raised, filled, or outlined panel. A visually flat form remains Grid/Flex content.
A page Section never receives a padding override, including `p:0` or responsive `p*` values.
For full-viewport or rail decisions, keep the Section default and place required spacing on the
inner Flex/Grid that owns the form or media region.

```text
Section minH:"vh-0"
  Flex direction:"column" align:"center" justify:"center" gap:{ xs:8 md:12 } minH:"vh-0"
    Image src:"/assets/images/brand.svg" alt:"Brand" w:{ xs:72 md:96 }
    Grid columns:1 gap:6 w:"full" maxW:96 px:{ xs:4 md:0 }
      Input bind:email label:"Email" placeholder:"name@example.com"
      Pin bind:pin label:"PIN" length:4
      Button w:"full" onClick:submitLogin
        "Log in"
```

### Split-panel auth layout

Let the outer Grid own the media and form halves. Let the form-side Flex own centering, and make one
bounded form Grid its direct child. Do not center a form with empty Grid columns, empty Boxes, or a
translation: those techniques encode one viewport guess and shrink the usable form width when the
outer split is already only half of the viewport.

```text
Section minH:"vh-0"
  Grid columns:{ xs:1 md:2 } gap:0 minH:"vh-0"
    Box cover:"/assets/images/auth-login.webp" minH:{ xs:40 md:"vh-0" }
    Flex:
      direction:"column"
      align:"center"
      justify:"center"
      p:{ xs:6 md:12 }
      minH:{ xs:"vh-40" md:"vh-0" }
      Grid columns:1 gap:5 w:"full" maxW:96
        Flex direction:"column" gap:2
          Title size:"3xl"
            "Log in"
          Text size:"sm"
            "Use your account to continue securely."
        Flex direction:"column" gap:4
          Input bind:email label:"Email address" placeholder:"you@example.com" w:"full"
          Password bind:password label:"Password" placeholder:"Enter your password" w:"full"
          Button w:"full" onClick:submitLogin
            "Log in"
        Flex direction:{ xs:"column" lg:"row" } gap:2
          Button variant:"outlined" w:"full"
            "Continue with Google"
          Button variant:"outlined" w:"full"
            "Continue with Apple"
```

Choose centering props from the resolved Flex direction:

| Flex direction | Horizontal center | Vertical center |
| --- | --- | --- |
| `row` or omitted | `justify:"center"` | `align:"center"` |
| `column` | `align:"center"` | `justify:"center"` |

`w:"full" maxW:96` bounds the form but does not center it by itself; the parent must center the
bounded child on the correct axis. Prefer the direct child pattern above. If another row Flex is
structurally necessary, give that row `justify:"center"`; `align:"center"` alone only centers its
child vertically.

Responsive breakpoints resolve from the viewport, not from a nested Grid track. At `md`, a split
panel may be roughly half the viewport before its own padding, even though `sm` and `md` rules are
already active. Keep long social or secondary actions stacked until their measured panel width can
hold every label; using `lg` for two action columns is often safer than promoting them at `sm`.
Validate the form and its action groups at `xs`, exactly `md`, and the reference viewport.

Use `cover` when artwork fills and crops with the panel as background geometry. Use `Image` when
the media is an independent foreground object with its own aspect, bounds, and `alt` role. A neutral
split-media track may be a `Box cover:...`; do not use `Card` unless the reference actually shows a
grouped surface with Card semantics.

## Visual direction and section richness

Semantic correctness is the floor, not the finish. Before composing a product or marketing page,
write one visual-direction sentence that combines product character, spatial behavior, and surface
treatment. For example: “precise financial interface with luminous orbital geometry, deep navy
fields, and compact proof surfaces.” Choose at most three recurring motifs and reuse them with
variation so the page feels authored rather than decorated component by component.

Give each substantial band four layers of intent:

| Layer | Question | Dowe expression |
| --- | --- | --- |
| Foundation | What makes this band distinct from its neighbors? | `Section bg`, `background`, `cover` plus `overlay`, or a deliberate flat token field |
| Composition | Where is the visual center and how does the eye move? | Asymmetric `Grid`, editorial `Flex`, or a relative `Box` stage |
| Payload | What can the user see besides copy? | Original `Image`, chart, product UI, icon composition, logo field, testimonial, process, or metric surface |
| Detail | What makes the composition specific to this product? | Number labels, Chips, dividers, floating proof Cards, transforms, shadows, borders, foreground media, or restrained motion |

Compact proof bars, legal copy, and FAQ bands may intentionally use fewer layers. A hero,
capability, product, tokenomics, evidence, or final-action band normally needs all four. If the user
asks to omit sections from a rich reference, preserve this richness floor in every retained band.

Avoid repeating one composition recipe. Consecutive sections should change at least two of these:
alignment, track ratio, surface tone, payload type, density, or foreground silhouette. A centered
heading over three equal Cards can be one band; it must not become the page's universal grammar.

## Layered visual scenes

Use `Box position:"relative"` when the reference's identity comes from overlap, floating proof, or
an illustration that behaves as a stage instead of a simple rectangular image. Keep meaningful UI
inside semantic components and place only their wrappers on the layer plane.

```text
Box position:"relative" minH:{ xs:80 md:96 } rounded:"xl" border:1 borderColor:"primary" shadow:"xl" shadowColor:"primary" p:{ xs:5 md:8 }
  Flex direction:"column" align:"center" justify:"center" gap:4 minH:{ xs:64 md:80 }
    Card variant:"solid" scheme:"surface" p:8 rounded:"xl" rotate:-3 animation:"scaleIn"
      Grid columns:1 gap:3
        Icon name:"layers-minimalistic-bold-duotone" fill:"primary" w:14 h:14
        Title size:"2xl"
          "Core product"
        Text size:"sm"
          "One focal surface anchors the scene."
  Box position:"absolute" top:4 right:4
    Chip variant:"solid" scheme:"primary" shadow:"md" shadowColor:"primary"
      "LIVE"
  Box position:"absolute" left:4 bottom:4
    Card variant:"solid" scheme:"background" p:4 shadow:"lg"
      Flex align:"center" gap:3
        Title size:"2xl"
          "+32%"
        Text size:"xs"
          "Verified activity"
```

The stage needs one dominant object and only a few supporting layers. Do not distribute ten equal
floating elements, stack Cards inside Cards, or use overlap when a normal Grid communicates the
relationship more clearly. On `xs`, keep floating proof inside the bounds, reduce transforms, and
preserve the content reading order even when the visual order changes.

## Modern band patterns

Choose a pattern because it expresses the band's job, not because it is familiar.

| Band | Rich composition options |
| --- | --- |
| Hero | Full-bleed cover or preset, asymmetric promise/media Grid, relative product stage, floating proof, compact metric rail |
| Immediate proof | Logo Marquee, rating-and-avatar row, ticker-like metrics, or one highlighted outcome Card |
| Ecosystem | Central brand or product visual with surrounding nodes, asymmetric feature mosaic, or media-led split with a compact capability list |
| Tokenomics or allocation | Split facts Card plus `ArcChart` or `PieChart`, map/texture field, large values, legends, and public-rule labels |
| Product capabilities | One dominant feature surface plus smaller supporting Cards; vary spans or track ratios instead of six identical tiles when evidence permits |
| Process | Numbered connected rhythm, alternating split steps, Stepper, or icon-led sequence with one visible artifact per step |
| Security or trust | Technical illustration, audit metrics, status Chips, compact controls, and explicit evidence instead of three abstract promises |
| Final action | Immersive cover or high-contrast Card with one outcome, one action, and one small proof or reassurance row |

Use authentic product screenshots or supplied illustrations when they exist. If an original asset
is not available, author its final path and placeholder contract; do not compensate with a wall of
generic Cards.

## Hero sections

A hero is the first page `Section`, not a separate component. Compose it from the same portable
containers and content components used elsewhere, but make its hierarchy unambiguous:

1. Put one primary promise in `Title`, one short supporting `Text`, and one primary `Button` or
   form action in the first reading path.
2. Add only proof that helps the first decision: a review row, customer avatars, a compact metric,
   a trial note, or a customer-mark row. Move detailed features and secondary explanations into
   later Sections.
3. Choose one dominant composition and express it directly:

| Hero intent | Dowe shape |
| --- | --- |
| Editorial or product statement | Centered `Flex direction:"column"` with constrained actions and optional customer marks |
| Copy plus media | Responsive two-track `Grid`; one content column and one `Box cover:` or `Image` region |
| Media plus copy | The same split Grid with media first when the image carries the initial visual weight |
| Lead capture | Responsive split Grid with promise and proof in one column and a flat Input flow or visibly contained form `Card` in the other |
| Immersive campaign | `Section cover:` plus `overlay`, then centered or split content above the generated visual stack |
| Product or analytics story | Relative media `Box` containing direct absolute `Box` wrappers around small Cards, Chips, Icons, or portable Svg data visuals |

When the reference hero is layered, a split Grid with one plain rectangular Image is incomplete
even if the image and copy are correct. Rebuild the visible stage, floating proof, foreground edge,
and tonal transition as separate layers. Keep only one dominant focal asset so the details support
the promise rather than compete with it.

Use `Section boxed:true` when the background or cover is full bleed but the hero content aligns to
the page rails. Give the Section a stable `id` only when navigation links target it. Use responsive
numeric column counts to preserve a portable structure, and add a gap only when the rendered tracks
need an explicit nonzero gutter. Grid columns are equal-width
tracks from `1` through `12`; track templates such as `fr` and `px` are not portable Grid values.
When a composition needs different visual weight, use nested containers or explicit Dowe scale
widths on the relevant content instead of a target-specific track template.

Prefer a scalar `Title size:"6xl"`; Text and Title sizes already use the fluid responsive scale.
Never add a responsive size object merely to make typography responsive. Use one only for an
explicit breakpoint typography change proven after rendering the scalar size. `Title` renders as
`h2` by default; use `as:"h1"` exactly once for the page's primary title, normally this hero title,
and omit it from every other Title. The `as:"h1"` Title must use one fixed scalar `size:"..."` value;
never combine it with a responsive size object or custom weight. This affects web SEO semantics only,
not visual weight. When specific line breaks are part of the composition, use one multiline child
rather than duplicate
compact and wide headline groups. Never hide the only copy or action at a breakpoint.

Treat a media-backed `Box` as a deliberate visual stage: give it a meaningful `minH`, portable
`cover`, radius, optional shadow, and `position:"relative"`. Place overlay content inside direct
`Box position:"absolute"` children with responsive `top`, `right`, `bottom`, or `left` offsets.
The Cards must remain real Dowe content, not flattened artwork. Keep contrast explicit with
`overlay` and semantic foreground tokens when the Section or Card owns a cover.

For a lead form, use a Card only when the form is visibly a contained surface; otherwise place the
Input or flat form flow directly in the content Flex/Grid. Use a one-column Grid when multiple
fields need one structural stack, make the primary submit action full width only when the reference
requires it, and keep legal or privacy copy with the form. Collapse the outer split Grid to one
column on `xs` so the promise remains before the form.

For a viewport-height split form, put `minH:"vh-0"` on the owning Section and `minH:"full"` on
the direct split Grid. The generated Section body accounts for its responsive padding, so both
panels can fill the usable inner height without a spacer or an extra sizing-only wrapper:

```text
Section minH:"vh-0"
  Grid columns:{ xs:1 md:2 } minH:"full"
    Box
      Text
        "Product promise"
    Grid columns:1 minH:"full"
      Input label:"Email"
      Button
        "Continue"
```

Use component-owned defaults first. When the default-first render proves an exception, use one
`gap`, responsive direction, or `w`/`maxW` on the real owner for the missing rhythm or measure. The
default Section body already provides responsive horizontal and vertical insets, so omit all
Section `p*` props for every band. This prohibition includes `p:0`, responsive objects, and
full-viewport forms. Put required spacing on the inner Grid/Flex/Card owner instead of repeating it
through Section, Grid, and Card. Never use unsupported margin props or insert size-only
Box spacers, empty Grid cells, or breakpoint-specific wrapper trees to reproduce offsets from one
screenshot.

```text
Section boxed:true cover:"/assets/images/hero-cover.webp"
  Grid columns:{ xs:1 lg:2 }
    Flex direction:"column"
      Title
        "Share every important idea"
      Text
        "Keep text, voice, photos, and video together."
      Flex
        Button
          "Get started"
        Button scheme:"secondary" variant:"outlined"
          "How it works"
      Text
        "14-day trial · no credit card"
    Image src:"/assets/images/hero-person.webp" alt:"Person using the product"
```

## Landing-page section sequence

Compose a landing page as an ordered argument. Each sibling Section has one conversion job and adds
information that the earlier bands did not already provide.

| Order | Band job | Common composition |
| --- | --- | --- |
| 1 | Promise and first action | Hero pattern above |
| 2 | Immediate credibility | Customer marks, ratings, compact metrics, or one proof row |
| 3 | Problem and outcome | Split copy/media Section or short before-and-after Grid |
| 4 | Capabilities | Responsive Grid of same-shape units with `each`; use Card only for visibly contained surfaces |
| 5 | Evidence | Testimonial, case study, comparison, chart, or results band |
| 6 | Objection handling | Process, FAQ, security, compatibility, or pricing details |
| 7 | Final action | Focused call-to-action Section that repeats the primary outcome and action |

This is a decision guide, not a required fixed count. Omit a band when the product has no meaningful
content for it, and preserve the exact order when reproducing a reference.

Keep the landing page coherent:

- Reuse one boxed content rail across Sections unless a band intentionally changes width.
- Establish a small vertical-padding ladder, using the default Section rhythm first and adding a
  local override only for a band that materially needs to be compact or spacious; do not invent
  unrelated values for every band.
- Alternate flat token backgrounds, Section presets, and media covers only to clarify the argument;
  do not decorate every Section independently.
- Preserve design density when reducing content from a reference. Fewer bands should produce a
  shorter but equally intentional page, not larger empty areas and simpler retained bands.
- Vary focal alignment and payload type across consecutive bands. Repetition should come from the
  theme, rail, spacing ladder, and motifs—not from copying the same section skeleton.
- Use a small number of high-quality details repeatedly: one glow family, one line/border language,
  one numbering style, and one motion character are usually enough.
- Keep repeated cards in Grid tracks, compact proof and action rows in Flex, and each standalone
  form, testimonial, metric, or pricing offer in one Card.
- Give navigable bands stable, unique Section ids and point shell navigation to those anchors.
- Keep AppBar and Footer in the layout even when the landing page is the only route.
- At every breakpoint, preserve the reading order: promise before proof, explanation before detail,
  and the primary action before secondary navigation.

## Equal-height rows and dead space

Sibling cells in a `Grid` row stretch to the tallest sibling. A stretched `Card` whose inner
content is a top-aligned stack leaves dead space at its bottom, which reads as a broken band. The
tallest card in a row is the height budget every sibling must fill. When cards share a row:

1. Give each card one inner `Flex direction:"column" justify:"between" gap:<n> h:"full"` so its
   header block and its trailing visual block distribute across the whole surface.
2. Scale the trailing visual block to the row budget: more icon-grid rows, a taller chart or
   placeholder, or `h:"full"` on the filler block, instead of one small strip floating in a large
   card.
3. Declare `align:"start"` on the parent Grid only when visibly unequal card heights are the
   intended design.

Balance visual weight between paired cards: if one card ends in a tall chart, its sibling needs a
media block of similar height, not just text. Check every stretched card for trailing empty space
before considering a band finished.

## Anti-patterns

| Wrong | Right | Why |
| --- | --- | --- |
| A flat form, metric, article, or profile automatically wrapped in Card | Keep it in Grid/Flex; use Card only when the reference shows a contained surface | Semantic grouping alone does not prove a Card surface |
| `Box` with border, radius, and shadow props rebuilt inline | `Card` | That prop cluster is a Card being imitated |
| `Grid columns:2` for one icon beside text | `Flex` | Tracks force equal columns; the row needs intrinsic sizes |
| `Grid` for a toolbar, chip row, or actions row | `Flex`, adding `wrap:true` only when overflow requires it | One-axis flow is Flex behavior |
| `Flex wrap:true` simulating a catalog of equal cards | `Grid columns:{ xs:1 md:3 }` | Repeated same-shape units are tracks |
| Same-kind container whose only purpose is another gap, padding, size, or visibility prop | Flatten it into the owning Grid or Flex | Same-kind nesting needs a distinct subgroup and layout responsibility |
| Column Flex containing an action-row Flex | Keep both | The inner action group owns a distinct row axis |
| `Card` inside `Card` | One Card containing `Grid` or `Flex` | Nested surfaces double borders and padding |
| `Box` as the default page or section container | `Section`, then Grid or Flex | Pages start at Section; Box is the exception |
| Empty `Box` children used as 12-column offsets | One centered Grid or Flex with `w:"full"` and `maxW` | Empty grid cells encode viewport guesses instead of content structure |
| A bounded form starts at the edge of its split panel | Form-side column Flex with `align:"center"`; use `justify:"center"` too for vertical centering | `maxW` limits measure but does not choose the child's position |
| `Flex align:"center"` expected to center horizontally while direction is `row` or omitted | Add `justify:"center"`, or use a column Flex whose cross axis is horizontal | Flex alignment follows the resolved axis, not the visual intent implied by the word “center” |
| Two long action labels switch to columns at `sm` inside a half-width panel | Keep one column until the labels fit, often `columns:{ xs:1 lg:2 }` | Breakpoints use viewport width while the nested panel may have less than half that usable width |
| A child `Image` is stretched to imitate a panel background | Put `cover` on the owning Section, Card, or neutral media-stage Box | Background media owns the panel crop; foreground Image owns independent content and `alt` semantics |
| `translateX` or `translateY` used to align normal content | AppBar regions, Grid/Flex alignment, `gap`, padding, `w`, or `maxW` on the real owner | Transforms change visual placement without defining the surrounding flow geometry |
| A translated `NavMenu`, AppBar action, or Drawer root | Keep the compound root untransformed and align it through its semantic owner | Anchored menus and overlays need stable component geometry |
| `Box` around each Input, Pin, Phone, or Button | Style the real control and group fields in Grid or Flex | Controls already own their surface, metrics, and responsive props |
| Separate mobile and desktop form trees | One tree with responsive Grid, Flex, spacing, sizing, and visibility | Duplicate bindings and workflows drift and encourage spacer wrappers |
| Visual props repeated on every Card or Button | Defaults in `theme.dowe` `design` | Local props are for one intentional exception |
| A page declaring `Scaffold`, `AppBar`, or `Footer` | Move shell to the layout | Shell chrome is layout-owned |
| The same nav tree copy-pasted into Sidebar and Drawer | One `component` mounted in both | Duplicated fragments drift apart |
| `NavMenu` used as the body of a Drawer or Sidebar | `SideNav` in the surface `body`; keep `NavMenu` in AppBar `center` or `end` | `NavMenu` is horizontal and does not express a vertical side navigation surface |
| One reusable navigation component with a `NavMenu` root mounted in Drawer and AppBar | A reusable `SideNav` root for Sidebar/Drawer plus a direct AppBar `NavMenu` | A component cannot receive mount-specific responsive props, and the two components own different orientations |
| `Flex` directly inside AppBar `start`, `center`, or `end` only to place siblings | Put `Brand`/`Logo`, `NavMenu`, `Button`, and `IconButton` directly in the AppBar region; use `IconButton` before Brand in `start` or directly in `end` | AppBar regions already own horizontal flex, vertical alignment, and spacing; the wrapper obscures semantic order |
| A `component` holding signals, functions, or bindings | Keep state in the owning layout or page | Components are static reusable trees |
| One Card declaration copied for every visible record | One collection, one `each`, and one Card template | Repeated content needs one data owner and one visual contract |
| A data-bound Card extracted with invented component props | Keep the `each` template in its page or layout | Reusable components do not accept dynamic caller inputs |
| A photo redrawn with `Svg` paths or `Canvas` commands | `Image` with its intended `src` path | Photographs are assets, not vector source |
| A `Box` with an icon standing in for a photo | `Image` with the named asset path and `scheme` | The unresolved frame is already the placeholder, and the path stays swappable |
| Every band is eyebrow + centered title + equal Cards | Alternate split, mosaic, visual-stage, proof-row, and text-led bands according to their jobs | Repeating one skeleton makes a long page look generated and flat |
| A rich reference scene reduced to one framed Image | Relative stage with the focal asset, floating proof, visible ornament, and foreground treatment | The missing layers carry the reference's depth and identity |
| Every Card uses the same border and quiet fill | Establish primary, supporting, and quiet surface roles | Surface hierarchy directs attention and prevents component-library sameness |
| Decoration added without a product concept | Choose two or three motifs from the product and reuse them deliberately | Specific visual language feels natural; arbitrary effects feel synthetic |
| Empty space used where the reference has visual evidence | Add the original payload, chart, logo field, process artifact, or declared asset placeholder | Whitespace cannot substitute for missing information or media |

## Composition checklist

- One Scaffold per layout; Sections only in pages; Splash bound and resolved in every branch.
- Every container choice is justified by the decision tree. Each `Box` owns a relative, absolute,
  fixed, or explicitly documented neutral media/drawing layer that Section, Grid, Flex, Card, or
  the real child cannot express.
- Same-kind nesting survives only for a distinct subgroup and independently verifiable track or
  axis, such as a column Flex containing one action-row Flex. No wrapper exists only to change gap,
  padding, sizing, or visibility.
- No empty Box grid gutters, control wrappers, or duplicated compact/wide form trees.
- Split forms are centered against their owning panel at the reference viewport and at `md`; a
  bounded form Grid is never assumed centered merely because it declares `maxW`.
- Flex centering uses `justify` on the main axis and `align` on the cross axis after resolving
  direction; nested row wrappers do not rely on `align` for horizontal placement.
- Background-filling media uses `cover` on its semantic owner or neutral media stage; foreground
  `Image` remains reserved for independently meaningful media.
- Nested action columns switch only where their complete labels fit the panel's usable width.
- Every repeated same-shape region, including non-Card feature rows, uses one `const`, typed `signal`,
  or shared Store and one `each` that wraps the complete unit inside the Grid tracks; no copy-pasted
  sibling survives visual QA.
- Static-only props remain valid inside repeated templates; `Icon.name` is never bound to an item path.
- Spacing starts with component defaults. A nonzero `gap` or one padding override is added only after
  the minimal render proves the real owner needs it; no stacked `p*` props, automatic gap on every
  Grid/Flex, empty Box spacers, or margin aliases such as `mt` survive.
- Every local prop passes the admission gate: required contract/content/accessibility, behavior,
  essential structure, explicit non-default choice, or a specific mismatch proven after rendering.
- `translateX` and `translateY` appear only on documented advanced visual layers, never on AppBar,
  Brand, NavMenu, Drawer, ordinary content, or normal-flow alignment corrections.
- No stretched card ends in trailing dead space; paired cards match visual weight, distributing
  content with `justify:"between"` or scaling their media blocks to the row height.
- `NavMenu` appears only directly in AppBar `center` or `end`; every vertical navigation inside a
  Sidebar or Drawer is a `SideNav`, including the reusable component mounted in both surfaces.
- AppBar regions contain direct semantic children; no wrapper `Flex` exists solely for AppBar
  alignment or gap. A mobile Drawer trigger is before Brand in `start` or directly in `end`.
- Section vertical padding follows one consistent ladder across the page instead of per-band
  improvisation.
- Visual identity lives in `theme.dowe`; page props are structural or intentional exceptions.
- Semantic color tokens and component variants replace literal colors and rebuilt chrome.
- One visual-direction sentence and at most three recurring motifs guide the page.
- Every substantial marketing band has a foundation, composition, payload, and product-specific
  detail; compact utility bands are intentionally simpler.
- Consecutive bands do not repeat the same alignment, track ratio, payload, and surface treatment.
- Layered reference scenes remain layered Dowe source instead of becoming one Image or Card.
