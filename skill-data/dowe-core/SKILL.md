---
name: dowe-core
description: Use for Dowe project roots, main.dowe, imports, types, translations, native tests, diagnostics, Harness, CodeGraph, or portable functions; skip for view-, server-, or theme-only edits.
---

# Dowe project authoring

Dowe Source Format is a small declarative language compiled through Dowe's shared Rust toolchain.

## Syntax model

Dowe declarations have only two shapes:

```text
utility key:value
utility binding key:value
```

The second word is a declared name or result binding. Later statements can import it, read it, or
pass it as another prop. Either shape may have indented children when the utility allows them.
Every prop uses `key:value`; arrays use `[]` with whitespace-separated items, objects use `{}` with
whitespace-separated `key:value` entries, and static strings use double quotes.
When props would make a declaration long, end the declaration header with `:` and put one prop on
each indented line. Do not mix inline props with that header form. Props must precede children, and
a child may open its own property suite at the next indentation level.

Restricted statements such as `import`, `let`, `if`, `else`, `return`, and quoted text children use
their documented signatures. Do not invent a third declaration style.

In views, static `Text`, `Title`, and `Button` children use ordinary quoted strings. Dynamic visible
text must be one complete braced binding such as `"{blog.title}"`; `"blog.title"` remains literal.
View props continue to use bare bindings such as `bind:form.title` and `show:ready`.

Use `[value value]` for new arrays. Existing comma-separated arrays remain accepted for migration,
but the formatter and generated templates emit the whitespace form.

## Workflow

1. Read `main.dowe`, then the imported modules for the requested surface.
2. Read `theme.dowe` before changing visual props or design defaults.
3. Use `dowe-server`, `dowe-views`, or `dowe-theme` for focused work. When a View request targets a
   project-owned route, or a Server route change affects a View caller, use the connected workflow:
   load both `dowe-views` and `dowe-server`. Trace the request-to-route contract before editing
   either side.
4. Preserve existing ownership and reuse declared bindings before adding new ones.
5. Run the narrowest compiler, test, Harness, or CodeGraph validation required by the change.

When a root theme is created or explicitly changed, delegate color authoring to `dowe-theme` and use
grouped `colors:` families with `color`, `text`, and `title` roles. When a page is generated without
a theme request, preserve the existing `theme.dowe` and consume its semantic tokens. A reference
image supplied for a layout, page, reusable component, or `Section` is not a theme request: do not
create `theme.dowe` or change its colors unless the user explicitly asks for theme or color changes.

## Reference routing

| Task | Read only |
| --- | --- |
| Root files, `main.dowe`, imports, types, translations, or project tree | `references/main.md` |
| Diagnostics, validation, native tests, Harness, CodeGraph, or generated output | `references/workflow.md` |
| Portable `str`, `math`, `parse`, `url`, `csv`, `sort`, `list`, `json`, or `date` function | `references/standard-library.md` |

## Boundaries

- Keep `main.dowe`, `theme.dowe`, `.env.example`, `.env`, `.env.live`, `.env.stage`, and `.env.uat` at the project root.
- Declare environment names in `.env.example`, keep development values in the ignored `.env`, keep
  build and Live values in `.env.live`, Stage values in `.env.stage`, UAT values in `.env.uat`, and use static `env.NAME` references only on
  supported view and server surfaces.
- Treat every environment name referenced from views as public client configuration.
- Frontend modules belong under `views`; backend modules belong under `server`. Keep this canonical
  separation for new source even though declarations and imports remain the compiler authority.
- Treat `main.dowe` and `theme.dowe` as the only Dowe files with fixed root locations.
- Do not write comments in `.dowe` source or in any generated code; Dowe source expresses intent
  through declarations, and generated output must stay comment-free.
- Do not edit generated `.dowe` artifacts as source of truth.
- Treat compiler diagnostics as the final authority for syntax and props.
- Keep server behavior in Rust-owned Dowe compilation and views target-neutral.
