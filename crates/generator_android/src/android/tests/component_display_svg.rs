#[test]
fn generates_android_solar_icon_paints() {
    let stroke = SvgPathFill::Stroke {
        color: Some(ColorToken::Accent),
        opacity: 128,
        width: 150,
        line_cap: SvgLineCap::Round,
        line_join: SvgLineJoin::Round,
    };
    assert!(compose_svg_fill(stroke).contains("DoweSvgFill.Stroke(DoweDesign.accent"));
    assert!(dev_svg_path_details(stroke).contains("true, 128, 1.5f"));
    assert!(android_runtime_data_code_svg().contains("drawscope.Stroke"));
    assert!(dev_activity_svg_view().contains("Paint.Style.STROKE"));
}

#[test]
fn generates_android_svg_logo_literal_paints() {
    let fill = SvgPathFill::LiteralFill {
        red: 36,
        green: 41,
        blue: 47,
        opacity: 255,
        even_odd: false,
    };
    assert!(compose_svg_fill(fill).contains("Color(0xFF24292F)"));
    assert!(dev_svg_path_color(fill).contains("Color.rgb(36, 41, 47)"));
}
