#[test]
fn derives_section_axis_padding_defaults_and_preserves_overrides() {
    let default_spacing = section_content_spacing(&SpacingProps::default());
    let padding = default_spacing.p.expect("default padding");
    assert_eq!(padding.entries[0].breakpoint, Breakpoint::Xs);
    assert_eq!(padding.entries[0].value, ScaleValue::from_half_steps(8));
    assert_eq!(padding.entries[1].breakpoint, Breakpoint::Lg);
    assert_eq!(padding.entries[1].value, ScaleValue::from_half_steps(10));

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
        ScaleValue::from_half_steps(10)
    );
}
