fn render_dev_android_carousel_display_node(
    props: &CarouselProps,
    slides: &[CarouselSlide],
    parent: &str,
    parent_gap: Option<&str>,
    parent_horizontal: bool,
    counter: &mut usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    context: &ComposeReactiveContext,
    children_method: Option<&str>,
) {
    let current_font = props.style.style.font.as_ref().or(inherited_font);
    let current_color = Some(dev_variant_content(&props.style).to_string());
    let visual_contract = props.visual_contract();
    let accent_color = java_color(visual_contract.accent);
    let control_border_alpha = format!("{:.2}", visual_contract.control_border_alpha);
    let indicator_inactive_alpha = format!("{:.2}", visual_contract.indicator_inactive_alpha);
    let vertical = props.orientation == CarouselOrientation::Vertical;
    let previous_icon = if vertical { "arrow-up" } else { "arrow-left" };
    let next_icon = if vertical { "arrow-down" } else { "arrow-right" };
    let shows_controls = props.shows_controls();
    let shows_indicators = props.shows_indicators() || props.has_variant_indicators();
    let variant = props.variant.as_str();
    let disable_loop = if props.disable_loop { "true" } else { "false" };
    let autoplay_interval = props.autoplay_interval;
    let control_contract = props.control_contract();
    let navigation_size = control_contract.navigation_size;
    let navigation_inset = control_contract.navigation_inset;
    let control_size = control_contract.control_size;
    let control_gap = control_contract.control_gap;
    let carousel_control_gap = control_gap.to_string();
    let indicator_gap = control_contract.indicator_gap;
    let indicator_height = control_contract.indicator_height;
    let indicator_inactive_width = control_contract.indicator_inactive_width;
    let indicator_active_width = control_contract.indicator_active_width;
    let indicator_dot_size = control_contract.indicator_dot_size;
    let indicator_dot_scale =
        f32::from(control_contract.indicator_dot_active_scale_percent) / 100.0;
    let geometry = props.geometry_contract();
    let content_gap = geometry.content_gap;
    let vertical_height = geometry.vertical_viewport_height;
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
            "        {scroll}.addView({track}, new android.widget.HorizontalScrollView.LayoutParams(ViewGroup.LayoutParams.WRAP_CONTENT, ViewGroup.LayoutParams.WRAP_CONTENT));\n        {viewport}.addView({scroll});\n        doweAdd({view}, {viewport}, {content_gap}, false);\n"
        ));
    } else {
        output.push_str(&format!(
            "        ScrollView {scroll} = new ScrollView(this);\n        {scroll}.setFillViewport(false);\n        {scroll}.setVerticalScrollBarEnabled(false);\n        {scroll}.setOverScrollMode(View.OVER_SCROLL_NEVER);\n        {scroll}.setLayoutParams(new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));\n        LinearLayout {track} = doweContainer(false);\n        {scroll}.addView({track}, new ScrollView.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));\n        {viewport}.addView({scroll});\n        doweAdd({view}, {viewport}, {content_gap}, false);\n"
        ));
    }
    if !horizontal {
        output.push_str(&format!("        {scroll}.getLayoutParams().height = doweDp({vertical_height});\n"));
    }
    let slide_width = props
        .slide_width
        .unwrap_or(if props.variant.is_free_scroll() {
            280
        } else {
            320
        });
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
                "        {slide_view}.setLayoutParams(new LinearLayout.LayoutParams({}, doweDp({height})));\n",
                if horizontal { format!("doweDp({slide_width})") } else { "ViewGroup.LayoutParams.MATCH_PARENT".into() },
            ));
        }
        output.push_str(&format!(
            "        doweAdd({track}, {slide_view}, {}, {});\n",
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
    if props.slide_width.is_none() && horizontal {
        let width_rule = if let Some(maximum) = geometry.slide_max_width {
            format!("Math.min(doweDp({maximum}), Math.round(availableWidth * {} / 100f))", geometry.slide_fraction_percent)
        } else {
            format!("Math.max(0, (availableWidth - doweDp({}) * {}) / {})", props.gap, props.slides_per_view.saturating_sub(1), props.slides_per_view.max(1))
        };
        output.push_str(&format!(
            "        {viewport}.addOnLayoutChangeListener((changed, left, top, right, bottom, oldLeft, oldTop, oldRight, oldBottom) -> {{\n            if (right - left == oldRight - oldLeft) return;\n            int availableWidth = Math.max(1, {scroll}.getWidth());\n            int resolvedWidth = {width_rule};\n            for (int index = 0; index < {track}.getChildCount(); index++) {{\n                View slide = {track}.getChildAt(index);\n                android.view.ViewGroup.LayoutParams params = slide.getLayoutParams();\n                params.width = resolvedWidth;\n                slide.setLayoutParams(params);\n            }}\n            {track}.requestLayout();\n        }});\n"
        ));
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
    let carousel_navigation_size = next_dev_view(counter);
    let carousel_navigation_inset = next_dev_view(counter);
    let carousel_control_size = next_dev_view(counter);
    let carousel_indicator_height = next_dev_view(counter);
    let carousel_indicator_inactive_width = next_dev_view(counter);
    let carousel_indicator_active_width = next_dev_view(counter);
    let carousel_indicator_dot_size = next_dev_view(counter);
    let carousel_indicator_dot_scale = next_dev_view(counter);
    let indicators_are_dots =
        props.variant == CarouselVariant::Dots || props.indicator_type.as_str() == "dot";
    let carousel_indicators_are_dots = next_dev_view(counter);
    output.push_str(&format!(
        "        int[] {carousel_index} = new int[] {{0}};\n        String {carousel_variant} = \"{}\";\n        final int {carousel_navigation_size} = doweDp({});\n        final int {carousel_navigation_inset} = doweDp({});\n        final int {carousel_control_size} = doweDp({});\n        final int {carousel_indicator_height} = doweDp({});\n        final int {carousel_indicator_inactive_width} = doweDp({});\n        final int {carousel_indicator_active_width} = doweDp({});\n        final int {carousel_indicator_dot_size} = doweDp({});\n        final float {carousel_indicator_dot_scale} = {}f;\n        final boolean {carousel_indicators_are_dots} = {};\n        ArrayList<View> {carousel_indicators} = new ArrayList<>();\n        TextView[] {carousel_counter} = new TextView[1];\n        View[] {carousel_previous} = new View[1];\n        View[] {carousel_next} = new View[1];\n        View[] {carousel_control_previous} = new View[1];\n        View[] {carousel_control_next} = new View[1];\n",
        escape_java(variant),
        navigation_size,
        navigation_inset,
        control_size,
        indicator_height,
        indicator_inactive_width,
        indicator_active_width,
        indicator_dot_size,
        indicator_dot_scale,
        indicators_are_dots,
    ));
    if props.show_navigation {
        render_dev_android_carousel_arrow(
            previous_icon,
            "Previous slide",
            -1,
            &carousel_index,
            &track,
            &scroll,
            &viewport,
            horizontal,
            vertical,
            disable_loop,
            &carousel_navigation_size,
            &carousel_navigation_inset,
            "0",
            accent_color,
            &control_border_alpha,
            &carousel_previous,
            true,
            counter,
            output,
        );
        render_dev_android_carousel_arrow(
            next_icon,
            "Next slide",
            1,
            &carousel_index,
            &track,
            &scroll,
            &viewport,
            horizontal,
            vertical,
            disable_loop,
            &carousel_navigation_size,
            &carousel_navigation_inset,
            "0",
            accent_color,
            &control_border_alpha,
            &carousel_next,
            true,
            counter,
            output,
        );
    }
    let controls = shows_controls.then(|| next_dev_view(counter));
    if let Some(controls) = controls.as_deref() {
        output.push_str(&format!(
            "        LinearLayout {controls} = doweContainer(true);\n        {controls}.setGravity(Gravity.CENTER);\n"
        ));
        render_dev_android_carousel_arrow(
            previous_icon,
            "Previous slide",
            -1,
            &carousel_index,
            &track,
            &scroll,
            controls,
            horizontal,
            vertical,
            disable_loop,
            &carousel_control_size,
            "0",
            &carousel_control_gap,
            accent_color,
            &control_border_alpha,
            &carousel_control_previous,
            false,
            counter,
            output,
        );
    }
    let indicators = if controls.is_none() && shows_indicators {
        Some(next_dev_view(counter))
    } else {
        None
    };
    let indicator_parent = controls.as_deref().or(indicators.as_deref());
    if let Some(indicator_parent) = indicator_parent {
        if let Some(indicators) = indicators.as_deref() {
            output.push_str(&format!(
                "        LinearLayout {indicators} = doweContainer(true);\n        {indicators}.setGravity(Gravity.CENTER);\n"
            ));
        }
        if shows_indicators {
            for (index, slide) in slides.iter().enumerate() {
                let indicator = next_dev_view(counter);
                if props.variant == CarouselVariant::Thumbnails {
                    output.push_str(&format!(
                        "        Button {indicator} = new Button(this);\n        {indicator}.setText(\"{}\");\n        {indicator}.setTextSize(android.util.TypedValue.COMPLEX_UNIT_DIP, 12f);\n        {indicator}.setAllCaps(false);\n        {indicator}.setPadding(doweDp(4), 0, doweDp(4), 0);\n        {indicator}.setTextColor({accent_color});\n        {indicator}.setMinimumHeight(doweDp(28));\n        {indicator}.setBackgroundColor(Color.TRANSPARENT);\n        {indicator}.setContentDescription(\"{}\");\n        final int {indicator}Index = {index};\n        {indicator}.setOnClickListener(target -> {scroll}.post(() -> {{\n            if ({track}.getChildCount() == 0) return;\n            View slide = {track}.getChildAt({indicator}Index);\n            if ({vertical}) {scroll}.smoothScrollTo(0, slide.getTop()); else {scroll}.smoothScrollTo(slide.getLeft(), 0);\n        }}));\n        {carousel_indicators}.add({indicator});\n        doweAdd({indicator_parent}, {indicator}, {indicator_gap}, true);\n",
                        escape_java(&slide.id),
                        escape_java(&slide.id),
                    ));
                } else {
                    let width = if indicators_are_dots {
                        carousel_indicator_dot_size.clone()
                    } else {
                        carousel_indicator_active_width.clone()
                    };
                    let height = if indicators_are_dots {
                        carousel_indicator_dot_size.clone()
                    } else {
                        carousel_indicator_height.clone()
                    };
                    let scale = if indicators_are_dots {
                        carousel_indicator_dot_scale.clone()
                    } else {
                        "1f".to_string()
                    };
                    output.push_str(&format!(
                        "        View {indicator} = new View(this);\n        {indicator}.setBackground(doweBackground({accent_color}, 999f));\n        {indicator}.setContentDescription(\"Go to slide {}\");\n        {indicator}.setFocusable(true);\n        {indicator}.setClickable(true);\n        {indicator}.setScaleX({});\n        {indicator}.setScaleY({});\n        final int {indicator}Index = {index};\n        LinearLayout.LayoutParams {indicator}Params = new LinearLayout.LayoutParams({}, {});\n        {indicator}.setLayoutParams({indicator}Params);\n        {indicator}.setOnClickListener(target -> {scroll}.post(() -> {{\n            if ({track}.getChildCount() == 0) return;\n            View slide = {track}.getChildAt({indicator}Index);\n            if ({vertical}) {scroll}.smoothScrollTo(0, slide.getTop()); else {scroll}.smoothScrollTo(slide.getLeft(), 0);\n        }}));\n        {carousel_indicators}.add({indicator});\n        doweAdd({indicator_parent}, {indicator}, {indicator_gap}, true);\n",
                        index + 1,
                        scale.clone(),
                        scale,
                        width,
                        height,
                    ));
                }
            }
        }
    }
    if let Some(controls) = controls.as_deref() {
        if props.show_counter {
            let counter_view = next_dev_view(counter);
            output.push_str(&format!(
                "        TextView {counter_view} = doweText(\"1 / {}\", {accent_color}, 13f, 600, 0f, 1.2f, {});\n        {counter_view}.setGravity(Gravity.CENTER);\n        {carousel_counter}[0] = {counter_view};\n        doweAdd({controls}, {counter_view}, {indicator_gap}, true);\n",
                slides.len(),
                dev_font_value(current_font),
            ));
        }
        render_dev_android_carousel_arrow(
            next_icon,
            "Next slide",
            1,
            &carousel_index,
            &track,
            &scroll,
            controls,
            horizontal,
            vertical,
            disable_loop,
            &carousel_control_size,
            "0",
            &carousel_control_gap,
            accent_color,
            &control_border_alpha,
            &carousel_control_next,
            false,
            counter,
            output,
        );
        output.push_str(&format!(
            "        doweAdd({view}, {controls}, {content_gap}, false);\n"
        ));
    } else if props.show_counter {
        let counter_view = next_dev_view(counter);
        output.push_str(&format!(
            "        TextView {counter_view} = doweText(\"1 / {}\", {accent_color}, 13f, 600, 0f, 1.2f, {});\n        {counter_view}.setGravity(Gravity.CENTER);\n        {carousel_counter}[0] = {counter_view};\n        doweAdd({view}, {counter_view}, {content_gap}, false);\n",
            slides.len(),
            dev_font_value(current_font),
        ));
    }
    if let Some(indicators) = indicators.as_deref() {
        output.push_str(&format!(
            "        doweAdd({view}, {indicators}, {content_gap}, false);\n"
        ));
    }
    output.push_str(&format!(
        "        Runnable {carousel_update} = () -> {{\n            int viewportCenter = {vertical} ? {scroll}.getScrollY() + {scroll}.getHeight() / 2 : {scroll}.getScrollX() + {scroll}.getWidth() / 2;\n            int viewportSize = Math.max(1, {vertical} ? {scroll}.getHeight() : {scroll}.getWidth());\n            int active = 0;\n            float activeDistance = Float.MAX_VALUE;\n            for (int index = 0; index < {track}.getChildCount(); index++) {{\n                View slide = {track}.getChildAt(index);\n                float center = {vertical} ? slide.getTop() + slide.getHeight() / 2f : slide.getLeft() + slide.getWidth() / 2f;\n                float phase = Math.max(-1f, Math.min(1f, (center - viewportCenter) / (float) viewportSize));\n                float distance = Math.min(1f, Math.abs(phase));\n                slide.setRotationY(0f);\n                slide.setRotation(0f);\n                slide.setScaleX(1f);\n                slide.setScaleY(1f);\n                slide.setTranslationX(0f);\n                slide.setTranslationY(0f);\n                slide.setAlpha(1f);\n                if (\"coverFlow\".equals({carousel_variant})) {{ slide.setCameraDistance(doweDp(24)); slide.setRotationY(phase * 24f); slide.setScaleX(1f - distance * 0.1f); slide.setScaleY(1f - distance * 0.1f); slide.setAlpha(1f - distance * 0.22f); }}\n                else if (\"stories\".equals({carousel_variant})) {{ slide.setCameraDistance(doweDp(24)); slide.setRotationY(phase * 30f); slide.setScaleX(1f - distance * 0.1f); slide.setScaleY(1f - distance * 0.1f); slide.setAlpha(1f - distance * 0.22f); }}\n                else if (\"smartStack\".equals({carousel_variant})) {{ slide.setRotation(phase * 1.5f); slide.setScaleX(1f - distance * 0.055f); slide.setScaleY(1f - distance * 0.055f); slide.setTranslationY(doweDp(8) * distance); }}\n                else if (\"cardStack\".equals({carousel_variant})) {{ slide.setScaleX(1f - distance * 0.055f); slide.setScaleY(1f - distance * 0.055f); slide.setTranslationY(doweDp(8) * distance); }}\n                else if (\"flipbook\".equals({carousel_variant})) {{ slide.setCameraDistance(doweDp(24)); slide.setRotationY(phase * 52f); slide.setScaleX(1f - distance * 0.1f); slide.setScaleY(1f - distance * 0.1f); slide.setAlpha(1f - distance * 0.22f); }}\n                else if (\"slideshow\".equals({carousel_variant})) {{ if ({vertical}) slide.setTranslationY(doweDp(24) * phase); else slide.setTranslationX(doweDp(24) * phase); slide.setAlpha(1f - distance * 0.12f); }}\n                if (Math.abs(phase) < activeDistance) {{ active = index; activeDistance = Math.abs(phase); }}\n            }}\n            {carousel_index}[0] = active;\n            if ({carousel_previous}[0] != null) {carousel_previous}[0].setEnabled(!{disable_loop} || active > 0);\n            if ({carousel_next}[0] != null) {carousel_next}[0].setEnabled(!{disable_loop} || active < Math.max(0, {track}.getChildCount() - 1));\n            if ({carousel_control_previous}[0] != null) {carousel_control_previous}[0].setEnabled(!{disable_loop} || active > 0);\n            if ({carousel_control_next}[0] != null) {carousel_control_next}[0].setEnabled(!{disable_loop} || active < Math.max(0, {track}.getChildCount() - 1));\n            for (int index = 0; index < {carousel_indicators}.size(); index++) {{ View indicator = {carousel_indicators}.get(index); boolean selected = index == active; if (indicator instanceof Button) {{ ((Button) indicator).setTextColor(selected ? {accent_color} : doweAlpha({accent_color}, 0.45f)); }} else {{ ViewGroup.LayoutParams params = indicator.getLayoutParams(); params.width = {carousel_indicators_are_dots} ? {carousel_indicator_dot_size} : (selected ? {carousel_indicator_active_width} : {carousel_indicator_inactive_width}); params.height = {carousel_indicators_are_dots} ? {carousel_indicator_dot_size} : {carousel_indicator_height}; indicator.setLayoutParams(params); indicator.setBackground(doweBackground(selected ? {accent_color} : doweAlpha({accent_color}, {indicator_inactive_alpha}f), 999f)); float scale = {carousel_indicators_are_dots} && selected ? {carousel_indicator_dot_scale} : 1f; indicator.setScaleX(scale); indicator.setScaleY(scale); }} }}\n            if ({carousel_counter}[0] != null) {carousel_counter}[0].setText(String.valueOf(active + 1) + \" / \" + String.valueOf({track}.getChildCount()));\n        }};\n        {scroll}.setOnScrollChangeListener((target, scrollX, scrollY, oldScrollX, oldScrollY) -> {carousel_update}.run());\n        {viewport}.post({carousel_update});\n",
    ));
    let carousel_interacting = next_dev_view(counter);
    let carousel_autoplay = next_dev_view(counter);
    output.push_str(&format!(
        "        boolean[] {carousel_interacting} = new boolean[] {{false}};\n"
    ));
    if props.variant.uses_snap() {
        output.push_str(&format!(
            "        Runnable {carousel_update}Snap = () -> {{\n            if ({track}.getChildCount() == 0) return;\n            int viewportCenter = {vertical} ? {scroll}.getScrollY() + {scroll}.getHeight() / 2 : {scroll}.getScrollX() + {scroll}.getWidth() / 2;\n            int nearest = 0;\n            int nearestDistance = Integer.MAX_VALUE;\n            for (int index = 0; index < {track}.getChildCount(); index++) {{\n                View slide = {track}.getChildAt(index);\n                int center = {vertical} ? slide.getTop() + slide.getHeight() / 2 : slide.getLeft() + slide.getWidth() / 2;\n                int distance = Math.abs(center - viewportCenter);\n                if (distance < nearestDistance) {{ nearest = index; nearestDistance = distance; }}\n            }}\n            View slide = {track}.getChildAt(nearest);\n            int target = ({vertical} ? slide.getTop() + slide.getHeight() / 2 : slide.getLeft() + slide.getWidth() / 2) - ({vertical} ? {scroll}.getHeight() : {scroll}.getWidth()) / 2;\n            int maximum = Math.max(0, ({vertical} ? {scroll}.getChildAt(0).getHeight() - {scroll}.getHeight() : {scroll}.getChildAt(0).getWidth() - {scroll}.getWidth()));\n            target = Math.max(0, Math.min(target, maximum));\n            if ({vertical}) {scroll}.smoothScrollTo(0, target); else {scroll}.smoothScrollTo(target, 0);\n        }};\n        {scroll}.setOnTouchListener((target, event) -> {{\n            int action = event.getActionMasked();\n            if (action == android.view.MotionEvent.ACTION_DOWN) {carousel_interacting}[0] = true;\n            if (action == android.view.MotionEvent.ACTION_UP || action == android.view.MotionEvent.ACTION_CANCEL) {{\n                {carousel_interacting}[0] = false;\n                {scroll}.removeCallbacks({carousel_update}Snap);\n                {scroll}.postDelayed({carousel_update}Snap, 120);\n            }}\n            return false;\n        }});\n"
        ));
        output.push_str(&format!(
            "        {scroll}.setOnScrollChangeListener((target, scrollX, scrollY, oldScrollX, oldScrollY) -> {{ {carousel_update}.run(); {scroll}.removeCallbacks({carousel_update}Snap); {scroll}.postDelayed({carousel_update}Snap, 120); }});\n"
        ));
    } else {
        output.push_str(&format!(
            "        {scroll}.setOnTouchListener((target, event) -> {{ int action = event.getActionMasked(); if (action == android.view.MotionEvent.ACTION_DOWN) {carousel_interacting}[0] = true; if (action == android.view.MotionEvent.ACTION_UP || action == android.view.MotionEvent.ACTION_CANCEL) {carousel_interacting}[0] = false; return false; }});\n"
        ));
    }
    if props.autoplay {
        output.push_str(&format!(
            "        Runnable[] {carousel_autoplay} = new Runnable[1];\n        {carousel_autoplay}[0] = () -> {{\n            if (!{view}.isShown() || {track}.getChildCount() == 0) return;\n            int last = Math.max(0, {track}.getChildCount() - 1);\n            if ({disable_loop} && {carousel_index}[0] >= last) return;\n            if ({carousel_interacting}[0]) {{ {scroll}.postDelayed({carousel_autoplay}[0], Math.max(500, {autoplay_interval})); return; }}\n            int targetIndex = {carousel_index}[0] + 1;\n            if (targetIndex > last) targetIndex = {disable_loop} ? last : 0;\n            View slide = {track}.getChildAt(targetIndex);\n            int target = ({vertical} ? slide.getTop() + slide.getHeight() / 2 : slide.getLeft() + slide.getWidth() / 2) - ({vertical} ? {scroll}.getHeight() : {scroll}.getWidth()) / 2;\n            int maximum = Math.max(0, ({vertical} ? {scroll}.getChildAt(0).getHeight() - {scroll}.getHeight() : {scroll}.getChildAt(0).getWidth() - {scroll}.getWidth()));\n            target = Math.max(0, Math.min(target, maximum));\n            if ({vertical}) {scroll}.smoothScrollTo(0, target); else {scroll}.smoothScrollTo(target, 0);\n            {scroll}.postDelayed({carousel_autoplay}[0], Math.max(500, {autoplay_interval}));\n        }};\n        {scroll}.postDelayed({carousel_autoplay}[0], Math.max(500, {autoplay_interval}));\n",
        ));
    }
}
