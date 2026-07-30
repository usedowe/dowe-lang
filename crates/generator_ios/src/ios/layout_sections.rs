struct IosLayoutSection<'a> {
    node: &'a ViewNode,
    flow: NativeFlow,
}

fn ios_layout_sections(layout: &ViewNode) -> Vec<IosLayoutSection<'_>> {
    let mut sections = Vec::new();
    let mut bindings = IosLayoutBindings::default();
    ios_collect_scope_bindings(layout, &mut bindings);
    collect_ios_layout_sections(
        layout,
        NativeFlow::Block,
        true,
        true,
        &bindings,
        &mut sections,
    );
    sections
}

fn collect_ios_layout_sections<'a>(
    node: &'a ViewNode,
    flow: NativeFlow,
    neutral_context: bool,
    root: bool,
    bindings: &IosLayoutBindings,
    sections: &mut Vec<IosLayoutSection<'a>>,
) {
    if !root
        && neutral_context
        && ios_layout_section_candidate(node)
        && !ios_layout_contains_children(node)
        && !ios_node_references_layout_bindings(node, bindings)
    {
        sections.push(IosLayoutSection { node, flow });
    }

    match node {
        ViewNode::Splash {
            content, children, ..
        } => {
            collect_ios_layout_section_children(content, flow, neutral_context, bindings, sections);
            collect_ios_layout_section_children(
                children,
                flow,
                neutral_context,
                bindings,
                sections,
            );
        }
        ViewNode::Scope { children, .. } => {
            collect_ios_layout_section_children(children, flow, neutral_context, bindings, sections)
        }
        ViewNode::Box { props, children } | ViewNode::Section { props, children } => {
            collect_ios_layout_section_children(
                children,
                NativeFlow::Block,
                neutral_context && props.font.is_none(),
                bindings,
                sections,
            )
        }
        ViewNode::Flex { props, children } => collect_ios_layout_section_children(
            children,
            NativeFlow::Inline,
            neutral_context && props.style.font.is_none(),
            bindings,
            sections,
        ),
        ViewNode::Grid { props, children } => collect_ios_layout_section_children(
            children,
            NativeFlow::Block,
            neutral_context && props.style.font.is_none(),
            bindings,
            sections,
        ),
        ViewNode::Card { props, children } => collect_ios_layout_section_children(
            children,
            NativeFlow::Block,
            neutral_context && props.style.font.is_none(),
            bindings,
            sections,
        ),
        ViewNode::Scaffold {
            props,
            app_bar,
            start,
            main,
            end,
            bottom_bar,
            overlays,
        } => {
            let neutral = neutral_context && props.style.font.is_none();
            collect_ios_layout_section_children(
                app_bar,
                NativeFlow::Block,
                neutral,
                bindings,
                sections,
            );
            collect_ios_layout_section_children(
                start,
                NativeFlow::Block,
                neutral,
                bindings,
                sections,
            );
            collect_ios_layout_section_children(
                main,
                NativeFlow::Block,
                neutral,
                bindings,
                sections,
            );
            collect_ios_layout_section_children(
                end,
                NativeFlow::Block,
                neutral,
                bindings,
                sections,
            );
            collect_ios_layout_section_children(
                bottom_bar,
                NativeFlow::Block,
                neutral,
                bindings,
                sections,
            );
            collect_ios_layout_section_children(
                overlays,
                NativeFlow::Block,
                neutral,
                bindings,
                sections,
            );
        }
        ViewNode::AppBar {
            props,
            top,
            start,
            center,
            end,
            bottom,
        }
        | ViewNode::Footer {
            props,
            top,
            start,
            center,
            end,
            bottom,
        } => {
            let neutral = neutral_context && props.style.style.font.is_none();
            collect_ios_layout_section_children(
                top,
                NativeFlow::Block,
                neutral,
                bindings,
                sections,
            );
            collect_ios_layout_section_children(
                start,
                NativeFlow::Inline,
                neutral,
                bindings,
                sections,
            );
            collect_ios_layout_section_children(
                center,
                NativeFlow::Inline,
                neutral,
                bindings,
                sections,
            );
            collect_ios_layout_section_children(
                end,
                NativeFlow::Inline,
                neutral,
                bindings,
                sections,
            );
            collect_ios_layout_section_children(
                bottom,
                NativeFlow::Block,
                neutral,
                bindings,
                sections,
            );
        }
        ViewNode::BottomBar { .. } => {}
        ViewNode::Drawer {
            props,
            header,
            body,
            footer,
        } => {
            let neutral = neutral_context && props.style.style.font.is_none();
            collect_ios_layout_section_slots(header, body, footer, neutral, bindings, sections);
        }
        ViewNode::Sidebar {
            props,
            header,
            body,
            footer,
        } => {
            let neutral = neutral_context && props.style.style.font.is_none();
            collect_ios_layout_section_slots(header, body, footer, neutral, bindings, sections);
        }
        ViewNode::Tooltip { props, children } => collect_ios_layout_section_children(
            children,
            NativeFlow::Block,
            neutral_context && props.style.style.font.is_none(),
            bindings,
            sections,
        ),
        ViewNode::Tabs { props, tabs } => {
            let neutral = neutral_context && props.style.font.is_none();
            for tab in tabs {
                collect_ios_layout_section_children(
                    &tab.children,
                    NativeFlow::Block,
                    neutral,
                    bindings,
                    sections,
                );
            }
        }
        _ => {}
    }
}

fn collect_ios_layout_section_slots<'a>(
    header: &'a [ViewNode],
    body: &'a [ViewNode],
    footer: &'a [ViewNode],
    neutral_context: bool,
    bindings: &IosLayoutBindings,
    sections: &mut Vec<IosLayoutSection<'a>>,
) {
    collect_ios_layout_section_children(
        header,
        NativeFlow::Block,
        neutral_context,
        bindings,
        sections,
    );
    collect_ios_layout_section_children(
        body,
        NativeFlow::Block,
        neutral_context,
        bindings,
        sections,
    );
    collect_ios_layout_section_children(
        footer,
        NativeFlow::Block,
        neutral_context,
        bindings,
        sections,
    );
}

fn collect_ios_layout_section_children<'a>(
    children: &'a [ViewNode],
    flow: NativeFlow,
    neutral_context: bool,
    bindings: &IosLayoutBindings,
    sections: &mut Vec<IosLayoutSection<'a>>,
) {
    for child in children {
        collect_ios_layout_sections(child, flow, neutral_context, false, bindings, sections);
    }
}

fn ios_layout_section_candidate(node: &ViewNode) -> bool {
    matches!(
        node,
        ViewNode::Box { .. }
            | ViewNode::Section { .. }
            | ViewNode::Flex { .. }
            | ViewNode::Grid { .. }
            | ViewNode::Card { .. }
            | ViewNode::Button { .. }
            | ViewNode::AppBar { .. }
            | ViewNode::Footer { .. }
            | ViewNode::BottomBar { .. }
            | ViewNode::SideNav { .. }
            | ViewNode::RailNav { .. }
            | ViewNode::Sidebar { .. }
            | ViewNode::NavMenu { .. }
            | ViewNode::Tabs { .. }
            | ViewNode::Scaffold { .. }
    )
}

fn ios_layout_contains_children(node: &ViewNode) -> bool {
    ios_children_boundary(node, true, false).count > 0
}
