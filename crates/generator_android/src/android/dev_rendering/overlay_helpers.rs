fn render_dev_android_overlay_entry(
    entry: &OverlayEntry,
    props: &VariantProps,
    parent: &str,
    counter: &mut usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    context: &ComposeReactiveContext,
) {
    match entry {
        OverlayEntry::Item(item) => render_dev_android_overlay_item(
            item,
            props,
            parent,
            counter,
            output,
            inherited_font,
            context,
        ),
        OverlayEntry::Divider => {
            let view = next_dev_view(counter);
            output.push_str(&format!(
                "        View {view} = new View(this);\n        {view}.setBackgroundColor({});\n        {view}.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, doweDp(1)));\n",
                java_color(ColorToken::Muted)
            ));
            output.push_str(&dev_add(parent, &view, None, false));
        }
    }
}

fn render_dev_android_command_entry(
    entry: &CommandEntry,
    props: &VariantProps,
    parent: &str,
    counter: &mut usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    context: &ComposeReactiveContext,
) {
    match entry {
        CommandEntry::Item(item) => render_dev_android_overlay_item(
            item,
            props,
            parent,
            counter,
            output,
            inherited_font,
            context,
        ),
        CommandEntry::Group { label, items, .. } => {
            render_dev_android_variant_label(
                label,
                props,
                parent,
                None,
                false,
                counter,
                output,
                inherited_font,
                context,
            );
            for item in items {
                render_dev_android_overlay_item(
                    item,
                    props,
                    parent,
                    counter,
                    output,
                    inherited_font,
                    context,
                );
            }
        }
    }
}

fn render_dev_android_overlay_item(
    item: &OverlayItemProps,
    props: &VariantProps,
    parent: &str,
    counter: &mut usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    context: &ComposeReactiveContext,
) {
    render_dev_android_variant_label(
        &item.label,
        props,
        parent,
        None,
        false,
        counter,
        output,
        inherited_font,
        context,
    );
}

fn render_dev_android_drawer(
    props: &DrawerProps,
    header: &[ViewNode],
    body: &[ViewNode],
    footer: &[ViewNode],
    counter: &mut usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    context: &ComposeReactiveContext,
    children_method: Option<&str>,
) {
    let overlay = next_dev_view(counter);
    let panel = next_dev_view(counter);
    let content = next_dev_view(counter);
    let body_scroll = next_dev_view(counter);
    let body_content = next_dev_view(counter);
    let navigation_close = format!("{overlay}NavigationClose");
    let popup = format!("{overlay}Popup");
    let path = escape_java(&context.signal_path(&props.open));
    let (width, height, gravity) = match props.position {
        DrawerPosition::Start => (
            "doweDp(320)",
            "ViewGroup.LayoutParams.MATCH_PARENT",
            "Gravity.START",
        ),
        DrawerPosition::End => (
            "doweDp(320)",
            "ViewGroup.LayoutParams.MATCH_PARENT",
            "Gravity.END",
        ),
        DrawerPosition::Top => (
            "ViewGroup.LayoutParams.MATCH_PARENT",
            "doweDp(320)",
            "Gravity.TOP",
        ),
        DrawerPosition::Bottom => (
            "ViewGroup.LayoutParams.MATCH_PARENT",
            "doweDp(320)",
            "Gravity.BOTTOM",
        ),
    };
    output.push_str(&format!(
        "        if (doweBool(\"{path}\")) {{\n        FrameLayout {overlay} = new FrameLayout(this);\n        {overlay}.setBackgroundColor(Color.argb(122, 15, 23, 42));\n        FrameLayout {panel} = new FrameLayout(this);\n        {panel}.setBackground(doweDrawerBackground({}, {}, \"{}\", {}));\n        FrameLayout.LayoutParams {panel}Params = new FrameLayout.LayoutParams({width}, {height}, {gravity});\n        {overlay}.addView({panel}, {panel}Params);\n        LinearLayout {content} = doweContainer(false);\n",
        dev_card_variant_container(&props.style),
        dev_card_border(&props.style),
        props.position.as_str(),
        dev_drawer_radius(&props.style.style)
    ));
    apply_dev_android_style(&props.style.style, &content, false, output);
    output.push_str(&format!(
        "        {content}.setPadding({content}.getPaddingLeft() + scrollView.getPaddingLeft(), {content}.getPaddingTop() + scrollView.getPaddingTop(), {content}.getPaddingRight() + scrollView.getPaddingRight(), {content}.getPaddingBottom() + scrollView.getPaddingBottom());\n        FrameLayout.LayoutParams {content}Params = doweFrameLayoutParams({content}.getLayoutParams());\n        if ({content}Params.width == ViewGroup.LayoutParams.WRAP_CONTENT) {{\n            {content}Params.width = ViewGroup.LayoutParams.MATCH_PARENT;\n        }}\n        if ({content}Params.height == ViewGroup.LayoutParams.WRAP_CONTENT) {{\n            {content}Params.height = ViewGroup.LayoutParams.MATCH_PARENT;\n        }}\n        {panel}.addView({content}, {content}Params);\n        ScrollView {body_scroll} = new ScrollView(this);\n        {body_scroll}.setFillViewport(true);\n        LinearLayout {body_content} = doweContainer(false);\n        {body_scroll}.addView({body_content}, new ScrollView.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));\n"
    ));
    if !header.is_empty() {
        let header_content = next_dev_view(counter);
        output.push_str(&format!(
            "        LinearLayout {header_content} = doweContainer(false);\n        doweAdd({content}, {header_content});\n"
        ));
        let current_font = props.style.style.font.as_ref().or(inherited_font);
        for child in header {
            render_dev_android_node(
                child,
                &header_content,
                None,
                false,
                counter,
                output,
                current_font,
                Some(dev_card_variant_content(&props.style).to_string()),
                context,
                children_method,
            );
        }
    }
    output.push_str(&format!(
        "        {content}.addView({body_scroll}, new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, 0, 1f));\n"
    ));
    let current_font = props.style.style.font.as_ref().or(inherited_font);
    for child in body {
        render_dev_android_node(
            child,
            &body_content,
            None,
            false,
            counter,
            output,
            current_font,
            Some(dev_card_variant_content(&props.style).to_string()),
            context,
            children_method,
        );
    }
    if !footer.is_empty() {
        let footer_content = next_dev_view(counter);
        output.push_str(&format!(
            "        LinearLayout {footer_content} = doweContainer(false);\n        doweAdd({content}, {footer_content});\n"
        ));
        for child in footer {
            render_dev_android_node(
                child,
                &footer_content,
                None,
                false,
                counter,
                output,
                current_font,
                Some(dev_card_variant_content(&props.style).to_string()),
                context,
                children_method,
            );
        }
    }
    output.push_str(&format!(
        "        PopupWindow {popup} = new PopupWindow({overlay}, ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT, true);\n        {popup}.setBackgroundDrawable(new android.graphics.drawable.ColorDrawable(Color.TRANSPARENT));\n        {popup}.setOutsideTouchable(false);\n        {panel}.setOnClickListener(v -> {{ }});\n"
    ));
    output.push_str(&format!(
        "        Runnable {navigation_close} = () -> {{ if ({popup}.isShowing()) {{ {popup}.dismiss(); }} doweWrite(\"{path}\", false); }};\n        doweDrawerNavigationClose = {navigation_close};\n        {popup}.setOnDismissListener(() -> {{ if (doweDrawerNavigationClose == {navigation_close}) {{ doweDrawerNavigationClose = null; }} }});\n"
    ));
    if !props.disable_overlay_close {
        output.push_str(&format!(
            "        {overlay}.setOnClickListener(v -> {{ {popup}.dismiss(); doweWrite(\"{path}\", false); renderCurrentRoute(false); }});\n"
        ));
    }
    if !props.hide_close_button {
        let close = next_dev_view(counter);
        let close_icon = next_dev_view(counter);
        let close_paths = format!("{close}Paths");
        output.push_str(&format!(
            "        FrameLayout {close} = new FrameLayout(this);\n        {close}.setBackground(doweBackground(DOWE_SOFT_MUTED, 999f));\n        {close}.setContentDescription(\"Close drawer\");\n        {close}.setFocusable(true);\n        {close}.setOnClickListener(v -> {{ {popup}.dismiss(); doweWrite(\"{path}\", false); renderCurrentRoute(false); }});\n        ArrayList<DoweSvgPathEntry> {close_paths} = new ArrayList<>();\n        {close_paths}.add(new DoweSvgPathEntry(\"M0 0h24v24H0z\", false, null));\n        {close_paths}.add(new DoweSvgPathEntry(\"m4.397 4.554l.073-.084a.75.75 0 0 1 .976-.073l.084.073L12 10.939l6.47-6.47a.75.75 0 1 1 1.06 1.061L13.061 12l6.47 6.47a.75.75 0 0 1 .072.976l-.073.084a.75.75 0 0 1-.976.073l-.084-.073L12 13.061l-6.47 6.47a.75.75 0 0 1-1.06-1.061L10.939 12l-6.47-6.47a.75.75 0 0 1-.072-.976l.073-.084z\", true, null));\n        DoweSvgView {close_icon} = new DoweSvgView(this, 0f, 0f, 24f, 24f, DOWE_ON_SOFT_MUTED, {close_paths});\n        {close_icon}.setImportantForAccessibility(View.IMPORTANT_FOR_ACCESSIBILITY_NO);\n        {close}.addView({close_icon}, new FrameLayout.LayoutParams(doweDp(18), doweDp(18), Gravity.CENTER));\n        FrameLayout.LayoutParams {close}Params = new FrameLayout.LayoutParams(doweDp(28), doweDp(28), Gravity.TOP | Gravity.END);\n        {close}Params.setMargins(0, scrollView.getPaddingTop() + doweDp(8), scrollView.getPaddingRight() + doweDp(8), 0);\n        {panel}.addView({close}, {close}Params);\n"
        ));
    }
    output.push_str(&format!(
        "        root.post(() -> {{ if (root.getWindowToken() != null) {{ {popup}.showAtLocation(root, Gravity.FILL, 0, 0); }} }});\n        }}\n"
    ));
}

fn dev_view_icon_label(icon: ViewIcon) -> &'static str {
    match icon {
        ViewIcon::Plus => "+",
        ViewIcon::Link => "link",
        ViewIcon::Edit => "edit",
        ViewIcon::Trash => "trash",
        ViewIcon::Search => "search",
        ViewIcon::Settings => "settings",
        ViewIcon::Upload => "upload",
        ViewIcon::File => "file",
        ViewIcon::Dismiss => "x",
        ViewIcon::Moon => "moon",
        ViewIcon::Sun => "sun",
    }
}

fn dev_dropzone_height(size: ButtonSize) -> u16 {
    match size {
        ButtonSize::Xs | ButtonSize::Sm => 128,
        ButtonSize::Md => 192,
        ButtonSize::Lg | ButtonSize::Xl => 256,
    }
}
