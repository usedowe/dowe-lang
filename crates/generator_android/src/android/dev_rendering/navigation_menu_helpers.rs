fn render_dev_android_nav_menu(
    props: &NavMenuProps,
    items: &[NavMenuItem],
    parent: &str,
    parent_gap: Option<&str>,
    parent_horizontal: bool,
    counter: &mut usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    context: &ComposeReactiveContext,
    children_method: Option<&str>,
) {
    let view = next_dev_view(counter);
    let current_font = props.style.style.font.as_ref().or(inherited_font);
    output.push_str(&format!(
        "        LinearLayout {view} = doweContainer(false);\n"
    ));
    apply_dev_android_style(&props.style.style, &view, true, output);
    output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
    let row = next_dev_view(counter);
    output.push_str(&format!(
        "        LinearLayout {row} = doweContainer(true);\n        doweAdd({view}, {row});\n"
    ));
    for item in items {
        match item {
            NavMenuItem::Item(props) => {
                render_dev_android_nav_menu_button(
                    props,
                    &row,
                    props.navigation.as_ref(),
                    counter,
                    output,
                    current_font,
                    context,
                );
            }
            NavMenuItem::Submenu { props, items } => {
                render_dev_android_nav_menu_button(
                    props,
                    &row,
                    None,
                    counter,
                    output,
                    current_font,
                    context,
                );
                let submenu = next_dev_view(counter);
                output.push_str(&format!(
                    "        LinearLayout {submenu} = doweContainer(false);\n        {submenu}.setPadding(doweDp(8), doweDp(8), doweDp(8), doweDp(8));\n        doweAdd({view}, {submenu});\n"
                ));
                for item in items {
                    render_dev_android_nav_menu_button(
                        item,
                        &submenu,
                        item.navigation.as_ref(),
                        counter,
                        output,
                        current_font,
                        context,
                    );
                }
            }
            NavMenuItem::Megamenu { props, content } => {
                render_dev_android_nav_menu_button(
                    props,
                    &row,
                    None,
                    counter,
                    output,
                    current_font,
                    context,
                );
                let panel = next_dev_view(counter);
                output.push_str(&format!(
                    "        LinearLayout {panel} = doweContainer(false);\n        {panel}.setPadding(doweDp(8), doweDp(8), doweDp(8), doweDp(8));\n        doweAdd({view}, {panel});\n"
                ));
                for child in content {
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
        }
    }
}

fn render_dev_android_nav_menu_button(
    props: &NavMenuItemProps,
    parent: &str,
    navigation: Option<&NavigationAction>,
    counter: &mut usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    context: &ComposeReactiveContext,
) {
    let view = next_dev_view(counter);
    output.push_str(&format!(
        "        LinearLayout {view} = doweContainer(true);\n        {view}.setGravity(Gravity.CENTER_VERTICAL);\n        {view}.setPadding(doweDp(12), doweDp(8), doweDp(12), doweDp(8));\n        doweAdd({parent}, {view});\n        TextView {view}Label = doweText(\"{}\", DOWE_ON_BACKGROUND, 14f, 400, 0f, 18f, {});\n        doweAdd({view}, {view}Label);\n",
        escape_java(&props.label),
        dev_font_value(inherited_font)
    ));
    if let Some(action) = props
        .on_click
        .as_deref()
        .and_then(|name| context.action_id(name))
        .map(|id| format!("doweRunAction(\"{}\", null)", escape_java(id)))
        .or_else(|| dev_android_navigation_action(navigation))
    {
        output.push_str(&format!(
            "        {view}.setOnClickListener(v -> {action});\n"
        ));
    }
}
