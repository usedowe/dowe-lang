use super::ViewSignalValue;

fn icon_demand_fixture(names: Vec<ViewSignalValue>) -> ViewNode {
    ViewNode::Scope {
        constants: vec![super::ViewConstant {
            id: "icons".to_string(),
            name: "icons".to_string(),
            value: ViewSignalValue::Array(names),
        }],
        signals: Vec::new(),
        actions: Vec::new(),
        children: vec![ViewNode::Each {
            item: "icon".to_string(),
            collection: "icons".to_string(),
            key: "icon".to_string(),
            children: vec![
                icon_component_node(vec![ComponentProp {
                    name: "name".to_string(),
                    value: PropValue::String("@icon-binding:icon".to_string()),
                }])
                .unwrap(),
            ],
        }],
    }
}

#[test]
fn computed_icon_catalog_deduplicates_demand_across_trees() {
    let first = icon_demand_fixture(
        ["home", "svg-logos:apple", "home"]
            .into_iter()
            .map(|name| ViewSignalValue::String(name.to_string()))
            .collect(),
    );
    let second = icon_demand_fixture(vec![ViewSignalValue::String("home".to_string())]);
    let catalog = super::dynamic_icon_catalog_for_trees([&first, &second]).unwrap();
    assert_eq!(catalog.len(), 2);
    assert_eq!(catalog[0].0, "home");
    assert_eq!(catalog[1].0, "svg-logos:apple");
    assert!(
        catalog
            .iter()
            .all(|(_, payload)| payload.contains("viewBox"))
    );
}

#[test]
fn empty_constant_each_generates_no_icon_payloads() {
    let tree = icon_demand_fixture(Vec::new());
    assert!(super::dynamic_icon_names(&tree).unwrap().is_empty());
    assert!(
        super::dynamic_icon_catalog_for_trees([&tree])
            .unwrap()
            .is_empty()
    );
}

#[test]
fn unresolved_icon_demand_never_selects_a_complete_catalog() {
    let tree = icon_component_node(vec![ComponentProp {
        name: "name".to_string(),
        value: PropValue::String("@icon-binding:unknown.icon".to_string()),
    }])
    .unwrap();
    let error = super::dynamic_icon_catalog_for_trees([&tree]).unwrap_err();
    assert!(error.to_string().contains("unknown.icon"));
    assert!(error.to_string().contains("const"));
}
