fn target_prop_keys(source: &str) -> (String, std::collections::BTreeSet<String>) {
    let mut lines = source.lines().filter(|line| !line.trim().is_empty());
    let target = lines
        .next()
        .and_then(|line| line.strip_prefix("target="))
        .expect("target manifest header")
        .to_string();
    let entries = lines.map(str::trim).map(ToOwned::to_owned).collect();
    (target, entries)
}

#[test]
fn exhaustive_prop_target_mode_matrix_is_declared() {
    let manifests = [
        include_str!("../../../generator_web/target_props.def"),
        include_str!("../../../generator_android/target_props.def"),
        include_str!("../../../generator_ios/target_props.def"),
    ]
    .map(target_prop_keys);
    let modes = ["static", "reactive", "responsive", "each"];

    for definition in dowe_components::VIEW_PROP_INVENTORY {
        let owner = match definition.owner {
            dowe_components::ViewPropOwner::CommonStyle => "CommonStyle".to_string(),
            dowe_components::ViewPropOwner::Component(component) => component.as_str().to_string(),
            dowe_components::ViewPropOwner::Item(item) => format!("Item:{}", item.as_str()),
        };
        let key = format!("{owner}|{}|{}", definition.prop, definition.ir_field.as_string());
        for mode in modes {
            if mode == "reactive" && !definition.reactive {
                continue;
            }
            for (_, entries) in &manifests {
                assert!(entries.contains(&key), "missing {mode} capability for {key}");
            }
        }
    }
}

#[test]
fn generators_report_their_runtime_targets() {
    let routes = Vec::new();
    assert_eq!(
        dowe_generator_web::render_report_for_desktop_routes(&routes).target,
        dowe_components::RenderTarget::Desktop
    );
    assert_eq!(
        dowe_generator_android::render_report_for_dev_routes(&routes).target,
        dowe_components::RenderTarget::AndroidDev
    );
    assert_eq!(
        dowe_generator_ios::render_report_for_dev_routes(&routes).target,
        dowe_components::RenderTarget::IosDev
    );
}

#[test]
fn generators_use_the_same_ir_schema_version() {
    assert_eq!(
        dowe_generator_web::VIEW_IR_SCHEMA_VERSION,
        dowe_components::VIEW_IR_SCHEMA_VERSION
    );
    assert_eq!(
        dowe_generator_android::VIEW_IR_SCHEMA_VERSION,
        dowe_components::VIEW_IR_SCHEMA_VERSION
    );
    assert_eq!(
        dowe_generator_ios::VIEW_IR_SCHEMA_VERSION,
        dowe_components::VIEW_IR_SCHEMA_VERSION
    );
}

#[test]
fn target_prop_manifests_match_the_ir_inventory() {
    let expected = dowe_components::VIEW_PROP_INVENTORY
        .iter()
        .map(|definition| {
            let owner = match definition.owner {
                dowe_components::ViewPropOwner::CommonStyle => "CommonStyle",
                dowe_components::ViewPropOwner::Component(component) => component.as_str(),
                dowe_components::ViewPropOwner::Item(item) => match item {
                    dowe_components::ViewItemKind::Tab => "Item:Tab",
                    dowe_components::ViewItemKind::Accordion => "Item:Accordion",
                    dowe_components::ViewItemKind::Carousel => "Item:Carousel",
                    dowe_components::ViewItemKind::Option => "Item:Option",
                    dowe_components::ViewItemKind::TableColumn => "Item:TableColumn",
                    dowe_components::ViewItemKind::NavMenu => "Item:NavMenu",
                    dowe_components::ViewItemKind::SideNav => "Item:SideNav",
                    dowe_components::ViewItemKind::RailNav => "Item:RailNav",
                    dowe_components::ViewItemKind::BottomBar => "Item:BottomBar",
                    dowe_components::ViewItemKind::SvgPath => "Item:SvgPath",
                },
            };
            format!("{owner}|{}|{}", definition.prop, definition.ir_field.as_string())
        })
        .collect::<std::collections::BTreeSet<_>>();

    for (target, manifest) in [
        (
            "web",
            include_str!("../../../generator_web/target_props.def"),
        ),
        (
            "android",
            include_str!("../../../generator_android/target_props.def"),
        ),
        (
            "ios",
            include_str!("../../../generator_ios/target_props.def"),
        ),
    ] {
        let (manifest_target, capabilities) = target_prop_keys(manifest);
        assert_eq!(manifest_target, target);
        assert!(!capabilities.is_empty(), "{target} prop manifest");
        assert!(capabilities.iter().all(|entry| expected.contains(entry)));
        assert!(expected.iter().all(|entry| capabilities.contains(entry)));
    }
}
