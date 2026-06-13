fn render_dev_android_tabs(
    props: &TabsProps,
    tabs: &[TabItem],
    parent: &str,
    parent_gap: Option<&str>,
    parent_horizontal: bool,
    counter: &mut usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    context: &ComposeReactiveContext,
    children_method: Option<&str>,
) {
    let root = next_dev_view(counter);
    let list = next_dev_view(counter);
    let panels = next_dev_view(counter);
    let vertical = matches!(
        props.position,
        dowe_components::TabsPosition::Start | dowe_components::TabsPosition::End
    );
    let root_horizontal = if vertical { "true" } else { "false" };
    let list_horizontal = if vertical { "false" } else { "true" };
    let list_radius = match props.variant {
        TabsVariant::Pills => "999f",
        TabsVariant::Solid | TabsVariant::Outlined => "DOWE_RADIUS_BOX",
        TabsVariant::Line | TabsVariant::Ghost => "0f",
    };
    let tab_radius = match props.variant {
        TabsVariant::Pills => "999f",
        _ => "DOWE_RADIUS_UI",
    };
    let active_background = dev_tab_background(props, true, tab_radius);
    let inactive_background = dev_tab_background(props, false, tab_radius);
    let active_content = dev_tabs_active_content(props);
    let inactive_content = dev_tabs_list_content(props);
    let font = dev_font_value(props.style.font.as_ref().or(inherited_font));
    output.push_str(&format!(
        "        LinearLayout {root} = doweContainer({root_horizontal});\n"
    ));
    apply_dev_android_style(&props.style, &root, true, output);
    output.push_str(&dev_add(parent, &root, parent_gap, parent_horizontal));
    output.push_str(&format!(
        "        LinearLayout {list} = doweContainer({list_horizontal});\n        {list}.setGravity(Gravity.CENTER_VERTICAL);\n        {list}.setPadding(doweDp({}), doweDp({}), doweDp({}), doweDp({}));\n",
        if matches!(props.variant, TabsVariant::Line | TabsVariant::Ghost) {
            0
        } else {
            4
        },
        if matches!(props.variant, TabsVariant::Line | TabsVariant::Ghost) {
            0
        } else {
            4
        },
        if matches!(props.variant, TabsVariant::Line | TabsVariant::Ghost) {
            0
        } else {
            4
        },
        if matches!(props.variant, TabsVariant::Line | TabsVariant::Ghost) {
            0
        } else {
            4
        }
    ));
    if !matches!(props.variant, TabsVariant::Line | TabsVariant::Ghost) {
        let background = if dev_tabs_border(props) == "null" {
            format!(
                "doweBackground({}, {list_radius})",
                dev_tabs_list_background(props)
            )
        } else {
            format!(
                "doweInputBackground({}, {}, {list_radius})",
                dev_tabs_list_background(props),
                dev_tabs_border(props)
            )
        };
        output.push_str(&format!("        {list}.setBackground({background});\n"));
    }
    output.push_str(&format!(
        "        FrameLayout {panels} = new FrameLayout(this);\n        {panels}.setLayoutParams(new LinearLayout.LayoutParams({}, ViewGroup.LayoutParams.WRAP_CONTENT{}));\n",
        if vertical { "0" } else { "ViewGroup.LayoutParams.MATCH_PARENT" },
        if vertical { ", 1f" } else { "" }
    ));
    match props.position {
        dowe_components::TabsPosition::Bottom | dowe_components::TabsPosition::End => {
            output.push_str(&dev_add(&root, &panels, None, vertical));
            output.push_str(&dev_add(&root, &list, Some("8"), vertical));
        }
        dowe_components::TabsPosition::Top | dowe_components::TabsPosition::Start => {
            output.push_str(&dev_add(&root, &list, None, vertical));
            output.push_str(&dev_add(&root, &panels, Some("8"), vertical));
        }
    }
    let mut button_names = Vec::new();
    let mut panel_names = Vec::new();
    let current_font = props.style.font.as_ref().or(inherited_font);
    for (index, tab) in tabs.iter().enumerate() {
        let button = next_dev_view(counter);
        let panel = next_dev_view(counter);
        button_names.push(button.clone());
        panel_names.push(panel.clone());
        output.push_str(&format!(
            "        TextView {button} = doweText(\"{}\", {}, 16f, 500, 0f, 1.25f, {font});\n        {button}.setGravity(Gravity.CENTER);\n        {button}.setPadding(doweDp(16), doweDp(6), doweDp(16), doweDp(6));\n        {button}.setTextColor({});\n        {button}.setBackground({});\n",
            escape_java(&tab.label),
            if index == 0 { active_content } else { inactive_content },
            if index == 0 { active_content } else { inactive_content },
            if index == 0 {
                active_background.as_str()
            } else {
                inactive_background.as_str()
            }
        ));
        output.push_str(&dev_add(&list, &button, Some("8"), !vertical));
        output.push_str(&format!(
            "        LinearLayout {panel} = doweContainer(false);\n        {panel}.setVisibility({});\n        {panels}.addView({panel}, new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));\n",
            if index == 0 {
                "View.VISIBLE"
            } else {
                "View.GONE"
            }
        ));
        for child in &tab.children {
            render_dev_android_node(
                child,
                &panel,
                None,
                false,
                counter,
                output,
                current_font,
                None,
                context,
                children_method,
            );
        }
    }
    output.push_str(&format!(
        "        TextView[] {root}Tabs = new TextView[]{{{}}};\n        View[] {root}Panels = new View[]{{{}}};\n",
        button_names.join(", "),
        panel_names.join(", ")
    ));
    for (index, button) in button_names.iter().enumerate() {
        output.push_str(&format!(
            "        {button}.setOnClickListener(v -> {{ for (int index = 0; index < {root}Tabs.length; index++) {{ boolean active = index == {index}; {root}Panels[index].setVisibility(active ? View.VISIBLE : View.GONE); {root}Tabs[index].setTextColor(active ? {active_content} : {inactive_content}); {root}Tabs[index].setBackground(active ? {active_background} : {inactive_background}); }} }});\n"
        ));
    }
}

fn dev_tab_background(props: &TabsProps, active: bool, radius: &str) -> String {
    if active {
        match props.variant {
            TabsVariant::Solid | TabsVariant::Outlined | TabsVariant::Pills => {
                format!(
                    "doweBackground({}, {radius})",
                    dev_tabs_active_background(props)
                )
            }
            TabsVariant::Line => {
                format!(
                    "doweInputBackground(Color.TRANSPARENT, {}, {radius})",
                    dev_tabs_accent(props)
                )
            }
            TabsVariant::Ghost => format!("doweBackground(Color.TRANSPARENT, {radius})"),
        }
    } else {
        format!("doweBackground(Color.TRANSPARENT, {radius})")
    }
}
