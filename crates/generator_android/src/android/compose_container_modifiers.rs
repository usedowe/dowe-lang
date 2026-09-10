fn compose_position_modifier(props: &PositionProps) -> String {
    let vertical = if props.bottom.is_some() {
        "Bottom"
    } else {
        "Top"
    };
    let horizontal = if props.right.is_some() {
        "End"
    } else {
        "Start"
    };
    let mut modifier = format!(".align(Alignment.{vertical}{horizontal})");
    let mut values = Vec::new();
    if let Some(value) = props.top.as_ref() {
        values.push(format!("top = {} ?: 0.dp", compose_scale_value(value)));
    }
    if let Some(value) = props.right.as_ref() {
        values.push(format!("end = {} ?: 0.dp", compose_scale_value(value)));
    }
    if let Some(value) = props.bottom.as_ref() {
        values.push(format!("bottom = {} ?: 0.dp", compose_scale_value(value)));
    }
    if let Some(value) = props.left.as_ref() {
        values.push(format!("start = {} ?: 0.dp", compose_scale_value(value)));
    }
    if !values.is_empty() {
        modifier.push_str(&format!(".padding({})", values.join(", ")));
    }
    modifier
}

fn modifier_for_section_container(props: &StyleProps, flow: ComposeFlow) -> String {
    let mut outer = props.clone();
    outer.spacing = Default::default();
    modifier_for_container_style(&outer, flow)
}

fn modifier_for_section_content(props: &StyleProps) -> String {
    let mut content = StyleProps::default();
    content.spacing = dowe_components::section_content_spacing(&props.spacing);
    content.sizing = props.sizing.clone();
    content.sizing.h = content
        .sizing
        .h
        .as_ref()
        .map(|value| android_section_bounded_size(value, &props.spacing));
    content.sizing.min_h = content
        .sizing
        .min_h
        .as_ref()
        .map(|value| android_section_bounded_size(value, &props.spacing));
    let modifier = if props.boxed {
        "Modifier.widthIn(max = 1536.dp).fillMaxWidth()".to_string()
    } else if props.center_x.is_some() {
        "Modifier.fillMaxWidth()".to_string()
    } else {
        "Modifier".to_string()
    };
    let modifier = if props.sizing.h.is_some() || props.sizing.min_h.is_some() {
        format!("{modifier}.fillMaxHeight()")
    } else {
        modifier
    };
    modifier_for_style_with_base(&content, modifier)
}

fn compose_section_vertical_arrangement(value: Option<&ResponsiveValue<GapValue>>) -> String {
    value
        .map(|value| {
            format!(
                ", verticalArrangement = Arrangement.spacedBy({} ?: 0.dp)",
                compose_gap_value(value)
            )
        })
        .unwrap_or_default()
}

fn compose_section_horizontal_alignment(value: &ResponsiveValue<bool>) -> String {
    format!(
        "({} ?: false) ? Alignment.CenterHorizontally : Alignment.Start",
        compose_bool_value(value)
    )
}

fn modifier_for_bar(props: &BarProps, flow: ComposeFlow) -> String {
    let mut modifier = String::from("Modifier");
    if flow.is_block() && props.style.style.sizing.w.is_none() {
        modifier.push_str(".fillMaxWidth()");
    }
    modifier.push_str(".heightIn(min = 48.dp)");
    if props.position != BarPosition::Static {
        modifier.push_str(".zIndex(1f)");
    }
    if props.floating && !props.dock_on_scroll {
        modifier.push_str(".padding(horizontal = 16.dp, vertical = 8.dp)");
    }
    modifier = modifier_for_style_with_base(&props.style.style, modifier);
    if props.dock_on_scroll {
        return format!(
            "doweDockingAppBarModifier({modifier}, scrollState, {})",
            variant_container(&props.style)
        );
    }
    if props.floating {
        modifier.push_str(".clip(RoundedCornerShape(DoweDesign.radius))");
    }
    modifier.push_str(&format!(".background({})", variant_container(&props.style)));
    if props.bordered || props.floating {
        let radius = if props.floating {
            "DoweDesign.radius"
        } else {
            "0.dp"
        };
        modifier.push_str(&format!(
            ".border(1.dp, DoweDesign.muted, RoundedCornerShape({radius}))"
        ));
    }
    modifier
}

fn modifier_for_divider(props: &DividerProps, flow: ComposeFlow) -> String {
    let mut modifier = String::from("Modifier");
    match props.orientation {
        DividerOrientation::Horizontal => {
            if flow.is_block() && props.style.sizing.w.is_none() {
                modifier.push_str(".fillMaxWidth()");
            }
            if props.style.sizing.h.is_none() {
                modifier.push_str(".height(1.dp)");
            }
        }
        DividerOrientation::Vertical => {
            if props.style.sizing.w.is_none() {
                modifier.push_str(".width(1.dp)");
            }
            if props.style.sizing.h.is_none() {
                modifier.push_str(".fillMaxHeight()");
            }
        }
    }
    modifier_for_style_with_base(&props.style, modifier)
}

