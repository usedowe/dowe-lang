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
        ViewNode::Splash {
            content, children, ..
        } => ios_merge_children_boundaries([
            ios_children_boundaries(content, neutral_context, parent_horizontal),
            ios_children_boundaries(children, neutral_context, parent_horizontal),
        ]),
        ViewNode::Children => IosChildrenBoundary {
            count: 1,
            valid: neutral_context && !parent_horizontal,
        },
        ViewNode::Scope { children, .. } => {
            ios_children_boundaries(children, neutral_context, parent_horizontal)
        }
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
        ViewNode::Flex { children, .. } => ios_children_boundaries(children, neutral_context, true),
        ViewNode::Brand { props, children } => ios_children_boundaries(
            children,
            neutral_context && props.style.font.is_none() && props.style.text.is_none(),
            true,
        ),
        ViewNode::Banner { props, children } => ios_children_boundaries(
            children,
            neutral_context && props.style.font.is_none() && props.style.text.is_none(),
            false,
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
            let neutral =
                neutral_context && props.style.font.is_none() && props.style.text.is_none();
            ios_merge_children_boundaries([
                ios_children_boundaries(app_bar, neutral, false),
                ios_children_boundaries(start, neutral, true),
                ios_children_boundaries(main, neutral, false),
                ios_children_boundaries(end, neutral, true),
                ios_children_boundaries(bottom_bar, neutral, false),
                ios_children_boundaries(overlays, neutral, false),
            ])
        }
        ViewNode::Tooltip { props, children } => ios_children_boundaries(
            children,
            neutral_context && props.style.style.font.is_none(),
            false,
        ),
        ViewNode::AppBar {
            top, start, center, end, bottom, ..
        }
        | ViewNode::Footer {
            top, start, center, end, bottom, ..
        } => ios_merge_children_boundaries([
            ios_children_boundaries(top, false, false),
            ios_children_boundaries(start, false, true),
            ios_children_boundaries(center, false, true),
            ios_children_boundaries(end, false, true),
            ios_children_boundaries(bottom, false, false),
        ]),
        ViewNode::Card { children, .. } | ViewNode::Badge { children, .. } => {
            ios_children_boundaries(children, false, false)
        }
        ViewNode::Drawer {
            header,
            body,
            footer,
            ..
        } => ios_merge_children_boundaries([
            ios_children_boundaries(header, false, false),
            ios_children_boundaries(body, false, false),
            ios_children_boundaries(footer, false, false),
        ]),
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
        ViewNode::NavMenu { items, .. } => {
            ios_merge_children_boundaries(items.iter().filter_map(|item| match item {
                NavMenuItem::Megamenu { content, .. } => {
                    Some(ios_children_boundaries(content, false, false))
                }
                _ => None,
            }))
        }
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
        ViewNode::Marquee { children, .. } => ios_children_boundaries(children, false, false),
        ViewNode::Collapsible { children, .. } => ios_children_boundaries(children, false, false),
        ViewNode::Each { children, .. } => ios_children_boundaries(children, false, false),
        ViewNode::Button { .. }
        | ViewNode::AvatarGroup { .. }
        | ViewNode::ChatBox { .. }
        | ViewNode::Empty { .. }
        | ViewNode::RichText { .. }
        | ViewNode::Record { .. }
        | ViewNode::ToggleGroup { .. }
        | ViewNode::Countdown { .. }
        | ViewNode::Map { .. }
        | ViewNode::ToggleTheme { .. }
        | ViewNode::SelectTheme { .. }
        | ViewNode::Fab { .. }
        | ViewNode::Input { .. }
        | ViewNode::ComboBox { .. }
        | ViewNode::CsvField { .. }
        | ViewNode::DragDrop { .. }
        | ViewNode::Editor { .. }
        | ViewNode::ImageCropper { .. }
        | ViewNode::Password { .. }
        | ViewNode::Phone { .. }
        | ViewNode::Pin { .. }
        | ViewNode::Textarea { .. }
        | ViewNode::Slider { .. }
        | ViewNode::Dropzone { .. }
        | ViewNode::Select { .. }
        | ViewNode::Audio { .. }
        | ViewNode::Image { .. }
        | ViewNode::Camera { .. }
        | ViewNode::Microphone { .. }
        | ViewNode::Code { .. }
        | ViewNode::Video { .. }
        | ViewNode::Iframe { .. }
        | ViewNode::Device { .. }
        | ViewNode::Canvas { .. }
        | ViewNode::Diagram { .. }
        | ViewNode::Candlestick { .. }
        | ViewNode::ArcChart { .. }
        | ViewNode::AreaChart { .. }
        | ViewNode::BarChart { .. }
        | ViewNode::LineChart { .. }
        | ViewNode::PieChart { .. }
        | ViewNode::Table { .. }
        | ViewNode::Tree { .. }
        | ViewNode::Divider { .. }
        | ViewNode::Title { .. }
        | ViewNode::Text { .. }
        | ViewNode::Alert { .. }
        | ViewNode::Svg { .. }
        | ViewNode::SideNav { .. }
        | ViewNode::RailNav { .. }
        | ViewNode::BottomBar { .. }
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
        | ViewNode::Toggle { .. }
        | ViewNode::TypeWriter { .. } => IosChildrenBoundary::default(),
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
