fn render_dev_android_carousel_arrow(
    icon_name: &str,
    label: &str,
    step: i32,
    carousel_index: &str,
    track: &str,
    scroll: &str,
    parent: &str,
    _horizontal: bool,
    vertical: bool,
    disable_loop: &str,
    size: &str,
    inset: &str,
    gap: &str,
    content_color: &str,
    border_alpha: &str,
    slot: &str,
    navigation: bool,
    counter: &mut usize,
    output: &mut String,
) {
    let icon_size = dowe_components::IconButtonGeometryContract::for_size(if navigation {
        ButtonSize::Md
    } else {
        ButtonSize::Sm
    })
    .icon_size;
    let button = next_dev_view(counter);
    let icon = solar_control_icon(icon_name).expect("bundled Carousel control icon");
    let icon_view = render_dev_android_icon_view(&icon, counter, output, Some(content_color));
    let enabled = if step < 0 {
        format!("!{disable_loop} || {carousel_index}[0] > 0")
    } else {
        format!("!{disable_loop} || {carousel_index}[0] < Math.max(0, {track}.getChildCount() - 1)")
    };
    output.push_str(&format!(
        "        FrameLayout {button} = doweIconButton({icon_view}, \"{label}\", {size}, doweDp({icon_size}), DOWE_SURFACE, {content_color}, doweAlpha({content_color}, {border_alpha}f));\n        {button}.setEnabled({enabled});\n        {button}.setOnClickListener(target -> {{\n            int targetIndex = {carousel_index}[0] + ({step});\n            int last = Math.max(0, {track}.getChildCount() - 1);\n            if (targetIndex < 0) targetIndex = {disable_loop} ? 0 : last;\n            if (targetIndex > last) targetIndex = {disable_loop} ? last : 0;\n            final int selectedIndex = targetIndex;\n            {scroll}.post(() -> {{\n                if ({track}.getChildCount() == 0) return;\n                View slide = {track}.getChildAt(selectedIndex);\n                if ({vertical}) {scroll}.smoothScrollTo(0, slide.getTop()); else {scroll}.smoothScrollTo(slide.getLeft(), 0);\n            }});\n        }});\n        {slot}[0] = {button};\n",
    ));
    if navigation {
        let gravity = if step < 0 {
            "Gravity.START | Gravity.CENTER_VERTICAL"
        } else {
            "Gravity.END | Gravity.CENTER_VERTICAL"
        };
        let margin = if step < 0 {
            format!("{button}Params.leftMargin = {inset};")
        } else {
            format!("{button}Params.rightMargin = {inset};")
        };
        output.push_str(&format!(
            "        FrameLayout.LayoutParams {button}Params = new FrameLayout.LayoutParams({size}, {size}, {gravity});\n        {margin}\n        {parent}.addView({button}, {button}Params);\n"
        ));
        if vertical {
            let vertical_gravity = if step < 0 {
                "Gravity.TOP | Gravity.CENTER_HORIZONTAL"
            } else {
                "Gravity.BOTTOM | Gravity.CENTER_HORIZONTAL"
            };
            let vertical_margin = if step < 0 {
                format!("{button}Params.topMargin = {inset};")
            } else {
                format!("{button}Params.bottomMargin = {inset};")
            };
            output.push_str(&format!(
                "        {button}Params.gravity = {vertical_gravity};\n        {button}Params.leftMargin = 0;\n        {button}Params.rightMargin = 0;\n        {vertical_margin}\n        {button}.setLayoutParams({button}Params);\n"
            ));
        }
    } else {
        output.push_str(&format!(
            "        {button}.setLayoutParams(new LinearLayout.LayoutParams({size}, {size}));\n        doweAdd({parent}, {button}, {gap}, true);\n"
        ));
    }
}
