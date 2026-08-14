#[test]
fn rejects_sidebar_navigation_entries() {
    let error = parse_page(
        r#"page navPage
  Sidebar variant:"soft" scheme:"surface"
    item label:"Home" href:"/""#,
    )
    .expect_err("sidebar item");

    assert!(
        error
            .to_string()
            .contains("Sidebar only accepts header, body or footer regions")
    );
}

#[test]
fn parses_nav_menu_items_submenus_and_megamenu_content() {
    let tree = parse_page(
        r##"page navMenuPage
  NavMenu variant:"ghost" scheme:"muted" size:"lg"
    item label:"Home" i18n:"navigation.home" href:"/"
    submenu label:"Docs"
      item label:"Guide" href:"/docs"
      item label:"Install" href:"#install"
    megamenu label:"Resources"
      content
        Card variant:"soft" scheme:"surface"
          Text
            "Resource hub""##,
    )
    .expect("tree");

    let ViewNode::NavMenu { props, items } = tree else {
        panic!("nav menu");
    };
    assert_eq!(props.style.variant, Some(ComponentVariant::Ghost));
    assert_eq!(props.style.color, Some(ColorFamily::Muted));
    assert_eq!(props.size, dowe_components::SideNavSize::Lg);
    assert!(matches!(
        &items[0],
        dowe_components::NavMenuItem::Item(props)
            if props.i18n.as_deref() == Some("navigation.home")
    ));
    assert!(matches!(
        &items[1],
        dowe_components::NavMenuItem::Submenu { items, .. } if items.len() == 2
    ));
    assert!(matches!(
        &items[2],
        dowe_components::NavMenuItem::Megamenu { content, .. } if content.len() == 1
    ));
}

#[test]
fn parses_scaffold_regions_with_required_main() {
    let tree = parse_page(
        r#"page shellPage
  Scaffold boxed:true
    appBar
      AppBar
        center
          Text
            "Top"
    start
      Sidebar
        body
          SideNav
            item label:"Home" href:"/"
    main
      Text
        "Main"
    overlays
      Text
        "Overlay"
    bottomBar
      BottomBar
        tab href:"/" label:"Bottom"
          Icon name:"home""#,
    )
    .expect("tree");

    let ViewNode::Scaffold {
        props,
        app_bar,
        start,
        main,
        bottom_bar,
        overlays,
        ..
    } = tree
    else {
        panic!("scaffold");
    };
    assert!(props.boxed);
    assert_eq!(app_bar.len(), 1);
    assert_eq!(start.len(), 1);
    assert_eq!(main.len(), 1);
    assert_eq!(bottom_bar.len(), 1);
    assert_eq!(overlays.len(), 1);

    let missing_main = parse_page(
        r#"page shellPage
  Scaffold
    start
      Text
        "Side""#,
    )
    .expect_err("missing main");
    assert!(
        missing_main
            .to_string()
            .contains("Scaffold requires a main region with content")
    );
}

#[test]
fn rejects_color_prop_for_nav_menu_and_sidebar() {
    let nav_menu = parse_page(
        r#"page navMenuPage
  NavMenu color:"primary"
    item label:"Home" href:"/""#,
    )
    .expect_err("nav menu color");
    assert!(
        nav_menu
            .to_string()
            .contains("unknown prop `color` on `NavMenu`; use `scheme`")
    );

    let sidebar = parse_page(
        r#"page navPage
  Sidebar color:"primary"
    body
      Text
        "Home""#,
    )
    .expect_err("sidebar color");
    assert!(
        sidebar
            .to_string()
            .contains("unknown prop `color` on `Sidebar`; use `scheme`")
    );
}

#[test]
fn rejects_invalid_side_nav_structure() {
    let icon_name = parse_page(
        r#"page navPage
  SideNav
    item label:"Home" icon:"not-an-icon""#,
    )
    .expect_err("icon name");
    assert!(
        icon_name
            .to_string()
            .contains("known Solar icon variant name")
    );

    let duplicate_icon = parse_page(
        r#"page navPage
  SideNav
    item label:"Home" icon:"home"
      icon
        Svg viewBox:"0 0 24 24"
          Path d:"M3 11l9-8 9 8v10H3z" fill:"currentColor""#,
    )
    .expect_err("duplicate icon");
    assert!(
        duplicate_icon
            .to_string()
            .contains("either an `icon` prop or an icon block")
    );

    let icon = parse_page(
        r#"page navPage
  SideNav
    item label:"Home"
      icon
        Text
          "Home""#,
    )
    .expect_err("icon");
    assert!(
        icon.to_string()
            .contains("SideNav icon requires exactly one Svg child")
    );

    let navigation = parse_page(
        r#"page navPage
  SideNav
    item label:"Home" href:"/" onClick:open"#,
    )
    .expect_err("navigation");
    assert!(
        navigation
            .to_string()
            .contains("`href` and `onClick` cannot be used on the same SideNav entry")
    );
}

#[test]
fn parses_tabs_entries_variants_and_panel_children() {
    let tree = parse_page(
        r#"page tabsPage
  Tabs variant:"line" scheme:"primary" position:"start"
    tab id:"overview" label:"Overview"
      Text
        "Overview content"
    tab id:"details" label:"Details"
      Button
        "Save""#,
    )
    .expect("tree");

    let ViewNode::Tabs { props, tabs } = tree else {
        panic!("tabs");
    };
    assert_eq!(props.variant, dowe_components::TabsVariant::Line);
    assert_eq!(props.color, ColorFamily::Primary);
    assert_eq!(props.position, dowe_components::TabsPosition::Start);
    assert_eq!(tabs.len(), 2);
    assert_eq!(tabs[0].id, "overview");
    assert_eq!(tabs[0].label, "Overview");
    assert_eq!(tabs[1].children.len(), 1);
}

#[test]
fn rejects_invalid_tabs_structure() {
    let color = parse_page(
        r#"page tabsPage
  Tabs color:"primary"
    tab id:"overview" label:"Overview"
      Text
        "Overview""#,
    )
    .expect_err("color");
    assert!(
        color
            .to_string()
            .contains("unknown prop `color` on `Tabs`; use `scheme` for visual family")
    );

    let duplicate = parse_page(
        r#"page tabsPage
  Tabs
    tab id:"overview" label:"Overview"
      Text
        "Overview"
    tab id:"overview" label:"Duplicate"
      Text
        "Duplicate""#,
    )
    .expect_err("duplicate");
    assert!(
        duplicate
            .to_string()
            .contains("duplicate Tabs tab id `overview`")
    );

    let child = parse_page(
        r#"page tabsPage
  Tabs
    Text
      "Overview""#,
    )
    .expect_err("child");
    assert!(child.to_string().contains("Tabs only accepts tab entries"));

    let outside = parse_page(
        r#"page tabsPage
  tab id:"overview" label:"Overview"
    Text
      "Overview""#,
    )
    .expect_err("outside");
    assert!(
        outside
            .to_string()
            .contains("tab can only be used inside Tabs")
    );
}

#[test]
fn parses_stepper_and_rejects_invalid_structure() {
    let tree = parse_page(
        r#"page onboardingPage
  Stepper scheme:"primary" orientation:"horizontal"
    step id:"account" label:"Account"
      Text
        "Account content"
    step id:"profile" label:"Profile"
      Text
        "Profile content""#,
    )
    .expect("tree");

    let ViewNode::Tabs { props, tabs } = tree else {
        panic!("stepper");
    };
    assert_eq!(props.variant, dowe_components::TabsVariant::Stepper);
    assert_eq!(props.position, dowe_components::TabsPosition::Top);
    assert_eq!(tabs.len(), 2);

    let child = parse_page(
        r#"page onboardingPage
  Stepper
    Text
      "Invalid""#,
    )
    .expect_err("child");
    assert!(
        child
            .to_string()
            .contains("Stepper only accepts step entries")
    );

    let outside = parse_page(
        r#"page onboardingPage
  step id:"account" label:"Account"
    Text
      "Invalid""#,
    )
    .expect_err("outside");
    assert!(
        outside
            .to_string()
            .contains("step can only be used inside Stepper")
    );
}

#[test]
fn parses_drawer_with_signal_open_and_responsive_show() {
    let tree = parse_page(
            r#"page navPage
  signal drawerOpen value:false
  Drawer open:drawerOpen position:"end" variant:"soft" scheme:"surface" show:{ xs:true md:false } disableOverlayClose:true hideCloseButton:true
    header
      Title
        "Menu"
    body
      Text
        "Navigation"
    footer
      Text
        "Footer""#,
        )
        .expect("tree");

    let ViewNode::Scope { children, .. } = tree else {
        panic!("scope");
    };
    let ViewNode::Drawer {
        props,
        header,
        body,
        footer,
    } = &children[0]
    else {
        panic!("drawer");
    };
    assert_eq!(props.open, "drawerOpen");
    assert_eq!(props.position, dowe_components::DrawerPosition::End);
    assert_eq!(props.style.variant, Some(ComponentVariant::Soft));
    assert_eq!(props.style.color, Some(ColorFamily::Surface));
    assert!(props.disable_overlay_close);
    assert!(props.hide_close_button);
    assert!(props.style.element.show.is_some());
    assert_eq!(header.len(), 1);
    assert_eq!(body.len(), 1);
    assert_eq!(footer.len(), 1);

    let legacy = parse_page(
        r#"page navPage
  signal drawerOpen value:false
  Drawer open:drawerOpen
    Text
      "Navigation""#,
    )
    .expect("legacy drawer");

    let ViewNode::Scope { children, .. } = legacy else {
        panic!("scope");
    };
    let ViewNode::Drawer { body, .. } = &children[0] else {
        panic!("drawer");
    };
    assert_eq!(body.len(), 1);
}
