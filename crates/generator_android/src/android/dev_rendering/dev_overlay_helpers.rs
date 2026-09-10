fn dev_android_modal_close(
    path: &str,
    action: Option<&str>,
    context: &ComposeReactiveContext,
    popup: &str,
) -> String {
    let action = action
        .and_then(|name| context.action_id(name))
        .map(|id| format!("doweRunAction(\"{}\", null); ", escape_java(id)))
        .unwrap_or_default();
    format!(
        "if ({popup} != null) {{ {popup}.dismiss(); }} doweWrite(\"{path}\", false); {action}renderCurrentRoute(false);"
    )
}

fn render_dev_android_overlay_close(
    panel: &str,
    label: &str,
    action: &str,
    gravity: &str,
    margins: &str,
    counter: &mut usize,
    output: &mut String,
) -> String {
    let close = next_dev_view(counter);
    let close_icon = next_dev_view(counter);
    let close_paths = format!("{close}Paths");
    output.push_str(&format!(
        "        FrameLayout {close} = new FrameLayout(this);\n        {close}.setBackground(doweBackground(DOWE_MUTED, 999f));\n        {close}.setContentDescription(\"{}\");\n        {close}.setFocusable(true);\n        {close}.setOnClickListener(v -> {{ {action} }});\n        ArrayList<DoweSvgPathEntry> {close_paths} = new ArrayList<>();\n        {close_paths}.add(new DoweSvgPathEntry(\"M0 0h24v24H0z\", false, null));\n        {close_paths}.add(new DoweSvgPathEntry(\"m4.397 4.554l.073-.084a.75.75 0 0 1 .976-.073l.084.073L12 10.939l6.47-6.47a.75.75 0 1 1 1.06 1.061L13.061 12l6.47 6.47a.75.75 0 0 1 .072.976l-.073.084a.75.75 0 0 1-.976.073l-.084-.073L12 13.061l-6.47 6.47a.75.75 0 0 1-1.06-1.061L10.939 12l-6.47-6.47a.75.75 0 0 1-.072-.976l.073-.084z\", true, null));\n        DoweSvgView {close_icon} = new DoweSvgView(this, 0f, 0f, 24f, 24f, DOWE_MUTED_TEXT, {close_paths});\n        {close_icon}.setImportantForAccessibility(View.IMPORTANT_FOR_ACCESSIBILITY_NO);\n        {close}.addView({close_icon}, new FrameLayout.LayoutParams(doweDp(18), doweDp(18), Gravity.CENTER));\n        FrameLayout.LayoutParams {close}Params = new FrameLayout.LayoutParams(doweDp(28), doweDp(28), {gravity});\n        {close}Params.setMargins({margins});\n        {panel}.addView({close}, {close}Params);\n",
        escape_java(label)
    ));
    close
}

fn dev_drawer_close_gravity(position: &DrawerPosition) -> (&'static str, &'static str) {
    match position {
        DrawerPosition::End => ("Gravity.TOP | Gravity.START", "doweDp(8), doweDp(8), 0, 0"),
        DrawerPosition::Top => ("Gravity.BOTTOM | Gravity.END", "0, 0, doweDp(8), doweDp(8)"),
        _ => ("Gravity.TOP | Gravity.END", "0, doweDp(8), doweDp(8), 0"),
    }
}
