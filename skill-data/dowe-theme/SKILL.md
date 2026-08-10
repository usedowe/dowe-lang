---
name: dowe-theme
description: Use only for root theme.dowe, semantic colors, theme inheritance, fonts, project-wide component defaults, or extracting a repeated visual system from a screenshot or UI reference; skip for one-off local view styling.
---

# Dowe theme authoring

Keep theme behavior in root `theme.dowe` and use semantic Dowe tokens from views. Preserve cross-platform meaning instead of targeting CSS-only behavior.

## Workflow

1. Inspect `theme.dowe` before changing component visual props in views.
2. Put repeated Card, Button, Avatar, Chip, control, Text font, and Title font defaults under `design`.
3. For reference-driven work, inventory recurring color families, foreground/background pairs,
   typography, radii, borders, and shadows before choosing tokens or defaults. Do not turn every
   sampled or anti-aliased shade into a token.
4. Use semantic colors and complete the base theme before adding inherited themes.
5. Keep local component visual props only when one instance intentionally differs.
6. Validate contrast, completeness, font tokens, and target support.
7. Use Dowe semantic names and the closed Dowe font token catalog, matched by typographic
   character when the reference family is unavailable.

## Reference routing

Read `references/theme.md` only when the task changes `theme.dowe`, semantic colors, inheritance,
fonts, component-default precedence, or the repeated visual system extracted from a UI reference.
