fn reusable_ios_layouts(routes: &[ViewRoute]) -> (Vec<&ViewNode>, Vec<Option<usize>>) {
    let mut layouts = Vec::new();
    let mut route_layouts = Vec::new();
    for route in routes {
        if !ios_layout_can_split(&route.layout_tree) {
            route_layouts.push(None);
            continue;
        }
        let index = layouts
            .iter()
            .position(|layout| *layout == &route.layout_tree)
            .unwrap_or_else(|| {
                layouts.push(&route.layout_tree);
                layouts.len() - 1
            });
        route_layouts.push(Some(index));
    }
    (layouts, route_layouts)
}

fn ios_layout_can_split(layout: &ViewNode) -> bool {
    let boundary = ios_children_boundary(layout, true, false);
    boundary.count == 1 && boundary.valid
}

struct IosChildrenBoundary {
    count: usize,
    valid: bool,
}

impl Default for IosChildrenBoundary {
    fn default() -> Self {
        Self {
            count: 0,
            valid: true,
        }
    }
}

fn ios_children_boundary(
    node: &ViewNode,
    neutral_context: bool,
    parent_horizontal: bool,
) -> IosChildrenBoundary {
    match node {
        ViewNode::Children => IosChildrenBoundary {
            count: 1,
            valid: neutral_context && !parent_horizontal,
        },
        ViewNode::Scope {
            signals,
            actions,
            children,
        } => ios_children_boundaries(
            children,
            neutral_context && signals.is_empty() && actions.is_empty(),
            parent_horizontal,
        ),
        ViewNode::Box { props, children } | ViewNode::Section { props, children } => {
            ios_children_boundaries(
                children,
                neutral_context && props.font.is_none() && props.text.is_none(),
                false,
            )
        }
        ViewNode::Grid { props, children } => ios_children_boundaries(
            children,
            neutral_context && props.style.font.is_none() && props.style.text.is_none(),
            false,
        ),
        ViewNode::Flex { children, .. } => {
            ios_children_boundaries(children, neutral_context, true)
        }
        ViewNode::Scaffold {
            props,
            app_bar,
            start,
            main,
            end,
            bottom_bar,
        } => {
            let neutral =
                neutral_context && props.style.font.is_none() && props.style.text.is_none();
            ios_merge_children_boundaries([
                ios_children_boundaries(app_bar, neutral, false),
                ios_children_boundaries(start, neutral, true),
                ios_children_boundaries(main, neutral, false),
                ios_children_boundaries(end, neutral, true),
                ios_children_boundaries(bottom_bar, neutral, false),
            ])
        }
        ViewNode::Tooltip { props, children } => ios_children_boundaries(
            children,
            neutral_context && props.style.style.font.is_none(),
            false,
        ),
        ViewNode::AppBar {
            start, center, end, ..
        }
        | ViewNode::Footer {
            start, center, end, ..
        }
        | ViewNode::BottomBar {
            start, center, end, ..
        } => ios_merge_children_boundaries([
            ios_children_boundaries(start, false, true),
            ios_children_boundaries(center, false, true),
            ios_children_boundaries(end, false, true),
        ]),
        ViewNode::Card { children, .. }
        | ViewNode::Drawer { children, .. }
        | ViewNode::Badge { children, .. } => ios_children_boundaries(children, false, false),
        ViewNode::Modal {
            header,
            body,
            footer,
            ..
        } => ios_merge_children_boundaries([
            ios_children_boundaries(header, false, false),
            ios_children_boundaries(body, false, false),
            ios_children_boundaries(footer, false, false),
        ]),
        ViewNode::Dropdown {
            trigger,
            header,
            footer,
            ..
        } => ios_merge_children_boundaries([
            ios_children_boundaries(trigger, false, false),
            ios_children_boundaries(header, false, false),
            ios_children_boundaries(footer, false, false),
        ]),
        ViewNode::Tabs { tabs, .. } => ios_merge_children_boundaries(
            tabs.iter()
                .map(|tab| ios_children_boundaries(&tab.children, false, false)),
        ),
        ViewNode::NavMenu { items, .. } => ios_merge_children_boundaries(
            items.iter().filter_map(|item| match item {
                NavMenuItem::Megamenu { content, .. } => {
                    Some(ios_children_boundaries(content, false, false))
                }
                _ => None,
            }),
        ),
        ViewNode::Accordion { items, .. } => ios_merge_children_boundaries(
            items
                .iter()
                .map(|item| ios_children_boundaries(&item.children, false, false)),
        ),
        ViewNode::Carousel { slides, .. } => ios_merge_children_boundaries(
            slides
                .iter()
                .map(|slide| ios_children_boundaries(&slide.children, false, false)),
        ),
        ViewNode::Each { children, .. } => ios_children_boundaries(children, false, false),
        ViewNode::Button { .. }
        | ViewNode::Input { .. }
        | ViewNode::Select { .. }
        | ViewNode::Audio { .. }
        | ViewNode::Image { .. }
        | ViewNode::Code { .. }
        | ViewNode::Video { .. }
        | ViewNode::Candlestick { .. }
        | ViewNode::Table { .. }
        | ViewNode::Divider { .. }
        | ViewNode::Title { .. }
        | ViewNode::Text { .. }
        | ViewNode::Alert { .. }
        | ViewNode::Svg { .. }
        | ViewNode::SideNav { .. }
        | ViewNode::Sidebar { .. }
        | ViewNode::Avatar { .. }
        | ViewNode::Chip { .. }
        | ViewNode::Skeleton { .. }
        | ViewNode::AlertDialog { .. }
        | ViewNode::Toast { .. }
        | ViewNode::Command { .. }
        | ViewNode::Checkbox { .. }
        | ViewNode::Color { .. }
        | ViewNode::Date { .. }
        | ViewNode::DateRange { .. }
        | ViewNode::RadioGroup { .. }
        | ViewNode::Toggle { .. } => IosChildrenBoundary::default(),
    }
}

fn ios_children_boundaries(
    children: &[ViewNode],
    neutral_context: bool,
    parent_horizontal: bool,
) -> IosChildrenBoundary {
    ios_merge_children_boundaries(
        children
            .iter()
            .map(|child| ios_children_boundary(child, neutral_context, parent_horizontal)),
    )
}

fn ios_merge_children_boundaries(
    boundaries: impl IntoIterator<Item = IosChildrenBoundary>,
) -> IosChildrenBoundary {
    boundaries.into_iter().fold(
        IosChildrenBoundary {
            count: 0,
            valid: true,
        },
        |mut combined, boundary| {
            combined.count += boundary.count;
            combined.valid &= boundary.valid;
            combined
        },
    )
}
