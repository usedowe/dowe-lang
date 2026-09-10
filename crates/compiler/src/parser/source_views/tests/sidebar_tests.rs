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
