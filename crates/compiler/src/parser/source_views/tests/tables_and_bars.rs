#[test]
fn lowers_pagination_to_reactive_page_control() {
    let tree = parse_page(
        r#"page blogPage
  signal page value:"1"
  fn loadBlogs
    reset page
  Pagination bind:page total:50 pageSize:10 onChange:loadBlogs"#,
    )
    .expect("tree");

    let ViewNode::Scope { children, .. } = tree else {
        panic!("scope");
    };
    let ViewNode::ToggleGroup { props, items } = &children[0] else {
        panic!("pagination toggle group");
    };
    assert_eq!(props.value.as_deref(), Some("page"));
    assert_eq!(props.kind, dowe_components::ToggleGroupKind::Pagination);
    assert_eq!(props.on_change.as_deref(), Some("loadBlogs"));
    assert_eq!(items.len(), 5);
    assert_eq!(items[4].id, "5");
}

#[test]
fn lowers_runtime_pagination_total_signal() {
    let tree = parse_page(
        r#"page catalogPage
  signal page value:1
  signal total value:0
  fn loadCatalog
    reset page
  Pagination bind:page total:total pageSize:60 onChange:loadCatalog"#,
    )
    .expect("tree");

    let ViewNode::Scope { children, .. } = tree else {
        panic!("scope");
    };
    let ViewNode::ToggleGroup { props, items } = &children[0] else {
        panic!("pagination toggle group");
    };
    let Some(dowe_components::PaginationProps {
        total: dowe_components::PaginationTotal::Signal(total),
        page_size,
    }) = props.pagination.as_ref()
    else {
        panic!("runtime pagination props");
    };
    assert_eq!(total, "total");
    assert_eq!(*page_size, 60);
    assert_eq!(items.len(), 25);
}

#[test]
fn rejects_invalid_table_usage() {
    let wrong_type = parse_page(
        r#"page usersPage
  signal users value:{ name:"Ana" }
  Table data:users
    column field:"name" label:"Name""#,
    )
    .expect_err("wrong type");
    assert!(
        wrong_type
            .to_string()
            .contains("signal `users` in `data` must be an array")
    );

    let color_prop = parse_page(
        r#"page usersPage
  signal users value:[{ name:"Ana" }]
  Table data:users color:"primary"
    column field:"name" label:"Name""#,
    )
    .expect_err("color prop");
    assert!(
        color_prop
            .to_string()
            .contains("unknown prop `color` on `Table`; use `scheme` for visual family")
    );

    let missing_field = parse_page(
        r#"type UserRow
  name:string

page usersPage
  signal users type:UserRow[] value:[]
  Table data:users
    column field:"status" label:"Status""#,
    )
    .expect_err("missing field");
    assert!(
        missing_field
            .to_string()
            .contains("unknown Table column field `status`")
    );
}

#[test]
fn parses_divider_component_with_orientation_and_scheme() {
    let tree = parse_page(
        r#"page dividerPage
  Divider orientation:"vertical" scheme:"primary" h:24"#,
    )
    .expect("tree");

    let ViewNode::Divider { props } = tree else {
        panic!("divider");
    };
    assert_eq!(props.orientation, DividerOrientation::Vertical);
    assert_eq!(props.color, ColorFamily::Primary);
    assert!(props.style.sizing.h.is_some());
}

#[test]
fn rejects_invalid_divider_usage() {
    let orientation = parse_page(
        r#"page dividerPage
  Divider orientation:"diagonal""#,
    )
    .expect_err("orientation");
    assert!(
        orientation
            .to_string()
            .contains("expected horizontal or vertical")
    );

    let child = parse_page(
        r#"page dividerPage
  Divider
    Text
      "Invalid""#,
    )
    .expect_err("child");
    assert!(
        child
            .to_string()
            .contains("children are not valid for this component")
    );
}

#[test]
fn rejects_invalid_video_usage() {
    let missing = parse_page(
        r#"page videoPage
  Video"#,
    )
    .expect_err("missing");
    assert!(missing.to_string().contains("invalid value for prop `src`"));

    let http = parse_page(
        r#"page videoPage
  Video src:"http://example.com/video.mp4""#,
    )
    .expect_err("http");
    assert!(http.to_string().contains("expected https URL"));

    let aspect = parse_page(
        r#"page videoPage
  Video src:"https://example.com/video.mp4" aspect:"wide""#,
    )
    .expect_err("aspect");
    assert!(
        aspect
            .to_string()
            .contains("expected horizontal, vertical or square")
    );

    let child = parse_page(
        r#"page videoPage
  Video src:"https://example.com/video.mp4"
    Text
      "Invalid""#,
    )
    .expect_err("child");
    assert!(
        child
            .to_string()
            .contains("children are not valid for this component")
    );
}

#[test]
fn parses_layout_bar_regions() {
    let tree = parse_page(
            r#"page barsPage
  AppBar variant:"soft" scheme:"surface" position:"fixed" bordered:true blurred:true boxed:true floating:true hideOnScroll:true dockOnScroll:true
    top
      Text
        "Notice"
    start
      Text
        "Menu"
    center
      Text
        "Brand"
    end
      Button href:"/"
        "Home"
    bottom
      Text
        "Status"
    mobileMenu mobileMenuOpen:menuOpen
      header
        Text
          "Navigation"
      body
        Button href:"/docs"
          "Docs"
      footer
        Text
          "Mobile navigation""#,
        )
        .expect("tree");

    let ViewNode::AppBar {
        props,
        top,
        start,
        center,
        end,
        bottom,
        mobile_menu,
    } = tree
    else {
        panic!("appbar");
    };

    assert_eq!(props.style.variant, Some(ComponentVariant::Solid));
    assert_eq!(props.style.color, Some(ColorFamily::Surface));
    assert!(props.bordered);
    assert!(props.blurred);
    assert!(props.boxed);
    assert!(props.floating);
    assert!(props.hide_on_scroll);
    assert!(props.dock_on_scroll);
    assert_eq!(mobile_menu.as_ref().and_then(|menu| menu.open.as_deref()), Some("menuOpen"));
    assert_eq!(top.len(), 1);
    assert_eq!(start.len(), 1);
    assert_eq!(center.len(), 1);
    assert_eq!(end.len(), 1);
    assert_eq!(bottom.len(), 1);
    let menu = mobile_menu.expect("mobile menu");
    assert_eq!(menu.header.len(), 1);
    assert_eq!(menu.body.len(), 1);
    assert_eq!(menu.footer.len(), 1);
}

#[test]
fn parses_footer_full_width_regions() {
    let tree = parse_page(
        r#"page footerPage
  Footer variant:"soft" scheme:"surface" boxed:true
    top
      Text
        "Directory"
    start
      Text
        "Company"
    center
      Text
        "Links"
    end
      Text
        "Social"
    bottom
      Text
        "Legal""#,
    )
    .expect("footer regions");

    let ViewNode::Footer {
        props,
        top,
        start,
        center,
        end,
        bottom,
        ..
    } = tree
    else {
        panic!("footer");
    };

    let horizontal = props.style.style.spacing.px.expect("default px");
    assert_eq!(
        horizontal.entries[0].breakpoint,
        dowe_components::Breakpoint::Xs
    );
    assert_eq!(
        horizontal.entries[0].value,
        dowe_components::ScaleValue::from_half_steps(8)
    );
    assert_eq!(
        horizontal.entries[1].breakpoint,
        dowe_components::Breakpoint::Md
    );
    assert_eq!(
        horizontal.entries[1].value,
        dowe_components::ScaleValue::from_half_steps(12)
    );
    let top_padding = props.style.style.spacing.pt.expect("default pt");
    assert_eq!(
        top_padding.entries[0].breakpoint,
        dowe_components::Breakpoint::Xs
    );
    assert_eq!(
        top_padding.entries[0].value,
        dowe_components::ScaleValue::from_half_steps(20)
    );
    assert_eq!(
        top_padding.entries[1].breakpoint,
        dowe_components::Breakpoint::Md
    );
    assert_eq!(
        top_padding.entries[1].value,
        dowe_components::ScaleValue::from_half_steps(32)
    );
    let bottom_padding = props.style.style.spacing.pb.expect("default pb");
    assert_eq!(
        bottom_padding.entries[0].value,
        dowe_components::ScaleValue::from_half_steps(8)
    );
    assert_eq!(
        bottom_padding.entries[1].value,
        dowe_components::ScaleValue::from_half_steps(12)
    );
    assert_eq!(top.len(), 1);
    assert_eq!(start.len(), 1);
    assert_eq!(center.len(), 1);
    assert_eq!(end.len(), 1);
    assert_eq!(bottom.len(), 1);
}

#[test]
fn parses_bottom_bar_tabs_with_icon_and_featured_state() {
    let tree = parse_page(
        r#"page tabsPage
  BottomBar variant:"soft" scheme:"surface"
    tab href:"/home" label:"Home"
      Icon name:"home"
    tab href:"/create" label:"Create" featured:true
      Icon name:"add-circle""#,
    )
    .expect("bottom bar tabs");

    let ViewNode::BottomBar { tabs, .. } = tree else {
        panic!("bottom bar");
    };
    assert_eq!(tabs.len(), 2);
    assert_eq!(tabs[0].label, "Home");
    assert!(!tabs[0].featured);
    assert_eq!(tabs[1].label, "Create");
    assert!(tabs[1].featured);
}

#[test]
fn rejects_bottom_bar_tabs_without_icon_or_with_multiple_featured_tabs() {
    let missing_icon = parse_page(
        r#"page tabsPage
  BottomBar
    tab href:"/home" label:"Home""#,
    )
    .expect_err("missing icon");
    assert!(missing_icon.to_string().contains("exactly one Icon child"));

    let multiple_featured = parse_page(
        r#"page tabsPage
  BottomBar
    tab href:"/home" label:"Home" featured:true
      Icon name:"home"
    tab href:"/create" label:"Create" featured:true
      Icon name:"add-circle""#,
    )
    .expect_err("multiple featured");
    assert!(
        multiple_featured
            .to_string()
            .contains("at most one featured tab")
    );
}

#[test]
fn rejects_invalid_layout_bar_regions() {
    let duplicate = parse_page(
        r#"page barsPage
  AppBar
    start
      Text
        "Menu"
    start
      Text
        "Brand""#,
    )
    .expect_err("duplicate");
    assert!(
        duplicate
            .to_string()
            .contains("duplicate `start` region in AppBar")
    );

    let direct_child = parse_page(
        r#"page barsPage
  AppBar
    Text
      "Brand""#,
    )
    .expect_err("direct child");
    assert!(
        direct_child
            .to_string()
            .contains("AppBar only accepts top, start, center, end or bottom regions")
    );
}

#[test]
fn parses_side_nav_entries_submenus_and_icons() {
    let tree = parse_page(
        r#"page navPage
  SideNav variant:"soft" scheme:"primary" size:"lg" wide:true
    header label:"Workspace" description:"Account navigation"
    item label:"Home" href:"/" icon:"home"
    divider
    submenu label:"Content" status:"2" open:true bordered:false
      item label:"Blogs" href:"/blogs" status:"12""#,
    )
    .expect("tree");

    let ViewNode::SideNav { props, items } = tree else {
        panic!("side nav");
    };
    assert_eq!(props.style.variant, Some(ComponentVariant::Solid));
    assert_eq!(props.style.color, Some(ColorFamily::Primary));
    assert_eq!(props.size, dowe_components::SideNavSize::Lg);
    assert!(props.wide);
    assert!(matches!(
        &items[1],
        dowe_components::SideNavItem::Item(props) if props.icon.is_some()
    ));
    assert!(matches!(
        &items[3],
        dowe_components::SideNavItem::Submenu { open: true, bordered: false, items, .. } if items.len() == 1
    ));
}

#[test]
fn parses_reactive_side_nav_visual_props() {
    let tree = parse_page(
        r#"page navigationPage
  signal variantChoice value:"ghost"
  signal schemeChoice value:"muted"
  signal sizeChoice value:"md"
  signal wideEnabled value:true
  SideNav variant:variantChoice scheme:schemeChoice size:sizeChoice wide:wideEnabled
    item label:"Overview" href:"/overview""#,
    )
    .expect("reactive side nav");
    let ViewNode::Scope { children, .. } = tree else {
        panic!("scope");
    };
    let ViewNode::SideNav { props, .. } = &children[0] else {
        panic!("side nav");
    };
    assert_eq!(
        props.style.reactive.variant.as_deref(),
        Some("variantChoice")
    );
    assert_eq!(props.style.reactive.scheme.as_deref(), Some("schemeChoice"));
    assert_eq!(props.style.reactive.size.as_deref(), Some("sizeChoice"));
    assert_eq!(props.reactive_wide.as_deref(), Some("wideEnabled"));
}

#[test]
fn parses_rail_nav_items_icons_and_labels() {
    let tree = parse_page(
        r#"page railPage
  RailNav variant:"soft" scheme:"primary" size:"lg" showLabels:true
    item label:"Home" i18n:"navigation.home" href:"/" icon:"home"
    divider
    item label:"Settings" onClick:openSettings icon:"settings""#,
    )
    .expect("rail nav");

    let ViewNode::RailNav { props, items } = tree else {
        panic!("rail nav");
    };
    assert_eq!(props.style.variant, Some(ComponentVariant::Solid));
    assert_eq!(props.style.color, Some(ColorFamily::Primary));
    assert_eq!(props.size, dowe_components::SideNavSize::Lg);
    assert!(props.show_labels);
    assert!(matches!(
        &items[0],
        dowe_components::RailNavItem::Item(props)
            if props.i18n.as_deref() == Some("navigation.home")
                && props.navigation.is_some()
    ));
    assert!(matches!(&items[1], dowe_components::RailNavItem::Divider));
    assert!(matches!(
        &items[2],
        dowe_components::RailNavItem::Item(props)
            if props.on_click.as_deref() == Some("openSettings")
    ));
}

#[test]
fn rejects_invalid_rail_nav_structure() {
    let missing_icon = parse_page(
        r#"page railPage
  RailNav
    item label:"Home" href:"/""#,
    )
    .expect_err("missing icon");
    assert!(
        missing_icon
            .to_string()
            .contains("invalid value for prop `icon`")
    );

    let invalid_child = parse_page(
        r#"page railPage
  RailNav
    header label:"Workspace""#,
    )
    .expect_err("invalid child");
    assert!(
        invalid_child
            .to_string()
            .contains("RailNav only accepts item or divider entries")
    );

    let dynamic_labels = parse_page(
        r#"page railPage
  RailNav showLabels:enabled
    item label:"Home" href:"/" icon:"home""#,
    )
    .expect_err("dynamic labels");
    assert!(dynamic_labels.to_string().contains("showLabels"));
}

#[test]
fn parses_sidebar_as_regional_shell_surface() {
    let tree = parse_page(
        r#"page navPage
  Sidebar variant:"solid" scheme:"primary"
    header
      Text
        "Workspace"
    body
      SideNav variant:"ghost" scheme:"primary" size:"sm" wide:true
        item label:"Home" href:"/"
        submenu label:"Content" open:true
          item label:"Blogs" href:"/blogs"
    footer
      Text
        "Footer""#,
    )
    .expect("tree");

    let ViewNode::Sidebar {
        props,
        header,
        body,
        footer,
    } = tree
    else {
        panic!("sidebar");
    };
    assert_eq!(props.style.variant, Some(ComponentVariant::Solid));
    assert_eq!(props.style.color, Some(ColorFamily::Primary));
    assert_eq!(header.len(), 1);
    assert_eq!(body.len(), 1);
    assert_eq!(footer.len(), 1);
    assert!(matches!(
        &body[0],
        ViewNode::SideNav { props, items }
            if props.size == dowe_components::SideNavSize::Sm
                && props.wide
                && matches!(&items[1], dowe_components::SideNavItem::Submenu { open: true, bordered: true, items, .. } if items.len() == 1)
    ));
}
