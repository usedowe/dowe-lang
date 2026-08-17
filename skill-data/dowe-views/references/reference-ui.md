# Reference-driven UI reconstruction

Use this reference whenever a screenshot, mockup, annotated image, or design capture defines the
requested UI. Treat the image as a visual contract and keep it as validation evidence, never as
application source.

## Contents

- Evidence, fidelity, and the required blueprint
- Exact reconstruction versus directed adaptation
- Visual-detail fidelity and anti-genericization
- Semantic component, ownership, collection, and media decisions
- Responsive inference, interaction states, accessibility, and theme extraction
- Static reuse and future dynamic-component candidates
- Implementation order and deterministic visual QA
- Completion checklist

## Evidence and fidelity

Record the reference viewport and inventory the complete visible surface before editing source:
shell chrome, ordered bands, exact copy, actions, navigation labels, media, decorative layers,
repeated units, dominant proportions, and alignment rails.

Preserve wording, capitalization, order, item count, density, hierarchy, and media intent. Do not
replace copy, invent actions, collapse bands, omit content, or substitute an easier generic layout.
Distinguish full-bleed backgrounds from centered bodies and compare every band, not only the first
viewport or hero.

The reference and every crop derived from it remain outside project assets. Use pixels only for
measurement and comparison.

## Exact reconstruction versus directed adaptation

Classify the request before implementation.

| Mode | Content contract | Visual contract | Validation |
| --- | --- | --- | --- |
| Exact reconstruction | Preserve every visible band, label, item, action, and asset role | Match measured geometry, density, layering, typography, surfaces, and detail | Treat every band mismatch as a defect unless an original asset is unavailable |
| Directed adaptation | Change copy, brand, or band count only where the user requests it | Preserve the reference's design grammar, focal hierarchy, depth, visual density, characteristic motifs, and quality in every retained band | Inspect retained bands against the reference and record intentional structural deviations; never claim pixel parity |

“Inspired by,” “same UI/UX,” “use only the necessary sections,” and “adapt this template” normally
mean directed adaptation. Content reduction does not authorize design reduction. If three of eight
bands remain, those three still need the reference's compositional depth and level of finish.

## Visual-detail fidelity

Inventory the complete visual stack, not only semantic content. A sophisticated reference often
contains five concurrent layers:

1. A full-bleed foundation: color field, gradient-like preset, texture, photograph, or space.
2. A focal composition: product media, dashboard, illustration, chart, device, or dimensional mark.
3. Supporting proof: metrics, badges, prices, ratings, customer marks, or a compact data card.
4. Structural detail: dividers, outlined frames, numbered labels, connecting lines, orbit nodes,
   maps, grids, or a foreground edge.
5. Controlled motion or interaction: entrance, hover lift, carousel, marquee, accordion, or active
   navigation when supported by the evidence.

Record each visible layer as its own blueprint region when it has independent geometry or meaning.
Do not flatten a layered hero to one rectangular Image, turn a product constellation into six plain
text rows, or replace chart-and-map evidence with a paragraph because the semantic content is
technically present.

For every retained marketing band, identify one primary visual payload beyond the heading and body
copy. Valid payloads include original media, product UI, a chart, metric composition, icon system,
logo field, testimonial, process visual, or a deliberate layered scene. Compact legal, navigation,
and FAQ bands may remain text-led when the reference does.

Reject these generic substitutions during review:

- Repeating the same centered eyebrow, large title, sentence, and equal Card grid for consecutive
  sections.
- Giving every Card the same border, background, padding, icon position, and height regardless of
  its role.
- Using whitespace as the only separation between bands that visibly use tonal fields, covers,
  maps, patterns, or media.
- Treating decorative geometry as optional when it creates the reference's silhouette or depth.
- Omitting small proof, labels, dividers, counts, or visual anchors that make the interface feel
  credible and intentional.

## Required blueprint

Create `.dowe/visual-qa/<screen>/blueprint.json` before application source. This is generated QA
evidence, not maintained source. Initialize it from the reference PNG:

```text
python3 .agents/skills/dowe-views/scripts/visual_qa.py init \
  --reference <reference.png> \
  --output .dowe/visual-qa/<screen>/blueprint.json
```

Complete the blueprint before implementation. Record every visible region and band.

| Field | Required decision |
| --- | --- |
| `viewport` | Exact reference width and height |
| `bands` | Ordered contiguous vertical bands covering the full capture |
| `regions[].band` | Declared band containing the region |
| `regions[].bounds` | Measured `x`, `y`, `width`, and `height` inside the viewport |
| `regions[].owner` | `layout`, `page`, or reusable static `component` |
| `regions[].component` | Exact implemented Dowe component |
| `regions[].container` | Scaffold region or Section/Grid/Flex/Card/Box structure |
| `regions[].dataOwner` | `none`, `const`, `signal`, or `store` |
| `regions[].responsive` | `evidence` as `observed` or `inferred`, plus exact reference, `xs`, and `md` rules |
| `regions[].states` | Visible and required supported states |
| `regions[].accessibility` | Labels, alternative text, reading order, and control semantics |
| `theme` | Colors, typography, spacing, radii, and shadows to centralize |
| `assets` | Final path, availability, and independent provenance |
| `candidateComponents` | Static extraction or unsupported dynamic reuse decision |

Add separate `regions` for meaningful decorative layers, floating proof, charts, logo fields, and
foreground media. Use the `container` and `component` fields to describe the actual Dowe layer
plane, such as `Box position:relative > Box position:absolute > Card`, instead of hiding the
composition inside a broad “hero media” region.

Begin the composition map with no Box regions. Add one only when the reference proves that a child
must leave normal flow, such as an absolute ornament or proof surface on a relative stage, or a
fixed route-viewport layer. A need for padding, background, border, width, centering, visibility,
or an empty Grid offset is not evidence for Box; assign those props to the Section, Grid, Flex,
Card, media, control, or content owner. Record the layer responsibility in `regions[].container`.

Begin with no `translateX` or `translateY` props as well. Measured `x` and `y` bounds are QA
evidence, not instructions to nudge every node into place. Reproduce ordinary bounds with AppBar
regions and Section rails; use Flex `direction`, `justify`, `align`, `gap`, and `wrap` for one-axis
flow, and Grid `columns`, `rows`, `justify`, `align`, and `gap` for shared tracks. Add padding,
responsive direction, and `w` or `maxW` on those real owners. Add translation only when the
blueprint identifies an advanced decorative or floating layer whose intentional overlap cannot be
expressed by Flex, Grid, normal flow, and absolute Box offsets.
Never translate `AppBar`, `Brand`, `NavMenu`, `Drawer`, or another compound overlay-owning root;
keep those semantic anchors stable and translate only a purely visual layer when justified.

Before source authoring, reduce every blueprint region to the smallest semantic component tree.
Treat `bounds`, typography measurements, whitespace, radius, border, shadow, and color observations
as comparison evidence, not as an automatic prop list. Admit a local prop only when it is required
by the component contract, content, behavior, binding, or accessibility; defines essential structure
that a default cannot infer; expresses an explicit non-default choice; or fixes a specific mismatch
proven after rendering the default-first tree. If none applies, omit the prop.

A completed blueprint has this shape:

```json
{
  "viewport": { "width": 1440, "height": 900 },
  "bands": [
    { "id": "shell", "top": 0, "bottom": 96 },
    { "id": "hero", "top": 96, "bottom": 900 }
  ],
  "regions": [
    {
      "id": "header",
      "band": "shell",
      "bounds": { "x": 0, "y": 0, "width": 1440, "height": 96 },
      "owner": "layout",
      "component": "AppBar",
      "container": "Scaffold.appBar",
      "dataOwner": "none",
      "responsive": {
        "evidence": "observed",
        "rules": [
          "reference: horizontal navigation and primary action",
          "xs: named menu action opens Drawer with the same destinations",
          "md: horizontal navigation and primary action remain visible"
        ]
      },
      "states": ["default"],
      "accessibility": ["brand label", "navigation labels", "named primary action"]
    }
  ],
  "theme": {
    "colors": ["background", "surface", "primary"],
    "typography": ["display title", "compact navigation"],
    "spacing": ["boxed rail", "section rhythm"],
    "radii": ["rounded action"],
    "shadows": ["hero media depth"]
  },
  "assets": [
    {
      "path": "/assets/images/hero-original.png",
      "status": "missing",
      "source": "independent-original"
    }
  ],
  "candidateComponents": [
    {
      "name": "MetricCard",
      "kind": "dynamic",
      "action": "future-feature",
      "reason": "The same data-bound unit appears on multiple routes."
    }
  ]
}
```

## Semantic reconstruction

Annotated labels are hints, not component names. Resolve visible behavior against
`references/components.md` before choosing a generic container.

| Visual cue | Dowe owner or component |
| --- | --- |
| Complete routed shell | One layout `Scaffold` |
| Persistent top chrome | `Scaffold appBar` containing `AppBar` |
| Product identity or logo group | `Brand` with `Text`, `Image`, or `Svg` children |
| Horizontal navigation | `NavMenu`; never invent `MenuBar` |
| Vertical navigation in a `Sidebar` or `Drawer` | `SideNav`; never place horizontal `NavMenu` in the side surface |
| Open mobile navigation surface | `Drawer` containing a vertical `SideNav` in `body` |
| Mobile Drawer trigger next to the brand | Direct `IconButton` before `Brand` in AppBar `start`, or direct `IconButton` in `end`; do not wrap both in `Flex` |
| Primary or secondary action | `Button`, or `IconButton` for an icon-only action |
| Major routed content band | Page `Section` |
| Split hero or repeated tracks | `Grid` |
| One-axis copy, metric, or action group | `Flex` |
| Relative stage with overlapping children or a fixed viewport layer | Positioned `Box` wrappers around the real semantic components |
| Compact label or status | `Chip` or `Badge` according to its behavior |
| Related bordered, raised, tinted, or otherwise contained unit | `Card`; a visually flat semantic group remains Grid/Flex content |

Rebuild navigation, headings, text, controls, cards, lists, metrics, forms, tables, charts,
dashboards, badges, icons, logos, and decorative geometry with Dowe components. Use `Canvas` only
for a portable drawing that semantic components and charts cannot express.

## Ownership and data

Anything visible across every route in a group belongs to the layout. Routed content belongs to
page Sections. Use `views/components` only for a caller-independent static tree reused in multiple
places.

Two or more same-shape units use one collection and one `each` template. Count any repeated semantic
tree, including icon-plus-copy feature rows, even when the visible reference contains only two items:

| Collection behavior | Data owner |
| --- | --- |
| Fixed records visible in the reference | Page or layout `const` |
| Records loaded, filtered, paged, appended, or replaced by one screen workflow | Typed page or layout `signal` |
| Reactive state consumed by multiple routes | Imported View Store |

Give every record a stable string `id`. The `each` boundary owns the entire repeated unit, not only
its text or one child. Never copy one Card, Flex row, icon/text group, or list unit per visible record.
Read `references/composition.md` for the complete layout, component-reuse, collection, and container
contracts.

## Responsive inference

A reference proves only the viewport it shows. Mark behavior at that viewport as `observed` and
every unshown breakpoint decision as `inferred`.

- Match the reference viewport exactly before optimizing other sizes.
- Validate `xs`, `md`, and the reference viewport.
- For a split layout, record the bounds of each panel and its bounded content separately. Compare a
  form or focal stack to the centerline of its owning panel, not to the centerline of the viewport.
- Remember that responsive breakpoints use viewport width, not the width of a nested Grid track. A
  two-column action group may therefore activate while its split panel is still too narrow; keep it
  stacked until every complete label fits at the measured panel width.
- Preserve source reading order when Grid tracks collapse or Flex changes direction.
- Keep wide shell navigation as a horizontal `NavMenu` in AppBar `center` or `end`. When space
  requires a mobile surface, open a `Drawer` whose `body` contains a vertical `SideNav`; keep the
  same destinations and labels.
- Keep AppBar sibling controls as direct children of their region; use `Flex` only for a real
  nested axis, wrapping, or independently structured group, never just to align `Logo` and the
  Drawer trigger. Place the mobile trigger before the brand in `start` or directly in `end`.
- Keep the primary promise, content, and action available at every breakpoint.
- Hide only secondary chrome. Never use responsive visibility to discard the only copy or action.
- Infer conservative behavior from Dowe container semantics; do not invent unseen mobile content.

## Interaction and state coverage

A static image does not define business logic. Do not invent endpoints, destructive actions,
authentication, or navigation destinations. Implement only user-requested behavior and supported
Dowe states.

| Surface | Review |
| --- | --- |
| Backend collection | Loading, populated, empty, and error presentation |
| Submit or refresh action | Named function, `Button loading`, duplicate-action prevention, success or error feedback |
| Form | Visible labels, help or error copy when required, disabled/read-only behavior already supported by the chosen control |
| Overlay or responsive navigation | Open, close, dismissal, and focus behavior owned by the built-in component |
| Interactive control | Default, disabled, focus, keyboard, pointer, and reduced-motion behavior provided by its portable contract |

Use `Skeleton` or `Splash` for loading boundaries, `Empty` for an empty collection, `Alert` for
inline status, and lowercase `toast` for action feedback. Do not recreate hover, focus, disabled,
loading, or overlay mechanics with Box and local styling when a built-in component owns them.

## Accessibility review

- Use semantic Dowe components so targets emit their built-in roles and interaction behavior.
- Give `Brand`, icon-only actions, navigation entries, form controls, and media explicit labels.
- Give original informative media meaningful `alt`; use an empty `alt` only for decorative media.
- Keep one clear primary page title and a visible reading hierarchy with `Title` and `Text`.
- Preserve logical source order when responsive layouts reorder visually.
- Do not communicate status only through color; keep visible text, labels, or semantic status
  components.
- Resolve foreground/background contrast through paired semantic theme tokens.
- Keep essential actions reachable by keyboard and touch through supported controls.
- Use Dowe motion and navigation contracts so reduced-motion preferences remain effective.

## Theme extraction

An image supplied to build or adapt a layout, page, reusable component, or `Section` does not by
itself authorize theme work. Use the image to infer composition, hierarchy, typography, spacing,
assets, and layering, but keep the existing theme colors unchanged. Do not create `theme.dowe`,
re-sample its palette, or add literal color overrides unless the user explicitly asks to generate or
change theme colors.

Inventory the reference's repeated visual decisions before adding local props:

| Visual system | Destination |
| --- | --- |
| Color families and contrast pairs | Named semantic colors in root `theme.dowe` |
| Typographic character | Closest supported Dowe font token and Text/Title defaults |
| Repeated Card, Button, Avatar, Chip, or control treatment | `design` component defaults |
| Section rhythm and track gaps | Section defaults first, then a small consistent Dowe spacing ladder |
| Repeated radii and shadows | Theme/component defaults where supported |

Use `dowe-theme` when authoring these defaults only after an explicit theme or color request. If
`theme.dowe` already exists, treat its palette as fixed for reference-driven layout, page,
component, and `Section` work: use its semantic tokens and do not replace it with colors re-sampled
from the image. If no theme exists, do not create one solely because the image has a visible palette.
Keep local visual props only for intentional non-color exceptions. Do not reproduce the reference
with unrelated literal colors or one-off styling on every instance. Before finishing, reread the
theme and verify that reference-driven view work did not alter it; when a theme change was
requested, verify every declared family has grouped `color`, `text`, and `title` roles.

Treat measured whitespace as evidence, not as permission to serialize padding or gap on every
container. Start with Section and Card defaults and render the minimal tree. Grid and Flex default
to zero gap; add one nonzero gap only when the sibling group demonstrably needs it, then add the
smallest local padding override only when the blueprint or user request proves that a default is
insufficient. Dowe Views does not support margin props; reject aliases such as `mt` instead of
fabricating them. A ghost Card with `p:0` is not a layout primitive when it only wraps Grid/Flex
content, and a flat group is not a Card merely because its children belong together.

## Media provenance

Use `Image` only for an independently obtained original photograph, illustration, texture, or an
authentic screenshot explicitly supplied or requested by the user. Never use the reference, a
crop, slice, rasterization, or recomposition derived from it as a project asset.

Classify the media role before choosing the component. When an original asset fills and crops with
a Section, Card, or neutral media panel as background geometry, use that owner's `cover` prop; do
not stretch a child `Image` to simulate the same background. Use `Image` when the original is a
foreground object with independent bounds, aspect behavior, and an `alt` contract. Record the
choice in the blueprint's `container`, `component`, and `assets` fields.

When an original is unavailable, author its final stable asset path, `alt`, aspect, object fit, and
visual weight. Record it as `missing` in the blueprint and report that exact parity remains blocked;
do not substitute unrelated stock media or claim completion.

## Reuse candidates

Extract a static fragment used in multiple places when its complete tree needs no caller data.
Current reusable components accept no props, slots, children, Signals, functions, requests, or
caller bindings.

When the same data-bound pattern appears across pages, record it in `candidateComponents` with
`kind:"dynamic"` and `action:"future-feature"`. Keep the `each` template in each current owner and
share only genuine cross-route state through a Store. Never invent component inputs.

## Implementation order

1. Complete and validate the blueprint.
2. Classify exact reconstruction or directed adaptation and write the visual-direction sentence.
3. Create or reuse the route graph and one layout Scaffold for shared chrome.
4. Author ordered page Sections and choose Grid/Flex/Card/Box through
   `references/composition.md`.
5. Write the minimal component tree and run the local prop-admission gate before detail work.
6. Build the major focal compositions and media stages before filling repeated details.
7. Declare collections and state before their repeated visual trees.
8. Apply theme defaults, then intentional local exceptions and the restrained detail pass.
9. Compile and repair from Dowe diagnostics.
10. Render the exact reference viewport, compare, and fix the largest retained-band mismatch first.
11. Validate `xs`, `md`, interaction states, accessibility, asset quality, and missing assets.

## Deterministic visual QA

The bundled script accepts 8-bit non-interlaced PNG evidence and writes only under `.dowe`.
Run its self-test once when the environment is new:

```text
python3 .agents/skills/dowe-views/scripts/visual_qa.py self-test
```

When local Chrome or Chromium is available, start Dowe, capture the exact viewport, and compare in
one command:

```text
python3 .agents/skills/dowe-views/scripts/visual_qa.py run \
  --project . \
  --reference <reference.png> \
  --blueprint .dowe/visual-qa/<screen>/blueprint.json \
  --output .dowe/visual-qa/<screen> \
  --url http://127.0.0.1:7655/<route>
```

If no supported local browser is available, start the web target, capture the rendered route at the
blueprint viewport with the agent's browser, and compare the two PNG files:

```text
python3 .agents/skills/dowe-views/scripts/visual_qa.py compare \
  --reference <reference.png> \
  --rendered <rendered.png> \
  --blueprint .dowe/visual-qa/<screen>/blueprint.json \
  --output .dowe/visual-qa/<screen>
```

Review `report.json` and `diff.png` band by band. The default gate marks a pixel different when one
RGB channel differs by more than 16 and fails a band when more than 8 percent of its pixels differ.
Fix geometry, wrapping, spacing, density, theme, and assets instead of weakening the threshold to
hide a visible mismatch.

## Completion checklist

- Blueprint covers every visible region and the complete capture height.
- The work is explicitly classified as exact reconstruction or directed adaptation.
- Exact built-in components are used; no annotated or guessed component name becomes source.
- Shared chrome is layout-owned; page content is Section-owned.
- Same-kind nesting remains only for a distinct subgroup and layout responsibility, such as a
  content-column Flex containing an action-row Flex; no wrapper exists only for another gap,
  padding, alignment, visibility, or size.
- Normal layout uses no `translateX` or `translateY`; any exception is a documented advanced visual
  layer, never a compound navigation or overlay root.
- Every repeated same-shape region, including icon-plus-copy rows, uses one collection and one `each`
  template around the complete unit; copied siblings are a structural defect.
- Static-only component props remain compiler-valid inside the loop; do not invent dynamic bindings
  such as `Icon name:<item.icon>`.
- Responsive behavior distinguishes observed evidence from conservative inference.
- Split-panel content is centered against its owning panel, and nested action tracks are checked at
  their actual usable width rather than inferred from the viewport breakpoint alone.
- Loading, empty, error, disabled, overlay, and feedback states are handled where real behavior
  requires them.
- Labels, alternative text, source order, contrast, keyboard/touch controls, and reduced motion are
  reviewed.
- Repeated visual identity is centralized in `theme.dowe`.
- Every substantial retained band has a primary visual payload and preserves the reference's depth.
- Consecutive bands do not collapse into the same generic heading-plus-Card-grid composition.
- Static fragments are extracted; unsupported dynamic reuse is recorded without invented syntax.
- Reference pixels never enter project assets; missing originals remain explicit.
- Component defaults are used before local `p*` props or gaps; no automatic gap on every Grid/Flex,
  stacked Section/Grid/Card padding, or unsupported margin aliases such as `mt` remain.
- Every local prop has a recorded contract, behavior, accessibility, structural, non-default, or
  rendered-mismatch reason; supported-but-unnecessary props are removed.
- Background media uses `cover` on its owner; foreground `Image` keeps an independent media role.
- The reference viewport passes band-by-band QA or the remaining evidence-backed blocker is
  reported precisely.
