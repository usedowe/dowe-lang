fn render_dev_android_interactive_display_node(
    node: &ViewNode,
    parent: &str,
    parent_gap: Option<&str>,
    parent_horizontal: bool,
    counter: &mut usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    _inherited_color: Option<String>,
    context: &ComposeReactiveContext,
    children_method: Option<&str>,
) -> bool {
    if !matches!(node, ViewNode::Accordion { .. } | ViewNode::Carousel { .. }) {
        return false;
    }
    match node {
        ViewNode::Accordion { props, items } => {
            let current_font = props.style.style.font.as_ref().or(inherited_font);
            let mut style = props.style.clone();
            style.variant.get_or_insert(ComponentVariant::Ghost);
            let variant = style.variant.unwrap_or(ComponentVariant::Ghost);
            let content_color = dev_card_variant_content(&style);
            let current_color = Some(content_color.to_string());
            let radius = dev_style_radius(&props.style.style);
            let item_background = match variant {
                ComponentVariant::Solid | ComponentVariant::Outlined => {
                    java_color(ColorToken::Surface)
                }
                _ => "Color.TRANSPARENT",
            };
            let outer_border = if variant == ComponentVariant::Outlined {
                java_color(family_color(style.color.unwrap_or(ColorFamily::Primary)))
            } else {
                "null"
            };
            let item_border = match variant {
                ComponentVariant::Solid => "null".to_string(),
                ComponentVariant::Outlined => {
                    format!(
                        "doweAlpha({}, 0.24f)",
                        java_color(family_color(style.color.unwrap_or(ColorFamily::Primary)))
                    )
                }
                ComponentVariant::Ghost | ComponentVariant::Line => {
                    let alpha = if variant == ComponentVariant::Ghost {
                        "0.22f"
                    } else {
                        "0.24f"
                    };
                    format!("doweAlpha({content_color}, {alpha})")
                }
            };
            let view = next_dev_view(counter);
            output.push_str(&format!(
                                        "        LinearLayout {view} = doweAccordion({}, \"{}\", {}, {}, {}, {}, {}, {}, {radius});\n",
                                        props.multiple,
                                        variant.as_str(),
                                        dev_card_variant_container(&style),
                                        content_color,
                                        outer_border,
                                        item_background,
                                        item_border,
                                        variant == ComponentVariant::Solid,
                                    ));
            apply_dev_android_style(&style.style, &view, true, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
            for item in items {
                let arrow = side_nav_submenu_arrow_icon();
                let arrow_view =
                    render_dev_android_icon_view(&arrow, counter, output, Some(content_color));
                let body = next_dev_view(counter);
                output.push_str(&format!(
                    "        LinearLayout {body} = doweAccordionItem({view}, \"{}\", {}, {}, {}, {arrow_view});\n",
                    escape_java(&item.label),
                    item.disabled,
                    item.default_open,
                    dev_font_value(current_font),
                ));
                for child in &item.children {
                    render_dev_android_node(
                        child,
                        &body,
                        Some("8"),
                        false,
                        counter,
                        output,
                        current_font,
                        current_color.clone(),
                        context,
                        children_method,
                    );
                }
            }
        }
        ViewNode::Carousel { props, slides } => {
            let current_font = props.style.style.font.as_ref().or(inherited_font);
            let current_color = Some(dev_variant_content(&props.style).to_string());
            let accent_color = dev_variant_title(&props.style);
            let vertical = props.orientation == CarouselOrientation::Vertical;
            let shows_controls = props.shows_controls();
            let shows_indicators = props.shows_indicators() || props.has_variant_indicators();
            let variant = props.variant.as_str();
            let disable_loop = if props.disable_loop { "true" } else { "false" };
            let view = next_dev_view(counter);
            output.push_str(&format!(
                                        "        LinearLayout {view} = doweContainer(false);\n        {view}.setBackground(doweBackground(Color.TRANSPARENT, DOWE_RADIUS));\n"
                                    ));
            apply_dev_android_style(&props.style.style, &view, true, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
            if let Some(title) = props.title.as_deref() {
                render_dev_android_variant_label(
                    title,
                    &props.style,
                    &view,
                    None,
                    false,
                    counter,
                    output,
                    current_font,
                    context,
                );
            }
            let viewport = next_dev_view(counter);
            let scroll = next_dev_view(counter);
            let track = next_dev_view(counter);
            let horizontal = props.orientation == CarouselOrientation::Horizontal;
            output.push_str(&format!(
                "        FrameLayout {viewport} = new FrameLayout(this);\n        {viewport}.setClipChildren(false);\n        {viewport}.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));\n"
            ));
            if horizontal {
                output.push_str(&format!(
                    "        android.widget.HorizontalScrollView {scroll} = new android.widget.HorizontalScrollView(this);\n        {scroll}.setFillViewport(false);\n        {scroll}.setHorizontalScrollBarEnabled(false);\n        {scroll}.setOverScrollMode(View.OVER_SCROLL_NEVER);\n        {scroll}.setNestedScrollingEnabled(true);\n        {scroll}.setLayoutParams(new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));\n        LinearLayout {track} = doweContainer(true);\n        {track}.setGravity(Gravity.CENTER_VERTICAL);\n"
                ));
                if props.variant == CarouselVariant::Rtl {
                    output.push_str(&format!(
                        "        {track}.setLayoutDirection(View.LAYOUT_DIRECTION_RTL);\n"
                    ));
                }
                output.push_str(&format!(
                    "        {scroll}.addView({track}, new android.widget.HorizontalScrollView.LayoutParams(ViewGroup.LayoutParams.WRAP_CONTENT, ViewGroup.LayoutParams.WRAP_CONTENT));\n        {viewport}.addView({scroll});\n        doweAdd({view}, {viewport});\n"
                ));
            } else {
                output.push_str(&format!(
                    "        ScrollView {scroll} = new ScrollView(this);\n        {scroll}.setFillViewport(false);\n        {scroll}.setVerticalScrollBarEnabled(false);\n        {scroll}.setOverScrollMode(View.OVER_SCROLL_NEVER);\n        {scroll}.setLayoutParams(new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));\n        LinearLayout {track} = doweContainer(false);\n        {scroll}.addView({track}, new ScrollView.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));\n        {viewport}.addView({scroll});\n        doweAdd({view}, {viewport});\n"
                ));
            }
            let slide_width = props.slide_width.unwrap_or(
                if matches!(
                    props.variant,
                    CarouselVariant::Simple
                        | CarouselVariant::Masonry
                        | CarouselVariant::Rtl
                        | CarouselVariant::Sticky
                ) {
                    280
                } else {
                    320
                },
            );
            for slide in slides {
                let slide_view = next_dev_view(counter);
                output.push_str(&format!(
                    "        LinearLayout {slide_view} = doweContainer(false);\n        {slide_view}.setClipToPadding(false);\n"
                ));
                if horizontal {
                    output.push_str(&format!(
                        "        {slide_view}.setLayoutParams(new LinearLayout.LayoutParams(doweDp({slide_width}), ViewGroup.LayoutParams.WRAP_CONTENT));\n"
                    ));
                }
                if let Some(height) = props.slide_height {
                    output.push_str(&format!(
                        "        {slide_view}.setMinimumHeight(doweDp({height}));\n"
                    ));
                }
                output.push_str(&format!(
                    "        doweAdd({track}, {slide_view}, doweDp({}), {});\n",
                    props.gap, horizontal
                ));
                for child in &slide.children {
                    render_dev_android_node(
                        child,
                        &slide_view,
                        None,
                        false,
                        counter,
                        output,
                        current_font,
                        current_color.clone(),
                        context,
                        children_method,
                    );
                }
            }
            let carousel_index = next_dev_view(counter);
            let carousel_variant = next_dev_view(counter);
            let carousel_indicators = next_dev_view(counter);
            let carousel_counter = next_dev_view(counter);
            let carousel_previous = next_dev_view(counter);
            let carousel_next = next_dev_view(counter);
            let carousel_control_previous = next_dev_view(counter);
            let carousel_control_next = next_dev_view(counter);
            let carousel_update = next_dev_view(counter);
            output.push_str(&format!(
                "        int[] {carousel_index} = new int[] {{0}};\n        String {carousel_variant} = \"{}\";\n        ArrayList<Button> {carousel_indicators} = new ArrayList<>();\n        TextView[] {carousel_counter} = new TextView[1];\n        Button[] {carousel_previous} = new Button[1];\n        Button[] {carousel_next} = new Button[1];\n        Button[] {carousel_control_previous} = new Button[1];\n        Button[] {carousel_control_next} = new Button[1];\n",
                escape_java(variant)
            ));
            if props.show_navigation {
                let previous = next_dev_view(counter);
                let next = next_dev_view(counter);
                output.push_str(&format!(
                    "        Button {previous} = new Button(this);\n        {previous}.setText(\"‹\");\n        {previous}.setTextSize(22f);\n        {previous}.setTextColor({accent_color});\n        {previous}.setAllCaps(false);\n        {previous}.setMinimumHeight(doweDp(32));\n        {previous}.setBackgroundColor(Color.TRANSPARENT);\n        {previous}.setPadding(0, 0, 0, 0);\n        {previous}.setContentDescription(\"Previous slide\");\n        {previous}.setOnClickListener(target -> {{\n            int targetIndex = {carousel_index}[0] - 1;\n            if (targetIndex < 0) targetIndex = {disable_loop} ? 0 : Math.max(0, {track}.getChildCount() - 1);\n            final int selectedIndex = targetIndex;\n            {scroll}.post(() -> {{\n                if ({track}.getChildCount() == 0) return;\n                View slide = {track}.getChildAt(selectedIndex);\n                if ({vertical}) {scroll}.smoothScrollTo(0, slide.getTop()); else {scroll}.smoothScrollTo(slide.getLeft(), 0);\n            }});\n        }});\n        {carousel_previous}[0] = {previous};\n        FrameLayout.LayoutParams {previous}Params = new FrameLayout.LayoutParams(doweDp(36), doweDp(36), Gravity.START | Gravity.CENTER_VERTICAL);\n        {previous}Params.leftMargin = doweDp(8);\n        {viewport}.addView({previous}, {previous}Params);\n        Button {next} = new Button(this);\n        {next}.setText(\"›\");\n        {next}.setTextSize(22f);\n        {next}.setTextColor({accent_color});\n        {next}.setAllCaps(false);\n        {next}.setMinimumHeight(doweDp(32));\n        {next}.setBackgroundColor(Color.TRANSPARENT);\n        {next}.setPadding(0, 0, 0, 0);\n        {next}.setContentDescription(\"Next slide\");\n        {next}.setOnClickListener(target -> {{\n            int targetIndex = {carousel_index}[0] + 1;\n            int last = Math.max(0, {track}.getChildCount() - 1);\n            if (targetIndex > last) targetIndex = {disable_loop} ? last : 0;\n            final int selectedNextIndex = targetIndex;\n            {scroll}.post(() -> {{\n                if ({track}.getChildCount() == 0) return;\n                View slide = {track}.getChildAt(selectedNextIndex);\n                if ({vertical}) {scroll}.smoothScrollTo(0, slide.getTop()); else {scroll}.smoothScrollTo(slide.getLeft(), 0);\n            }});\n        }});\n        {carousel_next}[0] = {next};\n        FrameLayout.LayoutParams {next}Params = new FrameLayout.LayoutParams(doweDp(36), doweDp(36), Gravity.END | Gravity.CENTER_VERTICAL);\n        {next}Params.rightMargin = doweDp(8);\n        {viewport}.addView({next}, {next}Params);\n",
                ));
            }
            if shows_controls {
                let controls = next_dev_view(counter);
                let previous = next_dev_view(counter);
                let next = next_dev_view(counter);
                output.push_str(&format!(
                    "        LinearLayout {controls} = doweContainer(true);\n        {controls}.setGravity(Gravity.CENTER_VERTICAL);\n        Button {previous} = new Button(this);\n        {previous}.setText(\"Previous\");\n        {previous}.setTextColor({accent_color});\n        {previous}.setAllCaps(false);\n        {previous}.setMinimumHeight(doweDp(32));\n        {previous}.setTextSize(14f);\n        {previous}.setBackgroundColor(Color.TRANSPARENT);\n        {previous}.setPadding(doweDp(6), 0, doweDp(6), 0);\n        {previous}.setOnClickListener(target -> {{\n            int targetIndex = {carousel_index}[0] - 1;\n            if (targetIndex < 0) targetIndex = {disable_loop} ? 0 : Math.max(0, {track}.getChildCount() - 1);\n            final int selectedIndex = targetIndex;\n            {scroll}.post(() -> {{\n                if ({track}.getChildCount() == 0) return;\n                View slide = {track}.getChildAt(selectedIndex);\n                if ({vertical}) {scroll}.smoothScrollTo(0, slide.getTop()); else {scroll}.smoothScrollTo(slide.getLeft(), 0);\n            }});\n        }});\n        {carousel_control_previous}[0] = {previous};\n        doweAdd({controls}, {previous}, 8, false);\n        Button {next} = new Button(this);\n        {next}.setText(\"Next\");\n        {next}.setTextColor({accent_color});\n        {next}.setAllCaps(false);\n        {next}.setMinimumHeight(doweDp(32));\n        {next}.setTextSize(14f);\n        {next}.setBackgroundColor(Color.TRANSPARENT);\n        {next}.setPadding(doweDp(6), 0, doweDp(6), 0);\n        {next}.setOnClickListener(target -> {{\n            int targetIndex = {carousel_index}[0] + 1;\n            int last = Math.max(0, {track}.getChildCount() - 1);\n            if (targetIndex > last) targetIndex = {disable_loop} ? last : 0;\n            final int selectedNextIndex = targetIndex;\n            {scroll}.post(() -> {{\n                if ({track}.getChildCount() == 0) return;\n                View slide = {track}.getChildAt(selectedNextIndex);\n                if ({vertical}) {scroll}.smoothScrollTo(0, slide.getTop()); else {scroll}.smoothScrollTo(slide.getLeft(), 0);\n            }});\n        }});\n        {carousel_control_next}[0] = {next};\n        doweAdd({controls}, {next}, 8, false);\n        doweAdd({view}, {controls}, 8, false);\n"
                ));
            }
            if shows_indicators {
                let indicators = next_dev_view(counter);
                output.push_str(&format!(
                    "        LinearLayout {indicators} = doweContainer(true);\n        {indicators}.setGravity(Gravity.CENTER);\n"
                ));
                for index in 0..slides.len() {
                    let indicator = next_dev_view(counter);
                    let label = if props.variant == CarouselVariant::Thumbnails {
                        format!("Slide {}", index + 1)
                    } else if props.variant == CarouselVariant::Dots
                        || props.indicator_type.as_str() == "dot"
                    {
                        "•".to_string()
                    } else {
                        (index + 1).to_string()
                    };
                    output.push_str(&format!(
                        "        Button {indicator} = new Button(this);\n        {indicator}.setText(\"{}\");\n        {indicator}.setTextSize({}f);\n        {indicator}.setAllCaps(false);\n        {indicator}.setPadding(doweDp(4), 0, doweDp(4), 0);\n        {indicator}.setTextColor({accent_color});\n        {indicator}.setMinimumHeight(doweDp(28));\n        {indicator}.setBackgroundColor(Color.TRANSPARENT);\n        final int {indicator}Index = {index};\n        {indicator}.setOnClickListener(target -> {scroll}.post(() -> {{\n            if ({track}.getChildCount() == 0) return;\n            View slide = {track}.getChildAt({indicator}Index);\n            if ({vertical}) {scroll}.smoothScrollTo(0, slide.getTop()); else {scroll}.smoothScrollTo(slide.getLeft(), 0);\n        }}));\n        {carousel_indicators}.add({indicator});\n        doweAdd({indicators}, {indicator}, 4, true);\n",
                        escape_java(&label),
                        if props.variant == CarouselVariant::Thumbnails { "12" } else { "18" },
                    ));
                }
                output.push_str(&format!(
                    "        doweAdd({view}, {indicators}, 8, false);\n"
                ));
            }
            if props.show_counter {
                let counter_view = next_dev_view(counter);
                output.push_str(&format!(
                    "        TextView {counter_view} = doweText(\"1 / {}\", {accent_color}, 13f, 600, 0f, 1.2f, {});\n        {counter_view}.setGravity(Gravity.CENTER);\n        {carousel_counter}[0] = {counter_view};\n        doweAdd({view}, {counter_view}, 4, false);\n",
                    slides.len(),
                    dev_font_value(current_font),
                ));
            }
            output.push_str(&format!(
                "        Runnable {carousel_update} = () -> {{\n            int viewportCenter = {vertical} ? {scroll}.getScrollY() + {scroll}.getHeight() / 2 : {scroll}.getScrollX() + {scroll}.getWidth() / 2;\n            int viewportSize = Math.max(1, {vertical} ? {scroll}.getHeight() : {scroll}.getWidth());\n            int active = 0;\n            float activeDistance = Float.MAX_VALUE;\n            for (int index = 0; index < {track}.getChildCount(); index++) {{\n                View slide = {track}.getChildAt(index);\n                float center = {vertical} ? slide.getTop() + slide.getHeight() / 2f : slide.getLeft() + slide.getWidth() / 2f;\n                float phase = Math.max(-1f, Math.min(1f, (center - viewportCenter) / (float) viewportSize));\n                float distance = Math.min(1f, Math.abs(phase));\n                slide.setRotationY(0f);\n                slide.setRotation(0f);\n                slide.setScaleX(1f);\n                slide.setScaleY(1f);\n                slide.setTranslationX(0f);\n                slide.setTranslationY(0f);\n                slide.setAlpha(1f);\n                if (\"coverFlow\".equals({carousel_variant})) {{ slide.setCameraDistance(doweDp(24)); slide.setRotationY(phase * 24f); slide.setScaleX(1f - distance * 0.1f); slide.setScaleY(1f - distance * 0.1f); slide.setAlpha(1f - distance * 0.22f); }}\n                else if (\"stories\".equals({carousel_variant})) {{ slide.setCameraDistance(doweDp(24)); slide.setRotationY(phase * 30f); slide.setScaleX(1f - distance * 0.1f); slide.setScaleY(1f - distance * 0.1f); slide.setAlpha(1f - distance * 0.22f); }}\n                else if (\"smartStack\".equals({carousel_variant})) {{ slide.setRotation(phase * 1.5f); slide.setScaleX(1f - distance * 0.055f); slide.setScaleY(1f - distance * 0.055f); slide.setTranslationY(doweDp(8) * distance); }}\n                else if (\"cardStack\".equals({carousel_variant})) {{ slide.setScaleX(1f - distance * 0.055f); slide.setScaleY(1f - distance * 0.055f); slide.setTranslationY(doweDp(8) * distance); }}\n                else if (\"flipbook\".equals({carousel_variant})) {{ slide.setCameraDistance(doweDp(24)); slide.setRotationY(phase * 52f); slide.setScaleX(1f - distance * 0.1f); slide.setScaleY(1f - distance * 0.1f); slide.setAlpha(1f - distance * 0.22f); }}\n                else if (\"slideshow\".equals({carousel_variant})) {{ if ({vertical}) slide.setTranslationY(doweDp(24) * phase); else slide.setTranslationX(doweDp(24) * phase); slide.setAlpha(1f - distance * 0.12f); }}\n                if (Math.abs(phase) < activeDistance) {{ active = index; activeDistance = Math.abs(phase); }}\n            }}\n            {carousel_index}[0] = active;\n            if ({carousel_previous}[0] != null) {carousel_previous}[0].setEnabled(!{disable_loop} || active > 0);\n            if ({carousel_next}[0] != null) {carousel_next}[0].setEnabled(!{disable_loop} || active < Math.max(0, {track}.getChildCount() - 1));\n            if ({carousel_control_previous}[0] != null) {carousel_control_previous}[0].setEnabled(!{disable_loop} || active > 0);\n            if ({carousel_control_next}[0] != null) {carousel_control_next}[0].setEnabled(!{disable_loop} || active < Math.max(0, {track}.getChildCount() - 1));\n            for (int index = 0; index < {carousel_indicators}.size(); index++) {{ Button indicator = {carousel_indicators}.get(index); boolean selected = index == active; indicator.setBackgroundColor(Color.TRANSPARENT); indicator.setTextColor(selected ? {accent_color} : doweAlpha({accent_color}, 0.45f)); }}\n        }};\n        {scroll}.setOnScrollChangeListener((target, scrollX, scrollY, oldScrollX, oldScrollY) -> {carousel_update}.run());\n        {viewport}.post({carousel_update});\n",
            ));
            if props.show_counter {
                output.push_str(&format!(
                    "        {scroll}.setOnScrollChangeListener((target, scrollX, scrollY, oldScrollX, oldScrollY) -> {{ {carousel_update}.run(); if ({carousel_counter}[0] != null) {carousel_counter}[0].setText(String.valueOf({carousel_index}[0] + 1) + \" / \" + String.valueOf({track}.getChildCount())); }});\n"
                ));
            }
            if !matches!(
                props.variant,
                CarouselVariant::Simple
                    | CarouselVariant::Masonry
                    | CarouselVariant::Rtl
                    | CarouselVariant::Sticky
            ) {
                let snap_step = if vertical {
                    props.slide_height.unwrap_or(280).saturating_add(props.gap)
                } else {
                    slide_width.saturating_add(props.gap)
                };
                output.push_str(&format!(
                    "        {scroll}.setOnTouchListener((target, event) -> {{\n            if (event.getAction() == android.view.MotionEvent.ACTION_UP || event.getAction() == android.view.MotionEvent.ACTION_CANCEL) {{\n                int step = doweDp({snap_step});\n                int page = Math.round((float) ({vertical} ? {scroll}.getScrollY() : {scroll}.getScrollX()) / Math.max(1, step));\n                if ({vertical}) {scroll}.post(() -> {scroll}.smoothScrollTo(0, page * step)); else {scroll}.post(() -> {scroll}.smoothScrollTo(page * step, 0));\n            }}\n            return false;\n        }});\n"
                ));
            }
        }
        _ => {}
    }
    true
}
