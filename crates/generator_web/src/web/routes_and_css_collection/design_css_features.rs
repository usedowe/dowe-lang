#[derive(Clone, Copy, Default)]
struct DesignCssFeatures {
    content: bool,
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
                | ViewNode::LineChart { .. }
                | ViewNode::Map { .. }
                | ViewNode::PieChart { .. }
        );
        self.disclosure |= matches!(
            node,
            ViewNode::Accordion { .. }
                | ViewNode::Carousel { .. }
                | ViewNode::Collapsible { .. }
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
