---
name: dowe-views
description: Use for Dowe view modules, routes, layouts, pages, UI composition, components, state, requests, responsive styles, Canvas, view targets, or exact reconstruction from an attached screenshot, mockup, or UI reference, including semantic component selection, shell/page ownership, reusable static fragments, and repeated collections rendered with each; skip for server-only work.
---

# Dowe views authoring

Dowe views are target-neutral source compiled to web, desktop, Android, and iOS outputs. Reuse one route graph and one source behavior model across targets.
Keep every new frontend module under `views/`; only root `main.dowe` and `theme.dowe` sit outside it.

## Workflow

1. Find the imported `views` binding connected by `main.dowe`, inspect the route graph, and read
   `theme.dowe`.
2. For reference-driven work, follow `references/reference-ui.md`: initialize the required
   `.dowe/visual-qa/<screen>/blueprint.json`, then inventory the reference viewport and record a
   composition map with ordered bands, region ownership, exact built-ins, collection owners,
   responsive evidence, states, accessibility, theme decisions, assets, and reusable-component
   candidates before authoring source.
3. Keep reference evidence distinct from inference. Preserve visible copy, geometry, hierarchy,
   density, and media intent; infer only behavior the supplied viewport cannot show.
4. Resolve UI roles against `references/components.md`. Prefer the semantic built-in that owns the
   behavior, use contextual children only under their declared owner, and never invent a component
   name from a label in the reference.
5. Rebuild every UI-shaped region with Dowe components. Never use the reference image or crops
   derived from it as assets. Use `Image` only for independently obtained photographs,
   illustrations, textures, or authentic screenshots explicitly supplied or requested by the user.
6. Create or reuse a layout whenever the reference has shared chrome. AppBar and Footer never
   belong in a page, and a one-page site still uses a layout-backed route group.
7. Put exactly one normal `Scaffold` root in every layout; add one direct `Splash` sibling only when
   startup replacement content is required.
8. Start every page with `Section` and use ordered sibling Sections for major page bands. Give
   every landing-page band one job, and make the hero establish the primary promise, support,
   action, and proof before later Sections add detail. Preserve visible copy, band order, actions,
   density, and media intent instead of inventing generic replacements.
9. Use `Grid` for tracks, `Flex` for one-axis flow, `Card` for grouped content, and `Box` only for
   a special neutral wrapper; when unsure, composing a hero or landing page, or working from a
   reference design, follow the decision tree and dedicated patterns in
   `references/composition.md`.
10. Model repeated same-shape UI once: use a `const` for immutable reference-defined content, a
    typed `signal` for a page collection refreshed or replaced by requests or local workflows, and
    an imported View Store only for state shared across routes. Render one unit with
    `each in:<collection> as:<item> key:<item-path>`; never copy sibling Cards or list units.
11. Extract a static fragment reused in two or more places, such as a logo or a navigation tree
    mounted in both Sidebar and Drawer, into a `component` under `views/components`; keep signals,
    functions, caller bindings, and data-bound `each` templates in the owning layout or page because
    reusable components are static and accept no invented props or slots.
12. Prefer component defaults from `theme.dowe`; add local visual props only for intentional
    exceptions.
13. Use Signals and View Stores for state, `fn` for event workflows, and one `init` for ordered
    mount-time work.
14. Write static visible text as `"Blog title"` and dynamic visible text as one complete braced
   binding such as `"{blog.title}"`.
15. Keep route groups one level: every `group` contains direct `route` declarations, never another
   `group`.
16. Use `store name:` with one indented prop per line when Store props would make one long line.
17. Validate bindings, component props, text children, routes, and target support with Dowe
    diagnostics.
18. For reference-driven work, run the installed `scripts/visual_qa.py` entrypoint at the exact
    viewport. Inspect its band report and diff, then iterate on geometry, line wrapping, spacing,
    density, states, and assets before finishing. It imports `scripts/visual_qa_blueprint.py` and
    `scripts/visual_qa_png.py`; do not run the helpers directly.

## Resource routing

Open the primary resource first. Load another only when the task crosses its contract.

| Task | Primary resource |
| --- | --- |
| Routes, layouts, pages, state, functions, requests, repeated views, or i18n | `references/views.md` |
| Exact screenshot, mockup, or UI-reference reconstruction | `references/reference-ui.md` |
| New screen, shell ownership, reusable fragments, container choice, hero, or landing composition | `references/composition.md` |
| Built-in component selection, children, bindings, interaction, or portability | `references/components.md` |
| Colors, variants, responsive props, typography, sizing, visibility, overlay, or motion | `references/styles.md` |
| Canvas drawing, input, animation, dynamic scenes, or limits | `references/canvas.md` |
| Deterministic reference capture and comparison | `scripts/visual_qa.py`, `scripts/visual_qa_blueprint.py`, `scripts/visual_qa_png.py` |
