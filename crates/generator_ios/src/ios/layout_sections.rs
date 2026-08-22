#[derive(Clone)]
struct IosLayoutScope<'a> {
    constants: &'a [ViewConstant],
    signals: &'a [ViewSignal],
    actions: &'a [ViewAction],
}

struct IosLayoutSection<'a> {
    node: &'a ViewNode,
    flow: NativeFlow,
    scopes: Vec<IosLayoutScope<'a>>,
}

fn ios_layout_sections(layout: &ViewNode) -> Vec<IosLayoutSection<'_>> {
    let mut sections = Vec::new();
    collect_ios_layout_sections(
        layout,
        NativeFlow::Block,
        true,
        true,
        &Vec::new(),
        &mut sections,
    );
    sections
}

fn collect_ios_layout_sections<'a>(
    node: &'a ViewNode,
    flow: NativeFlow,
    neutral_context: bool,
    root: bool,
    scopes: &Vec<IosLayoutScope<'a>>,
    sections: &mut Vec<IosLayoutSection<'a>>,
) {
    if !root
        && neutral_context
        && ios_layout_section_candidate(node)
        && !ios_layout_contains_children(node)
    {
        sections.push(IosLayoutSection {
            node,
            flow,
            scopes: scopes.clone(),
        });
    }

    match node {
        ViewNode::Splash {
            content, children, ..
        } => {
            collect_ios_layout_section_children(content, flow, neutral_context, scopes, sections);
            collect_ios_layout_section_children(children, flow, neutral_context, scopes, sections);
        }
        ViewNode::Scope {
            constants,
            signals,
            actions,
            children,
        } => {
            let mut nested = scopes.clone();
            nested.push(IosLayoutScope {
                constants,
                signals,
                actions,
            });
            collect_ios_layout_section_children(children, flow, neutral_context, &nested, sections)
        }
        ViewNode::Box { props, children } | ViewNode::Section { props, children } => {
            collect_ios_layout_section_children(
                children,
                NativeFlow::Block,
                neutral_context && props.font.is_none(),
                scopes,
                sections,
            )
        }
        ViewNode::Flex { props, children } => collect_ios_layout_section_children(
            children,
            NativeFlow::Inline,
            neutral_context && props.style.font.is_none(),
            scopes,
            sections,
        ),
        ViewNode::Grid { props, children } => collect_ios_layout_section_children(
            children,
            NativeFlow::Block,
            neutral_context && props.style.font.is_none(),
            scopes,
            sections,
        ),
        ViewNode::Card { props, children } => collect_ios_layout_section_children(
            children,
            NativeFlow::Block,
            neutral_context && props.style.font.is_none(),
            scopes,
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
                scopes,
                sections,
            );
            collect_ios_layout_section_children(start, NativeFlow::Block, neutral, scopes, sections);
            collect_ios_layout_section_children(main, NativeFlow::Block, neutral, scopes, sections);
            collect_ios_layout_section_children(end, NativeFlow::Block, neutral, scopes, sections);
            collect_ios_layout_section_children(
                bottom_bar,
                NativeFlow::Block,
                neutral,
                scopes,
                sections,
            );
            collect_ios_layout_section_children(
                overlays,
                NativeFlow::Block,
                neutral,
                scopes,
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
            collect_ios_layout_section_children(top, NativeFlow::Block, neutral, scopes, sections);
            collect_ios_layout_section_children(
                start,
                NativeFlow::Inline,
                neutral,
                scopes,
                sections,
            );
            collect_ios_layout_section_children(
                center,
                NativeFlow::Inline,
                neutral,
                scopes,
                sections,
            );
            collect_ios_layout_section_children(end, NativeFlow::Inline, neutral, scopes, sections);
            collect_ios_layout_section_children(
                bottom,
                NativeFlow::Block,
                neutral,
                scopes,
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
            collect_ios_layout_section_slots(header, body, footer, neutral, scopes, sections);
        }
        ViewNode::Sidebar {
            props,
            header,
            body,
            footer,
        } => {
            let neutral = neutral_context && props.style.style.font.is_none();
            collect_ios_layout_section_slots(header, body, footer, neutral, scopes, sections);
        }
        ViewNode::Tooltip { props, children } => collect_ios_layout_section_children(
            children,
            NativeFlow::Block,
            neutral_context && props.style.style.font.is_none(),
            scopes,
            sections,
        ),
        ViewNode::Tabs { props, tabs } => {
            let neutral = neutral_context && props.style.font.is_none();
            for tab in tabs {
                collect_ios_layout_section_children(
                    &tab.children,
                    NativeFlow::Block,
                    neutral,
                    scopes,
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
    scopes: &Vec<IosLayoutScope<'a>>,
    sections: &mut Vec<IosLayoutSection<'a>>,
) {
    collect_ios_layout_section_children(
        header,
        NativeFlow::Block,
        neutral_context,
        scopes,
        sections,
    );
    collect_ios_layout_section_children(body, NativeFlow::Block, neutral_context, scopes, sections);
    collect_ios_layout_section_children(
        footer,
        NativeFlow::Block,
        neutral_context,
        scopes,
        sections,
    );
}

fn collect_ios_layout_section_children<'a>(
    children: &'a [ViewNode],
    flow: NativeFlow,
    neutral_context: bool,
    scopes: &Vec<IosLayoutScope<'a>>,
    sections: &mut Vec<IosLayoutSection<'a>>,
) {
    for child in children {
        collect_ios_layout_sections(child, flow, neutral_context, false, scopes, sections);
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
