#[derive(Clone, Copy, Default)]
struct DesignCssFeatures {
    content: bool,
    section_center: bool,
    box_center: bool,
    forms: bool,
    media: bool,
    visualization: bool,
    disclosure: bool,
    feedback: bool,
    navigation: bool,
    overlays: bool,
}

impl DesignCssFeatures {
    fn all() -> Self {
        Self {
            content: true,
            section_center: true,
            box_center: true,
            forms: true,
            media: true,
            visualization: true,
            disclosure: true,
            feedback: true,
            navigation: true,
            overlays: true,
        }
    }

    fn collect<'a>(roots: impl IntoIterator<Item = &'a ViewNode>) -> Self {
        let mut features = Self::default();
        let mut pending = roots.into_iter().collect::<Vec<_>>();
        while let Some(node) = pending.pop() {
            features.observe(node);
            for group in dowe_components::node_child_groups(node) {
                pending.extend(group);
            }
        }
        features
    }

    fn observe(&mut self, node: &ViewNode) {
        match node {
            ViewNode::Section { props, .. } => self.section_center |= props.center_x.is_some(),
            ViewNode::Box { props, .. } => self.box_center |= props.center_x.is_some(),
            _ => {}
        }
        if let ViewNode::Scope { actions, .. } = node {
            self.overlays |= actions.iter().any(action_contains_toast);
        }
        self.content |= matches!(
            node,
            ViewNode::Code { .. }
                | ViewNode::Divider { .. }
                | ViewNode::RichText { .. }
                | ViewNode::Table { .. }
        );
        self.forms |= matches!(
            node,
            ViewNode::Checkbox { .. }
                | ViewNode::Color { .. }
                | ViewNode::ComboBox { .. }
                | ViewNode::CsvField { .. }
                | ViewNode::Date { .. }
                | ViewNode::DateRange { .. }
                | ViewNode::DragDrop { .. }
                | ViewNode::Dropzone { .. }
                | ViewNode::Editor { .. }
                | ViewNode::Fab { .. }
                | ViewNode::ImageCropper { .. }
                | ViewNode::Input { .. }
                | ViewNode::Password { .. }
                | ViewNode::Phone { .. }
                | ViewNode::Pin { .. }
                | ViewNode::RadioGroup { .. }
                | ViewNode::Select { .. }
                | ViewNode::SelectTheme { .. }
                | ViewNode::Slider { .. }
                | ViewNode::Textarea { .. }
                | ViewNode::Toggle { .. }
                | ViewNode::ToggleGroup { .. }
                | ViewNode::ToggleTheme { .. }
        );
        self.media |= matches!(
            node,
            ViewNode::Audio { .. }
                | ViewNode::Camera { .. }
                | ViewNode::Device { .. }
                | ViewNode::Iframe { .. }
                | ViewNode::Image { .. }
                | ViewNode::Microphone { .. }
                | ViewNode::Record { .. }
                | ViewNode::Video { .. }
        );
        self.visualization |= matches!(
            node,
            ViewNode::ArcChart { .. }
                | ViewNode::AreaChart { .. }
                | ViewNode::BarChart { .. }
                | ViewNode::Candlestick { .. }
                | ViewNode::Canvas { .. }
                | ViewNode::Countdown { .. }
                | ViewNode::Diagram { .. }
                | ViewNode::LineChart { .. }
                | ViewNode::Map { .. }
                | ViewNode::PieChart { .. }
        );
        self.disclosure |= matches!(
            node,
            ViewNode::Accordion { .. } | ViewNode::Carousel { .. } | ViewNode::Collapsible { .. }
        );
        self.feedback |= matches!(
            node,
            ViewNode::Alert { .. }
                | ViewNode::Avatar { .. }
                | ViewNode::AvatarGroup { .. }
                | ViewNode::Badge { .. }
                | ViewNode::ChatBox { .. }
                | ViewNode::Chip { .. }
                | ViewNode::Empty { .. }
                | ViewNode::Marquee { .. }
                | ViewNode::Skeleton { .. }
                | ViewNode::TypeWriter { .. }
        );
        self.navigation |= matches!(
            node,
            ViewNode::AppBar { .. }
                | ViewNode::BottomBar { .. }
                | ViewNode::Footer { .. }
                | ViewNode::NavMenu { .. }
                | ViewNode::RailNav { .. }
                | ViewNode::Scaffold { .. }
                | ViewNode::SideNav { .. }
                | ViewNode::Sidebar { .. }
                | ViewNode::Tabs { .. }
        );
        self.overlays |= matches!(
            node,
            ViewNode::AlertDialog { .. }
                | ViewNode::Command { .. }
                | ViewNode::Drawer { .. }
                | ViewNode::Dropdown { .. }
                | ViewNode::Modal { .. }
                | ViewNode::Toast { .. }
                | ViewNode::Tooltip { .. }
        );
    }
}

fn action_contains_toast(action: &ViewAction) -> bool {
    match &action.kind {
        ViewActionKind::Sequence(statements) => statements_contain_toast(statements),
        ViewActionKind::Request(_) | ViewActionKind::Assign(_) | ViewActionKind::Reset(_) => false,
    }
}

fn statements_contain_toast(statements: &[ViewFunctionStatement]) -> bool {
    statements.iter().any(|statement| match statement {
        ViewFunctionStatement::Toast(_) => true,
        ViewFunctionStatement::If { success, error, .. } => {
            statements_contain_toast(success) || statements_contain_toast(error)
        }
        ViewFunctionStatement::Validate { .. }
        | ViewFunctionStatement::Request { .. }
        | ViewFunctionStatement::Assign(_)
        | ViewFunctionStatement::Reset(_)
        | ViewFunctionStatement::Redirect { .. } => false,
    })
}
