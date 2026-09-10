fn validate_node_data_and_media(
    path: &Path,
    node: &ViewNode,
    signals: &HashMap<String, ViewSignalValue>,
    writable_signals: &HashSet<String>,
    actions: &HashSet<String>,
    locals: &HashMap<String, Option<ViewSignalValue>>,
) -> Option<DoweResult<()>> {
    match node {
        ViewNode::Candlestick { props } => Some((|| -> DoweResult<()> {
            validate_candlestick_data(path, signals, &props.data)?;
            Ok(())
        })()),
        ViewNode::Canvas { props } => Some((|| -> DoweResult<()> {
            validate_canvas_scene(path, signals, &props.scene)?;
            validate_optional_action(path, actions, props.on_pointer.as_deref())?;
            validate_optional_action(path, actions, props.on_key.as_deref())?;
            validate_optional_action(path, actions, props.on_motion.as_deref())?;
            Ok(())
        })()),
        ViewNode::Editor { props } => Some((|| -> DoweResult<()> {
            validate_optional_action(path, actions, props.on_save.as_deref())?;
            Ok(())
        })()),
        ViewNode::Camera { props } => Some((|| -> DoweResult<()> {
            validate_optional_action(path, actions, props.on_start.as_deref())?;
            validate_optional_action(path, actions, props.on_capture.as_deref())?;
            validate_optional_action(path, actions, props.on_error.as_deref())?;
            Ok(())
        })()),
        ViewNode::Microphone { props } => Some((|| -> DoweResult<()> {
            validate_optional_action(path, actions, props.on_start.as_deref())?;
            validate_optional_action(path, actions, props.on_stop.as_deref())?;
            validate_optional_action(path, actions, props.on_error.as_deref())?;
            Ok(())
        })()),
        ViewNode::Avatar { props, .. } => Some((|| -> DoweResult<()> {
            if let Some(binding) = props.name_binding.as_ref() {
                validate_typed_path(
                    path,
                    signals,
                    locals,
                    &binding.path,
                    "name",
                    ViewPathExpectation::String,
                )?;
            }
            if let Some(binding) = props.alt_binding.as_ref() {
                validate_typed_path(
                    path,
                    signals,
                    locals,
                    &binding.path,
                    "alt",
                    ViewPathExpectation::String,
                )?;
            }
            if let Some(binding) = props.size_binding.as_ref() {
                validate_typed_path(
                    path,
                    signals,
                    locals,
                    &binding.path,
                    "size",
                    ViewPathExpectation::String,
                )?;
            }
            Ok(())
        })()),
        ViewNode::Image { props } => Some((|| -> DoweResult<()> {
            if let Some(src) = props.reactive_src.as_deref() {
                validate_typed_path(
                    path,
                    signals,
                    locals,
                    src,
                    "src",
                    ViewPathExpectation::String,
                )?;
            }
            Ok(())
        })()),
        ViewNode::Iframe { props } => Some((|| -> DoweResult<()> {
            if let Some(src) = props.reactive_src.as_deref() {
                validate_typed_path(
                    path,
                    signals,
                    locals,
                    src,
                    "src",
                    ViewPathExpectation::String,
                )?;
            }
            Ok(())
        })()),
        ViewNode::ArcChart { props } => Some((|| -> DoweResult<()> {
            validate_category_chart_common(path, signals, &props.common, "ArcChart")?;
            Ok(())
        })()),
        ViewNode::PieChart { props } => Some((|| -> DoweResult<()> {
            validate_category_chart_common(path, signals, &props.common, "PieChart")?;
            Ok(())
        })()),
        ViewNode::BarChart { props } => Some((|| -> DoweResult<()> {
            validate_category_or_series_chart_common(path, signals, &props.common, "BarChart")?;
            Ok(())
        })()),
        ViewNode::AreaChart { props } => Some((|| -> DoweResult<()> {
            validate_point_or_series_chart_common(path, signals, &props.common, "AreaChart")?;
            Ok(())
        })()),
        ViewNode::LineChart { props } => Some((|| -> DoweResult<()> {
            validate_point_or_series_chart_common(path, signals, &props.common, "LineChart")?;
            Ok(())
        })()),
        ViewNode::Table { props } => Some((|| -> DoweResult<()> {
            validate_table_data(path, signals, &props.data, &props.columns)?;
            Ok(())
        })()),
        ViewNode::Tree { props } => Some((|| -> DoweResult<()> {
            validate_tree_data(path, signals, &props.data)?;
            if let Some(bind) = props.bind.as_deref() {
                if signals.contains_key(path_root(bind))
                    && !writable_signals.contains(path_root(bind))
                {
                    return Err(DoweError::at_path(
                        path,
                        format!("constant path `{bind}` cannot be used in `Tree bind`"),
                    ));
                }
                validate_typed_path(
                    path,
                    signals,
                    locals,
                    bind,
                    "Tree bind",
                    ViewPathExpectation::String,
                )?;
            }
            validate_optional_action(path, actions, props.on_select.as_deref())?;
            Ok(())
        })()),
        _ => None,
    }
}
