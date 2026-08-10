# Reference-driven UI reconstruction

Use this reference whenever a screenshot, mockup, annotated image, or design capture defines the
requested UI. Treat the image as a visual contract and keep it as validation evidence, never as
application source.

## Contents

- Evidence, fidelity, and the required blueprint
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
| Primary or secondary action | `Button`, or `IconButton` for an icon-only action |
| Major routed content band | Page `Section` |
| Split hero or repeated tracks | `Grid` |
| One-axis copy, metric, or action group | `Flex` |
| Compact label or status | `Chip` or `Badge` according to its behavior |
| Related bordered, raised, or tinted unit | `Card` |

Rebuild navigation, headings, text, controls, cards, lists, metrics, forms, tables, charts,
dashboards, badges, icons, logos, and decorative geometry with Dowe components. Use `Canvas` only
for a portable drawing that semantic components and charts cannot express.

## Ownership and data

Anything visible across every route in a group belongs to the layout. Routed content belongs to
page Sections. Use `views/components` only for a caller-independent static tree reused in multiple
places.

Two or more same-shape units use one collection and one `each` template:

| Collection behavior | Data owner |
| --- | --- |
| Fixed records visible in the reference | Page or layout `const` |
| Records loaded, filtered, paged, appended, or replaced by one screen workflow | Typed page or layout `signal` |
| Reactive state consumed by multiple routes | Imported View Store |

Give every record a stable string `id`. Never copy one Card per visible record. Read
`references/composition.md` for the complete layout, component-reuse, collection, and container
contracts.

## Responsive inference

A reference proves only the viewport it shows. Mark behavior at that viewport as `observed` and
every unshown breakpoint decision as `inferred`.

- Match the reference viewport exactly before optimizing other sizes.
- Validate `xs`, `md`, and the reference viewport.
- Preserve source reading order when Grid tracks collapse or Flex changes direction.
- Convert wide navigation to a supported Drawer, SideNav, or BottomBar pattern when space requires
  it; keep the same destinations and labels.
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

Inventory the reference's repeated visual decisions before adding local props:

| Visual system | Destination |
| --- | --- |
| Color families and contrast pairs | Named semantic colors in root `theme.dowe` |
| Typographic character | Closest supported Dowe font token and Text/Title defaults |
| Repeated Card, Button, Avatar, Chip, or control treatment | `design` component defaults |
| Section rhythm and track gaps | Small consistent Dowe spacing ladder |
| Repeated radii and shadows | Theme/component defaults where supported |

Use `dowe-theme` when authoring these defaults. Keep local visual props only for intentional
exceptions. Do not reproduce the reference with unrelated literal colors or one-off styling on
every instance.

## Media provenance

Use `Image` only for an independently obtained original photograph, illustration, texture, or an
authentic screenshot explicitly supplied or requested by the user. Never use the reference, a
crop, slice, rasterization, or recomposition derived from it as a project asset.

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
2. Create or reuse the route graph and one layout Scaffold for shared chrome.
3. Author ordered page Sections and choose Grid/Flex/Card/Box through
   `references/composition.md`.
4. Declare collections and state before the visual tree.
5. Apply theme defaults, then intentional local exceptions.
6. Compile and repair from Dowe diagnostics.
7. Render the exact reference viewport, compare, and fix the largest band mismatch first.
8. Validate `xs`, `md`, interaction states, accessibility, and missing assets.

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
- Exact built-in components are used; no annotated or guessed component name becomes source.
- Shared chrome is layout-owned; page content is Section-owned.
- Repeated records use one collection and one `each` template.
- Responsive behavior distinguishes observed evidence from conservative inference.
- Loading, empty, error, disabled, overlay, and feedback states are handled where real behavior
  requires them.
- Labels, alternative text, source order, contrast, keyboard/touch controls, and reduced motion are
  reviewed.
- Repeated visual identity is centralized in `theme.dowe`.
- Static fragments are extracted; unsupported dynamic reuse is recorded without invented syntax.
- Reference pixels never enter project assets; missing originals remain explicit.
- The reference viewport passes band-by-band QA or the remaining evidence-backed blocker is
  reported precisely.
