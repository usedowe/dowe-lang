#[test]
fn parses_side_nav_entries_submenus_and_icons() {
    let tree = parse_page(
        r#"page navPage
  SideNav variant:"solid" scheme:"primary" size:"lg" wide:true
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
fn side_nav_identity_belongs_to_root() {
    let tree = parse_page(
        r#"page navPage
  SideNav id:"mobile-navigation"
    item label:"Home" href:"/""#,
    )
    .expect("root identity");
    let ViewNode::SideNav { props, .. } = tree else {
        panic!("side nav");
    };
    assert_eq!(props.style.element.id.as_deref(), Some("mobile-navigation"));

    let error = parse_page(
        r#"page navPage
  SideNav
    item id:"home" label:"Home" href:"/""#,
    )
    .expect_err("item identity");
    assert!(error.to_string().contains(
        "SideNav item does not accept `id`; put `id` on the SideNav root"
    ));
}

#[test]
fn parses_rail_nav_items_icons_and_labels() {
    let tree = parse_page(
        r#"page railPage
  RailNav variant:"solid" scheme:"primary" size:"lg" showLabels:true
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
