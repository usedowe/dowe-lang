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
    if props.style.style.sizing.w.is_none() {
        output.push_str(&format!("        doweWrapContentWidth({view});\n"));
    }
    output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
    let row = next_dev_view(counter);
    output.push_str(&format!(
        "        LinearLayout {row} = doweContainer(true);\n        doweWrapContentWidth({row});\n        doweAdd({view}, {row});\n"
    ));
    for item in items {
        match item {
            NavMenuItem::Item(item_props) => {
                let active = dev_side_nav_active(item_props.navigation.as_ref());
                render_dev_android_nav_menu_button(
                    item_props,
                    &row,
                    props,
                    &active,
                    false,
                    item_props.navigation.as_ref(),
                    counter,
                    output,
                    current_font,
                    context,
                    None,
                );
            }
            NavMenuItem::Submenu {
                props: item_props,
                items,
            } => {
                let trigger = render_dev_android_nav_menu_button(
                    item_props,
                    &row,
                    props,
                    "false",
                    true,
                    None,
                    counter,
                    output,
                    current_font,
                    context,
                    None,
                );
                render_dev_android_nav_menu_submenu(
                    &trigger,
                    items,
                    props,
                    counter,
                    output,
                    current_font,
                    context,
                );
            }
            NavMenuItem::Megamenu {
                props: item_props,
                content,
            } => {
                let trigger = render_dev_android_nav_menu_button(
                    item_props,
                    &row,
                    props,
                    "false",
                    true,
                    None,
                    counter,
                    output,
                    current_font,
                    context,
                    None,
                );
                render_dev_android_nav_menu_megamenu(
                    &trigger,
                    content,
                    props,
                    counter,
                    output,
                    current_font,
                    context,
                    children_method,
                );
            }
        }
    }
}

fn render_dev_android_nav_menu_submenu(
    trigger: &str,
    items: &[NavMenuItemProps],
    nav: &NavMenuProps,
    counter: &mut usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    context: &ComposeReactiveContext,
) {
    let panel = next_dev_view(counter);
    let scroll = next_dev_view(counter);
    let popup_ref = format!("{trigger}PopupRef");
    let dismiss = format!(
        "if ({popup_ref}[0] != null) {{ {popup_ref}[0].dismiss(); }}"
    );
    begin_dev_android_nav_menu_popover(trigger, &panel, &popup_ref, output);
    for item in items {
        let active = dev_side_nav_active(item.navigation.as_ref());
        render_dev_android_nav_menu_button(
            item,
            &panel,
            nav,
            &active,
            false,
            item.navigation.as_ref(),
            counter,
            output,
            inherited_font,
            context,
            Some(&dismiss),
        );
    }
    finish_dev_android_nav_menu_popover(
        trigger,
        &panel,
        &scroll,
        &popup_ref,
        false,
        nav,
        output,
    );
}

fn render_dev_android_nav_menu_megamenu(
    trigger: &str,
    content: &[ViewNode],
    nav: &NavMenuProps,
    counter: &mut usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    context: &ComposeReactiveContext,
    children_method: Option<&str>,
) {
    let panel = next_dev_view(counter);
    let scroll = next_dev_view(counter);
    let popup_ref = format!("{trigger}PopupRef");
    begin_dev_android_nav_menu_popover(trigger, &panel, &popup_ref, output);
    for child in content {
        render_dev_android_node(
            child,
            &panel,
            None,
            false,
            counter,
            output,
            inherited_font,
            Some("DOWE_BACKGROUND_TEXT".to_string()),
            context,
            children_method,
        );
    }
    finish_dev_android_nav_menu_popover(
        trigger,
        &panel,
        &scroll,
        &popup_ref,
        true,
        nav,
        output,
    );
}

fn begin_dev_android_nav_menu_popover(
    trigger: &str,
    panel: &str,
    popup_ref: &str,
    output: &mut String,
) {
    output.push_str(&format!(
        "        final PopupWindow[] {popup_ref} = new PopupWindow[1];\n        {trigger}.setOnClickListener(anchor -> {{\n        if ({popup_ref}[0] != null && {popup_ref}[0].isShowing()) {{ {popup_ref}[0].dismiss(); return; }}\n        DoweDismissOnTouchLayout {panel} = new DoweDismissOnTouchLayout(this);\n        {panel}.setOrientation(LinearLayout.VERTICAL);\n        {panel}.setAlpha(0f);\n        {panel}.setScaleX(0.98f);\n        {panel}.setScaleY(0.98f);\n        {panel}.setTranslationY(-doweDp(4));\n        {panel}.setPadding(doweDp(8), doweDp(8), doweDp(8), doweDp(8));\n        {panel}.setBackground(doweInputBackground(DOWE_BACKGROUND, null, DOWE_RADIUS));\n"
    ));
}

fn finish_dev_android_nav_menu_popover(
    trigger: &str,
    panel: &str,
    scroll: &str,
    popup_ref: &str,
    wide: bool,
    nav: &NavMenuProps,
    output: &mut String,
) {
    let desired_width = if wide { 600 } else { 220 };
    let maximum_width = if wide { 720 } else { 360 };
    let border = if nav.style.variant.unwrap_or(ComponentVariant::Ghost)
        == ComponentVariant::Outlined
    {
        format!("Integer.valueOf({})", dev_variant_content(&nav.style))
    } else {
        "null".to_string()
    };
    let border_width = if border == "null" {
        "null"
    } else {
        "Integer.valueOf(doweDp(1))"
    };
    output.push_str(&format!(
        "        int {panel}AvailableWidth = Math.max(doweDp(192), getResources().getDisplayMetrics().widthPixels - doweDp(16));\n        int {panel}Width = Math.min(Math.min(Math.max({trigger}.getWidth(), doweDp({desired_width})), doweDp({maximum_width})), {panel}AvailableWidth);\n        ScrollView {scroll} = new ScrollView(this);\n        {scroll}.setFillViewport(false);\n        {scroll}.addView({panel}, new ScrollView.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));\n        {panel}.measure(View.MeasureSpec.makeMeasureSpec({panel}Width, View.MeasureSpec.EXACTLY), View.MeasureSpec.makeMeasureSpec(0, View.MeasureSpec.UNSPECIFIED));\n        {popup_ref}[0] = new PopupWindow({scroll}, {panel}Width, Math.min({panel}.getMeasuredHeight(), Math.min(doweDp(640), (int) (getResources().getDisplayMetrics().heightPixels * 0.8f))), true);\n        {popup_ref}[0].setBackgroundDrawable(new android.graphics.drawable.ColorDrawable(Color.TRANSPARENT));\n        {popup_ref}[0].setOutsideTouchable(true);\n        {popup_ref}[0].setElevation(doweDp(8));\n        {panel}.setDismissAction(() -> {{ if ({popup_ref}[0] != null) {{ {popup_ref}[0].dismiss(); }} }});\n        {trigger}.setBackground(doweStyledBackground({}, {border}, {border_width}, DOWE_RADIUS));\n        {trigger}Label.setTextColor({});\n        {trigger}Arrow.setCurrentColor({});\n        {trigger}Arrow.animate().rotation(-90f).setDuration(160).start();\n        {popup_ref}[0].setOnDismissListener(() -> {{\n            {trigger}.setBackgroundColor(Color.TRANSPARENT);\n            {trigger}Label.setTextColor(DOWE_BACKGROUND_TEXT);\n            {trigger}Arrow.setCurrentColor(DOWE_BACKGROUND_TEXT);\n            {trigger}Arrow.animate().rotation(90f).setDuration(160).start();\n        }});\n        {popup_ref}[0].showAsDropDown({trigger}, 0, doweDp(8));\n        {panel}.animate().alpha(1f).scaleX(1f).scaleY(1f).translationY(0f).setDuration(160).start();\n        }});\n",
        dev_variant_container(&nav.style),
        dev_nav_active_content(&nav.style),
        dev_nav_active_content(&nav.style),
    ));
}

#[allow(clippy::too_many_arguments)]
fn render_dev_android_nav_menu_button(
    props: &NavMenuItemProps,
    parent: &str,
    nav: &NavMenuProps,
    active: &str,
    arrow: bool,
    navigation: Option<&NavigationAction>,
    counter: &mut usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    context: &ComposeReactiveContext,
    dismiss: Option<&str>,
) -> String {
    let view = next_dev_view(counter);
    let content = format!(
        "({active}) ? {} : DOWE_BACKGROUND_TEXT",
        dev_nav_active_content(&nav.style)
    );
    let border = if nav.style.variant.unwrap_or(ComponentVariant::Ghost)
        == ComponentVariant::Outlined
    {
        format!("Integer.valueOf({})", dev_variant_content(&nav.style))
    } else {
        "null".to_string()
    };
    let border_width = if border == "null" {
        "null"
    } else {
        "Integer.valueOf(doweDp(1))"
    };
    output.push_str(&format!(
        "        LinearLayout {view} = doweContainer(true);\n        {view}.setGravity(Gravity.CENTER_VERTICAL);\n        {view}.setPadding(doweDp(12), doweDp(8), doweDp(12), doweDp(8));\n        doweWrapContentWidth({view});\n        if ({active}) {{ {view}.setBackground(doweStyledBackground({}, {border}, {border_width}, DOWE_RADIUS)); }}\n        doweAdd({parent}, {view});\n        TextView {view}Label = doweText({}, {content}, 14f, 400, 0f, 18f, {});\n        doweAdd({view}, {view}Label);\n",
        dev_variant_container(&nav.style),
        dev_text_expression(&props.label, props.i18n.as_deref(), context),
        dev_font_value(inherited_font)
    ));
    if arrow {
        output.push_str(&format!(
            "        DoweSvgView {view}Arrow = doweNavMenuArrow(DOWE_BACKGROUND_TEXT);\n        doweAdd({view}, {view}Arrow, 8, true);\n"
        ));
    }
    if let Some(action) = props
        .on_click
        .as_deref()
        .and_then(|name| context.action_id(name))
        .map(|id| format!("doweRunAction(\"{}\", null)", escape_java(id)))
        .or_else(|| dev_android_navigation_action(navigation))
    {
        let close = dismiss.map(|value| format!(" {value}")).unwrap_or_default();
        output.push_str(&format!(
            "        {view}.setOnClickListener(v -> {{ {action};{close} }});\n        {view}Label.setOnClickListener(v -> {view}.performClick());\n"
        ));
    }
    view
}
