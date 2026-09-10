#[test]
fn parses_each_over_an_immutable_view_constant() {
    let tree = parse_page(
        r#"page catalogPage
  const options value:[{ id:"primary" label:"Primary" }]
  Grid
    each in:options as:option key:option.id
      Button scheme:option.id
        "option.label""#,
    )
    .expect("constant each");

    let ViewNode::Scope { children, .. } = tree else {
        panic!("constant scope");
    };
    let ViewNode::Grid { children, .. } = &children[0] else {
        panic!("grid");
    };
    assert!(
        matches!(&children[0], ViewNode::Each { item, collection, key, .. } if item == "option" && collection == "options" && key == "option.id")
    );
}
#[test]
fn accepts_nested_each_collections_from_each_item_paths() {
    parse_page(
        r#"page treePage
  signal tree value:{ folders:[{ id:"views" folders:[{ id:"pages" }] }] }
  each in:tree.folders as:folder key:folder.id
    each in:folder.folders as:subfolder key:subfolder.id
      Text
        "{subfolder.id}""#,
    )
    .expect("nested each collections");
}

#[test]
fn accepts_image_source_from_a_constant_each_item_path() {
    let tree = parse_page(
        r#"page imageCatalogPage
  const homeFeatures value:[{ id:"intuitive-interface" cover:"/assets/img/soluciones-escalables.webp" }]
  Flex
    each in:homeFeatures as:feature key:feature.id
      Image src:feature.cover alt:"Feature cover""#,
    )
    .expect("dynamic image source");

    let ViewNode::Scope { children, .. } = tree else {
        panic!("constant scope");
    };
    let ViewNode::Flex { children, .. } = &children[0] else {
        panic!("flex");
    };
    let ViewNode::Each { children, .. } = &children[0] else {
        panic!("each");
    };
    let ViewNode::Image { props } = &children[0] else {
        panic!("image");
    };
    assert_eq!(props.reactive_src.as_deref(), Some("feature.cover"));
    assert!(props.src.is_empty());
}

#[test]
fn rejects_non_string_dynamic_image_source() {
    let error = parse_page(
        r#"page invalidImagePage
  const features value:[{ id:"feature" cover:42 }]
  each in:features as:feature key:feature.id
    Image src:feature.cover alt:"Feature""#,
    )
    .expect_err("non-string image source");
    assert!(
        error
            .to_string()
            .contains("invalid signal path `feature.cover` in `src`: expected string"),
        "{error}"
    );
}

#[test]
fn accepts_generic_reactive_style_props_from_each_item_paths() {
    let tree = parse_page(
        r#"page catalogPage
  const catalogSources value:[{ id:"one" fill:"primary" padding:8 width:16 title:"One" }]
  each in:catalogSources as:catalog key:catalog.id
    Card p:catalog.padding w:catalog.width bg:catalog.fill
      Text
        "{catalog.title}""#,
    ).expect("generic reactive style props");
    fn find(node: &ViewNode) -> Option<&dowe_components::StyleProps> {
        match node {
            ViewNode::Card { props, .. } => Some(&props.style),
            _ => dowe_components::node_child_groups(node).into_iter().find_map(|group| group.iter().find_map(find)),
        }
    }
    let style = find(&tree).expect("card style");
    assert!(style.spacing.p_binding.is_some());
    assert!(style.sizing.w_binding.is_some());
    assert!(style.bg_binding.is_some());
}

#[test]
fn accepts_icon_name_from_constant_signal_and_each_item_paths() {
    let tree = parse_page(
        r#"page platformPage
  const platforms value:[{ icon:"route-bold-duotone" title:"server" } { icon:"global-bold-duotone" title:"web" }]
  const fallbackIcon value:"home"
  signal activeIcon value:"home"
  Flex
    each in:platforms as:platform key:platform.title
      Icon name:platform.icon
    Icon name:fallbackIcon
    Icon name:activeIcon"#,
    )
    .expect("dynamic icon names");

    fn collect(node: &ViewNode, names: &mut Vec<String>) {
        if let ViewNode::Svg { props, .. } = node
            && let Some(name) = props.icon_name.as_ref()
        {
            names.push(name.clone());
            assert!(props.icon_fallback.is_some());
        }
        for group in dowe_components::node_child_groups(node) {
            for child in group {
                collect(child, names);
            }
        }
    }

    let mut names = Vec::new();
    collect(&tree, &mut names);
    assert_eq!(names, ["platform.icon", "fallbackIcon", "activeIcon"]);
}

#[test]
fn narrows_dynamic_icon_catalog_to_static_constant_values() {
    let tree = parse_page(
        r#"page platformPage
  const platforms value:[{ icon:"route-bold-duotone" title:"server" } { icon:"global-bold-duotone" title:"web" } { icon:"laptop-bold-duotone" title:"desktop" } { icon:"svg-logos:android-icon" title:"Android" } { icon:"svg-logos:apple" title:"Ios" }]
  each in:platforms as:platform key:platform.title
    Icon name:platform.icon"#,
    )
    .expect("dynamic icon names");

    assert_eq!(
        dowe_components::dynamic_icon_names(&tree).expect("static icon catalog"),
        [
            "global-bold-duotone".to_string(),
            "laptop-bold-duotone".to_string(),
            "route-bold-duotone".to_string(),
            "svg-logos:android-icon".to_string(),
            "svg-logos:apple".to_string(),
        ]
        .into_iter()
        .collect()
    );
}

#[test]
fn ignores_unrelated_mutable_each_when_narrowing_dynamic_icon_catalog() {
    let tree = parse_page(
        r#"page iconPage
  signal icons value:[{ id:"one" }]
  const selectedIcon value:"route-bold-duotone"
  each in:icons as:icon key:icon.id
    Text
      "Catalog entry"
  Icon name:selectedIcon"#,
    )
    .expect("dynamic icon names");

    assert_eq!(
        dowe_components::dynamic_icon_names(&tree).expect("static icon catalog"),
        ["route-bold-duotone".to_string()].into_iter().collect()
    );
}

#[test]
fn keeps_full_dynamic_icon_catalog_for_mutable_signals() {
    let tree = parse_page(
        r#"page iconPage
  signal selectedIcon value:"home"
  Icon name:selectedIcon"#,
    )
    .expect("dynamic icon names");

    assert!(dowe_components::dynamic_icon_names(&tree).is_none());
}

#[test]
fn rejects_non_string_icon_name_binding() {
    let error = parse_page(
        r#"page platformPage
  signal iconIndex value:1
  Icon name:iconIndex"#,
    )
    .expect_err("numeric icon binding");
    assert!(error.to_string().contains("Icon name"));
    assert!(error.to_string().contains("expected string"));
}

#[test]
fn hoists_constants_declared_in_a_visual_block_for_each() {
    let tree = parse_page(
        r#"page catalogPage
  Grid columns:1
    const options value:[{ id:"primary" label:"Primary" }]
    each in:options as:option key:option.id
      Text
        "{option.label}""#,
    )
    .expect("visual block constant");

    let ViewNode::Scope {
        constants,
        children,
        ..
    } = tree
    else {
        panic!("constant scope");
    };
    assert_eq!(constants.len(), 1);
    assert_eq!(constants[0].name, "options");
    let ViewNode::Grid { children, .. } = &children[0] else {
        panic!("grid");
    };
    assert!(
        matches!(&children[0], ViewNode::Each { item, collection, key, .. } if item == "option" && collection == "options" && key == "option.id")
    );
}

#[test]
fn parses_each_props_independently_of_order() {
    parse_page(
        r#"page catalogPage
  signal options value:[{ id:"primary" }]
  each key:option.id as:option in:options
    Text
      "option.id""#,
    )
    .expect("ordered each props");
}

#[test]
fn rejects_positional_each_syntax() {
    let error = parse_page(
        r#"page catalogPage
  signal options value:[{ id:"primary" }]
  each option in options key:option.id
    Text
      "option.id""#,
    )
    .expect_err("positional each");

    assert!(
        error
            .to_string()
            .contains("`each` must use `each in:collection as:item key:item.id`")
    );
}

#[test]
fn rejects_invalid_each_props() {
    let quoted = parse_page(
        r#"page catalogPage
  signal options value:[{ id:"primary" }]
  each in:"options" as:option key:option.id
    Text
      "option.id""#,
    )
    .expect_err("quoted collection");
    assert!(quoted.to_string().contains("`in` must be a reference"));

    let missing = parse_page(
        r#"page catalogPage
  signal options value:[{ id:"primary" }]
  each in:options key:option.id
    Text
      "option.id""#,
    )
    .expect_err("missing alias");
    assert!(missing.to_string().contains("missing `as`"));

    let quoted_key = parse_page(
        r#"page catalogPage
  signal options value:[{ id:"primary" }]
  each in:options as:option key:"option.id"
    Text
      "option.id""#,
    )
    .expect_err("quoted key");
    assert!(quoted_key.to_string().contains("`key` must be a reference"));

    let unknown = parse_page(
        r#"page catalogPage
  signal options value:[{ id:"primary" }]
  each in:options as:option key:option.id index:index
    Text
      "option.id""#,
    )
    .expect_err("unknown each prop");
    assert!(
        unknown
            .to_string()
            .contains("unknown prop `index` on `each`")
    );
}
