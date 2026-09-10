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
    let layer = next_dev_view(counter);
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
        "        if (doweBool(\"{path}\")) {{\n        FrameLayout {overlay} = new FrameLayout(this);\n        {overlay}.setBackgroundColor(Color.argb(122, 15, 23, 42));\n        FrameLayout {layer} = new FrameLayout(this);\n        FrameLayout {panel} = new FrameLayout(this);\n        {panel}.setBackground(doweDrawerBackground({}, {}, \"{}\", {}));\n        {panel}.setLayoutParams(new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT));\n        {layer}.addView({panel});\n        FrameLayout.LayoutParams {panel}Params = new FrameLayout.LayoutParams({width}, {height}, {gravity});\n        {overlay}.addView({layer}, {panel}Params);\n        LinearLayout {content} = doweContainer(false);\n",
        dev_card_variant_container(&props.style),
        dev_card_border(&props.style),
        props.position.as_str(),
        dev_drawer_radius(&props.style.style)
    ));
    apply_dev_android_style(&props.style.style, &content, false, output);
    output.push_str(&format!("        {content}.setClipChildren(true);\n        {content}.setClipToPadding(true);\n"));
    output.push_str(&format!(
        "        FrameLayout.LayoutParams {content}Params = doweFrameLayoutParams({content}.getLayoutParams());\n        if ({content}Params.width == ViewGroup.LayoutParams.WRAP_CONTENT) {{\n            {content}Params.width = ViewGroup.LayoutParams.MATCH_PARENT;\n        }}\n        if ({content}Params.height == ViewGroup.LayoutParams.WRAP_CONTENT) {{\n            {content}Params.height = ViewGroup.LayoutParams.MATCH_PARENT;\n        }}\n        {panel}.addView({content}, {content}Params);\n        ScrollView {body_scroll} = new ScrollView(this);\n        {body_scroll}.setFillViewport(true);\n        LinearLayout {body_content} = doweContainer(false);\n        {body_scroll}.addView({body_content}, new ScrollView.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));\n"
    ));
    if !header.is_empty() {
        let header_content = next_dev_view(counter);
        output.push_str(&format!(
            "        LinearLayout {header_content} = doweContainer(false);\n        {header_content}.setBackgroundColor({});\n        {header_content}.setClipChildren(true);\n        doweAdd({content}, {header_content});\n",
            dev_card_variant_container(&props.style)
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
        "        Runnable {navigation_close} = () -> {{ PopupWindow activePopup = doweActiveOverlay; if (activePopup != null && activePopup.isShowing()) {{ activePopup.dismiss(); }} doweWrite(\"{path}\", false); }};\n        doweDrawerNavigationClose = {navigation_close};\n        {popup}.setOnDismissListener(() -> {{ doweDrawerNavigationClose = null; if (doweActiveOverlay == {popup}) {{ doweActiveOverlay = null; }} }});\n"
    ));
    if !props.disable_overlay_close {
        output.push_str(&format!(
            "        {overlay}.setOnClickListener(v -> {{ {navigation_close}.run(); renderCurrentRoute(false); }});\n"
        ));
    }
    let mut drawer_close_claim = String::new();
    if !props.hide_close_button {
        let (close_gravity, close_margins) = dev_drawer_close_gravity(&props.position);
        let drawer_close = render_dev_android_overlay_close(
            &overlay,
            "Close drawer",
            &format!("{navigation_close}.run(); renderCurrentRoute(false);"),
            close_gravity,
            close_margins,
            counter,
            output,
        );
        drawer_close_claim = format!(
            "            {overlay}.removeView({drawer_close});\n            existingOverlay.addView({drawer_close}, {drawer_close}Params);\n"
        );
    }
    output.push_str(&format!(
        "        if (doweActiveOverlay != null && doweActiveOverlay.isShowing() && doweActiveOverlay.getContentView() instanceof FrameLayout) {{\n            FrameLayout existingOverlay = (FrameLayout) doweActiveOverlay.getContentView();\n            existingOverlay.removeAllViews();\n            {overlay}.removeView({layer});\n            existingOverlay.addView({layer}, {panel}Params);\n{drawer_close_claim}\n            doweOverlayClaimed = doweOverlayRender;\n        }} else {{\n            root.post(() -> {{ if (root.getWindowToken() != null) {{ doweActiveOverlay = {popup}; {popup}.showAtLocation(root, Gravity.FILL, 0, 0); doweOverlayClaimed = doweOverlayRender; }} }});\n        }}\n        }}\n"));
}

fn dev_fab_content_gravity(position: OverlayCornerPosition) -> &'static str {
    match position {
        OverlayCornerPosition::TopLeft | OverlayCornerPosition::BottomLeft => "Gravity.START",
        OverlayCornerPosition::TopRight | OverlayCornerPosition::BottomRight => "Gravity.END",
    }
}

fn dev_fab_layout_gravity(position: OverlayCornerPosition) -> &'static str {
    match position {
        OverlayCornerPosition::TopLeft => "Gravity.TOP | Gravity.START",
        OverlayCornerPosition::TopRight => "Gravity.TOP | Gravity.END",
        OverlayCornerPosition::BottomLeft => "Gravity.BOTTOM | Gravity.START",
        OverlayCornerPosition::BottomRight => "Gravity.BOTTOM | Gravity.END",
    }
}

fn dev_fab_size(size: ButtonSize) -> u16 {
    match size {
        ButtonSize::Xs => 40,
        ButtonSize::Sm => 48,
        ButtonSize::Md => 52,
        ButtonSize::Lg => 56,
        ButtonSize::Xl => 64,
    }
}

fn dev_dropzone_height(size: ButtonSize) -> u16 {
    match size {
        ButtonSize::Xs | ButtonSize::Sm => 128,
        ButtonSize::Md => 192,
        ButtonSize::Lg | ButtonSize::Xl => 256,
    }
}
