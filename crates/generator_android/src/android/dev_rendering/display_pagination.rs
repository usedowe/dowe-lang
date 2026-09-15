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
    let pagination_variant = props
        .pagination
        .as_ref()
        .map(|pagination| pagination.variant)
        .unwrap_or(PaginationVariant::Pages);
    let pagination_contract = PaginationControlContract::for_size(props.size);
    let visual_contract = props.pagination_visual_contract();
    let accent_color = java_color(visual_contract.accent);
    let control_border_alpha = format!("{:.2}", visual_contract.control_border_alpha);
    let disabled_alpha = format!("{:.2}", visual_contract.disabled_alpha);
    let dimension = pagination_contract.control_size;
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
    if matches!(pagination_variant, PaginationVariant::Pages | PaginationVariant::Controls) {
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
            &control_border_alpha,
            &disabled_alpha,
            counter,
            output,
        );
    }
    if pagination_variant == PaginationVariant::Pages {
        for page in 1..=rendered_pages {
            if rendered_pages > 7 && page == 2 {
                let ellipsis = next_dev_view(counter);
                output.push_str(&format!(
                    "        TextView {ellipsis} = doweText(\"…\", doweAlpha(DOWE_BACKGROUND_TEXT, 0.6f), 14f, 400, 0f, 1.2f, null);\n        {ellipsis}.setGravity(Gravity.CENTER);\n        {ellipsis}.setVisibility({view}Selected > 3 ? View.VISIBLE : View.GONE);\n        doweAdd({view}, {ellipsis}, 4, true);\n"
                ));
            }
            if rendered_pages > 7 && page == rendered_pages {
                let ellipsis = next_dev_view(counter);
                output.push_str(&format!(
                    "        TextView {ellipsis} = doweText(\"…\", doweAlpha(DOWE_BACKGROUND_TEXT, 0.6f), 14f, 400, 0f, 1.2f, null);\n        {ellipsis}.setGravity(Gravity.CENTER);\n        {ellipsis}.setVisibility({view}Selected < {view}Pages - 2 ? View.VISIBLE : View.GONE);\n        doweAdd({view}, {ellipsis}, 4, true);\n"
                ));
            }
            let button = next_dev_view(counter);
            let visible = format!(
                "{page} <= {view}Pages && ({view}Pages <= 7 || {page} == 1 || {page} == {view}Pages || Math.abs({page} - {view}Selected) <= 1)"
            );
            output.push_str(&format!(
                "        TextView {button} = doweText(\"{page}\", ({view}Selected == {page}) ? {} : DOWE_BACKGROUND_TEXT, 14f, 500, 0f, 1.2f, null);\n        {button}.setGravity(Gravity.CENTER);\n        {button}.setContentDescription(\"Page {page}\");\n        {button}.setBackground(doweBackground(({view}Selected == {page}) ? {} : Color.TRANSPARENT, DOWE_RADIUS));\n        {button}.setVisibility({visible} ? View.VISIBLE : View.GONE);\n        {button}.setEnabled({});\n        {button}.setOnClickListener(v -> {{ if ({view}Selected != {page}) {{ doweWrite(\"{path}\", \"{page}\"); {action}renderCurrentRoute(false); }} }});\n        LinearLayout.LayoutParams {button}Params = new LinearLayout.LayoutParams(doweDp({dimension}), doweDp({dimension}));\n        {view}.addView({button}, {button}Params);\n",
                accent_color,
                dev_variant_container(&props.style),
                !props.disabled,
            ));
        }
    } else {
        let dot = pagination_variant == PaginationVariant::Dots;
        for page in 1..=rendered_pages {
            render_dev_android_pagination_indicator(
                props,
                page,
                dot,
                &format!("{view}Pages"),
                &path,
                &action,
                &view,
                pagination_contract,
                counter,
                output,
            );
        }
        if pagination_variant == PaginationVariant::Controls {
            let count = next_dev_view(counter);
            output.push_str(&format!(
                "        TextView {count} = doweText({view}Selected + \" / \" + {view}Pages, DOWE_BACKGROUND_TEXT, 13f, 600, 0f, 1.2f, null);\n        {count}.setGravity(Gravity.CENTER);\n        doweAdd({view}, {count}, 8, true);\n"
            ));
        }
    }
    if matches!(pagination_variant, PaginationVariant::Pages | PaginationVariant::Controls) {
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
            &control_border_alpha,
            &disabled_alpha,
            counter,
            output,
        );
    }
}

fn render_dev_android_pagination_indicator(
    props: &ToggleGroupProps,
    page: usize,
    dot: bool,
    pages: &str,
    path: &str,
    action: &str,
    parent: &str,
    contract: PaginationControlContract,
    counter: &mut usize,
    output: &mut String,
) {
    let indicator = next_dev_view(counter);
    let visual_contract = props.pagination_visual_contract();
    let accent_color = java_color(visual_contract.accent);
    let inactive_alpha = format!("{:.2}", visual_contract.inactive_alpha);
    let width = if dot {
        contract.indicator_dot_size
    } else {
        contract.indicator_inactive_width
    };
    let active_width = if dot {
        contract.indicator_dot_size
    } else {
        contract.indicator_active_width
    };
    let height = if dot {
        contract.indicator_dot_size
    } else {
        contract.indicator_height
    };
    output.push_str(&format!(
        "        TextView {indicator} = doweText(\"\", ({parent}Selected == {page}) ? {} : doweAlpha({}, {inactive_alpha}f), 1f, 400, 0f, 1f, null);\n        {indicator}.setContentDescription(\"Go to page {page}\");\n        {indicator}.setGravity(Gravity.CENTER);\n        {indicator}.setBackground(doweBackground(({parent}Selected == {page}) ? {} : doweAlpha({}, {inactive_alpha}f), 999f));\n        {indicator}.setEnabled({});\n        {indicator}.setVisibility({page} <= {pages} ? View.VISIBLE : View.GONE);\n        {indicator}.setOnClickListener(v -> {{ if ({parent}Selected != {page}) {{ doweWrite(\"{path}\", \"{page}\"); {action}renderCurrentRoute(false); }} }});\n        LinearLayout.LayoutParams {indicator}Params = new LinearLayout.LayoutParams(doweDp(({parent}Selected == {page}) ? {active_width} : {width}), doweDp({height}));\n        {indicator}Params.setMargins(doweDp(4), 0, doweDp(4), 0);\n        {parent}.addView({indicator}, {indicator}Params);\n",
        accent_color,
        accent_color,
        accent_color,
        accent_color,
        !props.disabled,
    ));
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
    border_alpha: &str,
    disabled_alpha: &str,
    counter: &mut usize,
    output: &mut String,
) {
    let icon_size = dowe_components::IconButtonGeometryContract::for_size(ButtonSize::Sm).icon_size;
    let accent_color = java_color(props.pagination_visual_contract().accent);
    let button = next_dev_view(counter);
    let icon = solar_control_icon(icon_name).expect("bundled Pagination icon");
    let icon_view = render_dev_android_icon_view(
        &icon,
        counter,
        output,
        Some(accent_color),
    );
    let enabled = if step < 0 {
        format!("{parent}Selected > 1")
    } else {
        format!("{parent}Selected < {pages}")
    };
    output.push_str(&format!(
        "        FrameLayout {button} = doweIconButton({icon_view}, \"{label}\", doweDp({dimension}), doweDp({icon_size}), DOWE_SURFACE, {}, doweAlpha({}, {border_alpha}f));\n        {button}.setEnabled({} && {enabled});\n        {button}.setAlpha({enabled} ? 1f : {disabled_alpha}f);\n        {button}.setOnClickListener(v -> {{ int page = Math.max(1, Math.min({pages}, {parent}Selected + ({step}))); doweWrite(\"{path}\", String.valueOf(page)); {action}renderCurrentRoute(false); }});\n        LinearLayout.LayoutParams {button}Params = new LinearLayout.LayoutParams(doweDp({dimension}), doweDp({dimension}));\n        {button}Params.setMargins(doweDp(4), 0, 0, 0);\n        {parent}.addView({button}, {button}Params);\n",
        accent_color,
        accent_color,
        !props.disabled,
    ));
}
