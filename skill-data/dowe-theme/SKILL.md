---
name: dowe-theme
description: Author root theme.dowe fonts, named semantic color themes, dark mode inheritance, and project-wide component defaults including Text and Title fonts.
---

# Dowe theme authoring

Keep theme behavior in root `theme.dowe` and use semantic Dowe tokens from views. Preserve cross-platform meaning instead of targeting CSS-only behavior.

## Workflow

1. Inspect `theme.dowe` before changing component visual props in views.
2. Put repeated Card, Button, Avatar, Chip, control, Text font, and Title font defaults under `design`.
3. Use semantic colors and complete the base theme before adding inherited themes.
4. Keep local component visual props only when one instance intentionally differs.
5. Validate contrast, completeness, font tokens, and target support.
6. Avoid Tailwind names, CSS framework configuration, URLs, custom font imports, and browser-only
   assumptions; fonts always come from the closed Dowe token catalog, matched by typographic
   character when the reference family is unavailable.

Read `references/theme.md` for the complete root shape, accepted values, semantic colors, inheritance,
and default precedence.
