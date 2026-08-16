# Table and data-grid authoring

Use this reference when a view needs a table, data grid, directory, ledger, project tracker, or
another data-heavy surface. The goal is a strong information hierarchy and a predictable portable
experience across web, desktop, Android, and iOS.

## Contents

- [Choose the right pattern](#choose-the-right-pattern)
- [Table contract](#table-contract)
- [Visual recipe](#visual-recipe)
- [Documented block recipes](#documented-block-recipes)
- [Compose the advanced experience](#compose-the-advanced-experience)
- [Responsive directory pattern](#responsive-directory-pattern)
- [State and accessibility checklist](#state-and-accessibility-checklist)

## Choose the right pattern

There are two supported patterns. Choose before writing the visual tree.

| Need | Pattern | Why |
| --- | --- | --- |
| Scalar cells, explicit headers, semantic tabular data | `Table` | Dowe emits a real web table or equivalent native rows from one target-neutral model. |
| Avatars, chips, nested text, row actions, custom controls, or breakpoint-specific fields | Responsive `Grid` rows | `Table` has no custom cell renderers or visual cell children. A keyed Grid gives each row a complete authored template. |

Use `Table` for invoices, projects, metrics, logs, and simple directories. Use the responsive Grid
pattern for a team directory like the documented block: the primary identity can combine an avatar,
name, and role, while secondary fields can disappear on narrow screens and an action can remain at
the row end. Keep semantic accessibility in mind; when rich row composition is not essential,
prefer `Table`.

## Table contract

`Table` accepts one array path through `data` and one or more direct structural `column` children.
`column` has no children and is valid only inside `Table`.

```text
type InvoiceRow
  id:string
  invoice:string
  customer:string
  issued:string
  due:string
  amount:string
  status:string

page InvoiceTablePage
  signal invoices type:InvoiceRow[] value:[]
  Section id:"invoice-ledger"
    Grid columns:1 gap:6
      Title size:{ xs:"3xl" md:"5xl" } weight:"black"
        "Invoice ledger"
      Table:
        data:invoices
        variant:"soft"
        scheme:"surface"
        size:"md"
        striped:true
        bordered:true
        dividers:true
        rounded:"md"
        column field:"invoice" label:"Invoice" width:"0.9fr"
        column field:"customer" label:"Customer" width:"1.5fr"
        column field:"issued" label:"Issued" width:"1fr"
        column field:"due" label:"Due" width:"1fr"
        column field:"amount" label:"Amount" align:"end" width:"0.8fr"
        column field:"status" label:"Status" align:"end" width:"0.9fr"
```

Rules:

- `data` resolves to a Signal array or an accepted immutable array path. Use a typed `signal` for
  request-backed or changing rows and a `const` for fixed, documentation-like records.
- `field` is a quoted relative row path such as `name` or `profile.email`. Never prefix it with
  the Signal name, leave it empty, or add empty path segments.
- Known row types are checked before generation. Use only fields that exist on the row type and
  resolve to strings, numbers, booleans, or values whose runtime shape is intentionally scalar.
- Objects, arrays, null, and missing runtime fields render as empty cells. Format currency, dates,
  status labels, and other display values in the data contract before handing them to `Table`.
- `label` is static and should be concise. Put the primary identifying column first; put numeric,
  date, and amount columns at the end with `align:"end"` when that improves scanning.
- `width` is static and accepts portable CSS-like hints such as `px`, `%`, `rem`, `fr`, `auto`,
  `min-content`, or `max-content`. Declared width includes cell padding on every target.
- `Table` does not sort, filter, paginate, select, search, or render custom cell children. Do not
  add `onSort`, `page`, `query`, `render`, `toolbar`, or similar invented props.

## Visual recipe

Use the smallest source that expresses the intended hierarchy. The following are deliberate recipes,
not defaults to copy into every table.

| Situation | Recommended direction |
| --- | --- |
| Dashboard or workspace table | `variant:"soft" scheme:"surface" size:"md" striped:true dividers:true rounded:"md"` |
| Dense finance or audit ledger | `variant:"outlined" scheme:"surface" size:"sm" bordered:true dividers:true rounded:"md"` |
| Small, low-emphasis table | `variant:"ghost"` or a minimal surface with the project theme defaults |
| Few columns with spacious reading | `size:"lg"`, restrained widths, and no unnecessary stripes |

Use `scheme`, never `color`, for the Table family. `striped` improves row tracking in longer lists;
`dividers` preserves row boundaries; `bordered` frames the outer table and cell separators. Keep the
surface neutral unless the table itself represents a meaningful status or decision. Do not put every
table inside a Card: a table can be the primary surface inside its page Section, while a surrounding
Card is appropriate only when it groups the table with a title, summary, or actions.

On web and desktop, Dowe emits semantic table markup in a horizontal overflow container. On native
targets, the same headers and rows become readable Compose or SwiftUI output. Narrow screens scroll
the table when its minimum column widths exceed the viewport; do not clip text, squeeze every column
to an unreadable width, or maintain a duplicated mobile table tree. Native targets treat declared
widths as preferred or minimum track sizes: they distribute remaining space when the table is narrower
than the viewport and preserve the minimum widths with horizontal scrolling when it is wider.

`variant` and `scheme` resolve consistently across targets. The header uses the neutral `softMuted`
surface, while stripes, dividers, and bordered cell separators remain neutral so an action-family
scheme does not make the table structure visually noisy. Use `surface` or `background` for structural
schemes when the table should recede into the page.

## Documented block recipes

Use the maintained block examples as four starting points, then adapt the row shape and copy to the
product domain:

| Block | Starting point | UX purpose |
| --- | --- | --- |
| Team directory | Responsive Grid rows with `Input`, `Skeleton`, `IconButton`, and `Pagination` | Rich identity cells, mobile field reduction, explicit row actions, and server-backed results |
| Invoice ledger | `Table` with `soft`/`surface`, `md`, `striped`, `bordered`, `dividers`, and explicit `fr` widths | Fast scanning of identifiers, customers, dates, amounts, and status |
| Project status | `Table` with `outlined`/`surface`, `sm`, `striped`, `bordered`, `dividers`, and end-aligned progress/status | Dense operational tracking with deliberate column proportions |
| Empty workspace | `Table` with `soft`/`surface`, `bordered`, `emptyTitle`, and `emptyDescription` | Make the no-records state intentional and explain the next action |

The directory block is intentionally a responsive Grid rather than a `Table`: its composed avatar,
role, status, hidden fields, and action menu are outside the Table contract. The other three blocks
stay inside the semantic Table contract.

## Compose the advanced experience

Keep data state, transport state, and table presentation separate. A production table commonly has:

1. A heading and one-sentence context in a `Grid`.
2. A `Flex` toolbar with search/filter controls and a primary action when needed.
3. A loading surface using `Skeleton` rows while the request is pending.
4. A `Table` for loaded scalar rows, or one keyed responsive Grid row template for rich cells.
5. An `Alert` for request failure and a clear retry action outside the table.
6. A `Pagination` control below the table when the server returns `total` and page data.

The `Table` owns only the fourth item. Search, filtering, sorting, and pagination belong to a view
function and the server contract. Keep request parameters in Signals, update the typed row Signal on
success, and keep `loading` or an equivalent numeric state separate so an in-flight empty array does
not immediately show the final empty copy.

```text
type DirectoryMember
  id:string
  name:string
  role:string
  company:string
  status:string

page MembersPage
  signal search value:""
  signal currentPage value:1
  signal totalMembers value:0
  signal loadState value:1
  signal members type:DirectoryMember[] value:[]
  fn loadMembers
    set loadState value:1
    request result method:"GET" route:"/api/team-members/q-:search/:currentPage"
    if result.ok
      set members value:result.data.data
      set totalMembers value:result.data.total
      set loadState value:0
    else
      set loadState value:0
  Section id:"members" boxed:true
    Flex direction:"column" gap:4
      Grid columns:{ xs:1 md:2 } gap:2 align:"end"
        Input bind:search label:"Search" placeholder:"Search people" iconStart:"magnifier" w:"full"
        IconButton icon:"magnifier" label:"Load results" onClick:loadMembers
      Skeleton show:{ when:loadState gt:0 } variant:"rounded" h:36 w:"full"
      Table:
        show:{ when:loadState lt:1 }
        data:members
        variant:"soft"
        scheme:"surface"
        size:"md"
        striped:true
        dividers:true
        rounded:"md"
        emptyTitle:"No members found"
        emptyDescription:"Invite a teammate or change the search to see results."
        column field:"name" label:"Name" width:"1.4fr"
        column field:"role" label:"Role" width:"1.2fr"
        column field:"company" label:"Company" width:"1fr"
        column field:"status" label:"Status" align:"end" width:"0.8fr"
      Pagination show:{ when:loadState lt:1 } bind:currentPage total:totalMembers pageSize:10 onChange:loadMembers ariaLabel:"Member pages"
```

Keep the toolbar aligned to the table's readable width. Let the search control grow on desktop but
use a full-width control at `xs`; keep actions labeled and give every `IconButton` a meaningful
static `label`. Show skeletons only while loading, and keep an empty result distinct from a request
failure.

## Responsive directory pattern

Use a responsive Grid instead of `Table` when one row has a composed identity block, hidden secondary
columns, a status surface, or row actions. The documented pattern uses one header track and one keyed
row track, with the same `columns:{ xs:3 md:7 }` geometry for both. The first track is a selection or
identity affordance, the middle tracks contain fields, and the final track owns the row action.

```text
Flex direction:"column" gap:0
  Grid columns:{ xs:3 md:7 } gap:3 align:"center" p:3 border:1 borderColor:"muted"
    Text size:"xs" weight:"bold"
      "Author"
    Text size:"xs" weight:"bold" show:{ xs:false md:true }
      "Company"
    Text size:"xs" weight:"bold" show:{ xs:false md:true }
      "Email"
    Text size:"xs" weight:"bold" show:{ xs:false md:true }
      "Status"
    Text size:"xs" weight:"bold"
      "Actions"
  each in:members as:member key:member.id
    Grid columns:{ xs:3 md:7 } gap:3 align:"center" p:3 border:1 borderColor:"muted"
      Flex align:"center" gap:3
        Avatar alt:"Team member avatar" variant:"soft" scheme:"muted" size:"md"
        Grid columns:1 gap:0
          Text size:"sm" weight:"bold"
            "{member.name}"
          Text size:"xs" color:"muted"
            "{member.role}"
      Text size:"sm" show:{ xs:false md:true }
        "{member.company}"
      Text size:"sm" show:{ xs:false md:true }
        "{member.email}"
      Badge show:{ xs:false md:true }
        Text size:"xs"
          "{member.status}"
      IconButton icon:"menu-dots" label:"Open member actions" variant:"ghost" scheme:"muted" size:"sm"
```

Keep the row template complete inside the single `each`; never copy a desktop row and a mobile row.
Use stable string ids, preserve the same header/row track definitions, hide only secondary fields at
small breakpoints, and keep the primary identity plus action discoverable. If the row remains scalar,
use `Table` instead so assistive technology receives the native table structure.

## State and accessibility checklist

- Define typed rows and stable ids before authoring the visual tree.
- Keep `Table` columns direct, structural, static, and free of children.
- Provide clear column labels and keep the primary identifying field first.
- Use an explicit `emptyTitle` and `emptyDescription` that explain the next useful action.
- Keep loading, success with rows, success with zero rows, and error states visually distinct.
- Use semantic `Table` output whenever rich cell composition is not required.
- Use labeled controls, readable status text, and sufficient contrast from the selected `scheme`.
- Check `xs`, `md`, and a wide viewport: verify overflow, row density, wrapping, toolbar fit, and
  pagination placement.
- Validate the source so unknown fields, invalid widths, invalid alignment, missing columns, and
  unsupported Table props fail before target generation.
