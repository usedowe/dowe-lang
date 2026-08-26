fn render_dev_android_display_media_data_node(
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
) {
    match node {
        ViewNode::Audio { props } => {
            let play = solar_control_icon("play").expect("bundled Audio play icon");
            let pause = solar_control_icon("pause").expect("bundled Audio pause icon");
            let content_color = dev_card_variant_content(&props.style);
            let mut button_style = props.style.clone();
            button_style.variant = Some(
                if props.style.variant.unwrap_or(ComponentVariant::Solid) == ComponentVariant::Solid
                {
                    ComponentVariant::Solid
                } else {
                    ComponentVariant::Solid
                },
            );
            let button_background = dev_variant_container(&button_style);
            let button_content = dev_variant_content(&button_style);
            let play_view =
                render_dev_android_icon_view(&play, counter, output, Some(button_content));
            let pause_view =
                render_dev_android_icon_view(&pause, counter, output, Some(button_content));
            let view = next_dev_view(counter);
            let subtitle = props
                .subtitle
                .as_deref()
                .map(escape_java)
                .map(|value| format!("\"{value}\""))
                .unwrap_or_else(|| "null".to_string());
            let avatar = props
                .avatar_src
                .as_deref()
                .map(escape_java)
                .map(|value| format!("\"{value}\""))
                .unwrap_or_else(|| "null".to_string());
            output.push_str(&format!(
                "        LinearLayout {view} = doweAudio(\"{}\", {}, {}, {}, {}, {}, {}, {}, {play_view}, {pause_view});\n",
                escape_java(&props.src),
                subtitle,
                avatar,
                dev_card_variant_container(&props.style),
                content_color,
                button_background,
                button_content,
                dev_card_border(&props.style),
            ));
            apply_dev_android_style(&props.style.style, &view, false, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        ViewNode::Camera { props } => {
            let view = next_dev_view(counter);
            let on_start = props
                .on_start
                .as_deref()
                .and_then(|value| context.action_id(value))
                .map(|value| format!("\"{}\"", escape_java(value)))
                .unwrap_or_else(|| "null".to_string());
            let on_capture = props
                .on_capture
                .as_deref()
                .and_then(|value| context.action_id(value))
                .map(|value| format!("\"{}\"", escape_java(value)))
                .unwrap_or_else(|| "null".to_string());
            let on_error = props
                .on_error
                .as_deref()
                .and_then(|value| context.action_id(value))
                .map(|value| format!("\"{}\"", escape_java(value)))
                .unwrap_or_else(|| "null".to_string());
            output.push_str(&format!(
                "        Button {view} = new Button(this);\n        {view}.setText(\"{}\");\n        {view}.setAllCaps(false);\n        {view}.setEnabled({});\n        {view}.setOnClickListener(target -> doweOpenCamera(\"{}\", {on_start}, {on_capture}, {on_error}));\n",
                escape_java(&props.label),
                !props.disabled,
                props.facing.as_str(),
            ));
            apply_dev_android_style(&props.style.style, &view, false, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        ViewNode::Microphone { props } => {
            let view = next_dev_view(counter);
            let start = props
                .on_start
                .as_deref()
                .and_then(|value| context.action_id(value))
                .map(|value| format!("\"{}\"", escape_java(value)))
                .unwrap_or_else(|| "null".to_string());
            let stop = props
                .on_stop
                .as_deref()
                .and_then(|value| context.action_id(value))
                .map(|value| format!("\"{}\"", escape_java(value)))
                .unwrap_or_else(|| "null".to_string());
            let error = props
                .on_error
                .as_deref()
                .and_then(|value| context.action_id(value))
                .map(|value| format!("\"{}\"", escape_java(value)))
                .unwrap_or_else(|| "null".to_string());
            let duration = props
                .max_duration
                .map(|value| value.to_string())
                .unwrap_or_else(|| "0".to_string());
            output.push_str(&format!(
                "        LinearLayout {view} = doweContainer(true);\n        Button {view}Start = new Button(this);\n        {view}Start.setText(\"{}\");\n        {view}Start.setAllCaps(false);\n        {view}Start.setEnabled({});\n        {view}Start.setOnClickListener(target -> doweStartMicrophone({start}, {stop}, {error}, {duration}));\n        doweAdd({view}, {view}Start);\n",
                escape_java(&props.label),
                !props.disabled,
            ));
            apply_dev_android_style(&props.style.style, &view, false, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        ViewNode::Image { props } => {
            let view = next_dev_view(counter);
            let source = props
                .reactive_src
                .as_deref()
                .map(|path| dev_text_expression(path, None, context))
                .unwrap_or_else(|| format!("\"{}\"", escape_java(&props.src)));
            output.push_str(&format!(
                                        "        FrameLayout {view} = doweImage({source}, \"{}\", \"{}\", \"{}\", {}, {});\n",
                                        escape_java(&props.alt),
                                        props.aspect.as_str(),
                                        props.object_fit.as_str(),
                                        dev_card_variant_container(&props.style),
                                        dev_card_border(&props.style)
                                    ));
            apply_dev_android_style(&props.style.style, &view, false, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
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
        ViewNode::Code { props } => {
            let view = next_dev_view(counter);
            let source = if props.template_segments.is_empty() {
                format!("\"{}\"", escape_java(&props.source))
            } else {
                props
                    .template_segments
                    .iter()
                    .map(|segment| match segment {
                        CodeTemplateSegment::Static { text, .. } => {
                            format!("\"{}\"", escape_java(text))
                        }
                        CodeTemplateSegment::Binding(path) => format!(
                            "doweTextValue(\"{}\", null)",
                            escape_java(&context.signal_path(path))
                        ),
                    })
                    .collect::<Vec<_>>()
                    .join(" + ")
            };
            let (texts, colors) = if props.template_segments.is_empty() {
                (
                    java_string_array(props.tokens.iter().map(|token| token.text.as_str())),
                    java_int_array(props.tokens.iter().map(|token| {
                        dev_code_token_color(token.kind, dev_card_variant_content(&props.style))
                    })),
                )
            } else {
                let mut texts = Vec::new();
                let mut colors = Vec::new();
                for segment in &props.template_segments {
                    match segment {
                        CodeTemplateSegment::Static { tokens, .. } => {
                            for token in tokens {
                                texts.push(format!("\"{}\"", escape_java(&token.text)));
                                colors.push(dev_code_token_color(
                                    token.kind,
                                    dev_card_variant_content(&props.style),
                                ));
                            }
                        }
                        CodeTemplateSegment::Binding(path) => {
                            texts.push(format!(
                                "doweTextValue(\"{}\", null)",
                                escape_java(&context.signal_path(path))
                            ));
                            colors.push(dev_card_variant_content(&props.style).to_string());
                        }
                    }
                }
                (
                    format!("new String[]{{{}}}", texts.join(", ")),
                    format!("new int[]{{{}}}", colors.join(", ")),
                )
            };
            output.push_str(&format!(
                                        "        LinearLayout {view} = doweCode({source}, \"{}\", {texts}, {colors}, \"{}\", \"{}\", {}, {}, {});\n",
                                        props.language.as_str(),
                                        escape_java(&props.copy_label),
                                        escape_java(&props.copied_label),
                                        dev_card_variant_container(&props.style),
                                        dev_card_variant_content(&props.style),
                                        dev_card_border(&props.style)
                                    ));
            apply_dev_android_style(&props.style.style, &view, false, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        ViewNode::Video { props } => {
            let view = next_dev_view(counter);
            let play_icon = render_dev_android_icon_view(
                &solar_control_icon("play").expect("bundled Video play icon"),
                counter,
                output,
                Some("Color.WHITE"),
            );
            let pause_icon = render_dev_android_icon_view(
                &solar_control_icon("pause").expect("bundled Video pause icon"),
                counter,
                output,
                Some("Color.WHITE"),
            );
            let volume_icon = render_dev_android_icon_view(
                &solar_control_icon("volume-loud").expect("bundled Video volume icon"),
                counter,
                output,
                Some("Color.WHITE"),
            );
            let muted_icon = render_dev_android_icon_view(
                &solar_control_icon("volume-cross").expect("bundled Video muted icon"),
                counter,
                output,
                Some("Color.WHITE"),
            );
            let picture_in_picture_icon = render_dev_android_icon_view(
                &solar_control_icon("pip").expect("bundled Video picture-in-picture icon"),
                counter,
                output,
                Some("Color.WHITE"),
            );
            let fullscreen_icon = render_dev_android_icon_view(
                &solar_control_icon("full-screen").expect("bundled Video fullscreen icon"),
                counter,
                output,
                Some("Color.WHITE"),
            );
            let poster = props
                .poster
                .as_deref()
                .map(|value| format!("\"{}\"", escape_java(value)))
                .unwrap_or_else(|| "null".to_string());
            output.push_str(&format!(
                "        FrameLayout {view} = doweVideo(\"{}\", {poster}, {}, \"{}\", {}, {}, {play_icon}, {pause_icon}, {volume_icon}, {muted_icon}, {picture_in_picture_icon}, {fullscreen_icon});\n",
                escape_java(&props.src),
                props.autoplay,
                props.aspect.as_str(),
                dev_card_variant_container(&props.style),
                dev_card_border(&props.style)
            ));
            apply_dev_android_style(&props.style.style, &view, false, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        ViewNode::Iframe { props } => {
            let view = next_dev_view(counter);
            let scripts = props
                .sandbox
                .as_ref()
                .map(|tokens| tokens.iter().any(|token| token == "scripts"))
                .unwrap_or(true);
            output.push_str(&format!(
                "        FrameLayout {view} = doweIframe(\"{}\", \"{}\", {}, {});\n",
                escape_java(&props.src),
                escape_java(&props.title),
                scripts,
                props.allow.iter().any(|token| token == "autoplay"),
            ));
            apply_dev_android_style(&props.style, &view, false, output);
            if props.style.border.is_some() {
                output.push_str(&format!(
                    "        {view}.setPadding(doweDp(1), doweDp(1), doweDp(1), doweDp(1));\n"
                ));
            }
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        ViewNode::Device { props, iframe } => {
            let view = next_dev_view(counter);
            let scripts = iframe
                .sandbox
                .as_ref()
                .map(|tokens| tokens.iter().any(|token| token == "scripts"))
                .unwrap_or(true);
            let mut options = Vec::new();
            for (index, option) in props.options.iter().enumerate() {
                let paths_name = format!("{view}DevicePaths{index}");
                let icon_name = format!("{view}DeviceIcon{index}");
                output.push_str(&format!(
                    "        ArrayList<DoweSvgPathEntry> {paths_name} = new ArrayList<>();\n"
                ));
                for path in &option.icon.paths {
                    output.push_str(&format!(
                        "        {paths_name}.add(new DoweSvgPathEntry(\"{}\", {}, {}, {}, {}));\n",
                        escape_java(&path.data),
                        dev_svg_path_current_color(path.fill),
                        dev_svg_path_color(path.fill),
                        dev_svg_path_details(path.fill),
                        dev_svg_path_transform(path.transform.as_ref())
                    ));
                }
                output.push_str(&format!(
                    "        DoweSvgView {icon_name} = new DoweSvgView(this, {}f, {}f, {}f, {}f, DOWE_BACKGROUND_TEXT, {paths_name});\n",
                    option.icon.props.view_box.min_x,
                    option.icon.props.view_box.min_y,
                    option.icon.props.view_box.width,
                    option.icon.props.view_box.height,
                ));
                options.push(format!(
                    "new DoweDeviceOption(\"{}\", {icon_name})",
                    option.profile.as_str()
                ));
            }
            output.push_str(&format!(
                "        FrameLayout {view} = doweDevice(\"{}\", \"{}\", \"{}\", {}, {}, new DoweDeviceOption[] {{{}}});\n",
                props.device.as_str(),
                escape_java(&iframe.src),
                escape_java(&iframe.title),
                scripts,
                iframe.allow.iter().any(|token| token == "autoplay"),
                options.join(", "),
            ));
            apply_dev_android_style(&props.style, &view, true, output);
            if props.style.border.is_some() {
                output.push_str(&format!(
                    "        {view}.setPadding(doweDp(1), doweDp(1), doweDp(1), doweDp(1));\n"
                ));
            }
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        ViewNode::Canvas { props } => {
            let view = next_dev_view(counter);
            let on_pointer = props
                .on_pointer
                .as_deref()
                .and_then(|value| context.action_id(value))
                .map(|value| format!("\"{}\"", escape_java(value)))
                .unwrap_or_else(|| "null".to_string());
            let on_key = props
                .on_key
                .as_deref()
                .and_then(|value| context.action_id(value))
                .map(|value| format!("\"{}\"", escape_java(value)))
                .unwrap_or_else(|| "null".to_string());
            let on_motion = props
                .on_motion
                .as_deref()
                .and_then(|value| context.action_id(value))
                .map(|value| format!("\"{}\"", escape_java(value)))
                .unwrap_or_else(|| "null".to_string());
            let background = match props.background {
                CanvasBackground::Transparent => "Color.TRANSPARENT".to_string(),
                CanvasBackground::Color(color) => java_color(color).to_string(),
            };
            let border_width = props
                .style
                .border
                .as_ref()
                .map(dev_border_value)
                .unwrap_or_else(|| "null".to_string());
            let border_color = props
                .style
                .border_color
                .map(family_color)
                .map(java_color)
                .unwrap_or("DOWE_BACKGROUND_TEXT");
            output.push_str(&format!(
                "        DoweCanvasView {view} = doweCanvas(\"{}\", {}f, {}f, \"{}\", {}, {}, {}, {}, \"{}\", {on_pointer}, {on_key}, {on_motion}, {}, {border_width}, {border_color}, {});\n",
                escape_java(&context.signal_path(&props.scene)),
                props.view_width,
                props.view_height,
                props.fit.as_str(),
                props.fps,
                props.autoplay,
                props.pixelated,
                background,
                escape_java(&props.label),
                props.motion_rate,
                dev_style_radius(&props.style),
            ));
            apply_dev_android_style(&props.style, &view, false, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        ViewNode::Diagram { props } => {
            let view = next_dev_view(counter);
            let on_node_click = props
                .on_node_click
                .as_deref()
                .and_then(|value| context.action_id(value))
                .map(|value| format!("\"{}\"", escape_java(value)))
                .unwrap_or_else(|| "null".to_string());
            let on_node_drag = props
                .on_node_drag
                .as_deref()
                .and_then(|value| context.action_id(value))
                .map(|value| format!("\"{}\"", escape_java(value)))
                .unwrap_or_else(|| "null".to_string());
            let on_connect = props
                .on_connect
                .as_deref()
                .and_then(|value| context.action_id(value))
                .map(|value| format!("\"{}\"", escape_java(value)))
                .unwrap_or_else(|| "null".to_string());
            output.push_str(&format!(
                "        DoweDiagramView {view} = doweDiagram(\"{}\", \"{}\", {}, {}, {}, {}, {}, {}, {}, {on_node_click}, {on_node_drag}, {on_connect}, DOWE_SURFACE, DOWE_BACKGROUND_TEXT, {});\n",
                escape_java(&context.signal_path(&props.nodes)),
                escape_java(&context.signal_path(&props.edges)),
                props.fit_view,
                props.pan_on_drag,
                props.zoom_on_scroll,
                props.controls,
                props.minimap,
                props.show_grid,
                format!("\"{}\"", escape_java(&props.empty_label)),
                dev_style_radius(&props.style.style),
            ));
            apply_dev_android_style(&props.style.style, &view, false, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        ViewNode::Candlestick { props } => {
            let view = next_dev_view(counter);
            let stream = props
                .stream
                .as_deref()
                .map(|value| format!("\"{}\"", escape_java(value)))
                .unwrap_or_else(|| "null".to_string());
            output.push_str(&format!(
                                        "        DoweCandlestickView {view} = doweCandlestick(\"{}\", {stream}, {}, {}, \"{}\", {}, {}, {}, {});\n",
                                        escape_java(&context.signal_path(&props.data)),
                                        java_color(props.up_color),
                                        java_color(props.down_color),
                                        escape_java(&props.empty_label),
                                        props.max_points,
                                        dev_card_variant_container(&props.style),
                                        dev_card_variant_content(&props.style),
                                        dev_card_border(&props.style)
                                    ));
            apply_dev_android_style(&props.style.style, &view, false, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        ViewNode::ArcChart { props } => {
            render_dev_android_chart(
                "arc",
                &props.common,
                None,
                Some(props),
                parent,
                parent_gap,
                parent_horizontal,
                counter,
                context,
                output,
            );
        }
        ViewNode::AreaChart { props } => {
            render_dev_android_chart(
                "area",
                &props.common,
                None,
                None,
                parent,
                parent_gap,
                parent_horizontal,
                counter,
                context,
                output,
            );
        }
        ViewNode::BarChart { props } => {
            render_dev_android_chart(
                "bar",
                &props.common,
                None,
                None,
                parent,
                parent_gap,
                parent_horizontal,
                counter,
                context,
                output,
            );
        }
        ViewNode::LineChart { props } => {
            render_dev_android_chart(
                "line",
                &props.common,
                None,
                None,
                parent,
                parent_gap,
                parent_horizontal,
                counter,
                context,
                output,
            );
        }
        ViewNode::PieChart { props } => {
            render_dev_android_chart(
                "pie",
                &props.common,
                Some(props),
                None,
                parent,
                parent_gap,
                parent_horizontal,
                counter,
                context,
                output,
            );
        }
        ViewNode::Table { props } => {
            let view = next_dev_view(counter);
            render_dev_android_table(props, &view, &context.signal_path(&props.data), output);
            apply_dev_android_style(&props.style.style, &view, false, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        _ => {}
    }
}

fn render_dev_android_chart(
    chart_type: &str,
    props: &ChartCommonProps,
    pie_props: Option<&PieChartProps>,
    arc_props: Option<&ArcChartProps>,
    parent: &str,
    parent_gap: Option<&str>,
    parent_horizontal: bool,
    counter: &mut usize,
    context: &ComposeReactiveContext,
    output: &mut String,
) {
    let view = next_dev_view(counter);
    let data_path = props
        .data
        .as_deref()
        .map(|value| format!("\"{}\"", escape_java(&context.signal_path(value))))
        .unwrap_or_else(|| "null".to_string());
    let series_path = props
        .series
        .as_deref()
        .map(|value| format!("\"{}\"", escape_java(&context.signal_path(value))))
        .unwrap_or_else(|| "null".to_string());
    let pie_args = pie_props
        .map(|pie| {
            format!(
                ", {}, {}, {}, {}, {}, {}, {}, {}, {}, {}",
                pie.donut,
                pie.donut_width,
                pie.center_label
                    .as_deref()
                    .map(|value| format!("\"{}\"", escape_java(value)))
                    .unwrap_or_else(|| "null".to_string()),
                pie.center_value
                    .as_deref()
                    .map(|value| format!("\"{}\"", escape_java(value)))
                    .unwrap_or_else(|| "null".to_string()),
                pie.start_angle,
                pie.pad_angle,
                pie.hide_labels,
                pie.hide_values,
                pie.hide_percentages,
                pie.show_glow,
            )
        })
        .or_else(|| {
            arc_props.map(|arc| {
                format!(
                    ", false, 60, null, null, {}, 0, false, false, false, false",
                    arc.start_angle
                )
            })
        })
        .unwrap_or_else(|| {
            ", false, 60, null, null, -90, 0, false, false, false, false".to_string()
        });
    let arc_args = arc_props
        .map(|arc| {
            format!(
                ", {}, {}, {}, {}, {}, {}, {}",
                arc.center_text
                    .as_deref()
                    .map(|value| format!("\"{}\"", escape_java(value)))
                    .unwrap_or_else(|| "null".to_string()),
                arc.thickness,
                arc.gap,
                arc.end_angle,
                arc.show_inline_labels,
                arc.hide_values,
                arc.show_glow,
            )
        })
        .unwrap_or_else(|| ", null, 16, 8, 270, false, false, false".to_string());
    output.push_str(&format!(
        "        DoweChartView {view} = doweChart(\"{}\", {data_path}, {series_path}, \"{}\", \"{}\", \"{}\", {}, {}, {}, {}, {}{pie_args}{arc_args});\n",
        escape_java(chart_type),
        props.palette.as_str(),
        props.legend_position.as_str(),
        escape_java(&props.empty_label),
        props.loading,
        props.hide_legend,
        dev_card_variant_container(&props.style),
        dev_card_variant_content(&props.style),
        dev_card_border(&props.style)
    ));
    apply_dev_android_style(&props.style.style, &view, false, output);
    output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
}
