struct IosLayoutSection<'a> {
    node: &'a ViewNode,
    flow: NativeFlow,
}

fn ios_layout_sections(layout: &ViewNode) -> Vec<IosLayoutSection<'_>> {
    let mut sections = Vec::new();
    collect_ios_layout_sections(layout, NativeFlow::Block, true, true, &mut sections);
    sections
}

fn collect_ios_layout_sections<'a>(
    node: &'a ViewNode,
    flow: NativeFlow,
    neutral_context: bool,
    root: bool,
    sections: &mut Vec<IosLayoutSection<'a>>,
) {
    if !root
        && neutral_context
        && ios_layout_section_candidate(node)
        && !ios_layout_contains_children(node)
    {
        sections.push(IosLayoutSection { node, flow });
    }

    match node {
        ViewNode::Scope {
            signals,
            actions,
            children,
        } => collect_ios_layout_section_children(
            children,
            flow,
            neutral_context && signals.is_empty() && actions.is_empty(),
            sections,
        ),
        ViewNode::Box { props, children } | ViewNode::Section { props, children } => {
            collect_ios_layout_section_children(
                children,
                NativeFlow::Block,
                neutral_context && props.font.is_none(),
                sections,
            )
        }
        ViewNode::Flex { props, children } => collect_ios_layout_section_children(
            children,
            NativeFlow::Inline,
            neutral_context && props.style.font.is_none(),
            sections,
        ),
        ViewNode::Grid { props, children } => {
            collect_ios_layout_section_children(
                children,
                NativeFlow::Block,
                neutral_context && props.style.font.is_none(),
                sections,
            )
        }
        ViewNode::Card { props, children } => {
            collect_ios_layout_section_children(
                children,
                NativeFlow::Block,
                neutral_context && props.style.font.is_none(),
                sections,
            )
        }
        ViewNode::Scaffold {
            props,
            app_bar,
            start,
            main,
            end,
            bottom_bar,
        } => {
            let neutral = neutral_context && props.style.font.is_none();
            collect_ios_layout_section_children(
                app_bar,
                NativeFlow::Block,
                neutral,
                sections,
            );
            collect_ios_layout_section_children(
                start,
                NativeFlow::Block,
                neutral,
                sections,
            );
            collect_ios_layout_section_children(
                main,
                NativeFlow::Block,
                neutral,
                sections,
            );
            collect_ios_layout_section_children(end, NativeFlow::Block, neutral, sections);
            collect_ios_layout_section_children(
                bottom_bar,
                NativeFlow::Block,
                neutral,
                sections,
            );
        }
        ViewNode::AppBar {
            props,
            start,
            center,
            end,
        }
        | ViewNode::Footer {
            props,
            start,
            center,
            end,
        }
        | ViewNode::BottomBar {
            props,
            start,
            center,
            end,
        } => {
            let neutral = neutral_context && props.style.style.font.is_none();
            collect_ios_layout_section_children(
                start,
                NativeFlow::Inline,
                neutral,
                sections,
            );
            collect_ios_layout_section_children(
                center,
                NativeFlow::Inline,
                neutral,
                sections,
            );
            collect_ios_layout_section_children(end, NativeFlow::Inline, neutral, sections);
        }
        ViewNode::Tooltip { props, children } => collect_ios_layout_section_children(
            children,
            NativeFlow::Block,
            neutral_context && props.style.style.font.is_none(),
            sections,
        ),
        ViewNode::Tabs { props, tabs } => {
            let neutral = neutral_context && props.style.font.is_none();
            for tab in tabs {
                collect_ios_layout_section_children(
                    &tab.children,
                    NativeFlow::Block,
                    neutral,
                    sections,
                );
            }
        }
        _ => {}
    }
}

fn collect_ios_layout_section_children<'a>(
    children: &'a [ViewNode],
    flow: NativeFlow,
    neutral_context: bool,
    sections: &mut Vec<IosLayoutSection<'a>>,
) {
    for child in children {
        collect_ios_layout_sections(child, flow, neutral_context, false, sections);
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
            | ViewNode::Sidebar { .. }
            | ViewNode::NavMenu { .. }
            | ViewNode::Tabs { .. }
            | ViewNode::Scaffold { .. }
    )
}

fn ios_layout_contains_children(node: &ViewNode) -> bool {
    ios_children_boundary(node, true, false).count > 0
}
