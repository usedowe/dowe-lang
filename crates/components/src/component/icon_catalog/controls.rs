pub fn solar_control_icon(name: &str) -> ComponentResult<SideNavIcon> {
    match icon_component_node(vec![ComponentProp {
        name: "name".to_string(),
        value: PropValue::String(name.to_string()),
    }])? {
        ViewNode::Svg { props, paths } => Ok(SideNavIcon { props, paths }),
        _ => unreachable!(),
    }
}

pub fn empty_icon(kind: EmptyKind) -> ComponentResult<SideNavIcon> {
    let name = match kind {
        EmptyKind::Playlist => "playlist-bold-duotone",
        EmptyKind::Result => "magnifier-bold-duotone",
        EmptyKind::Data => "database-bold-duotone",
        EmptyKind::Template => "widget-add-bold-duotone",
    };
    solar_control_icon(name)
}

pub const SIDE_NAV_SUBMENU_ARROW_PATH: &str = "m19.704 12l-8.491-8.727a.75.75 0 1 1 1.075-1.046l9 9.25a.75.75 0 0 1 0 1.046l-9 9.25a.75.75 0 1 1-1.075-1.046z";

pub fn side_nav_submenu_arrow_icon() -> SideNavIcon {
    let mut style = StyleProps::default();
    let size = ResponsiveValue::scalar(SizeValue::Scale(ScaleValue::from_half_steps(8)));
    style.sizing.w = Some(size.clone());
    style.sizing.h = Some(size);
    SideNavIcon {
        props: SvgProps {
            style,
            view_box: SvgViewBox {
                min_x: "0".to_string(),
                min_y: "0".to_string(),
                width: "24".to_string(),
                height: "24".to_string(),
            },
            data: None,
            icon_name: None,
            icon_fallback: None,
            icon_fill: None,
            icon_fill_binding: None,
            icon_stroke: None,
            icon_stroke_binding: None,
            motion: None,
        },
        paths: vec![
            SvgPath {
                data: "M0 0h24v24H0z".to_string(),
                fill: SvgPathFill::None,
                transform: None,
            },
            SvgPath {
                data: SIDE_NAV_SUBMENU_ARROW_PATH.to_string(),
                fill: SvgPathFill::CurrentColor,
                transform: None,
            },
        ],
    }
}

pub fn svg_spinner_control_icon(name: &str) -> ComponentResult<SideNavIcon> {
    match icon_component_node(vec![ComponentProp {
        name: "name".to_string(),
        value: PropValue::String(format!("svg-spinners:{name}")),
    }])? {
        ViewNode::Svg { props, paths } => Ok(SideNavIcon { props, paths }),
        _ => unreachable!(),
    }
}

pub fn view_icon(icon: ViewIcon) -> SideNavIcon {
    let svg = match icon {
        ViewIcon::Plus => {
            r#"<svg viewBox="0 0 24 24"><path d="M12 5v14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"/><path d="M5 12h14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"/></svg>"#
        }
        ViewIcon::Link => {
            r#"<svg viewBox="0 0 24 24"><path d="M10 13a5 5 0 0 0 7.07 0l2.12-2.12a5 5 0 0 0-7.07-7.07L10.9 5.03" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"/><path d="M14 11a5 5 0 0 0-7.07 0L4.81 13.12a5 5 0 0 0 7.07 7.07l1.22-1.22" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"/></svg>"#
        }
        ViewIcon::Edit => {
            r#"<svg viewBox="0 0 24 24"><path d="M4 20h4l10.5-10.5a2.12 2.12 0 0 0-3-3L5 17v3Z" fill="none" stroke="currentColor" stroke-width="2" stroke-linejoin="round"/><path d="m13.5 7.5 3 3" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"/></svg>"#
        }
        ViewIcon::Trash => {
            r#"<svg viewBox="0 0 24 24"><path d="M5 7h14M10 11v6M14 11v6M8 7l1-3h6l1 3M7 7l1 13h8l1-13" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg>"#
        }
        ViewIcon::Search => {
            r#"<svg viewBox="0 0 24 24"><circle cx="11" cy="11" r="6" fill="none" stroke="currentColor" stroke-width="2"/><path d="m16 16 4 4" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"/></svg>"#
        }
        ViewIcon::Settings => {
            r#"<svg viewBox="0 0 24 24"><path d="M12 8a4 4 0 1 0 0 8 4 4 0 0 0 0-8Z" fill="none" stroke="currentColor" stroke-width="2"/><path d="M4 12h2m12 0h2M12 4v2m0 12v2M6.3 6.3l1.4 1.4m8.6 8.6 1.4 1.4m0-11.4-1.4 1.4m-8.6 8.6-1.4 1.4" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"/></svg>"#
        }
        ViewIcon::Upload => {
            r#"<svg viewBox="0 0 24 24"><path d="M12 16V4m0 0 5 5m-5-5-5 5M4 16v3a1 1 0 0 0 1 1h14a1 1 0 0 0 1-1v-3" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg>"#
        }
        ViewIcon::File => {
            r#"<svg viewBox="0 0 24 24"><path d="M6 3h8l4 4v14H6V3Z" fill="none" stroke="currentColor" stroke-width="2" stroke-linejoin="round"/><path d="M14 3v5h5" fill="none" stroke="currentColor" stroke-width="2" stroke-linejoin="round"/></svg>"#
        }
        ViewIcon::Dismiss => {
            r#"<svg viewBox="0 0 24 24"><path d="m6 6 12 12M18 6 6 18" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"/></svg>"#
        }
        ViewIcon::Moon => {
            r#"<svg viewBox="0 0 24 24"><path d="M20 15.3A8 8 0 0 1 8.7 4 8.5 8.5 0 1 0 20 15.3Z" fill="none" stroke="currentColor" stroke-width="2" stroke-linejoin="round"/></svg>"#
        }
        ViewIcon::Sun => {
            r#"<svg viewBox="0 0 24 24"><circle cx="12" cy="12" r="4" fill="none" stroke="currentColor" stroke-width="2"/><path d="M12 2v2m0 16v2M4.93 4.93l1.42 1.42m11.3 11.3 1.42 1.42M2 12h2m16 0h2M4.93 19.07l1.42-1.42m11.3-11.3 1.42-1.42" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"/></svg>"#
        }
    };
    let (view_box, paths) = parse_solar_svg(svg, None, None).expect("bundled ViewIcon geometry");
    let mut props = parse_svg_props(
        BuiltinComponent::Icon,
        &[ComponentProp {
            name: "viewBox".to_string(),
            value: PropValue::String(view_box.as_str()),
        }],
    )
    .expect("bundled ViewIcon props");
    let size = ResponsiveValue::scalar(SizeValue::Scale(ScaleValue::from_half_steps(10)));
    props.style.sizing.w = Some(size.clone());
    props.style.sizing.h = Some(size);
    SideNavIcon { props, paths }
}
