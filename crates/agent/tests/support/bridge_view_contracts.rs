#[test]
fn view_skill_requires_faithful_reference_driven_composition() {
    let compact = get_public_skill("views", false).expect("compact views skill");
    let full = get_public_skill("views", true).expect("full views skill");
    let compact_content = compact
        .content
        .to_ascii_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    assert!(compact_content.contains("inventory"));
    assert!(compact_content.contains("preserve visible"));
    assert!(compact_content.contains("reference viewport"));
    assert!(
        full.content
            .contains("even when the route graph has one page")
    );
    assert!(full.content.contains("When an original is unavailable"));
    assert!(full.content.contains("AppBar and Footer"));
}

#[test]
fn view_skill_requires_dowe_native_reference_reconstruction() {
    let compact = get_public_skill("views", false).expect("compact views skill");
    let full = get_public_skill("views", true).expect("full views skill");
    let compact_content = compact
        .content
        .to_ascii_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    assert!(
        compact_content
            .contains("never use the reference image or crops derived from it as assets")
    );
    assert!(full.content.contains("validation evidence, never as"));
    assert!(
        full.content
            .contains("Rebuild navigation, headings, text, controls, cards, lists, metrics")
    );
    assert!(full.content.contains("independently obtained"));
    assert!(full.content.contains("rasterization, or recomposition"));
}

#[test]
fn view_skill_requires_semantic_ownership_and_collection_modeling() {
    let compact = get_public_skill("views", false).expect("compact views skill");
    let full = get_public_skill("views", true).expect("full views skill");

    assert!(
        compact
            .content
            .contains("composition map with ordered bands")
    );
    assert!(
        compact
            .content
            .contains("sibling Cards, feature rows, icon/text groups, or list units")
    );
    assert!(full.content.contains("never invent `MenuBar`"));
    assert!(
        full.content
            .contains("Fixed copy and records visible in the reference")
    );
    assert!(
        full.content
            .contains("Data loaded, filtered, paged, appended, or replaced")
    );
    assert!(
        full.content
            .contains("Reusable components do not accept dynamic caller inputs")
    );
    assert!(full.content.contains("page-only"));
    assert!(full.content.contains("theme unchanged"));
    assert!(full.content.contains("grouped `colors:` form"));
    assert!(
        compact
            .content
            .contains("layout, page, reusable component, or")
    );
    assert!(
        compact.content.contains(
            "Generate a theme or modify its colors only when the user explicitly requests"
        )
    );
}

#[test]
fn view_skill_reserves_box_for_advanced_layer_planes() {
    let compact = get_public_skill("views", false).expect("compact views skill");
    let full = get_public_skill("views", true).expect("full views skill");
    let index: Value = serde_json::from_str(VIEW_BLOCK_INDEX).expect("block index json");
    let auth_rules = index["families"]["auth"]["authoringRules"]
        .as_array()
        .expect("auth authoring rules")
        .iter()
        .map(|rule| rule.as_str().expect("auth rule"))
        .collect::<Vec<_>>()
        .join(" ");

    assert!(compact.content.contains("Begin with no `Box` nodes"));
    assert!(compact.content.contains("fixed viewport layer"));
    assert!(compact.content.contains("one responsive source tree"));
    assert!(
        full.content
            .contains("Do not use empty Boxes as Grid gutters")
    );
    assert!(
        full.content.contains(
            "Do not wrap `Input`, `Password`, `Phone`, `Pin`, `Button`, `Image`, or `Svg`"
        )
    );
    assert!(
        full.content
            .contains("Separate mobile and desktop form trees")
    );
    assert!(auth_rules.contains("without individual Box wrappers"));
    assert!(auth_rules.contains("advanced auth-layout layer plane"));
}

#[test]
fn view_skill_keeps_same_kind_layout_nesting_only_for_distinct_subgroups() {
    let compact = get_public_skill("views", false).expect("compact views skill");
    let composition = get_public_skill_resource("views", "references/composition.md")
        .expect("composition resource");
    let reference_ui = get_public_skill_resource("views", "references/reference-ui.md")
        .expect("reference UI resource");

    assert!(
        compact
            .content
            .contains("Same-kind nesting is allowed only for a distinct subgroup")
    );
    assert!(
        compact
            .content
            .contains("a column `Flex` may contain one row `Flex`")
    );
    assert!(
        composition
            .content
            .contains("Same-kind layout nesting requires a distinct subgroup")
    );
    assert!(
        composition
            .content
            .contains("A feature row Flex containing a column")
    );
    assert!(
        composition
            .content
            .contains("| Column Flex containing an action-row Flex | Keep both |")
    );
    assert!(
        reference_ui
            .content
            .contains("Same-kind nesting remains only for a distinct subgroup")
    );
}

#[test]
fn view_skill_requires_default_first_minimal_props_for_reference_patterns() {
    let compact = get_public_skill("views", false).expect("compact views skill");
    let components = get_public_skill_resource("views", "references/components.md")
        .expect("components resource");
    let composition = get_public_skill_resource("views", "references/composition.md")
        .expect("composition resource");
    let reference_ui = get_public_skill_resource("views", "references/reference-ui.md")
        .expect("reference UI resource");
    let styles =
        get_public_skill_resource("views", "references/styles.md").expect("styles resource");

    assert!(compact.content.contains("strict prop-admission gate"));
    assert!(
        compact
            .content
            .contains("`Grid` and `Flex` default to zero gap")
    );
    assert!(
        components
            .content
            .contains("If every answer is no, omit the prop")
    );
    assert!(components.content.contains("visibly independent surface"));
    for pattern in [
        "Hero with lead input",
        "Hero with cover and actions",
        "Media features",
        "Icon features",
        "FAQ split",
        "Pricing",
    ] {
        assert!(composition.content.contains(pattern), "missing {pattern}");
    }
    assert!(
        composition
            .content
            .contains("These maps are a prop ceiling for the first pass")
    );
    assert!(
        reference_ui
            .content
            .contains("not as an automatic prop list")
    );
    assert!(
        styles
            .content
            .contains("Admit a local prop only when at least one condition is true")
    );
}

#[test]
fn view_skill_keeps_horizontal_and_vertical_shell_navigation_separate() {
    let compact = get_public_skill("views", false).expect("compact views skill");
    let components = get_public_skill_resource("views", "references/components.md")
        .expect("components resource");
    let composition = get_public_skill_resource("views", "references/composition.md")
        .expect("composition resource");
    let views = get_public_skill_resource("views", "references/views.md").expect("views resource");
    let reference_ui = get_public_skill_resource("views", "references/reference-ui.md")
        .expect("reference UI resource");
    let example = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/embedded/examples/reference-ui/views/components/site-navigation.dowe"
    ));

    assert!(
        compact
            .content
            .contains("`NavMenu` is horizontal shell navigation")
    );
    assert!(
        compact
            .content
            .contains("put a vertical `SideNav` in its `body`")
    );
    assert!(
        compact
            .content
            .contains("AppBar `start`, `center`, and `end` already lay out their direct children")
    );
    assert!(
        components
            .content
            .contains("never use it as the body of a `Drawer`, `Sidebar`")
    );
    assert!(
        components
            .content
            .contains("AppBar regions already provide a horizontal flex row")
    );
    assert!(
        composition
            .content
            .contains("mount a prop-free reusable `SideNav`")
    );
    assert!(
        composition
            .content
            .contains("Do not write `start > Flex` merely to place `Logo` beside `IconButton`")
    );
    assert!(
        views
            .content
            .contains("`SideNav` is vertical and is the navigation child")
    );
    assert!(
        views
            .content
            .contains("`IconButton` as direct region children; never add a wrapper `Flex`")
    );
    assert!(
        reference_ui
            .content
            .contains("`Drawer` containing a vertical `SideNav` in `body`")
    );
    assert!(
        reference_ui
            .content
            .contains("Direct `IconButton` before `Brand` in AppBar `start`")
    );
    assert!(example.contains("SideNav"));
    assert!(!example.contains("NavMenu"));
}

#[test]
fn view_skill_reserves_translation_for_advanced_visual_layers() {
    let compact = get_public_skill("views", false).expect("compact views skill");
    let full = get_public_skill("views", true).expect("full views skill");
    let index: Value = serde_json::from_str(VIEW_BLOCK_INDEX).expect("block index json");
    let app_bar_rules = index["families"]["app-bar"]["authoringRules"]
        .as_array()
        .expect("app bar authoring rules")
        .iter()
        .map(|rule| rule.as_str().expect("app bar rule"))
        .collect::<Vec<_>>()
        .join(" ");

    assert!(
        compact
            .content
            .contains("Zero authored translations is the default")
    );
    assert!(
        compact
            .content
            .contains("Never translate `AppBar`, `Brand`, `NavMenu`, `Drawer`")
    );
    assert!(
        compact
            .content
            .contains("First solve one-axis placement with `Flex`")
    );
    assert!(
        compact
            .content
            .contains("solve shared tracks and responsive")
    );
    assert!(
        full.content
            .contains("Availability is not a layout recommendation")
    );
    assert!(full.content.contains("Screenshot measurements describe"));
    assert!(full.content.contains("Measured `x` and `y` bounds are QA"));
    assert!(app_bar_rules.contains("Do not use translateX or translateY on AppBar"));
}

#[test]
fn view_skill_requires_reference_blueprints_and_visual_qa() {
    let compact = get_public_skill("views", false).expect("compact views skill");
    let full = get_public_skill("views", true).expect("full views skill");
    let script = get_public_skill_resource("views", "scripts/visual_qa.py").expect("script");

    assert!(
        compact
            .content
            .contains(".dowe/visual-qa/<screen>/blueprint.json")
    );
    assert!(full.content.contains("observed` or `inferred"));
    assert!(full.content.contains("`regions[].bounds`"));
    assert!(
        full.content
            .contains("Loading, populated, empty, and error")
    );
    assert!(full.content.contains("## Accessibility review"));
    assert!(full.content.contains("## Theme extraction"));
    assert!(full.content.contains(
        "An image supplied to build or adapt a layout, page, reusable component, or `Section`"
    ));
    assert!(
        full.content
            .contains("keep the existing theme colors unchanged")
    );
    assert!(full.content.contains("kind:\"dynamic\""));
    assert!(full.content.contains("report.json"));
    assert!(full.content.contains("diff.png"));
    assert!(
        script
            .content
            .contains("commands.add_parser(\"self-test\")")
    );
    assert!(
        script
            .content
            .contains("[dowe, \"dev\", \"--target\", \"web\"]")
    );
}

