# Dowe Source Format syntax

This compact contract is the pre-write syntax reference for Dowe agents. It is derived from the
compiler-owned language examples and is intentionally self-contained.

## Declarations

Most declarations have one of these shapes:

```text
utility key:value
utility binding key:value
```

Use indentation for children. Every declaration header is followed by its props, then its
children. A long declaration ends with `:` and puts one prop per indented line; do not mix inline
props with that form.

```text
page HomePage
  Section
    Grid columns:{ xs:1 md:2 } gap:4
      Title as:"h1"
        "Welcome"
      Button href:"#start"
        "Get started"
```

Props use `key:value`. Static strings use double quotes. Arrays use whitespace-separated values,
for example `[dashboardRoutes docsRoutes]`; objects use whitespace-separated `key:value` entries,
for example `{ xs:1 md:2 }`. Comma-separated arrays are migration input only.

## Imports and roots

```text
import viewRoutes from "@/views/routes/view"

main
  app name:"Example" bundle:"com.example.app"
  views:viewRoutes
```

`@/` resolves to the project root. `main.dowe` and `theme.dowe` stay at the root and cannot be
imported. View modules belong under `views/`; server modules belong under `server/`.

## Views and data

```text
views siteRoutes
  group path:"/" layout:SiteLayout
    route path:"" page:HomePage

const features:[
  { id:"lang" title:"Dowe Lang" }
  { id:"cloud" title:"Dowe Cloud" }
]

each in:features as:feature key:feature.id
  Card
    Title
      "{feature.title}"
```

Dynamic visible text is a single quoted binding such as `"{feature.title}"`; an unquoted binding
is not a text node and `"feature.title"` is literal. Prop bindings remain bare: `show:ready`,
`bind:form.title`, `onClick:save`, and `Icon name:feature.icon`.

Use `const` for immutable reference content, a typed `signal` for page-owned reactive collections,
and a View Store only for state shared across routes. Give every repeated record a stable scalar
string or number `id`; never use `key:feature` or another whole object as a key.

## Server shape

Server declarations, routes, functions, and capabilities use the same indentation and prop rules.
Use the focused `dowe-server` references for their exact statements. Do not invent JSX, JavaScript,
CSS, XML, or a third declaration style inside `.dowe` files. Compiler diagnostics are authoritative
when an example and a newer implementation disagree.
