fn render_compose_nav_menu(
    props: &NavMenuProps,
    items: &[NavMenuItem],
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    let current_font = props.style.style.font.as_ref().or(inherited_font);
    let (padding_horizontal, padding_vertical, gap, label_size, description_size) =
        compose_side_nav_metrics(props.size);
    let border =
        if props.style.variant.unwrap_or(ComponentVariant::Ghost) == ComponentVariant::Outlined {
            variant_content(&props.style)
        } else {
            "null"
        };
    output.push_str(&format!(
        "{pad}DoweNavMenu(modifier = {}, gap = {gap}.dp, popoverBackgroundColor = DoweDesign.background, popoverContentColor = DoweDesign.onBackground, content = {{ openIndex, toggle ->\n",
        modifier_for_container_style(&props.style.style, flow)
    ));
    for (index, item) in items.iter().enumerate() {
        render_compose_nav_menu_trigger(
            index,
            item,
            indent + 4,
            output,
            props,
            padding_horizontal,
            padding_vertical,
            label_size,
            border,
            current_font,
            default_family,
            context,
        );
    }
    output.push_str(&format!("{pad}}}, popover = {{ openIndex ->\n"));
    for (index, item) in items.iter().enumerate() {
        render_compose_nav_menu_popover(
            index,
            item,
            indent + 4,
            output,
            props,
            padding_horizontal,
            padding_vertical,
            label_size,
            description_size,
            border,
            current_font,
            default_family,
            context,
        );
    }
    output.push_str(&format!("{pad}}})\n"));
}

fn render_compose_nav_menu_trigger(
    index: usize,
    item: &NavMenuItem,
    indent: usize,
    output: &mut String,
    nav: &NavMenuProps,
    padding_horizontal: u16,
    padding_vertical: u16,
    label_size: u16,
    border: &str,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    match item {
        NavMenuItem::Item(props) => render_compose_nav_menu_button(
            props,
            compose_nav_menu_action(props, context),
            compose_nav_menu_active(props.navigation.as_ref()),
            false,
            indent,
            output,
            nav,
            padding_horizontal,
            padding_vertical,
            label_size,
            border,
            inherited_font,
            default_family,
        ),
        NavMenuItem::Submenu { props, .. } | NavMenuItem::Megamenu { props, .. } => {
            render_compose_nav_menu_button(
                props,
                format!("{{ toggle({index}) }}"),
                format!("openIndex == {index}"),
                true,
                indent,
                output,
                nav,
                padding_horizontal,
                padding_vertical,
                label_size,
                border,
                inherited_font,
                default_family,
            );
        }
    }
}

fn render_compose_nav_menu_popover(
    index: usize,
    item: &NavMenuItem,
    indent: usize,
    output: &mut String,
    nav: &NavMenuProps,
    padding_horizontal: u16,
    padding_vertical: u16,
    label_size: u16,
    description_size: u16,
    border: &str,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    match item {
        NavMenuItem::Submenu { items, .. } => {
            output.push_str(&format!("{pad}if (openIndex == {index}) {{\n"));
            output.push_str(&format!(
                "{pad}    Column(verticalArrangement = Arrangement.spacedBy(2.dp)) {{\n"
            ));
            for item in items {
                render_compose_nav_menu_subitem(
                    item,
                    indent + 8,
                    output,
                    nav,
                    padding_horizontal,
                    padding_vertical,
                    label_size,
                    description_size,
                    border,
                    inherited_font,
                    default_family,
                    context,
                );
            }
            output.push_str(&format!("{pad}    }}\n"));
            output.push_str(&format!("{pad}}}\n"));
        }
        NavMenuItem::Megamenu { content, .. } => {
            output.push_str(&format!("{pad}if (openIndex == {index}) {{\n"));
            output.push_str(&format!(
                "{pad}    Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {{\n"
            ));
            for child in content {
                render_compose_node_in_flow(
                    child,
                    indent + 8,
                    output,
                    ComposeFlow::Block,
                    inherited_font,
                    default_family,
                    context,
                );
            }
            output.push_str(&format!("{pad}    }}\n"));
            output.push_str(&format!("{pad}}}\n"));
        }
        NavMenuItem::Item(_) => {}
    }
}

fn render_compose_nav_menu_subitem(
    props: &NavMenuItemProps,
    indent: usize,
    output: &mut String,
    nav: &NavMenuProps,
    padding_horizontal: u16,
    padding_vertical: u16,
    label_size: u16,
    description_size: u16,
    border: &str,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    render_compose_nav_menu_button(
        props,
        compose_nav_menu_action(props, context),
        compose_nav_menu_active(props.navigation.as_ref()),
        false,
        indent,
        output,
        nav,
        padding_horizontal,
        padding_vertical,
        label_size,
        border,
        inherited_font,
        default_family,
    );
    if let Some(description) = props.description.as_deref() {
        output.push_str(&format!(
            "{pad}Text(text = \"{}\", modifier = Modifier.padding(start = 12.dp), fontSize = {description_size}.sp, fontFamily = {}, color = LocalContentColor.current.copy(alpha = 0.72f))\n",
            escape_kotlin(description),
            compose_font_value(inherited_font, default_family)
        ));
    }
}

fn render_compose_nav_menu_button(
    props: &NavMenuItemProps,
    action: String,
    active: String,
    arrow: bool,
    indent: usize,
    output: &mut String,
    nav: &NavMenuProps,
    padding_horizontal: u16,
    padding_vertical: u16,
    label_size: u16,
    border: &str,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
) {
    let pad = " ".repeat(indent);
    output.push_str(&format!(
        "{pad}DoweNavMenuItem(active = {active}, paddingHorizontal = {padding_horizontal}.dp, paddingVertical = {padding_vertical}.dp, backgroundColor = {}, contentColor = {}, borderColor = {border}, onClick = {action}) {{\n",
        variant_container(&nav.style),
        nav_active_content(&nav.style),
    ));
    if let Some(icon) = props.icon.as_ref() {
        output.push_str(&format!(
            "{pad}    DoweSvg(viewBox = {}, modifier = {}, color = {}, paths = {})\n",
            compose_svg_view_box(&icon.props.view_box),
            modifier_for_style(&icon.props.style),
            compose_svg_color(&icon.props.style),
            compose_svg_paths(&icon.paths)
        ));
    }
    output.push_str(&format!(
        "{pad}    Text(text = \"{}\", fontSize = {label_size}.sp, fontFamily = {}, fontWeight = FontWeight.Normal)\n",
        escape_kotlin(&props.label),
        compose_font_value(inherited_font, default_family),
    ));
    if arrow {
        output.push_str(&format!(
            "{pad}    Text(text = \"⌄\", fontSize = {label_size}.sp, fontWeight = FontWeight.SemiBold)\n"
        ));
    }
    output.push_str(&format!("{pad}}}\n"));
}

fn compose_nav_menu_action(props: &NavMenuItemProps, context: &ComposeReactiveContext) -> String {
    props
        .on_click
        .as_deref()
        .and_then(|name| context.action_id(name))
        .map(|id| {
            format!(
                "{{ actionScope.launch {{ state.run(\"{}\") }} }}",
                escape_kotlin(id)
            )
        })
        .or_else(|| {
            props
                .navigation
                .as_ref()
                .map(|action| compose_navigation_action(Some(action)))
        })
        .unwrap_or_else(|| "null".to_string())
}

fn compose_nav_menu_active(action: Option<&NavigationAction>) -> String {
    match action {
        Some(NavigationAction::Internal { path, .. }) => {
            format!("activePath == \"{}\"", escape_kotlin(path))
        }
        _ => "false".to_string(),
    }
}
