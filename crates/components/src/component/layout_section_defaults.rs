#[test]
fn derives_section_axis_padding_defaults_and_preserves_overrides() {
    let default_spacing = section_content_spacing(&SpacingProps::default());
    let horizontal = default_spacing.px.expect("default horizontal padding");
    let vertical = default_spacing.py.expect("default vertical padding");
    assert_eq!(horizontal.entries[0].value, ScaleValue::from_half_steps(8));
    assert_eq!(horizontal.entries[1].value, ScaleValue::from_half_steps(12));
    assert_eq!(vertical.entries[0].value, ScaleValue::from_half_steps(20));
    assert_eq!(vertical.entries[1].value, ScaleValue::from_half_steps(32));

    let authored = SpacingProps {
        py: Some(super::ResponsiveValue::scalar(ScaleValue::from_half_steps(
            12,
        ))),
        ..Default::default()
    };
    let effective = section_content_spacing(&authored);
    assert_eq!(
        effective.py.expect("authored vertical padding").entries[0].value,
        ScaleValue::from_half_steps(12)
    );
    assert_eq!(
        effective.px.expect("default horizontal padding").entries[1].value,
        ScaleValue::from_half_steps(12)
    );
}
