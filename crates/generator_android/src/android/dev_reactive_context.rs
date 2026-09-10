struct DevReactiveRoute {
    initial: Vec<String>,
    metadata: Vec<String>,
    forms: Vec<String>,
    actions: Vec<String>,
    init: Vec<String>,
    autoload: Vec<String>,
}

fn dev_reactive_route(tree: &ViewNode) -> DevReactiveRoute {
    let mut initial = Vec::new();
    let mut metadata = Vec::new();
    let forms = collect_view_forms(tree)
        .iter()
        .map(|form| java_form_value(form, &dev_reactive_context(tree)))
        .collect();
    let mut actions = Vec::new();
    let mut init = Vec::new();
    let mut autoload = Vec::new();
    collect_dev_reactive(
        tree,
        &ComposeReactiveContext::default(),
        &mut initial,
        &mut metadata,
        &mut actions,
        &mut init,
        &mut autoload,
    );
    DevReactiveRoute {
        initial,
        metadata,
        forms,
        actions,
        init,
        autoload,
    }
}

fn dev_reactive_context(tree: &ViewNode) -> ComposeReactiveContext {
    match tree {
        ViewNode::Scope {
            constants,
            signals,
            actions,
            ..
        } => ComposeReactiveContext::default().with_scope(constants, signals, actions),
        _ => ComposeReactiveContext::default(),
    }
}

fn java_form_value(form: &ViewForm, context: &ComposeReactiveContext) -> String {
    let signal = escape_java(&context.signal_path(&form.signal));
    let fields = form
        .fields
        .iter()
        .map(|field| {
            let rules = field
                .rules
                .iter()
                .map(|rule| {
                    format!(
                        "{{\"{}\", {}, \"{}\"}}",
                        escape_java(rule.kind.name()),
                        rule.kind
                            .argument()
                            .map(|value| format!("\"{}\"", escape_java(&value)))
                            .unwrap_or_else(|| "null".to_string()),
                        escape_java(&rule.message)
                    )
                })
                .collect::<Vec<_>>()
                .join(", ");
            format!(
                "new DoweFormFieldMetadata(\"{}\", {}, new String[][] {{ {} }})",
                escape_java(&field.path),
                matches!(field.kind, ViewFormFieldKind::Boolean),
                rules
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "dowePutForm(\"{}\", new DoweFormFieldMetadata[] {{ {} }});",
        signal, fields
    )
}

fn collect_dev_reactive(
    node: &ViewNode,
    context: &ComposeReactiveContext,
    initial: &mut Vec<String>,
    metadata: &mut Vec<String>,
    actions: &mut Vec<String>,
    init: &mut Vec<String>,
    autoload: &mut Vec<String>,
) {
    match node {
        ViewNode::Scope {
            constants,
            signals,
            actions: scope_actions,
            children,
        } => {
            let context = context.with_scope(constants, signals, scope_actions);
            initial.extend(constants.iter().map(|constant| {
                format!(
                    "dowePutInitial(\"{}\", {});",
                    escape_java(&constant.id),
                    java_signal_value(&constant.value)
                )
            }));
            initial.extend(signals.iter().map(|signal| {
                format!(
                    "dowePutInitial(\"{}\", {});",
                    escape_java(&signal.id),
                    java_signal_value(&signal.initial)
                )
            }));
            metadata.extend(signals.iter().map(|signal| {
                format!(
                    "dowePutSignalMetadata(\"{}\", \"{}\", \"{}\", \"{}\");",
                    escape_java(&signal.id),
                    escape_java(&signal.storage_key),
                    signal.scope.as_str(),
                    signal.storage.as_str()
                )
            }));
            for action in scope_actions {
                actions.push(format!(
                    "doweActions.put(\"{}\", {});",
                    escape_java(&action.id),
                    java_action_value(action, &context)
                ));
                if action.is_init() {
                    init.push(action.id.clone());
                } else if action_autoloads(action) {
                    autoload.push(action.id.clone());
                }
            }
            for child in children {
                collect_dev_reactive(child, &context, initial, metadata, actions, init, autoload);
            }
        }
        ViewNode::Splash {
            content, children, ..
        } => {
            for child in content.iter().chain(children) {
                collect_dev_reactive(child, context, initial, metadata, actions, init, autoload);
            }
        }
        ViewNode::Box { children, .. }
        | ViewNode::Section { children, .. }
        | ViewNode::Flex { children, .. }
        | ViewNode::Grid { children, .. }
        | ViewNode::Card { children, .. }
        | ViewNode::Badge { children, .. }
        | ViewNode::Tooltip { children, .. }
        | ViewNode::Brand { children, .. }
        | ViewNode::Banner { children, .. }
        | ViewNode::Button { children, .. }
        | ViewNode::Each { children, .. } => {
            for child in children {
                collect_dev_reactive(child, context, initial, metadata, actions, init, autoload);
            }
        }
        ViewNode::Drawer {
            header,
            body,
            footer,
            ..
        } => {
            for child in header.iter().chain(body).chain(footer) {
                collect_dev_reactive(child, context, initial, metadata, actions, init, autoload);
            }
        }
        ViewNode::Modal {
            header,
            body,
            footer,
            ..
        } => {
            for child in header.iter().chain(body).chain(footer) {
                collect_dev_reactive(child, context, initial, metadata, actions, init, autoload);
            }
        }
        ViewNode::Dropdown {
            trigger,
            header,
            footer,
            ..
        } => {
            for child in trigger.iter().chain(header).chain(footer) {
                collect_dev_reactive(child, context, initial, metadata, actions, init, autoload);
            }
        }
        ViewNode::AppBar {
            top,
            start,
            center,
            end,
            bottom,
            ..
        }
        | ViewNode::Footer {
            top,
            start,
            center,
            end,
            bottom,
            ..
        } => {
            for child in top.iter().chain(start).chain(center).chain(end).chain(bottom) {
                collect_dev_reactive(child, context, initial, metadata, actions, init, autoload);
            }
        }
        ViewNode::BottomBar { .. } => {}
        ViewNode::Tabs { tabs, .. } => {
            for tab in tabs {
                for child in &tab.children {
                    collect_dev_reactive(child, context, initial, metadata, actions, init, autoload);
                }
            }
        }
        ViewNode::Accordion { items, .. } => {
            for item in items {
                for child in &item.children {
                    collect_dev_reactive(child, context, initial, metadata, actions, init, autoload);
                }
            }
        }
        ViewNode::Carousel { slides, .. } => {
            for slide in slides {
                for child in &slide.children {
                    collect_dev_reactive(child, context, initial, metadata, actions, init, autoload);
                }
            }
        }
        ViewNode::Marquee { children, .. } | ViewNode::Collapsible { children, .. } => {
            for child in children {
                collect_dev_reactive(child, context, initial, metadata, actions, init, autoload);
            }
        }
        ViewNode::NavMenu { items, .. } => {
            for item in items {
                if let dowe_components::NavMenuItem::Megamenu { content, .. } = item {
                    for child in content {
                        collect_dev_reactive(child, context, initial, metadata, actions, init, autoload);
                    }
                }
            }
        }
        ViewNode::Scaffold {
            app_bar,
            start,
            main,
            end,
            bottom_bar,
            overlays,
            ..
        } => {
            for child in app_bar
                .iter()
                .chain(start)
                .chain(main)
                .chain(end)
                .chain(bottom_bar)
                .chain(overlays)
            {
                collect_dev_reactive(child, context, initial, metadata, actions, init, autoload);
            }
        }
        ViewNode::Input { .. }
        | ViewNode::ComboBox { .. }
        | ViewNode::CsvField { .. }
        | ViewNode::DragDrop { .. }
        | ViewNode::Editor { .. }
        | ViewNode::ImageCropper { .. }
        | ViewNode::Password { .. }
        | ViewNode::Phone { .. }
        | ViewNode::Pin { .. }
        | ViewNode::Textarea { .. }
        | ViewNode::AvatarGroup { .. }
        | ViewNode::ChatBox { .. }
        | ViewNode::Empty { .. }
        | ViewNode::ToggleTheme { .. }
        | ViewNode::SelectTheme { .. }
        | ViewNode::Fab { .. }
        | ViewNode::Slider { .. }
        | ViewNode::Dropzone { .. }
        | ViewNode::Select { .. }
        | ViewNode::Audio { .. }
        | ViewNode::Image { .. }
        | ViewNode::Camera { .. }
        | ViewNode::Microphone { .. }
        | ViewNode::Checkbox { .. }
        | ViewNode::Color { .. }
        | ViewNode::Date { .. }
        | ViewNode::DateRange { .. }
        | ViewNode::RadioGroup { .. }
        | ViewNode::Toggle { .. }
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
        | ViewNode::Avatar { .. }
        | ViewNode::Chip { .. }
        | ViewNode::Skeleton { .. }
        | ViewNode::AlertDialog { .. }
        | ViewNode::Toast { .. }
        | ViewNode::Command { .. }
        | ViewNode::Svg { .. }
        | ViewNode::SideNav { .. }
        | ViewNode::RailNav { .. }
        | ViewNode::Sidebar { .. }
        | ViewNode::TypeWriter { .. }
        | ViewNode::RichText { .. }
        | ViewNode::Record { .. }
        | ViewNode::ToggleGroup { .. }
        | ViewNode::Countdown { .. }
        | ViewNode::Map { .. }
        | ViewNode::Children => {}
    }
}

