---
name: dowe-views
description: Use for Dowe view modules, routes, layouts, pages, UI composition, components, state, requests, responsive styles, reference-driven fidelity, Canvas, and view targets; skip for server-only work.
---

# Dowe views authoring

Dowe views are target-neutral source compiled to web, desktop, Android, and iOS outputs. Reuse one route graph and one source behavior model across targets.
Keep every new frontend module under `views/`; only root `main.dowe` and `theme.dowe` sit outside it.

## Workflow

1. Find the imported `views` binding connected by `main.dowe`, inspect the route graph, and read
   `theme.dowe`.
2. For reference-driven work, inventory every visible shell region, ordered page band, exact text
   block, media asset, dominant proportion, and the reference viewport before authoring source.
3. Rebuild every UI-shaped region with Dowe components. Never use the reference image or crops
   derived from it as assets. Use `Image` only for independently obtained photographs,
   illustrations, textures, or authentic screenshots explicitly supplied or requested by the user.
4. Create or reuse a layout whenever the reference has shared chrome. AppBar and Footer never
   belong in a page, and a one-page site still uses a layout-backed route group.
5. Put exactly one normal `Scaffold` root in every layout; add one direct `Splash` sibling only when startup replacement content is required.
6. Start every page with `Section` and use ordered sibling Sections for major page bands. Give
   every landing-page band one job, and make the hero establish the primary promise, support,
   action, and proof before later Sections add detail. Preserve visible copy, band order, actions,
   density, and media intent instead of inventing generic replacements.
7. Use `Grid` for tracks, `Flex` for one-axis flow, `Card` for grouped content, and `Box` only for
   a special neutral wrapper; when unsure, composing a hero or landing page, or working from a
   reference design, follow the decision tree and dedicated patterns in
   `references/composition.md`.
8. Extract a static fragment reused in two or more places, such as a logo or a navigation tree
   mounted in both Sidebar and Drawer, into a `component` under `views/components`; keep signals
   and functions in the owning layout or page.
9. Prefer component defaults from `theme.dowe`; add local visual props only for intentional exceptions.
10. Use Signals and View Stores for state, `fn` for event workflows, and one `init` for ordered mount-time work.
11. Write static visible text as `"Blog title"` and dynamic visible text as one complete braced
   binding such as `"{blog.title}"`.
12. Keep route groups one level: every `group` contains direct `route` declarations, never another
   `group`.
13. Use `store name:` with one indented prop per line when Store props would make one long line.
14. Validate bindings, component props, text children, routes, and target support with Dowe diagnostics.
15. Render reference-driven work at the reference viewport and compare it band by band against the
    source image. Iterate on geometry, line wrapping, spacing, density, and assets before finishing.

## Reference routing

| Task | Read only |
| --- | --- |
| Routes, layouts, pages, state, functions, requests, repeated views, or i18n | `references/views.md` |
| New screen, reference image, shell ownership, container choice, hero, or landing composition | `references/composition.md` |
| Built-in component selection, children, bindings, interaction, or portability | `references/components.md` |
| Colors, variants, responsive props, typography, sizing, visibility, overlay, or motion | `references/styles.md` |
| Canvas drawing, input, animation, dynamic scenes, or limits | `references/canvas.md` |
