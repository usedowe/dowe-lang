fn append_dev_activity_runtime(output: &mut String, has_dynamic_icons: bool, has_phones: bool) {
    output.push_str(dev_activity_layout_widgets());
    output.push_str(dev_activity_flex_layout());
    output.push_str(dev_activity_grid_layout());
    if has_dynamic_icons {
        output.push_str(dev_activity_dynamic_icon_runtime());
    }
    output.push_str(dev_activity_svg_parser());
    output.push_str(dev_activity_svg_view());
    output.push_str(dev_activity_drawables_media());
    output.push_str(dev_activity_image_cropper());
    output.push_str(dev_activity_candlestick_runtime());
    output.push_str(dev_activity_chart_runtime());
    output.push_str(dev_activity_canvas_runtime());
    output.push_str(dev_activity_diagram_runtime());
    output.push_str(dev_activity_diagram_view());
    output.push_str(&dev_activity_code_and_forms());
    if has_phones {
        output.push_str(dev_activity_phone_flag_runtime());
    } else {
        output.push_str(dev_activity_empty_phone_flag_runtime());
    }
    output.push_str(dev_activity_responsive_helpers());
}
