---
name: dowe-core
description: Author or restructure a Dowe project, understand Dowe Source Format, configure main.dowe, use imports, types, translation catalogs, the standard library, native tests, diagnose source, and apply Harness or CodeGraph when needed.
---

# Dowe project authoring

Dowe Source Format is a small declarative language compiled by Rust. It does not execute user source
through JavaScript or Node.js.

## Syntax model

Dowe declarations have only two shapes:

```text
utility key:value
utility binding key:value
```

The second word is a declared name or result binding. Later statements can import it, read it, or
pass it as another prop. Either shape may have indented children when the utility allows them.
Every prop uses `key:value`; arrays use `[]`, objects use `{}`, and static strings use double quotes.
When props would make a declaration long, end the declaration header with `:` and put one prop on
each indented line. Do not mix inline props with that header form. Props must precede children, and
a child may open its own property suite at the next indentation level.

Restricted statements such as `import`, `let`, `if`, `else`, `return`, and quoted text children use
their documented signatures. Do not invent a third declaration style.

In views, static `Text`, `Title`, and `Button` children use ordinary quoted strings. Dynamic visible
text must be one complete braced binding such as `"{blog.title}"`; `"blog.title"` remains literal.
View props continue to use bare bindings such as `bind:form.title` and `show:ready`.

## Workflow

1. Read `main.dowe`, then the imported modules for the requested surface.
2. Read `theme.dowe` before changing visual props or design defaults.
3. Use `dowe-server`, `dowe-views`, or `dowe-theme` for focused work.
4. Preserve existing ownership and reuse declared bindings before adding new ones.
5. Run the narrowest compiler, test, Harness, or CodeGraph validation required by the change.

## Boundaries

- Keep `main.dowe`, `theme.dowe`, `.env.example`, and `.env` at the project root.
- Declare environment names in `.env.example`, keep local values in the ignored `.env`, and use
  static `env.NAME` references only on supported view and server surfaces.
- Treat every environment name referenced from views as public client configuration.
- Frontend modules belong under `views`; backend modules belong under `server`. Keep this canonical
  separation for new source even though declarations and imports remain the compiler authority.
- Treat `main.dowe` and `theme.dowe` as the only Dowe files with fixed root locations.
- Do not require Node.js, `node_modules`, Tailwind, React, or browser-only runtime behavior.
- Do not write comments in `.dowe` source or in any generated code; Dowe source expresses intent
  through declarations, and generated output must stay comment-free.
- Do not edit generated `.dowe` artifacts as source of truth.
- Treat compiler diagnostics as the final authority for syntax and props.
- Keep server behavior in Rust-owned Dowe compilation and views target-neutral.

Read `references/main.md` for root files, imports, type declarations, translation catalogs, and
the canonical project tree. Read `references/workflow.md` for validation, native tests, Harness,
CodeGraph, generated output, and security. Read `references/standard-library.md` for the complete
portable function catalog shared by server statements and view `set` sources.
