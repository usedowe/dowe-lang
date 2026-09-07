#[test]
fn generates_intrinsic_android_icon_button_labels_with_explicit_width_overrides() {
    for (start, end) in [(false, false), (true, false), (false, true), (true, true)] {
        for width in [
            None,
            Some(SizeValue::Full),
            Some(SizeValue::Scale(ScaleValue::from_half_steps(64))),
        ] {
            let mut route = route();
            route.layout_tree = ViewNode::Children;
            route.page_tree = ViewNode::Grid {
                props: Default::default(),
                children: vec![ViewNode::Button {
                    props: VariantProps {
                        style: StyleProps {
                            sizing: SizingProps {
                                w: width.clone().map(ResponsiveValue::scalar),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        icon_start: start.then(|| solar_control_icon("settings").unwrap()),
                        icon_end: end.then(|| solar_control_icon("settings").unwrap()),
                        ..Default::default()
                    },
                    children: vec![text("Read native build docs")],
                }],
            };
            let output = generate_android(
                &[route],
                &FontConfig::default(),
                &DesignConfig::default(),
                &[],
            );
            let compose = output
                .files
                .iter()
                .find(|file| file.relative_path.ends_with("DowePages.kt"))
                .unwrap();
            assert!(
                compose
                    .content
                    .lines()
                    .any(|line| line.contains("Button(modifier =")
                        && line.contains(".doweGridCompactWidth()"))
            );
            if width == Some(SizeValue::Full) {
                assert!(compose.content.contains(".fillMaxWidth()"));
            }
            let dev = output
                .files
                .iter()
                .filter(|file| {
                    file.relative_path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .is_some_and(|name| {
                            name.starts_with("DoweDevRoute") && name.ends_with(".java")
                        })
                })
                .map(|file| {
                    file.content
                        .replace("runtime.", "")
                        .replace("new Button(runtime)", "new Button(this)")
                        .replace("DoweDevActivity.", "")
                })
                .collect::<String>();
            let declaration = dev
                .lines()
                .find(|line| {
                    (line.contains("LinearLayout view") && line.contains(" = doweContainer(true);"))
                        || (line.contains("Button view") && line.contains(" = new Button(this);"))
                })
                .expect("icon button row");
            let view = declaration.split_whitespace().nth(1).unwrap();
            let intrinsic = format!(
                "{view}.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.WRAP_CONTENT, ViewGroup.LayoutParams.WRAP_CONTENT));"
            );
            let intrinsic_offset = dev
                .find(&intrinsic)
                .unwrap_or_else(|| panic!("missing {intrinsic}; declaration: {declaration}"));
            let compact = format!(
                "{view}.setTag(DOWE_COMPACT_WIDTH_TAG, {view}.getLayoutParams().width == ViewGroup.LayoutParams.WRAP_CONTENT);"
            );
            let compact_offset = dev
                .find(&compact)
                .expect("Grid and vertical Flex must preserve intrinsic button width");
            if width.is_some() {
                let override_offset = dev
                    .find(&format!(
                        "{view}SizeParams.width = doweDimension({view}Width)"
                    ))
                    .expect("explicit width override");
                assert!(intrinsic_offset < override_offset);
                assert!(override_offset < compact_offset);
            } else {
                assert!(!dev.contains(&format!("Integer {view}Width =")));
            }
        }
    }
}
