fn collect_joined_text(children: &[ViewNode]) -> String {
    let mut texts = Vec::new();
    for child in children {
        collect_texts(child, &mut texts);
    }
    texts.join(" ")
}

fn java_string_array<'a>(values: impl Iterator<Item = &'a str>) -> String {
    let values = values
        .map(|value| format!("\"{}\"", escape_java(value)))
        .collect::<Vec<_>>()
        .join(", ");
    format!("new String[]{{{values}}}")
}

fn java_nullable_string_array<'a>(values: impl Iterator<Item = Option<&'a str>>) -> String {
    let values = values
        .map(|value| {
            value
                .map(|value| format!("\"{}\"", escape_java(value)))
                .unwrap_or_else(|| "null".to_string())
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("new String[]{{{values}}}")
}

fn java_int_array(values: impl Iterator<Item = String>) -> String {
    format!("new int[]{{{}}}", values.collect::<Vec<_>>().join(", "))
}

fn dev_code_token_color(kind: CodeTokenKind, plain: &str) -> String {
    match kind {
        CodeTokenKind::Plain => plain.to_string(),
        CodeTokenKind::Keyword => "DOWE_PRIMARY".to_string(),
        CodeTokenKind::Type => "DOWE_INFO".to_string(),
        CodeTokenKind::String => "DOWE_SUCCESS".to_string(),
        CodeTokenKind::Number => "DOWE_WARNING".to_string(),
        CodeTokenKind::Attribute => "DOWE_TERTIARY".to_string(),
        CodeTokenKind::Comment => "DOWE_MUTED".to_string(),
        CodeTokenKind::Punctuation => "DOWE_DANGER".to_string(),
    }
}

fn collect_texts<'a>(node: &'a ViewNode, output: &mut Vec<&'a str>) {
    match node {
        ViewNode::Scope { children, .. }
        | ViewNode::Each { children, .. }
        | ViewNode::Box { children, .. }
        | ViewNode::Section { children, .. }
        | ViewNode::Flex { children, .. }
        | ViewNode::Grid { children, .. }
        | ViewNode::Card { children, .. }
        | ViewNode::Drawer { children, .. }
        | ViewNode::Badge { children, .. }
        | ViewNode::Tooltip { children, .. }
        | ViewNode::Button { children, .. } => {
            for child in children {
                collect_texts(child, output);
            }
        }
        ViewNode::Modal {
            header,
            body,
            footer,
            ..
        } => {
            for child in header.iter().chain(body).chain(footer) {
                collect_texts(child, output);
            }
        }
        ViewNode::Dropdown {
            trigger,
            header,
            entries,
            footer,
            ..
        } => {
            for child in trigger.iter().chain(header).chain(footer) {
                collect_texts(child, output);
            }
            for entry in entries {
                match entry {
                    OverlayEntry::Item(item) => output.push(&item.label),
                    OverlayEntry::Divider => {}
                }
            }
        }
        ViewNode::Command { entries, .. } => {
            for entry in entries {
                match entry {
                    CommandEntry::Item(item) => output.push(&item.label),
                    CommandEntry::Group { label, items, .. } => {
                        output.push(label);
                        output.extend(items.iter().map(|item| item.label.as_str()));
                    }
                }
            }
        }
        ViewNode::SideNav { items, .. } | ViewNode::Sidebar { items, .. } => {
            for item in items {
                match item {
                    SideNavItem::Header(props) | SideNavItem::Item(props) => {
                        output.push(&props.label);
                    }
                    SideNavItem::Submenu { props, items, .. } => {
                        output.push(&props.label);
                        output.extend(items.iter().map(|props| props.label.as_str()));
                    }
                    SideNavItem::Divider => {}
                }
            }
        }
        ViewNode::NavMenu { items, .. } => {
            for item in items {
                match item {
                    NavMenuItem::Item(props) => output.push(&props.label),
                    NavMenuItem::Submenu { props, items } => {
                        output.push(&props.label);
                        output.extend(items.iter().map(|props| props.label.as_str()));
                    }
                    NavMenuItem::Megamenu { props, content } => {
                        output.push(&props.label);
                        for child in content {
                            collect_texts(child, output);
                        }
                    }
                }
            }
        }
        ViewNode::Tabs { tabs, .. } => {
            for tab in tabs {
                output.push(&tab.label);
                for child in &tab.children {
                    collect_texts(child, output);
                }
            }
        }
        ViewNode::Accordion { items, .. } => {
            for item in items {
                output.push(&item.label);
                for child in &item.children {
                    collect_texts(child, output);
                }
            }
        }
        ViewNode::Carousel { props, slides } => {
            if let Some(title) = props.title.as_deref() {
                output.push(title);
            }
            for slide in slides {
                for child in &slide.children {
                    collect_texts(child, output);
                }
            }
        }
        ViewNode::Marquee { children, .. } => {
            for child in children {
                collect_texts(child, output);
            }
        }
        ViewNode::Collapsible { props, children } => {
            output.push(&props.label);
            for child in children {
                collect_texts(child, output);
            }
        }
        ViewNode::AppBar {
            start, center, end, ..
        }
        | ViewNode::Footer {
            start, center, end, ..
        }
        | ViewNode::BottomBar {
            start, center, end, ..
        } => {
            for child in start.iter().chain(center.iter()).chain(end.iter()) {
                collect_texts(child, output);
            }
        }
        ViewNode::Scaffold {
            app_bar,
            start,
            main,
            end,
            bottom_bar,
            ..
        } => {
            for child in app_bar
                .iter()
                .chain(start)
                .chain(main)
                .chain(end)
                .chain(bottom_bar)
            {
                collect_texts(child, output);
            }
        }
        ViewNode::Title { value, .. } | ViewNode::Text { value, .. } => output.push(value),
        ViewNode::RichText { marks, .. } => {
            output.extend(marks.iter().map(|mark| mark.text.as_str()));
        }
        ViewNode::Alert { props } => output.push(&props.message),
        ViewNode::Avatar { props, .. } => output.push(&props.alt),
        ViewNode::AvatarGroup { items, .. } => {
            for item in items {
                if let Some(alt) = item.alt.as_deref().or(item.name.as_deref()) {
                    output.push(alt);
                }
            }
        }
        ViewNode::ChatBox { props } => {
            output.push(&props.user_name);
            output.push(&props.assistant_name);
        }
        ViewNode::Empty { props } => {
            if let Some(title) = props.title.as_deref() {
                output.push(title);
            }
            if let Some(description) = props.description.as_deref() {
                output.push(description);
            }
        }
        ViewNode::Record { props } => {
            if let Some(label) = props.style.label.as_deref() {
                output.push(label);
            }
            output.push(&props.name);
        }
        ViewNode::ToggleGroup { props, items } => {
            if let Some(label) = props.style.label.as_deref() {
                output.push(label);
            }
            if let Some(aria_label) = props.aria_label.as_deref() {
                output.push(aria_label);
            }
            output.extend(items.iter().map(|item| item.label.as_str()));
        }
        ViewNode::Countdown { props } => {
            output.push(&props.days_label);
            output.push(&props.hours_label);
            output.push(&props.minutes_label);
            output.push(&props.seconds_label);
        }
        ViewNode::Map { markers, .. } => {
            for marker in markers {
                output.push(&marker.id);
                if let Some(label) = marker.label.as_deref() {
                    output.push(label);
                }
                if let Some(popup) = marker.popup.as_deref() {
                    output.push(popup);
                }
            }
        }
        ViewNode::Image { props } => output.push(&props.alt),
        ViewNode::Audio { props } => {
            if let Some(subtitle) = props.subtitle.as_deref() {
                output.push(subtitle);
            }
        }
        ViewNode::Chip { value, .. } => output.push(value),
        ViewNode::AlertDialog { props } => {
            output.push(&props.title);
            output.push(&props.description);
        }
        ViewNode::Toast { props } => {
            if let Some(title) = props.title.as_deref() {
                output.push(title);
            }
            output.push(&props.description);
        }
        ViewNode::ComboBox { props, options } => {
            if let Some(label) = props.style.label.as_deref() {
                output.push(label);
            }
            output.extend(options.iter().map(|option| option.label.as_str()));
        }
        ViewNode::CsvField { props, columns } => {
            output.push(&props.button_text);
            output.push(&props.modal_title);
            output.extend(columns.iter().filter_map(|column| column.label.as_deref()));
        }
        ViewNode::DragDrop {
            props,
            items,
            groups,
        } => {
            output.push(&props.empty_text);
            output.extend(items.iter().filter_map(|item| item.label.as_deref()));
            for group in groups {
                if let Some(title) = group.title.as_deref() {
                    output.push(title);
                }
                output.extend(group.items.iter().filter_map(|item| item.label.as_deref()));
            }
        }
        ViewNode::Editor { props } => {
            if let Some(label) = props.style.label.as_deref() {
                output.push(label);
            }
        }
        ViewNode::ImageCropper { props } => output.push(&props.alt),
        ViewNode::PasswordField { props } => {
            if let Some(label) = props.style.label.as_deref() {
                output.push(label);
            }
        }
        ViewNode::PhoneField { props } => {
            if let Some(label) = props.style.label.as_deref() {
                output.push(label);
            }
        }
        ViewNode::PinField { props } => {
            if let Some(label) = props.style.label.as_deref() {
                output.push(label);
            }
        }
        ViewNode::Textarea { props } => {
            if let Some(label) = props.style.label.as_deref() {
                output.push(label);
            }
        }
        ViewNode::Input { .. }
        | ViewNode::ToggleTheme { .. }
        | ViewNode::Fab { .. }
        | ViewNode::Slider { .. }
        | ViewNode::Dropzone { .. }
        | ViewNode::Select { .. }
        | ViewNode::Checkbox { .. }
        | ViewNode::Color { .. }
        | ViewNode::Date { .. }
        | ViewNode::DateRange { .. }
        | ViewNode::RadioGroup { .. }
        | ViewNode::Toggle { .. }
        | ViewNode::Code { .. }
        | ViewNode::Video { .. }
        | ViewNode::Candlestick { .. }
        | ViewNode::ArcChart { .. }
        | ViewNode::AreaChart { .. }
        | ViewNode::BarChart { .. }
        | ViewNode::LineChart { .. }
        | ViewNode::PieChart { .. }
        | ViewNode::Table { .. }
        | ViewNode::Divider { .. }
        | ViewNode::Skeleton { .. }
        | ViewNode::TypeWriter { .. }
        | ViewNode::Svg { .. }
        | ViewNode::Children => {}
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ComposeFlow {
    Block,
    Inline,
}
