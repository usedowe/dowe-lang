# UI composition reference

This reference decides structure: what belongs to the layout, what belongs to the page, and which
container owns each region. Read it before composing a screen, and always before translating a
reference design or screenshot into source.

## Fidelity contract

When the user asks to reproduce a reference exactly, treat the reference as a visual contract, not
as loose inspiration.

The reference image is validation evidence, not an asset. Its pixels may guide measurements and
visual comparison, but they must not appear in the generated application.

1. Record the reference viewport and inventory the complete visible surface before writing source:
   shell chrome, ordered horizontal bands, headings, body copy, actions, navigation labels, media,
   decorative layers, and repeated units.
2. Preserve visible wording, capitalization, order, item count, alignment, and hierarchy. Do not add
   generic actions, replace copy, collapse bands, or omit a region because a simpler substitute is
   available.
3. Distinguish full-bleed bands from centered content bodies. Use `Section boxed:true` for a
   centered capped body while leaving its outer background full width; use `AppBar boxed:true` and
   `Footer boxed:true` when their inner rows align to the same content rails.
4. Rebuild UI-shaped content with Dowe. Navigation, text, buttons, cards, charts, dashboards,
   tables, badges, icons, logos, and decorative geometry are source structure, even when the
   reference presents several of them as one flattened region.
5. Match irreducible media by subject, crop, aspect, and visual weight. Reuse original supplied or
   existing project assets, or obtain a suitable photograph, illustration, or texture independently.
   Do not crop or slice the reference image into production assets. If an original is unavailable,
   author a stable local asset path with the intended aspect and report it as required; do not claim
   final visual parity.
6. Render at the reference viewport. Compare the shell and each band in order, checking content
   width, section height, column ratio, line wrapping, gaps, image crop, border radius, color,
   typography, and vertical rhythm. Fix the largest mismatch first and repeat until no major
   geometric or content difference remains.

Keep a short internal inventory while working. For a long landing page it should account for every
visible band and its dominant composition, not merely the first viewport.

## Dowe-native reconstruction boundary

Classify every visual before creating files:

| Reference content | Authoring contract |
| --- | --- |
| Navigation, headings, body text, controls, cards, lists, metrics, badges, tables, charts, dashboards, forms, and repeated services | Rebuild with semantic Dowe components and visible text children |
| Simple marks, logos, icons, dividers, lines, dots, and decorative geometry | Use `Brand`, `Icon`, `Svg`, `Path`, `Divider`, or a small semantic container composition |
| Custom data visualization unsupported by chart components | Use target-neutral `Canvas` commands after checking `references/canvas.md` |
| Photograph, illustration, texture, or genuine media artwork | Use `Image` only from an independently obtained original, an existing project asset, or media explicitly supplied by the user |
| Authentic product screenshot that must remain a screenshot | Use `Image` only when the user explicitly requests the screenshot treatment or supplies the original capture |
| The design reference itself or any crop, slice, rasterization, or recomposition derived from it | Never ship it; keep it outside project assets and use it only for measurement and visual comparison |

A flattened region is not evidence that `Image` is the right component. If the pixels depict UI
that Dowe can express, reconstruct the region in Dowe so it remains responsive, accessible,
themeable, inspectable, and portable across web, desktop, Android, and iOS.

Do not use a crop of the reference for a logo, service row, testimonial strip, dashboard, chart,
marketing card group, footer, or complete Section. Do not hide reference captures behind Dowe
containers. A correct implementation remains recognizable when all photographic assets are
temporarily replaced by their Dowe placeholder frames.

## Layout versus page ownership

| Surface | Root contract | Owns | Never contains |
| --- | --- | --- | --- |
| `layout` | Exactly one `Scaffold`, plus one optional direct `Splash` | AppBar, SideNav or Sidebar, Footer, BottomBar, shell overlays, the `children` boundary, session or shell state | `Section` page bands, page content, routed data loading that belongs to one page |
| `page` | One or more sibling `Section` roots, plus one optional direct `Splash` | Page bands, page state, page functions, page data loading | `Scaffold`, `AppBar`, `Footer`, `BottomBar`, `children` |

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
      Footer boxed:true variant:"soft" scheme:"surface"
        start
          Text weight:"bold"
            "SOLTECH"
        end
          Text size:"sm" color:"muted"
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
        Text size:"sm" color:"muted"
          "Validating your session"
```

Use a layout `Splash` for whole-application gates such as session validation, and a page `Splash`
for that page's own loading state. Both bind a boolean set explicitly in every `init` branch.

## Reusable components

`component <Name>` in `views/components` declares an importable reusable static view tree. A
component owns no signals, functions, stores, or bindings to caller state; it is pure structure
and visual props, and theme defaults apply inside it normally.

Extract a component when the same static fragment appears in two or more places — a brand or logo
(`Brand` plus `Svg`), a navigation tree, a social link block, a footer column — or when a deep
static fragment makes a layout or page hard to read. The canonical responsive navigation pattern
declares the menu once and mounts it in both shell surfaces, so desktop and mobile navigation can
never drift apart:

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
    start
      Sidebar show:{ xs:false md:true } variant:"ghost" scheme:"muted" w:72
        body
          ViewsNavigation
    main
      children
    overlays
      Drawer open:openDrawer
        body
          ViewsNavigation
```

The `openDrawer` signal and its inline toggle stay in the layout; the component stays stateless.
If a fragment needs bindings to page state, an `each`, or event functions, keep it in the owning
page or layout instead of extracting it. Do not rebuild built-in components as custom components,
and do not extract a fragment used in only one place.

## Container decision tree

Decide top-down for every region, in this order:

1. Is it a major horizontal band of the page (hero, features, catalog, form area, pricing,
   testimonials, call to action)? Use a sibling `Section`.
2. Do sibling children align to shared tracks — repeated same-shape units, a dashboard, a catalog,
   responsive columns, or a stack with one uniform rhythm? Use `Grid` with `columns`, `gap`, and
   responsive values such as `columns:{ xs:1 md:3 }`.
3. Do children flow on one axis with individual intrinsic sizes — an icon beside text, a label
   with a trailing action, a toolbar, a chip row, centered content? Use `Flex` with `direction`,
   `align`, `justify`, `gap`, and `wrap:true` for rows that may overflow.
4. Is the group one related semantic unit the user reads as a surface — a form, metric, article,
   product, profile, setting group? Wrap it in `Card` and let `theme.dowe` style it.
5. Only when no semantic component fits — a plain background band, cover or overlay holder, or a
   neutral absolute wrapper — use `Box`.

`Grid` and `Flex` arrange; `Card` groups and styles; `Box` is the documented exception, not a
default. `Grid columns:1 gap:<n>` and `Flex direction:"column" gap:<n>` are both valid vertical
stacks: prefer Grid when the stack is structural rhythm between blocks, and Flex when the column
also needs `align` or `justify` behavior, such as centering Splash content.

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
| Lead capture | Responsive split Grid with the promise and proof in one column and one form `Card` in the other |
| Immersive campaign | `Section cover:` plus `overlay`, then centered or split content above the generated visual stack |
| Product or analytics story | Relative media `Box` containing direct absolute `Box` wrappers around small Cards, Chips, Icons, or portable Svg data visuals |

Use `Section boxed:true` when the background or cover is full bleed but the hero content aligns to
the page rails. Give the Section a stable `id` when navigation links target it. Use responsive
column templates and gaps to preserve the design's real proportions; equal `1fr 1fr` columns are
not a default when the reference clearly gives copy, media, or a form more space.

Prefer one responsive `Title size:{ xs:"4xl" md:"6xl" }` when natural wrapping is acceptable.
When specific line breaks are part of the composition, author separate compact and wide headline
groups with complementary `show` values. Never hide the only copy or action at a breakpoint.

Treat a media-backed `Box` as a deliberate visual stage: give it a meaningful `minH`, portable
`cover`, radius, optional shadow, and `position:"relative"`. Place overlay content inside direct
`Box position:"absolute"` children with responsive `top`, `right`, `bottom`, or `left` offsets.
The Cards must remain real Dowe content, not flattened artwork. Keep contrast explicit with
`overlay` and semantic foreground tokens when the Section or Card owns a cover.

For a lead form, the form is one Card rather than a generic Box. Stack its fields with
`Grid columns:1 gap:<n>`, make the primary submit action full width when appropriate, and keep
legal or privacy copy inside the same Card. Collapse the outer split Grid to one column on `xs` so
the promise remains before the form.

Use `gap`, Section padding, or explicit `pt` and `pb` for ordinary rhythm. A size-only responsive
`Box` is an exception for an intentional track in a wider asymmetric Grid, or for editorial
spacing that must differ by breakpoint around fixed headline groups, forms, or proof rows. Give
the spacer an explicit size and complementary `show` values where compact and wide layouts need
different heights; do not scatter empty Boxes through ordinary stacks.

```text
Section id:"hero" background:"aurora" boxed:true py:{ xs:8 md:12 }
  Grid columns:{ xs:1 md:"5fr 6fr" } gap:{ xs:8 md:16 } align:"center"
    Flex direction:"column" align:"start" gap:6
      Title size:{ xs:"4xl" md:"6xl" } weight:"black"
        "Turn useful ideas into durable growth"
      Text size:"lg" color:"muted"
        "Plan, publish, and learn from one focused workspace."
      Flex direction:{ xs:"column" sm:"row" } gap:3
        Button size:"lg"
          "Start free"
        Button variant:"outlined" scheme:"muted" size:"lg"
          "See how it works"
      Flex align:"center" gap:2
        Icon name:"check-circle" style:"bold" fill:"success"
        Text size:"sm" color:"muted"
          "14-day trial · no credit card"
    Box position:"relative" cover:"/assets/images/hero-team.jpg" rounded:"xl" minH:"vh-48"
      Box position:"absolute" right:6 bottom:6
        Card variant:"solid" scheme:"surface" shadow:"xl" shadowColor:"success"
          Grid columns:1 gap:1
            Title size:"3xl" weight:"black"
              "32%"
            Text size:"sm" color:"muted"
              "Audience growth this month"
```

## Landing-page section sequence

Compose a landing page as an ordered argument. Each sibling Section has one conversion job and adds
information that the earlier bands did not already provide.

| Order | Band job | Common composition |
| --- | --- | --- |
| 1 | Promise and first action | Hero pattern above |
| 2 | Immediate credibility | Customer marks, ratings, compact metrics, or one proof row |
| 3 | Problem and outcome | Split copy/media Section or short before-and-after Grid |
| 4 | Capabilities | Responsive Grid of same-shape Cards, preferably data-driven with `each` |
| 5 | Evidence | Testimonial, case study, comparison, chart, or results band |
| 6 | Objection handling | Process, FAQ, security, compatibility, or pricing details |
| 7 | Final action | Focused call-to-action Section that repeats the primary outcome and action |

This is a decision guide, not a required fixed count. Omit a band when the product has no meaningful
content for it, and preserve the exact order when reproducing a reference.

Keep the landing page coherent:

- Reuse one boxed content rail across Sections unless a band intentionally changes width.
- Establish a small vertical-padding ladder, such as compact proof, standard content, and spacious
  hero or final CTA, instead of inventing unrelated values for every band.
- Alternate flat token backgrounds, Section presets, and media covers only to clarify the argument;
  do not decorate every Section independently.
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

Balance visual weight between paired cards the way the reference does: if one card ends in a tall
chart, its sibling needs a media block of similar height, not just text. Check every stretched
card for trailing empty space before considering a band finished.

## Anti-patterns

| Wrong | Right | Why |
| --- | --- | --- |
| `Box` around a form, metric, article, or profile | `Card` | The theme styles Cards once; Box has no surface semantics and no theme slot |
| `Box` with border, radius, and shadow props rebuilt inline | `Card` | That prop cluster is a Card being imitated |
| `Grid columns:2` for one icon beside text | `Flex align:"center" gap:2` | Tracks force equal columns; the row needs intrinsic sizes |
| `Grid` for a toolbar, chip row, or actions row | `Flex gap:<n> wrap:true` | One-axis flow with alignment is Flex behavior |
| `Flex wrap:true` simulating a catalog of equal cards | `Grid columns:{ xs:1 md:3 }` | Repeated same-shape units are tracks |
| `Card` inside `Card` | One Card containing `Grid` or `Flex` | Nested surfaces double borders and padding |
| `Box` as the default page or section container | `Section`, then Grid or Flex | Pages start at Section; Box is the exception |
| Visual props repeated on every Card or Button | Defaults in `theme.dowe` `design` | Local props are for one intentional exception |
| A page declaring `Scaffold`, `AppBar`, or `Footer` | Move shell to the layout | Shell chrome is layout-owned |
| The same nav tree copy-pasted into Sidebar and Drawer | One `component` mounted in both | Duplicated fragments drift apart |
| A `component` holding signals, functions, or bindings | Keep state in the owning layout or page | Components are static reusable trees |
| A reference screenshot or crop used as a Section, card grid, service row, dashboard, chart, logo, or footer | Rebuild the region with Dowe components | Reference pixels are validation evidence, not application source |
| A photograph cropped from the design reference | Use an independently obtained original or a named missing asset path | The design capture is never an asset source |
| A photo redrawn with `Svg` paths or `Canvas` commands | `Image` with its intended `src` path | Photographs are assets, not vector source |
| A `Box` with an icon standing in for a photo | `Image` with the named asset path and `scheme` | The unresolved frame is already the placeholder, and the path stays swappable |

## From reference design to source

When given a reference image, mockup, or screenshot, decompose it in this order instead of
transcribing it top-left to bottom-right:

1. Separate shell from content. Top bar, side navigation, footer, bottom tabs, and floating
   actions become the layout Scaffold regions; everything that scrolls per route becomes the page.
2. Slice the page into horizontal bands. Each visually distinct band is one sibling `Section`;
   full-bleed color, `cover`, or gradient bands stay on the Section while `boxed:true` caps the
   content body.
3. Inside each band, find the repetition. Repeated same-shape units become
   `Grid columns:{ ... } gap:<n>` with one `each` and one `Card` per unit.
4. Inside each unit, read the axes. Icon-and-text pairs, label-and-value rows, and action rows are
   `Flex`; stacked blocks with even rhythm are `Grid columns:1 gap:<n>`.
5. Classify every bordered, elevated, or tinted surface as `Card` with a variant and scheme; only
   surfaces with no grouping meaning stay `Box`.
6. Rebuild every UI-shaped region in Dowe: navigation, headings, buttons, service rows, card groups,
   charts, dashboards, metrics, tables, badges, icons, logos, and decorative geometry. Never use
   the reference or a crop from it as an `Image` source.
7. Map only irreducible media to `Image`: independently obtained photographs, illustrations,
   textures, or an authentic screenshot explicitly supplied or requested by the user. When an
   original does not exist yet, author the final contract anyway: a named project path such as
   `src:"/assets/images/hero-team.jpg"` plus `alt`, an `aspect` matching the intended proportions,
   `objectFit:"cover"`, and a `scheme` tint. An unresolved source renders its styled frame as a
   built-in placeholder without crashing, so dropping the original file at that path activates the
   image with no source change; list the expected asset paths when finishing. Full-bleed
   photographs use `Section cover:` with the same path or HTTPS contract.
8. Map text roles to `Title` and `Text` sizes and weights, actions to `Button`, `IconButton`, or
   navigation props, and status accents to `Chip`, `Badge`, or `Alert`. Use built-in navigation,
   form, and overlay components instead of rebuilding them from containers.
9. Extract the visual system once: put the repeated Card, Button, Chip, Avatar, and font choices
   in `theme.dowe` `design` so page source keeps only structural props and intentional exceptions.
   Fonts come from the closed Dowe token catalog; match the reference's typographic character to
   the closest token using the dowe-theme font table instead of importing the original family.
10. Add responsive behavior from the widest breakpoint down: collapse Grid columns, switch
   `Flex direction:{ xs:"column" md:"row" }`, and hide secondary chrome with
   `show:{ xs:false md:true }`.
11. Render at the reference viewport and compare all visible bands, including content after the
    initial viewport. Do not stop after the first valid compile or after matching only the hero.

A faithful decomposition of a typical dashboard band looks like:

```text
Section boxed:true px:{ xs:4 md:8 } py:{ xs:6 md:10 }
  Grid columns:1 gap:8
    Flex align:"center" justify:"between" wrap:true gap:4
      Title size:{ xs:"3xl" md:"5xl" }
        "Overview"
      Button onClick:openCreateModal
        "New report"
    Grid columns:{ xs:1 md:3 } gap:4
      each in:metrics as:metric key:metric.id
        Card
          Grid columns:1 gap:2
            Text size:"sm" color:"muted"
              "{metric.label}"
            Title size:"3xl"
              "{metric.value}"
```

## Composition checklist

- One Scaffold per layout; Sections only in pages; Splash bound and resolved in every branch.
- Every container choice justified by the decision tree; any `Box` must be explainable as the
  neutral exception.
- Repetition uses `each` inside Grid tracks with Card units, never copy-pasted siblings.
- Spacing normally comes from `gap`, Section padding, and explicit padding props. A size-only Box
  is limited to an intentional responsive Grid rail or breakpoint-specific editorial spacer.
- No stretched card ends in trailing dead space; paired cards match visual weight, distributing
  content with `justify:"between"` or scaling their media blocks to the row height.
- Section vertical padding follows one consistent ladder across the page instead of per-band
  improvisation.
- Visual identity lives in `theme.dowe`; page props are structural or intentional exceptions.
- Semantic color tokens and component variants replace literal colors and rebuilt chrome.
- Visible copy, navigation, actions, section count, and media intent match the reference; missing
  exact assets remain explicit local paths rather than unrelated substitutions.
- No project asset is the reference image, a crop from it, or a flattened replacement for UI that
  Dowe components can express.
- For reference-driven work, run the web target, capture the rendered page, and compare it band by
  band against the reference; fix dead space, alignment, and density differences before finishing.
