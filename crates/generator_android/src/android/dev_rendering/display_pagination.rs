fn render_dev_android_pagination(
    props: &ToggleGroupProps,
    items: &[ToggleGroupItem],
    parent: &str,
    parent_gap: Option<&str>,
    parent_horizontal: bool,
    counter: &mut usize,
    output: &mut String,
    context: &ComposeReactiveContext,
) {
    let view = next_dev_view(counter);
    let rendered_pages = items.len().max(1);
    let path = props
        .value
        .as_deref()
        .map(|value| escape_java(&context.signal_path(value)))
        .unwrap_or_default();
    let action = props
        .on_change
        .as_deref()
        .and_then(|name| context.action_id(name))
        .map(|name| format!("doweRunAction(\"{}\", null); ", escape_java(name)))
        .unwrap_or_default();
    let dimension = match props.size {
        ButtonSize::Xs => 24,
        ButtonSize::Sm => 32,
        ButtonSize::Lg => 48,
        _ => 40,
    };
    let (total_setup, page_count) = props
        .pagination
        .as_ref()
        .map(|pagination| match &pagination.total {
            dowe_components::PaginationTotal::Static(total) => (
                String::new(),
                total.div_ceil(pagination.page_size).max(1).to_string(),
            ),
            dowe_components::PaginationTotal::Signal(total) => {
                let total = escape_java(&context.signal_path(total));
                let offset = pagination.page_size - 1;
                (
                    format!(
                        "        int {view}Total = 0;\n        try {{ {view}Total = Integer.parseInt(doweTextValue(\"{total}\", null)); }} catch (NumberFormatException ignored) {{}}\n"
                    ),
                    format!(
                        "Math.max(1, Math.min(25, (Math.max(0, {view}Total) + {offset}) / {}))",
                        pagination.page_size
                    ),
                )
            }
        })
        .unwrap_or_else(|| (String::new(), rendered_pages.to_string()));
    output.push_str(&format!(
        "        LinearLayout {view} = doweContainer(true);\n        {view}.setGravity(Gravity.CENTER_VERTICAL);\n{total_setup}        final int {view}Pages = {page_count};\n        int {view}Current = 1;\n        try {{ {view}Current = Integer.parseInt(doweTextValue(\"{path}\", null)); }} catch (NumberFormatException ignored) {{}}\n        {view}Current = Math.max(1, Math.min({view}Pages, {view}Current));\n        final int {view}Selected = {view}Current;\n"
    ));
    apply_dev_android_style(&props.style.style, &view, true, output);
    output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
    render_dev_android_pagination_arrow(
        props,
        "arrow-left",
        "Previous page",
        -1,
        &format!("{view}Pages"),
        dimension,
        &path,
        &action,
        &view,
        counter,
        output,
    );
    for page in 1..=rendered_pages {
        if rendered_pages > 7 && page == 2 {
            let ellipsis = next_dev_view(counter);
            output.push_str(&format!(
                "        TextView {ellipsis} = doweText(\"…\", doweAlpha(DOWE_BACKGROUND_TEXT, 0.6f), 14f, 400, 0f, 1.2f, null);\n        {ellipsis}.setGravity(Gravity.CENTER);\n        {ellipsis}.setVisibility({view}Selected > 3 ? View.VISIBLE : View.GONE);\n        doweAdd({view}, {ellipsis}, doweDp(4), true);\n"
            ));
        }
        if rendered_pages > 7 && page == rendered_pages {
            let ellipsis = next_dev_view(counter);
            output.push_str(&format!(
                "        TextView {ellipsis} = doweText(\"…\", doweAlpha(DOWE_BACKGROUND_TEXT, 0.6f), 14f, 400, 0f, 1.2f, null);\n        {ellipsis}.setGravity(Gravity.CENTER);\n        {ellipsis}.setVisibility({view}Selected < {view}Pages - 2 ? View.VISIBLE : View.GONE);\n        doweAdd({view}, {ellipsis}, doweDp(4), true);\n"
            ));
        }
        let button = next_dev_view(counter);
        let visible = format!(
            "{page} <= {view}Pages && ({view}Pages <= 7 || {page} == 1 || {page} == {view}Pages || Math.abs({page} - {view}Selected) <= 1)"
        );
        output.push_str(&format!(
            "        TextView {button} = doweText(\"{page}\", ({view}Selected == {page}) ? {} : DOWE_BACKGROUND_TEXT, 14f, 500, 0f, 1.2f, null);\n        {button}.setGravity(Gravity.CENTER);\n        {button}.setContentDescription(\"Page {page}\");\n        {button}.setBackground(doweBackground(({view}Selected == {page}) ? {} : Color.TRANSPARENT, DOWE_RADIUS));\n        {button}.setVisibility({visible} ? View.VISIBLE : View.GONE);\n        {button}.setEnabled({});\n        {button}.setOnClickListener(v -> {{ if ({view}Selected != {page}) {{ doweWrite(\"{path}\", \"{page}\"); {action}renderCurrentRoute(false); }} }});\n        LinearLayout.LayoutParams {button}Params = new LinearLayout.LayoutParams(doweDp({dimension}), doweDp({dimension}));\n        {view}.addView({button}, {button}Params);\n",
            dev_variant_content(&props.style),
            dev_variant_container(&props.style),
            !props.disabled,
        ));
    }
    render_dev_android_pagination_arrow(
        props,
        "arrow-right",
        "Next page",
        1,
        &format!("{view}Pages"),
        dimension,
        &path,
        &action,
        &view,
        counter,
        output,
    );
}

fn render_dev_android_pagination_arrow(
    props: &ToggleGroupProps,
    icon_name: &str,
    label: &str,
    step: i32,
    pages: &str,
    dimension: u16,
    path: &str,
    action: &str,
    parent: &str,
    counter: &mut usize,
    output: &mut String,
) {
    let button = next_dev_view(counter);
    let icon = solar_control_icon(icon_name).expect("bundled Pagination icon");
    let icon_view = render_dev_android_icon_view(
        &icon,
        counter,
        output,
        Some(dev_variant_content(&props.style)),
    );
    let enabled = if step < 0 {
        format!("{parent}Selected > 1")
    } else {
        format!("{parent}Selected < {pages}")
    };
    output.push_str(&format!(
        "        FrameLayout {button} = new FrameLayout(this);\n        {button}.setContentDescription(\"{label}\");\n        {button}.setFocusable(true);\n        {button}.setEnabled({} && {enabled});\n        {button}.setAlpha({enabled} ? 1f : 0.42f);\n        {button}.setBackground(doweBackground({}, DOWE_RADIUS));\n        {button}.setOnClickListener(v -> {{ int page = Math.max(1, Math.min({pages}, {parent}Selected + ({step}))); doweWrite(\"{path}\", String.valueOf(page)); {action}renderCurrentRoute(false); }});\n        {button}.addView({icon_view}, new FrameLayout.LayoutParams(doweDp(18), doweDp(18), Gravity.CENTER));\n        LinearLayout.LayoutParams {button}Params = new LinearLayout.LayoutParams(doweDp({dimension}), doweDp({dimension}));\n        {button}Params.setMargins(doweDp(4), 0, 0, 0);\n        {parent}.addView({button}, {button}Params);\n",
        !props.disabled,
        dev_variant_container(&props.style),
    ));
}
