#[test]
fn swiftui_block_bars_accept_parent_width_without_changing_explicit_sizing() {
    let mut props = BarProps::default();
    let block = super::swift_modifiers_for_bar(&props, super::NativeFlow::Block);
    assert_eq!(
        block[0],
        ".frame(minWidth: CGFloat(0), maxWidth: .infinity, minHeight: CGFloat(48), alignment: .center)"
    );
    let inline = super::swift_modifiers_for_bar(&props, super::NativeFlow::Inline);
    assert_eq!(
        inline[0],
        ".frame(minHeight: CGFloat(48), alignment: .center)"
    );
    props.style.style.sizing.w = Some(ResponsiveValue::scalar(SizeValue::Scale(
        ScaleValue::from_half_steps(120),
    )));
    let explicit = super::swift_modifiers_for_bar(&props, super::NativeFlow::Block);
    assert_eq!(explicit[0], inline[0]);
    assert!(
        explicit
            .iter()
            .any(|value| value.contains("DoweSize.fixed(CGFloat(240))"))
    );
}

#[test]
#[cfg(target_os = "macos")]
fn swiftui_bar_overflow_does_not_expand_sibling_geometry() {
    let modifiers = super::swift_modifiers_for_bar(&BarProps::default(), super::NativeFlow::Block);
    let source = include_str!("bar_width_harness.swift").replace("__BAR_FRAME__", &modifiers[0]);
    let directory = std::env::temp_dir().join(format!("dowe-bar-width-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let file = directory.join("BarWidth.swift");
    let executable = directory.join("bar-width");
    std::fs::write(&file, source).unwrap();
    let compiled = std::process::Command::new("xcrun")
        .arg("swiftc")
        .arg(&file)
        .arg("-o")
        .arg(&executable)
        .output()
        .expect("Swift compiler");
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let executed = std::process::Command::new(&executable).output().unwrap();
    assert!(
        executed.status.success(),
        "{}",
        String::from_utf8_lossy(&executed.stderr)
    );
    std::fs::remove_dir_all(directory).unwrap();
}
