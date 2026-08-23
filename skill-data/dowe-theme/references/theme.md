# `theme.dowe` reference

`theme.dowe` contains exactly one `theme` block. It accepts `fonts` and `design`. Application name
and bundle metadata belong in `main.dowe`, not in `theme.dowe`.

Color families use the grouped-role syntax shown below. The family name is the semantic fill token,
`text` is the ordinary content and control-label role, and `title` is the heading role. This is the
only theme color form to author or emit.

```text
theme
  fonts default:"manrope" install:["manrope" "inter"]
  design defaultTheme:"light"
    Card variant:"outlined" scheme:"primary" radius:"xs" shadow:"xs"
    Button variant:"solid" scheme:"secondary" size:"md"
    Avatar radius:"full" size:"md"
    Chip variant:"soft" scheme:"secondary" radius:"full" size:"sm"
    Text font:"manrope"
    Title font:"syne"
    theme name:"light"
      colors:
        primary color:"#0a0d12" text:"#f8f8f6" title:"#f8f8f6"
        softPrimary color:"#e7e9ec" text:"#0a0d12" title:"#0a0d12"
        secondary color:"#050607" text:"#f7f7f4" title:"#f7f7f4"
        softSecondary color:"#171a1f" text:"#f7f7f4" title:"#f7f7f4"
        accent color:"#1f3a5f" text:"#f7f7f4" title:"#f7f7f4"
        softAccent color:"#dce7f5" text:"#0b1624" title:"#0b1624"
        muted color:"#6f716f" text:"#f7f7f4" title:"#f7f7f4"
        softMuted color:"#eeeeec" text:"#252525" title:"#252525"
        background color:"#f7f7f5" text:"#0d0f12" title:"#0d0f12"
        surface color:"#ffffff" text:"#101114" title:"#101114"
        success color:"#2f6f4f" text:"#f7f7f4" title:"#f7f7f4"
        softSuccess color:"#dfeee5" text:"#102519" title:"#102519"
        info color:"#385b87" text:"#f7f7f4" title:"#f7f7f4"
        softInfo color:"#e1eaf4" text:"#102033" title:"#102033"
        warning color:"#7a6534" text:"#f7f7f4" title:"#f7f7f4"
        softWarning color:"#f1ead9" text:"#2b230f" title:"#2b230f"
        danger color:"#8a2f35" text:"#f7f7f4" title:"#f7f7f4"
        softDanger color:"#f2dddd" text:"#361216" title:"#361216"
    theme name:"dark" extends:"light"
      colors:
        background color:"#050607" text:"#f7f7f4" title:"#f7f7f4"
        surface color:"#101114" text:"#f7f7f4" title:"#f7f7f4"
        muted color:"#8a8d93" text:"#050607" title:"#050607"
        softMuted color:"#181a1e" text:"#f7f7f4" title:"#f7f7f4"
```

## Custom color families

Add project semantic families directly under `colors:` using lower camel case. Each base family
defines a complete fill, text, and title triple and can then be used as a component scheme.

```text
colors:
  happy color:"#176c75" text:"#fffffe" title:"#fffffe"
  sad color:"#394867" text:"#fffffe" title:"#fffffe"
  softHappy color:"#d9f3f1" text:"#124d53" title:"#124d53"
```

```text
Card scheme:"happy"
  Title "Saved"
  Text "Your changes are ready."
```

The base family creates `happy`, `happyText`, and `happyTitle`. `softHappy` is a separate optional
triple used by `variant:"soft" scheme:"happy"`; Dowe never derives it from `happy`. A referenced
custom family must be complete in the resolved default theme. Alternate themes may override its
roles and otherwise retain the default theme values. Names contain at most 48 ASCII letters or
numbers, start with a lowercase letter, and use lower camel case. Do not declare normalized role
names such as `happyText` or `happyTitle` as families.
The structural names `theme`, `design`, `fonts`, `colors`, `color`, `text`, and `title` are reserved.

## Fonts

`fonts.default` is one quoted font token. `fonts.install` is an ordered unique array. Supported
tokens are `system`, `inter`, `roboto`, `montserrat`, `lato`, `poppins`, `manrope`, `quicksand`,
`lora`, `syne`, `jost`, and `puritan`.

The catalog is closed: there are no custom fonts, font files, Google Fonts imports, or font URLs.
When a reference design uses a family outside the catalog, select the token whose typographic
character is closest instead of trying to import the original; never invent a token name.

| Reference character | Closest tokens |
| --- | --- |
| Native platform look, OS-consistent UI | `system` |
| Neutral grotesque for interfaces, dashboards, and data | `inter`, `roboto` |
| Modern semi-geometric sans for product and marketing pages | `manrope` |
| Geometric rounded sans with a friendly voice | `poppins`, `quicksand` |
| Geometric display sans for headlines and branding | `montserrat`, `jost` |
| Warm humanist sans for readable body copy | `lato`, `puritan` |
| Editorial serif for longform and literary tone | `lora` |
| Wide display face with strong personality for hero statements | `syne` |

Pair at most two families per project: one for `Title` and one for `Text`, configured once in
`design`. A single family for both is a valid, quieter default.

## Reference-system extraction

Treat a screenshot or mockup as evidence of relationships, not as a bag of unrelated pixel values.
Inventory repeated choices before editing `theme.dowe`. An image supplied for a layout, page,
reusable component, or `Section` is not by itself a request to create or change a theme. If the file
already exists, consume its semantic tokens without changing its palette; if it does not exist, do
not create one solely from the image. Re-extract or generate colors only when the user explicitly
asks for a theme or visual-system change.

| Reference evidence | Theme decision |
| --- | --- |
| Page canvas, body copy, and headings | `background color:… text:… title:…` |
| Cards, bars, menus, and raised panels | `surface color:… text:… title:…` |
| Brand and primary action family | Complete `primary` and `softPrimary` fill, text, and title triples |
| Supporting accent family | `secondary` or `accent` complete family |
| Secondary copy and quiet fills | Base `muted` on `primary` as a lighter, lower-emphasis tonal counterpart; choose `mutedText` and `mutedTitle` for clear contrast, then complete the `softMuted` triple when soft variants are needed |
| Repeated success, information, warning, or error meaning | Matching semantic status family |
| Repeated Card, Button, Avatar, or Chip treatment | Supported dedicated `design` slot |
| Repeated control or surface treatment without a dedicated slot | Supported `Ui` defaults |
| Heading and body character | Closest supported `Title` and `Text` font tokens |

- Sample flat interior areas rather than anti-aliased edges, text fringes, blur transitions, or
  compressed-image noise.
- Consolidate visually equivalent samples into the smallest semantic palette that preserves the
  reference hierarchy.
- Validate both content props against their family fill. `text` serves ordinary content and
  control labels; `title` serves `Title` and semantic component headers.
- Preserve brand meaning across light and inherited dark themes; an inherited theme overrides only
  the roles whose relationship actually changes.
- Move a treatment to `design` only when it repeats. Keep a one-off hero or campaign exception on
  the owning Dowe component.
- Use only the supported default slots and props below. A visual pattern does not authorize an
  invented theme slot.

## Tonal architecture for polished interfaces

A modern palette needs hierarchy, not a long list of colors. Establish these relationships before
styling individual components:

| Visual layer | Semantic destination | Relationship |
| --- | --- | --- |
| Canvas | `background` family | Quietest broad field with readable body copy and headings |
| Primary surface | `surface` family | Clearly separable from the canvas without requiring a border everywhere |
| Quiet panel or divider field | `muted` or `softMuted` fill, text, and title triple | `muted` is a lighter tonal counterpart of `primary`; use it when a solid primary surface is too heavy, including solid form controls such as `Input` |
| Brand emphasis | `primary` family | Saturated accent for actions, values, focal labels, and occasional glow—not every Card |
| Supporting visual accent | `secondary` or `accent` family | Complements the brand and distinguishes charts, data, or a second product concept |

In a dark theme, test broad adjacent fills at page scale. Values that differ only in name but look
identical on a large monitor do not create depth. Conversely, do not brighten every Card until the
page becomes a checkerboard. Use one canvas, one principal surface, one quiet field, and small
saturated accents.

Theme defaults should produce a calm baseline. A global `Card` default may set radius and a quiet
variant, but strong colored borders, `xl` shadows, or brand-colored surfaces usually belong to the
few focal Cards that need them. A global effect applied everywhere ceases to communicate priority.

Typography participates in the visual system. Select a display family with the reference's
character, then let size, measure, and layout create hierarchy in views. Do not compensate for weak
composition by adding more font families or using the maximum Title size in every section.

Every action and status family is a child of `colors:` with `color`, `text`, and `title` props, plus
the corresponding grouped soft family. `muted` should be authored as a lighter tonal counterpart
of `primary`, not as an unrelated neutral; its `text` and `title` roles must remain clearly legible
against the lighter fill. This gives solid controls such as `Input` a quieter alternative to a
heavy primary surface. `background` and `surface` are structural triples.
A filled component supplies its resolved text role to normal descendants and its title role to
`Title`. Buttons use the text role for their label. The transparent `SideNav` header is an
exception: it uses the visible base color of its `scheme` so its content remains readable. An
explicit descendant `color` remains the local override.

## Component defaults

Dedicated slots are available for `Button`, `IconButton`, `Card`, `Drawer`, `Toast`, `Section`,
`Accordion`, `Checkbox`, `Input`, `Date`, `Password`, `Select`, `Pin`, `AppBar`, `Footer`, `Modal`,
`Dropdown`, `Tooltip`, and `Tabs`. `Card`, `Button`, `Avatar`, `Chip`, and `Ui` remain supported for
existing shared visual policy, while `Text` and `Title` remain independent typography slots.

When no `design` entry configures a component, use these built-in defaults:

| Component | Built-in defaults |
| --- | --- |
| `Button`, `IconButton` | `scheme:"primary" variant:"solid" rounded:"md"` |
| `Card` | `scheme:"surface" variant:"solid" rounded:"md"` |
| `Drawer` | `scheme:"surface" variant:"solid"` |
| `Toast` | `scheme:"info" variant:"solid" rounded:"md"` |
| `Section` | `scheme:"background" variant:"solid"` |
| `Accordion` | `variant:"ghost"` |
| `Checkbox` | `scheme:"primary"` |
| `Input`, `Date`, `Password`, `Select`, `Pin` | `variant:"outlined" scheme:"primary"` |
| `AppBar`, `Footer`, `Modal`, `Dropdown`, `Tooltip` | `scheme:"surface" variant:"solid"` |
| `Tabs` | `variant:"pills" scheme:"primary"` |

These built-in defaults add no `border` or `shadow`. Resolve each property independently in this
order: explicit component prop, matching `design` slot, then the built-in component default. This
contract is defined by `/specs/features/00149-normalize-view-component-defaults`.

The `Toast` `variant` default also applies to the lowercase global `toast` statement used inside a
View `fn` or `init`, even though that statement does not create a `Toast` node in the visual tree.
Use `Toast variant:"soft"` for a project-wide global feedback surface; an explicit `variant` on a
statement still wins, and the built-in fallback is `solid`.

Code generation should omit resolved defaults from View source. Prefer `Button "Log in"` or
`Button w:"full" "Log in"` over repeating `variant:"solid" scheme:"primary" size:"md"`, and
prefer `Input bind:email label:"Email"` over repeating `variant:"outlined" scheme:"primary"`.
Keep local props only for non-default design decisions, reactive values, required content or
accessibility, layout, and behavior. Configure repeated visual policy here instead of copying the
same props into every page, layout, or component.

| Prop | Accepted values |
| --- | --- |
| `variant` | `solid`, `soft`, `outlined`, `ghost`; `Tabs` additionally accepts `line` and `pills` |
| `scheme` | Semantic color family |
| `radius` or `rounded` | `xs`, `sm`, `md`, `lg`, `xl`, `full` |
| `size` | `xs`, `sm`, `md`, `lg`, `xl` |
| `shadow` | `xs`, `sm`, `md`, `lg`, `xl` |
| `shadowColor` | Semantic color family |
| `border` | Integer from `1` to `4` |
| `borderColor` | Semantic color family |
| `font` | Dowe font token; only for `Text` and `Title` |

Each slot accepts only props applicable to that component. Compiler diagnostics identify invalid
slot and prop combinations. An explicit prop on one component instance overrides the theme default;
all omitted visual props continue to use project or built-in defaults.

`Text` and `Title` accept only `font` in `design`. Each uses one quoted token from the font catalog.
The slots are independent, and their configured families are included in generated assets. A local
`font` prop on a page, layout, or reusable component instance overrides the matching theme default.

## Named themes

`design defaultTheme:"name"` must match a declared theme. A base theme defines the complete semantic
color set. A later `theme name:"dark" extends:"light"` can override only the roles that differ.
Inheritance cycles and incomplete resolved themes are invalid.

Use semantic roles instead of literal colors in pages and layouts. This lets the same source render
consistently on web, desktop, Android, and iOS.
