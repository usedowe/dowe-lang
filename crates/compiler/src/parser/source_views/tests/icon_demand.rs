#[test]
fn rejects_unbounded_computed_icon_names_before_generation() {
    for icon in [
        "Icon name:selectedIcon.name",
        "Avatar icon:selectedIcon.name",
        "Avatar\n    icon\n      Icon name:selectedIcon.name",
    ] {
        let error = parse_page(&format!(
            "page iconPage\n  signal selectedIcon value:{{ name:\"home\" }}\n  {icon}"
        ))
        .expect_err("unbounded icon names");
        assert!(error.to_string().contains("selectedIcon"));
        assert!(error.to_string().contains("const"));
    }
}

#[test]
fn validates_computed_icon_names_in_every_constant_item() {
    let error = parse_page(
        r#"page iconPage
  const icons value:[{ id:"first" name:"home" } { id:"second" name:"not-a-dowe-icon" }]
  each in:icons as:icon key:icon.id
    Icon name:icon.name"#,
    )
    .expect_err("invalid later icon");
    assert!(error.to_string().contains("not-a-dowe-icon"));
    assert!(error.to_string().contains("icon.name"));
}

#[test]
fn resolves_computed_icon_names_through_nested_constant_each() {
    let tree = parse_page(
        r#"page iconPage
  const groups value:[{ id:"first" items:[{ id:"one" icon:"home" }] } { id:"second" items:[{ id:"two" icon:"svg-logos:apple" } { id:"three" icon:"home" }] }]
  each in:groups as:group key:group.id
    each in:group.items as:entry key:entry.id
      Icon name:entry.icon"#,
    )
    .expect("bounded icon names");
    assert_eq!(
        dowe_components::dynamic_icon_names(&tree).unwrap(),
        ["home".to_string(), "svg-logos:apple".to_string()]
            .into_iter()
            .collect()
    );
}

#[test]
fn mutable_each_can_repeat_an_independent_constant_icon() {
    let tree = parse_page(
        r#"page iconPage
  signal rows value:[{ id:"one" }]
  const itemIcon value:"home"
  each in:rows as:row key:row.id
    Icon name:itemIcon"#,
    )
    .expect("independent constant icon");
    assert_eq!(
        dowe_components::dynamic_icon_names(&tree).unwrap(),
        ["home".to_string()].into_iter().collect()
    );
}

#[test]
fn rejects_icon_names_from_mutable_each_items() {
    let error = parse_page(
        r#"page iconPage
  signal rows value:[{ id:"one" icon:"home" }]
  each in:rows as:row key:row.id
    Icon name:row.icon"#,
    )
    .expect_err("mutable item icon names");
    assert!(error.to_string().contains("row.icon"));
    assert!(error.to_string().contains("const"));
}

#[test]
fn validates_computed_icon_types_after_the_first_item() {
    let error = parse_page(
        r#"page iconPage
  const icons value:[{ id:"one" name:"home" } { id:"two" name:42 }]
  each in:icons as:icon key:icon.id
    Icon name:icon.name"#,
    )
    .expect_err("non-string later name");
    assert!(error.to_string().contains("icon.name"));
    assert!(error.to_string().contains("string"));
}
