# Layouts

Layouts own persistent shell structure; pages own route-specific content and state. Create a layout when two routes share chrome, or when a single route has an application shell that should not be repeated. A one-page site still uses a layout-backed route group when it has shell chrome.

## Ownership boundaries

- Put exactly one normal `Scaffold` in each layout. A page must not contain `Scaffold`, `AppBar`, `Sidebar`, `Drawer`, `Footer`, `BottomBar`, or `children`.
- `Scaffold` owns `appBar`, `start`, `main`, `end`, `bottomBar`, and `overlays`; `AppBar` owns navigation regions; `Sidebar` owns desktop navigation; `Drawer` owns the mobile navigation surface; `Footer` owns persistent footer content.
- Keep one shared shell tree for mobile and desktop. Use responsive `show` on the actual `NavMenu`, trigger, `Sidebar`, or `Drawer`; do not duplicate AppBars or page forms.
- `NavMenu` is horizontal AppBar navigation. A vertical `SideNav` may be reused in both `Sidebar body` and `Drawer body`.
- Route groups wire the shell: keep groups one level, declare direct child routes, and attach `layout:<Layout>` to the group. Keep page content as sibling `Section` roots.

## Minimal template

Omit theme-resolved visual and padding defaults. Availability or screenshot measurements do not justify `variant`, `scheme`, `rounded`, `p`, `px`, `py`, `pt`, `pb`, `pl`, or `pr`. Retain a prop only for required contract/content/accessibility/binding/behavior, an explicit non-default request, or a proven post-render exception.

For vertical navigation, put an optional `id` on the `SideNav` root when two structurally
identical navigations need independent submenu memory. `item` entries do not accept `id`.

```text
layout SiteLayout
  signal openNavigation value:false
  Scaffold
    appBar
      AppBar
        start
          Brand href:"/" label:"Site home"
        center
          NavMenu show:{ xs:false md:true }
            item label:"Home" href:"/"
        end
          IconButton show:{ xs:true md:false } icon:"menu-dots" label:"Open navigation" onClick:{ set:openNavigation value:!openNavigation }
    start
      Sidebar show:{ xs:false md:true }
        body
          SideNav
            item label:"Home" href:"/"
    main
      children
    overlays
      Drawer bind:openNavigation show:{ xs:true md:false }
        body
          SideNav
            item label:"Home" href:"/"
    
 group path:"/" layout:SiteLayout
   route path:"" page:HomePage
```

The example shows boundaries, not a required navigation shape. Add authored spacing only to the actual region or content owner when a contract or proven exception requires it; never add padding to compensate for a shell wrapper.
