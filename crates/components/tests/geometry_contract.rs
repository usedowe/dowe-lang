use dowe_components::*;

#[test]
fn geometry_catalog_covers_every_builtin_and_contextual_owner() {
    for component in BuiltinComponent::ALL
        .iter()
        .copied()
        .chain([BuiltinComponent::Swap])
    {
        let contract = component_geometry_contract(component);
        assert_eq!(contract.component, component);
        assert!(!contract.regions.is_empty(), "{component:?}");
        let mut names = std::collections::BTreeSet::new();
        for region in contract.regions {
            assert!(names.insert(region.name), "{component:?}: {}", region.name);
        }
    }
    assert_eq!(
        component_geometry_contract(BuiltinComponent::Path).family,
        GeometryFamily::Contextual
    );
    assert_eq!(
        component_geometry_contract(BuiltinComponent::Carousel).family,
        GeometryFamily::Collection
    );
}

fn snapshot(target: RenderTarget, density: f64) -> GeometrySnapshot {
    GeometrySnapshot {
        target,
        case_id: "carousel-controls/first".into(),
        environment: GeometryEnvironment {
            width: 360.0,
            height: 800.0,
            text_scale: 1.0,
            locale: "es".into(),
            theme_id: "fixture-theme-v1".into(),
            content_id: "fixture-content-v1".into(),
        },
        density,
        regions: vec![GeometryMeasurement {
            node_id: "carousel/controls/previous".into(),
            parent_id: "carousel/controls".into(),
            rect: GeometryRect {
                x: 10.0 * density,
                y: 20.0 * density,
                width: 32.0 * density,
                height: 32.0 * density,
            },
        }],
    }
}

#[test]
fn geometry_comparison_normalizes_density_and_reports_edge_drift() {
    let reference = snapshot(RenderTarget::Web, 1.0);
    let mut actual = snapshot(RenderTarget::Ios, 3.0);
    assert!(
        compare_geometry(&reference, &actual, 1.0)
            .unwrap()
            .is_empty()
    );
    actual.regions[0].rect.width += 6.0;
    let differences = compare_geometry(&reference, &actual, 1.0).unwrap();
    assert_eq!(differences.len(), 1);
    assert_eq!(differences[0].node_id, "carousel/controls/previous");
}

#[test]
fn geometry_comparison_rejects_incomparable_or_invalid_evidence() {
    let reference = snapshot(RenderTarget::Web, 1.0);
    let mut actual = snapshot(RenderTarget::Android, 2.0);
    actual.environment.text_scale = 1.2;
    assert!(compare_geometry(&reference, &actual, 1.0).is_err());
    actual = snapshot(RenderTarget::Android, 2.0);
    actual.regions.push(actual.regions[0].clone());
    assert!(compare_geometry(&reference, &actual, 1.0).is_err());
    actual.regions.clear();
    assert!(compare_geometry(&reference, &actual, 1.0).is_err());
    actual = snapshot(RenderTarget::Android, 2.0);
    actual.density = f64::NAN;
    assert!(compare_geometry(&reference, &actual, 1.0).is_err());
    actual = snapshot(RenderTarget::Android, 2.0);
    actual.regions[0].rect.width = -1.0;
    assert!(compare_geometry(&reference, &actual, 1.0).is_err());
    assert!(compare_geometry(&reference, &reference, f64::INFINITY).is_err());
}

#[test]
fn geometry_comparison_detects_missing_regions_and_changed_composition() {
    let reference = snapshot(RenderTarget::Web, 1.0);
    let mut actual = snapshot(RenderTarget::Ios, 1.0);
    actual.regions[0].parent_id = "carousel/stage".into();
    assert!(
        !compare_geometry(&reference, &actual, 1.0)
            .unwrap()
            .is_empty()
    );
    actual.regions[0].node_id = "carousel/controls/next".into();
    assert_eq!(compare_geometry(&reference, &actual, 1.0).unwrap().len(), 2);
}

#[test]
fn carousel_geometry_resolves_from_container_and_preserves_explicit_width() {
    let geometry = CarouselGeometryContract::for_variant(CarouselVariant::Stories);
    assert_eq!(geometry.slide_width(200.0, None, 1, 0), 164.0);
    assert_eq!(geometry.slide_width(1000.0, None, 1, 0), 384.0);
    assert_eq!(geometry.slide_width(200.0, Some(300), 1, 0), 300.0);
    let basic = CarouselGeometryContract::for_variant(CarouselVariant::Controls);
    assert_eq!(basic.slide_width(320.0, None, 3, 10), 100.0);
    assert_eq!(basic.slide_width(10.0, None, 3, 20), 0.0);
    assert_eq!(basic.slide_width(f64::NAN, None, 1, 0), 0.0);
    for variant in CarouselVariant::all() {
        let contract = CarouselGeometryContract::for_variant(*variant);
        assert_eq!(contract.content_gap, 12);
        assert_eq!(contract.viewport_padding, 0);
        assert_eq!(contract.vertical_viewport_height, 448);
    }
}

#[test]
fn composed_navigation_reuses_icon_button_geometry() {
    for (size, control, icon) in [
        (ButtonSize::Xs, 24, 16),
        (ButtonSize::Sm, 32, 20),
        (ButtonSize::Md, 40, 24),
        (ButtonSize::Lg, 48, 32),
        (ButtonSize::Xl, 56, 40),
    ] {
        let geometry = IconButtonGeometryContract::for_size(size);
        assert_eq!((geometry.control_size, geometry.icon_size), (control, icon));
        assert_eq!(size.icon_button_control_size().native_units(), control);
        assert_eq!(size.icon_button_icon_size().native_units(), icon);
        let pagination = PaginationControlContract::for_size(size);
        let carousel = CarouselControlContract::for_size(size);
        assert_eq!(
            pagination.control_size,
            IconButtonGeometryContract::for_size(ButtonSize::Sm).control_size
        );
        assert_eq!(carousel.control_size, pagination.control_size);
        assert_eq!(
            carousel.navigation_size,
            IconButtonGeometryContract::for_size(ButtonSize::Md).control_size
        );
    }
    let pagination = view_geometry_contract("Pagination").expect("public Pagination contract");
    assert_eq!(pagination.name, "Pagination");
    assert_eq!(pagination.dependencies, &["IconButton"]);
    assert_eq!(
        component_geometry_contract(BuiltinComponent::Carousel).dependencies,
        &["IconButton", "Pagination"]
    );
    assert_eq!(
        component_geometry_contract(BuiltinComponent::IconButton).dependencies,
        &["Icon"]
    );
}
