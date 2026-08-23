fn dev_layout_can_split(layout: &ViewNode) -> bool {
    let boundary = dev_children_boundary(layout, true, false);
    boundary.count == 1 && boundary.valid
}

struct DevChildrenBoundary {
    count: usize,
    valid: bool,
}

impl Default for DevChildrenBoundary {
    fn default() -> Self {
        Self {
            count: 0,
            valid: true,
        }
    }
}

fn dev_children_boundary(
    node: &ViewNode,
    neutral_context: bool,
    parent_horizontal: bool,
) -> DevChildrenBoundary {
    match node {
        ViewNode::Splash {
            content, children, ..
        } => dev_merge_children_boundaries([
            dev_children_boundaries(content, neutral_context, parent_horizontal),
            dev_children_boundaries(children, neutral_context, parent_horizontal),
        ]),
        ViewNode::Children => DevChildrenBoundary {
            count: 1,
            valid: neutral_context && !parent_horizontal,
        },
        ViewNode::Scope { children, .. } => {
            dev_children_boundaries(children, neutral_context, parent_horizontal)
        }
        ViewNode::Box { props, children } | ViewNode::Section { props, children } => {
            dev_children_boundaries(
                children,
                neutral_context && props.font.is_none() && props.text.is_none(),
                false,
            )
        }
        ViewNode::Grid { props, children } => dev_children_boundaries(
            children,
            neutral_context && props.style.font.is_none() && props.style.text.is_none(),
            false,
        ),
        ViewNode::Flex { children, .. } => dev_children_boundaries(children, neutral_context, true),
        ViewNode::Brand { props, children } => dev_children_boundaries(
            children,
            neutral_context && props.style.font.is_none() && props.style.text.is_none(),
            true,
        ),
        ViewNode::Banner { props, children } => dev_children_boundaries(
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
            dev_merge_children_boundaries([
                dev_children_boundaries(app_bar, neutral, false),
                dev_children_boundaries(start, neutral, true),
                dev_children_boundaries(main, neutral, false),
                dev_children_boundaries(end, neutral, true),
                dev_children_boundaries(bottom_bar, neutral, false),
                dev_children_boundaries(overlays, neutral, false),
            ])
        }
        ViewNode::Tooltip { props, children } => dev_children_boundaries(
            children,
            neutral_context && props.style.style.font.is_none(),
            false,
        ),
        ViewNode::AppBar {
            top, start, center, end, bottom, ..
        }
        | ViewNode::Footer {
            top, start, center, end, bottom, ..
        } => dev_merge_children_boundaries([
            dev_children_boundaries(top, false, false),
            dev_children_boundaries(start, false, true),
            dev_children_boundaries(center, false, true),
            dev_children_boundaries(end, false, true),
            dev_children_boundaries(bottom, false, false),
        ]),
        ViewNode::Card { children, .. } | ViewNode::Badge { children, .. } => {
            dev_children_boundaries(children, false, false)
        }
        ViewNode::Drawer {
            header,
            body,
            footer,
            ..
        } => dev_merge_children_boundaries([
            dev_children_boundaries(header, false, false),
            dev_children_boundaries(body, false, false),
            dev_children_boundaries(footer, false, false),
        ]),
        ViewNode::Modal {
            header,
            body,
            footer,
            ..
        } => dev_merge_children_boundaries([
            dev_children_boundaries(header, false, false),
            dev_children_boundaries(body, false, false),
            dev_children_boundaries(footer, false, false),
        ]),
        ViewNode::Dropdown {
            trigger,
            header,
            footer,
            ..
        } => dev_merge_children_boundaries([
            dev_children_boundaries(trigger, false, false),
            dev_children_boundaries(header, false, false),
            dev_children_boundaries(footer, false, false),
        ]),
        ViewNode::Tabs { tabs, .. } => dev_merge_children_boundaries(
            tabs.iter()
                .map(|tab| dev_children_boundaries(&tab.children, false, false)),
        ),
        ViewNode::NavMenu { items, .. } => {
            dev_merge_children_boundaries(items.iter().filter_map(|item| match item {
                NavMenuItem::Megamenu { content, .. } => {
                    Some(dev_children_boundaries(content, false, false))
                }
                _ => None,
            }))
        }
        ViewNode::Accordion { items, .. } => dev_merge_children_boundaries(
            items
                .iter()
                .map(|item| dev_children_boundaries(&item.children, false, false)),
        ),
        ViewNode::Carousel { slides, .. } => dev_merge_children_boundaries(
            slides
                .iter()
                .map(|slide| dev_children_boundaries(&slide.children, false, false)),
        ),
        ViewNode::Marquee { children, .. } | ViewNode::Collapsible { children, .. } => {
            dev_children_boundaries(children, false, false)
        }
        ViewNode::Each { children, .. } => dev_children_boundaries(children, false, false),
        ViewNode::Button { .. }
        | ViewNode::AvatarGroup { .. }
        | ViewNode::ChatBox { .. }
        | ViewNode::Empty { .. }
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
        | ViewNode::Candlestick { .. }
        | ViewNode::ArcChart { .. }
        | ViewNode::AreaChart { .. }
        | ViewNode::BarChart { .. }
        | ViewNode::LineChart { .. }
        | ViewNode::PieChart { .. }
        | ViewNode::Table { .. }
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
        | ViewNode::RichText { .. }
        | ViewNode::Record { .. }
        | ViewNode::ToggleGroup { .. }
        | ViewNode::Countdown { .. }
        | ViewNode::Map { .. }
        | ViewNode::TypeWriter { .. } => DevChildrenBoundary::default(),
    }
}

fn dev_children_boundaries(
    children: &[ViewNode],
    neutral_context: bool,
    parent_horizontal: bool,
) -> DevChildrenBoundary {
    dev_merge_children_boundaries(
        children
            .iter()
            .map(|child| dev_children_boundary(child, neutral_context, parent_horizontal)),
    )
}

fn dev_merge_children_boundaries(
    boundaries: impl IntoIterator<Item = DevChildrenBoundary>,
) -> DevChildrenBoundary {
    boundaries.into_iter().fold(
        DevChildrenBoundary {
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

