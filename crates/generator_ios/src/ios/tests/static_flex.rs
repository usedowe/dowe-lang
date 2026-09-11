use dowe_components::FlexDirection;

fn flex_direction_page(direction: ResponsiveValue<FlexDirection>, wrap: bool) -> String {
    let mut page = route();
    page.layout_tree = ViewNode::Children;
    page.page_tree = ViewNode::Each {
        item: "item".to_string(),
        collection: "items".to_string(),
        key: "item.id".to_string(),
        children: vec![ViewNode::Flex {
            props: dowe_components::LayoutProps {
                direction,
                wrap,
                ..Default::default()
            },
            children: vec![text("Single-render marker")],
        }],
    };
    let output = generate_ios(
        &[page],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    output
        .files
        .into_iter()
        .find(|file| file.relative_path.ends_with("DowePageLoginView.swift"))
        .unwrap()
        .content
}

#[test]
fn static_flex_emits_only_the_reachable_axis_inside_each() {
    for (direction, stack) in [
        (FlexDirection::Row, "HStack("),
        (FlexDirection::Column, "VStack("),
    ] {
        let source = flex_direction_page(ResponsiveValue::scalar(direction), false);
        assert_eq!(source.matches("Single-render marker").count(), 1);
        assert!(source.contains(stack));
        assert!(!source.contains("== DoweFlexDirection.column"));
        assert!(source.contains("ForEach(state.rows(\"items\")) { row in"));
    }
    let source = flex_direction_page(ResponsiveValue::scalar(FlexDirection::Row), true);
    assert!(source.contains("DoweFlowLayout("));
    assert_eq!(source.matches("Single-render marker").count(), 1);
}

#[test]
fn nested_static_flex_keeps_one_reactive_row_subtree() {
    let mut child = text("{item.title}");
    for direction in [FlexDirection::Row, FlexDirection::Column]
        .into_iter()
        .cycle()
        .take(6)
    {
        child = ViewNode::Flex {
            props: dowe_components::LayoutProps {
                direction: ResponsiveValue::scalar(direction),
                ..Default::default()
            },
            children: vec![child],
        };
    }
    let mut page = route();
    page.layout_tree = ViewNode::Children;
    page.page_tree = ViewNode::Each {
        item: "item".to_string(),
        collection: "items".to_string(),
        key: "item.id".to_string(),
        children: vec![child],
    };
    let output = generate_ios(
        &[page],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let page = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePageLoginView.swift"))
        .unwrap();
    assert_eq!(
        page.content
            .matches("state.text(\"item.title\", item: row.value)")
            .count(),
        1
    );
    assert!(!page.content.contains("== DoweFlexDirection.column"));
}

#[test]
fn responsive_flex_specialization_preserves_the_default_before_breakpoints() {
    for (entries, variants) in [
        (vec![(Breakpoint::Md, FlexDirection::Row)], 1),
        (
            vec![
                (Breakpoint::Xs, FlexDirection::Column),
                (Breakpoint::Md, FlexDirection::Column),
            ],
            1,
        ),
        (vec![(Breakpoint::Md, FlexDirection::Column)], 2),
        (
            vec![
                (Breakpoint::Xs, FlexDirection::Column),
                (Breakpoint::Md, FlexDirection::Row),
            ],
            2,
        ),
    ] {
        let direction = ResponsiveValue::ordered(
            entries
                .into_iter()
                .map(|(breakpoint, value)| ResponsiveEntry { breakpoint, value })
                .collect(),
        );
        let source = flex_direction_page(direction, false);
        assert_eq!(source.matches("Single-render marker").count(), variants);
        assert_eq!(
            source.contains("== DoweFlexDirection.column"),
            variants == 2
        );
    }
}
