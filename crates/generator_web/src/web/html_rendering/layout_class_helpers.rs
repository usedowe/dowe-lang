fn box_classes(props: &StyleProps) -> Vec<String> {
    let mut classes = vec!["box".to_string()];
    append_style_classes(&mut classes, props);
    append_responsive_classes(&mut classes, "box-center-x", props.center_x.as_ref(), |value| value.to_string());
    append_responsive_classes(&mut classes, "box-center-y", props.center_y.as_ref(), |value| value.to_string());
    append_container_visual_classes(&mut classes, props);
    classes
}

fn brand_classes(props: &StyleProps) -> Vec<String> {
    let mut classes = vec!["brand".to_string()];
    append_style_classes(&mut classes, props);
    classes
}

fn banner_classes(props: &StyleProps) -> Vec<String> {
    let mut classes = vec!["banner".to_string()];
    append_style_classes(&mut classes, props);
    append_container_visual_classes(&mut classes, props);
    classes
}

fn section_classes(props: &StyleProps) -> Vec<String> {
    let mut classes = vec!["section".to_string()];
    append_style_classes(&mut classes, props);
    classes.retain(|class_name| !section_spacing_class(class_name));
    append_container_visual_classes(&mut classes, props);
    classes
}

fn section_body_classes(props: &StyleProps) -> Vec<String> {
    let mut classes = vec!["section-body".to_string()];
    if props.sizing.h.is_some() || props.sizing.min_h.is_some() {
        classes.push("section-body-has-height".to_string());
    }
    if props.boxed {
        classes.push("is-boxed".to_string());
    }
    append_responsive_classes(&mut classes, "section-center-x", props.center_x.as_ref(), |value| {
        value.to_string()
    });
    append_responsive_classes(&mut classes, "section-center-y", props.center_y.as_ref(), |value| {
        value.to_string()
    });
    append_responsive_classes(&mut classes, "gap", props.gap.as_ref(), |value| {
        value.class_suffix()
    });
    let mut content = props.clone();
    content.spacing = dowe_components::section_content_spacing(&props.spacing);
    let mut style_classes = Vec::new();
    append_style_classes(&mut style_classes, &content);
    classes.extend(
        style_classes
            .into_iter()
            .filter(|class_name| section_spacing_class(class_name)),
    );
    classes
}

fn section_spacing_class(class_name: &str) -> bool {
    let class_name = class_name.rsplit(':').next().unwrap_or(class_name);
    ["p-", "px-", "py-", "pl-", "pr-", "pt-", "pb-"]
        .iter()
        .any(|prefix| class_name.starts_with(prefix))
}

fn layout_classes(base: &str, props: &LayoutProps) -> Vec<String> {
    let mut classes = vec![base.to_string()];
    append_style_classes(&mut classes, &props.style);
    append_responsive_classes(
        &mut classes,
        "direction",
        Some(&props.direction),
        |value| value.as_str().to_string(),
    );
    if props.wrap {
        classes.push("flex-wrap".to_string());
    }
    append_responsive_classes(&mut classes, "justify", props.justify.as_ref(), |value| {
        value.as_str().to_string()
    });
    append_responsive_classes(&mut classes, "align", props.align.as_ref(), |value| {
        value.as_str().to_string()
    });
    append_responsive_classes(&mut classes, "gap", props.gap.as_ref(), |value| {
        value.class_suffix()
    });
    classes
}
