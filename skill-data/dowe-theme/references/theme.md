# `theme.dowe` reference

`theme.dowe` contains exactly one `theme` block. It accepts `fonts` and `design`. Application name
and bundle metadata belong in `main.dowe`, not in `theme.dowe`.

```text
theme
  fonts default:"manrope" install:["manrope","inter"]
  design defaultTheme:"light"
    Card variant:"outlined" scheme:"primary" radius:"xs" shadow:"xs"
    Button variant:"solid" scheme:"secondary" size:"md"
    Avatar radius:"full" size:"md"
    Chip variant:"soft" scheme:"secondary" radius:"full" size:"sm"
    Text font:"manrope"
    Title font:"syne"
    theme name:"light"
      colors primary:"#0a0d12" onPrimary:"#f8f8f6" softPrimary:"#e7e9ec" onSoftPrimary:"#0a0d12"
      colors secondary:"#050607" onSecondary:"#f7f7f4" softSecondary:"#171a1f" onSoftSecondary:"#f7f7f4"
      colors tertiary:"#1f3a5f" onTertiary:"#f7f7f4" softTertiary:"#dce7f5" onSoftTertiary:"#0b1624"
      colors muted:"#6f716f" onMuted:"#f7f7f4" softMuted:"#eeeeec" onSoftMuted:"#252525"
      colors background:"#f7f7f5" onBackground:"#0d0f12"
      colors surface:"#ffffff" onSurface:"#101114"
      colors success:"#2f6f4f" onSuccess:"#f7f7f4" softSuccess:"#dfeee5" onSoftSuccess:"#102519"
      colors info:"#385b87" onInfo:"#f7f7f4" softInfo:"#e1eaf4" onSoftInfo:"#102033"
      colors warning:"#7a6534" onWarning:"#f7f7f4" softWarning:"#f1ead9" onSoftWarning:"#2b230f"
      colors danger:"#8a2f35" onDanger:"#f7f7f4" softDanger:"#f2dddd" onSoftDanger:"#361216"
    theme name:"dark" extends:"light"
      colors background:"#050607" onBackground:"#f7f7f4"
      colors surface:"#101114" onSurface:"#f7f7f4"
      colors muted:"#8a8d93" onMuted:"#050607" softMuted:"#181a1e" onSoftMuted:"#f7f7f4"
```

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

## Component defaults

The default slots are `Card`, `Button`, `Avatar`, `Chip`, `Ui`, `Text`, and `Title`. `Ui` sets
shared defaults for the interface controls and surfaces that accept the common visual props but
have no dedicated slot.

| Prop | Accepted values |
| --- | --- |
| `variant` | `solid`, `soft`, `outlined`, `ghost`; the normalized model uses `outlined` |
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
