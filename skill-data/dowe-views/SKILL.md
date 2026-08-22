---
name: dowe-views
compatibility: Requires Python 3 for the optional visual QA scripts and the Dowe CLI for project validation.
description: Use for Dowe view modules, routes, layouts, pages, UI composition, components, state, requests, responsive styles, Canvas, portable and advanced data tables, loading/empty/error states, search and pagination compositions, view targets, modern product or marketing visual direction, layered scenes, or exact and adapted reconstruction from an attached screenshot, mockup, template, or UI reference, including semantic component selection, shell/page ownership, reusable static fragments, and repeated collections rendered with each. Pair with dowe-server when a View request needs a project-owned route, server logic, persistence, or security change; skip only work that is entirely server-owned.
---

# Dowe views authoring

Dowe views are target-neutral source compiled to web, desktop, Android, and iOS outputs. Reuse one route graph and one source behavior model across targets.
Keep every new frontend module under `views/`; only root `main.dowe` and `theme.dowe` sit outside it.

## Fullstack request gate

A prompt is not View-only when the requested interaction reads or writes project-owned server data,
calls an internal API route, or requires server authorization, validation, persistence, or provider
behavior. This includes tasks that begin from a screenshot or page request when the visible form,
table, search, pagination, authentication, upload, or action depends on an endpoint. In those cases,
load the companion `dowe-server` skill before editing either side.

Create a request-to-route matrix for every affected internal request. Record its owning page or
layout and function or `init`; HTTP method and resolved path; body, headers, and route parameters;
expected safe JSON response; loading, success, empty, error, and unauthorized states; endpoint,
handler, middleware, service, and repository owners; data impact; and authorization boundary.

Inspect the existing Server route and full import chain. If the route is missing or its method,
input, response, authorization, or data behavior does not satisfy the requested View, update the
Server in the same task. The required scope may include endpoints, handlers, middleware, services,
repositories, entities, migrations, Database, Cache, Vector, Queue, configuration, or environment
names. Keep the change limited to the requested capability and reuse existing owners before adding
new modules.

Views still own Signals, Stores, request dispatch, and presentation states. Server owns request
parsing, authoritative validation, authorization, business rules, secrets, providers, persistence,
and responses. Never import Server bindings into Views or duplicate Server rules in client state.
A request to an independently owned external API does not justify inventing a project Server route.

## Non-duplication gate

Repetition is a structural contract, not a style preference. Treat two or more sibling units with the
same semantic or layout shape as a collection, even when the reference contains only two items or the
only differences are text, icons, links, or status values. This includes feature rows such as
`Flex` + icon + title/description, not only `Card` grids.

Before authoring the repeated view:

- Declare immutable reference content in a page or layout `const`, or use a typed `signal`/Store when
  the collection is reactive. Give every record a stable string `id`.
- Render one complete repeated unit with exactly one `each in:<collection> as:<item> key:<item.id>`.
  The `each` must wrap the whole unit, including its icon, copy, actions, and state—not just one text
  node inside a set of copied siblings.
- Bind every varying supported field from the current item and keep the `Grid`/`Flex` structure
  outside or inside that single template according to the reference's actual track ownership.
- Reject the result if the source contains copied sibling rows, Cards, list units, or feature groups
  that could be represented by one collection and one template. Fix the data boundary before doing
  visual polish.

Keep static-only props static. In particular, `Icon.name` is a quoted, compiler-validated name and
must not be invented as `name:<item.icon>`. For varying runtime vector data, use the supported
`Svg data:<reference>` contract; otherwise preserve the component's static contract and report a
missing language capability instead of duplicating the complete repeated unit or inventing syntax.

## Reference-image theme boundary

When a reference image is supplied to build or adapt a layout, page, reusable component, or
`Section`, use it as evidence for structure, geometry, typography, spacing, assets, layering, and
visual hierarchy. Do not sample its colors, create a new `theme.dowe`, change existing theme colors,
or add literal color overrides just to imitate the image. Preserve the project's existing theme and
semantic color tokens. Generate a theme or modify its colors only when the user explicitly requests
theme or color changes.

## Workflow

1. Find the imported `views` binding connected by `main.dowe`, inspect the route graph, and read
   `theme.dowe`. If the task is page-only, treat the existing theme as read-only and use its
   semantic tokens; do not regenerate its palette from the page reference.
2. For reference-driven work, follow `references/reference-ui.md`: initialize the required
   `.dowe/visual-qa/<screen>/blueprint.json`, then inventory the reference viewport and record a
   composition map with ordered bands, region ownership, exact built-ins, collection owners,
   responsive evidence, states, accessibility, theme decisions, assets, and reusable-component
   candidates before authoring source. Reduce that map to a minimal component tree before adding
   local visual props. Measurements are comparison evidence, not a list of padding, sizing, radius,
   shadow, typography, alignment, or gap declarations to serialize.
   For block-driven work based on Dowe's documented UI patterns, read
   `references/blocks/index.json` first. Select at most five candidates, then combine one primary
   block with at most one supporting pattern; use the family composition rules and variant tags as
   design guidance, not as permission to copy documentation gallery wrappers.
3. Before writing source, state one visual-direction sentence and choose at most three recurring
   motifs from the evidence: for example orbital geometry, luminous data surfaces, editorial
   typography, translucent panels, or technical linework. Map each retained band to a distinct
   composition and visual payload; do not let the implementation default to repeated headings over
   uniform Card grids.
4. Decide whether the request is an exact reconstruction or a directed adaptation. Exact work
   preserves every visible band and measured relationship. Adaptation may change copy or omit
   content only as requested, while preserving the reference's visual density, depth, focal
   hierarchy, asset intent, and characteristic details in every retained band.
5. Keep reference evidence distinct from inference. Preserve visible copy, geometry, hierarchy,
   density, layered depth, and media intent; infer only behavior the supplied viewport cannot show.
   When the reference shows a scene made from background, focal media, floating proof, ornament,
   and foreground, record and rebuild those as separate Dowe layers rather than collapsing them to
   one Image or Card.
6. Resolve UI roles against `references/components.md`. Prefer the semantic built-in that owns the
   behavior, use contextual children only under their declared owner, and never invent a component
   name from a label in the reference.
7. Rebuild every UI-shaped region with Dowe components. Never use the reference image or crops
   derived from it as assets. Use `Image` only for independently obtained photographs,
   illustrations, textures, or authentic screenshots explicitly supplied or requested by the user.
   When media is a cropped background that fills a panel or band, put `cover` on the owning
   `Section`, `Card`, or neutral media-stage `Box` instead of rendering a child `Image`; reserve
   `Image` for foreground media with its own content and accessibility role.
   Public project media follows a fixed URL mapping: a source file at
   `assets/<relative-path>` is referenced from a view as `/assets/<relative-path>`. For example,
   `assets/img/hero.webp` becomes `Image src:"/assets/img/hero.webp"` or
   `cover:"/assets/img/hero.webp"`. Do not use an absolute filesystem path such as
   `/Users/name/project/assets/img/hero.webp`, a source-relative path such as `assets/img/hero.webp`,
   or a stripped path such as `/img/hero.webp`; Dowe's built-in web runtime serves the project
   `assets/**` tree under `/assets/**`. Verify the file exists with the exact case before authoring
   the URL. Keep the same root-relative URL in the shared source for web, desktop, Android, and iOS;
   target packaging resolves the local file separately.
8. Create or reuse a layout whenever the reference has shared chrome. AppBar and Footer never
   belong in a page, and a one-page site still uses a layout-backed route group.
   Treat `AppBar` as the shell's semantic navigation bar: keep exactly one `AppBar` directly under
   `Scaffold appBar`, map full-width announcements or secondary bands to `top` and `bottom`, and
   put the main row in `start`, `center`, and `end`. `NavMenu` is horizontal shell navigation: place
   it only as a direct child of AppBar `center` (primary navigation) or `end` (compact/secondary
   navigation). Never put `NavMenu` inside `Drawer`, `Sidebar`, `Scaffold start/end`, or a generic
   content region. AppBar `start`, `center`, and `end` already lay out their direct children as
   horizontal flex rows with centered cross-axis alignment and component-owned spacing. Put
   `Brand`/logo, `NavMenu`, `Button`, and `IconButton` directly in their region; do not add a
   `Flex` only to align, gap, or place those siblings. Use a nested `Flex` only when it owns a
   distinct layout responsibility such as a column, wrap, or independently structured control
   cluster. Put `show` on the actual `NavMenu`, `Button`, or `IconButton`; do not wrap those slot
   children in `Box` only to provide responsive visibility, alignment, or width. A `Box` inside an
   AppBar slot is reserved for a meaningful positioned or decorative layer, not as a generic slot
   adapter. For mobile navigation, put the Drawer trigger before the `Brand`/logo when both are in
   `start`, or put it directly in `end`; do not place it after the brand inside a wrapper `Flex` as
   the default. Do not rebuild the same AppBar with `Box` nodes in `overlays`; use `Drawer` there
   only for the mobile navigation surface, and put a vertical `SideNav` in its `body`. Reusable
   components accept no props, so keep the static `SideNav` navigation component reusable in both
   the desktop `Sidebar` and mobile `Drawer`, while the AppBar owns its direct `NavMenu` instance.
9. Put exactly one normal `Scaffold` root in every layout; add one direct `Splash` sibling only when
   startup replacement content is required.
10. Start every page with `Section` and use ordered sibling Sections for major page bands. Give
   every landing-page band one job, and make the hero establish the primary promise, support,
   action, and proof before later Sections add detail. Preserve visible copy, band order, actions,
   density, and media intent instead of inventing generic replacements.
11. Begin with no `Box` nodes. Use `Section` for page bands, `Grid` for tracks and responsive
    structure, `Flex` for flow, alignment, and centering, and `Card` for one grouped surface. Add
    `Box` only when normal flow cannot express the composition, normally as a relative layer plane
    with direct absolute children or as a fixed viewport layer. Padding, sizing, backgrounds,
    borders, visibility, grid gutters, and control wrappers do not justify `Box`; put those props on
    the real owner. Reject layout wrappers that exist only to carry another `gap`, padding, size, or
    visibility value. Same-kind nesting is allowed only for a distinct subgroup with its own layout
    responsibility: for example, a column `Flex` may contain one row `Flex` that owns the actions.
    A column Flex inside another column Flex only to change spacing is wrapper noise and must be
    flattened. Do not alternate Grid and Flex merely to hide the same redundant wrapper: every
    container must own a distinct track, axis, centering, wrapping, or responsive responsibility.
    Keep one responsive source tree instead of duplicating mobile and desktop forms. Follow the
    decision tree and minimal reference patterns in `references/composition.md`.
    For a split auth screen, let the outer Grid own the two panels, let the form-side column Flex
    center one bounded `Grid w:"full" maxW:<scale>`, and never create empty Grid tracks or Boxes as
    offsets. Remember that `justify` controls the resolved main axis and `align` the cross axis: a
    column Flex centers horizontally with `align:"center"`, while a default row Flex centers
    horizontally with `justify:"center"`. Prefer the direct bounded form child over an extra
    wrapper whose axis can silently anchor the form to the panel edge.
12. Keep normal-flow geometry free of `translateX` and `translateY`. Do not use translation to
    center, align, space, size, or nudge AppBar regions, navigation, controls, content, Sections,
    Grids, or Flex stacks toward screenshot coordinates. First solve one-axis placement with `Flex`
    using `direction`, `justify`, `align`, `gap`, and `wrap`; solve shared tracks and responsive
    columns with `Grid` using `columns`, `rows`, `gap`, `align`, and `justify`. Use AppBar regions,
    `boxed`, padding, responsive direction, and `w` or `maxW` on the real layout owner as needed.
    Never translate `AppBar`, `Brand`, `NavMenu`, `Drawer`, or another compound overlay trigger or
    root; its menus and floating surfaces must remain anchored to untransformed semantic geometry.
    Translation is an advanced layer effect only: use it sparingly inside a documented relative
    `Box` scene for a decorative or floating element whose intentional overlap cannot be expressed
    by Flex, Grid, normal flow, or absolute offsets. Zero authored translations is the default.
13. Give every substantial marketing band a visual payload beyond title and body copy: original
    media, a data visual, product UI, icon composition, logo field, testimonial, process diagram,
    or layered proof surface. Add a restrained detail pass with supported covers, overlays,
    positioning, transforms, shadows, borders, motion, and tonal contrast from
    `references/styles.md`; decoration must reinforce the concept rather than fill empty space.
14. Apply the non-duplication gate to every repeated same-shape unit: use a `const` for immutable
    reference-defined content, a typed `signal` for a page collection refreshed or replaced by
    requests or local workflows, and an imported View Store only for state shared across routes.
    Render the complete unit with one `each in:<collection> as:<item> key:<item-path>`; never copy
    sibling Cards, feature rows, icon/text groups, or list units. A result with repeated siblings is
    incomplete even when it looks visually correct.
15. Extract a static fragment reused in two or more places, such as a logo or a vertical `SideNav`
    tree mounted in both Sidebar and Drawer, into a `component` under `views/components`; keep signals,
    functions, caller bindings, and data-bound `each` templates in the owning layout or page because
    reusable components are static and accept no invented props or slots.
16. Prefer component defaults from `theme.dowe`; add local visual props only for intentional
    exceptions. A reference image for a layout, page, reusable component, or `Section` is not
    permission to create or recolor the theme. Consume the existing theme instead of rewriting it.
    Only when the user explicitly requests a theme or color change, delegate palette extraction to
    `dowe-theme` and use grouped `colors:` families with `color`, `text`, and `title` roles. Use
    normalized family-role tokens only in view props.
17. Before finalizing reference-driven work, reread `theme.dowe` and compare it with the version
    inspected before authoring. A layout, page, component, or `Section` task based on an image must
    leave the theme unchanged unless the user explicitly requested theme or color changes; an
    explicit theme task must leave every declared family in grouped `colors:` form with all three
    roles.
18. Generate the smallest valid component declaration. Omit built-in and theme-resolved visual
    defaults instead of serializing them: `Button "Log in"` is preferred to
    `Button variant:"solid" scheme:"primary" size:"md"`, and `Input bind:email label:"Email"`
    is preferred to repeating `variant:"outlined" scheme:"primary"`. Keep a prop only when it is
    a non-default design decision, a reactive binding, required content or accessibility metadata,
    layout or behavior, or the example is explicitly comparing that prop. This rule applies to
    generated source, documentation examples, and reusable view fragments; use `theme.dowe` for
    repeated visual policy rather than copying the same values into every instance. See
    `references/styles.md` for the current default matrix and minimal-prop examples.
    For `Text` and `Title`, use `align:"start"`, `align:"center"`, `align:"end"`, or
    `align:"justify"` for logical text alignment. A scalar typography size such as `size:"lg"`
    is already fluid/responsive; write a responsive size object only when the design intentionally
    changes at named breakpoints, not merely to make the size responsive.
    Keep one semantic text node for intentional line boundaries by using a multiline string child;
    use `maxW` when natural wrapping is acceptable. Do not duplicate `Text` or `Title` nodes or add
    a `Flex` only to force a heading onto multiple lines.
    Apply a strict prop-admission gate before writing any local prop. Keep it only when it is
    required by the component contract or accessibility, owns data or behavior, defines essential
    structure that no default can infer, expresses an explicit non-default choice, or fixes a
    mismatch proven after rendering the default-first tree. Prop availability and a measured value
    in a screenshot are not reasons by themselves. If the reason cannot be stated, omit the prop.
19. Enforce spacing economy before adding container padding. Dowe Views has no margin contract:
    never invent or emit `m`, `mx`, `my`, `mt`, `mr`, `mb`, or `ml`; express separation with the
    parent's `gap`, responsive flow, alignment, sizing, or the real Section owner's padding. Start
    with component defaults: an ordinary `Section` already provides responsive `px`/`py`, and a
    `Card` already provides responsive inner padding. `Grid` and `Flex` default to zero gap, so add
    one `gap` only when their siblings need an explicit nonzero rhythm after the default-first tree
    is rendered; do not pre-encode every measured whitespace value. Never use padding on a Grid or
    Flex merely to separate its children. Add `p`, `px`, `py`, `pt`, or `pb` only when a specific
    user requirement or rendered comparison proves that the default is insufficient, and put the
    smallest override on one real owner instead of stacking equivalent padding on `Section`,
    `Grid`, and `Card`. Treat `Card variant:"ghost" p:0` as invalid wrapper noise when it only
    groups a layout tree; remove it unless the reference shows a real independent surface.
20. Use Signals and View Stores for state, `fn` for event workflows, and one `init` for ordered
    mount-time work.
21. Write static visible text as `"Blog title"` and dynamic visible text as one complete braced
   binding such as `"{blog.title}"`.
22. Keep route groups one level: every `group` contains direct `route` declarations, never another
   `group`.
23. Use `store name:` with one indented prop per line when Store props would make one long line.
24. Validate bindings, component props, text children, routes, and target support with Dowe
   diagnostics. For internal requests, also verify the request-to-route matrix against the actual
   Server source and validate the complete View and Server import graph; do not assume the compiler
   proves a response shape that the source does not declare.
25. Before visual QA, audit every repeated region: name its collection and owner, verify stable ids,
   confirm one `each` wraps the complete repeated subtree, and check that no copied sibling has
   survived. Verify static-only props such as `Icon.name` remain compiler-valid. Audit every local
   prop against the admission gate and remove any prop whose only rationale is that it is accepted,
   commonly generated, or numerically measurable in the reference.
26. Review the rendered page at `xs`, `md`, and the reference viewport. Audit focal hierarchy,
   section-to-section rhythm, visible layering, Card variety, text measure, asset quality, and
   interaction states before accepting a technically valid layout. For split layouts, compare the
   form centerline with the centerline of its owning panel, not the whole viewport, and verify that
   nested action columns fit the available panel width at every active breakpoint.
27. For reference-driven work, run the installed `scripts/visual_qa.py` entrypoint at the exact
   viewport. Inspect its band report and diff, then iterate on geometry, line wrapping, spacing,
    density, states, layers, and assets before finishing. For directed adaptations, use the report
    to inspect retained bands and document intentional structural deviations instead of weakening
    thresholds or claiming pixel parity. It imports `scripts/visual_qa_blueprint.py` and
    `scripts/visual_qa_png.py`; do not run the helpers directly.

## Table authoring

When a view needs a data-heavy surface, read `references/table.md` before writing the source. Treat
`Table` as a semantic, portable scalar-data component and compose advanced product behavior around
it. Do not invent sorting, pagination, selection, search, toolbar, or custom-cell props on `Table`.

- Use `Table` for typed rows whose cells resolve to strings, numbers, booleans, or scalar relative
  field paths. Give it one or more direct `column` entries with quoted `field` and `label` props.
- Use a typed page `signal` for data loaded, filtered, paged, or replaced by a request. Use a
  `const` only for immutable table data when the compiler accepts the array path.
- Build the enhanced table experience with surrounding `Flex`/`Grid` composition: `Input` and a
  labeled `IconButton` for search, `Skeleton` for loading, `Alert` for errors, `Pagination` for
  server-backed pages, and an explicit empty state on the Table or adjacent action.
- Keep table copy, status values, dates, and amounts concise and directly readable. Align numeric,
  date, and amount columns with `align:"end"`; use intentional `width` hints when columns need
  stable proportions. Let narrow viewports scroll horizontally instead of clipping or duplicating
  the table.
- If a row needs avatars, chips, nested controls, per-row actions, or breakpoint-specific field
  visibility, follow the enhanced directory pattern in `references/table.md`: render one keyed
  responsive `Grid` row with explicit headers instead of forcing rich cells into `Table`.
- Always handle loading, loaded rows, errors, and a genuinely empty result as separate visual
  states. Do not let a temporary empty array flash the empty copy while the first request is in
  flight, and keep the next action visible when no records exist.
- Prefer the documented neutral `soft`/`surface` recipe for dashboards, `outlined` for dense
  ledgers, `sm` for many columns, and `md` for normal reading density. Add `striped`, `dividers`,
  `bordered`, and `rounded` only when they improve scanning or boundary clarity; use `scheme`, not
  `color`, for the Table family.

## Resource routing

Open the primary resource first. Load another only when the task crosses its contract.

| Task | Primary resource |
| --- | --- |
| Routes, layouts, pages, state, functions, requests, repeated views, or i18n | `references/views.md` |
| Exact screenshot, mockup, or UI-reference reconstruction | `references/reference-ui.md` |
| Dowe documentation block patterns and variant selection | `references/blocks/index.json` |
| New screen, shell ownership, reusable fragments, container choice, hero, or landing composition | `references/composition.md` |
| Built-in component selection, children, bindings, interaction, or portability | `references/components.md` |
| Tables, data grids, loading/empty/error states, toolbars, search, pagination, and responsive table UX | `references/table.md` |
| Colors, variants, responsive props, typography, sizing, visibility, overlay, or motion | `references/styles.md` |
| Canvas drawing, input, animation, dynamic scenes, or limits | `references/canvas.md` |
| Deterministic reference capture and comparison | `scripts/visual_qa.py`, `scripts/visual_qa_blueprint.py`, `scripts/visual_qa_png.py` |
