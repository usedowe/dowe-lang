---
name: dowe-theme
description: Use only for root theme.dowe, semantic colors, tonal surface hierarchy, theme inheritance, fonts, project-wide component defaults, or extracting a repeated visual system for a modern interface from a screenshot, template, or UI reference; skip for one-off local view styling.
---

# Dowe theme authoring

Keep theme behavior in root `theme.dowe` and use semantic Dowe tokens from views. Preserve cross-platform meaning instead of targeting CSS-only behavior.

Theme colors use grouped family roles only. Every declared family is written under `colors:` with
`color`, `text`, and `title`. This grouped form is the only theme color syntax to author or emit.

The following excerpt shows the family shape; a named theme must still declare the complete
required semantic color set described in `references/theme.md`.

```text
theme
  design defaultTheme:"dark"
    theme name:"dark"
      colors:
        primary color:"#D6F966" text:"#08101A" title:"#08101A"
        background color:"#010314" text:"#FFFFFF" title:"#FFFFFF"
        surface color:"#063453" text:"#FFFFFF" title:"#FFFFFF"
```

## Theme/page contract

- Extract or change the palette only when the user asks for a theme or visual-system change.
- A reference image supplied for a layout, page, reusable component, or `Section` does not by
  itself authorize creating `theme.dowe` or changing its colors. Use the image for structural and
  visual evidence while preserving the existing theme; generate or recolor the theme only after an
  explicit user request for theme or color changes.
- When `theme.dowe` already exists, treat its colors as the source of truth for page generation.
  Do not replace, flatten, or re-sample that palette while authoring a view from an image.
- A page consumes semantic family tokens from the existing theme. It does not recreate the theme
  from local component props or literal color values.
- When a theme must be created or explicitly changed, write one complete grouped `colors:` block
  with `color`, `text`, and `title` for every declared family.
- Before finishing, reread `theme.dowe`. If the request was page-only, its theme content must be
  unchanged; if the request included theme changes, every family row must still contain all three
  grouped roles.

## Workflow

1. Inspect `theme.dowe` before changing component visual props in views. Preserve its palette for
   page-only work.
2. Put repeated Card, Button, Avatar, Chip, control, Text font, and Title font defaults under `design`.
3. For reference-driven work, inventory recurring color families, foreground/background pairs,
   typography, radii, borders, shadows, glow usage, and surface hierarchy before choosing tokens or
   defaults. Do not turn every sampled or anti-aliased shade into a token.
4. Use semantic colors and complete the base theme before adding inherited themes. Do not recreate
   an existing base theme during page generation.
5. Give modern dark interfaces distinct canvas, surface, quiet-surface, and accent roles. Do not
   assign nearly identical dark values to every family or make every Card use the brand color.
6. Base `muted` on the project's `primary` color as a lighter, lower-emphasis tonal counterpart,
   similar to an inverse relationship without mechanically inverting channels. Choose `mutedText`
   and `mutedTitle` to stand out clearly against that lighter muted fill. This is useful for solid
   controls such as `Input` where a solid primary surface would be too heavy; use muted to preserve
   hierarchy while reducing visual weight.
7. Choose defaults that establish a quiet baseline. Reserve stronger borders, shadows, glows,
   covers, transforms, and motion for intentional focal instances in views.
8. Keep local component visual props only when one instance intentionally differs.
9. Treat `design` as the source of repeated visual policy. View generation must omit component
   props already supplied by `design` or by the built-in component contract; emit only local
   exceptions, reactive bindings, layout/behavior props, and required content or accessibility.
10. Validate contrast, completeness, font tokens, surface separation, and target support.
11. Use Dowe semantic names and the closed Dowe font token catalog, matched by typographic
   character when the reference family is unavailable.

## Reference routing

Read `references/theme.md` only when the task changes `theme.dowe`, semantic colors, inheritance,
fonts, component-default precedence, or the repeated visual system extracted from a UI reference.
