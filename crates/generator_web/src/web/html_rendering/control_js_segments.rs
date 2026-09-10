fn collect_control_js_node_segments(node: &ViewNode, segments: &mut Vec<JsSegment>, context: &ReactiveRenderContext) -> bool {
    match node {
        ViewNode::NavMenu { props, items } => {
            collect_nav_menu_js_segments(props, items, segments, context)
        }
        ViewNode::Button { props, children } => {
            let (open, close) = button_tags(props, context);
            push_literal(segments, &open);
            if let Some(icon) = props.loading_icon.as_ref() {
                push_literal(
                    segments,
                    r#"<span class="button-loading" data-dowe-button-loading hidden aria-hidden="true">"#,
                );
                push_literal(
                    segments,
                    &render_svg_html(&icon.props, &icon.paths, context),
                );
                push_literal(segments, "</span>");
                push_literal(segments, "<span data-dowe-button-content>");
            }
            if let Some(icon) = props.icon_start.as_ref() {
                push_literal(segments, "<span data-dowe-button-icon-start data-dowe-swap-on>");
                push_literal(
                    segments,
                    &render_svg_html(&icon.props, &icon.paths, context),
                );
                push_literal(segments, "</span>");
            }
            if let Some(icon) = props.swap_icon_off.as_ref() {
                push_literal(segments, "<span data-dowe-button-icon-start data-dowe-swap-off hidden>");
                push_literal(
                    segments,
                    &render_svg_html(&icon.props, &icon.paths, context),
                );
                push_literal(segments, "</span>");
            }
            for child in children {
                collect_js_segments(child, segments, context);
            }
            if let Some(icon) = props.icon_end.as_ref() {
                push_literal(segments, "<span data-dowe-button-icon-end>");
                push_literal(
                    segments,
                    &render_svg_html(&icon.props, &icon.paths, context),
                );
                push_literal(segments, "</span>");
            }
            if props.loading_icon.is_some() {
                push_literal(segments, "</span>");
            }
            push_literal(segments, close);
        }
        ViewNode::Brand { props, children } => {
            let (open, close) = brand_tags(props, context);
            push_literal(segments, &open);
            for child in children {
                collect_js_segments(child, segments, context);
            }
            push_literal(segments, close);
        }
        ViewNode::Banner { props, children } => {
            let (open, close) = banner_tags(props, context);
            push_literal(segments, &open);
            for child in children {
                collect_js_segments(child, segments, context);
            }
            push_literal(segments, close);
        }
        ViewNode::ToggleTheme { props } => {
            push_literal(segments, &render_theme_toggle_html(props, context));
        }
        ViewNode::SelectTheme { props } => {
            push_literal(segments, &render_theme_select_html(props, context));
        }
        ViewNode::Fab { props, actions } => {
            push_literal(segments, &render_fab_html(props, actions, context));
        }
        ViewNode::Input { props } => {
            push_literal(segments, &render_input_html(props, context));
        }
        ViewNode::Slider { props } => {
            push_literal(segments, &render_slider_html(props, context));
        }
        ViewNode::Dropzone { props } => {
            push_literal(segments, &render_dropzone_html(props, context));
        }
        ViewNode::Select {
            props,
            options,
            option_each,
        } => {
            push_literal(
                segments,
                &render_select_html(props, options, option_each.as_ref(), context),
            );
        }
        ViewNode::ComboBox { props, options } => {
            push_literal(segments, &render_combo_box_html(props, options, context));
        }
        ViewNode::CsvField { props, columns } => {
            push_literal(segments, &render_csv_field_html(props, columns, context));
        }
        ViewNode::DragDrop {
            props,
            items,
            groups,
        } => {
            push_literal(
                segments,
                &render_drag_drop_html(props, items, groups, context),
            );
        }
        ViewNode::Editor { props } => {
            push_literal(segments, &render_editor_html(props, context));
        }
        ViewNode::ImageCropper { props } => {
            push_literal(segments, &render_image_cropper_html(props, context));
        }
        ViewNode::Password { props } => {
            push_literal(segments, &render_password_html(props, context));
        }
        ViewNode::Phone { props } => {
            push_literal(segments, &render_phone_html(props, context));
        }
        ViewNode::Pin { props } => {
            push_literal(segments, &render_pin_html(props, context));
        }
        ViewNode::Textarea { props } => {
            push_literal(segments, &render_textarea_html(props, context));
        }
        ViewNode::Code { props } => {
            push_literal(segments, &render_code_html(props, context));
        }
        ViewNode::Video { props } => {
            push_literal(segments, &render_video_html(props, context));
        }
        ViewNode::Iframe { props } => {
            push_literal(segments, &render_iframe_html(props, context));
        }
        ViewNode::Device { props, iframe } => {
            push_literal(segments, &render_device_html(props, iframe, context));
        }
        ViewNode::Canvas { props } => {
            push_literal(segments, &render_canvas_html(props, context));
        }
        ViewNode::Diagram { props } => {
            push_literal(segments, &render_diagram_html(props, context));
        }
        ViewNode::Audio { props } => {
            push_literal(segments, &render_audio_html(props, context));
        }
        ViewNode::Camera { props } => {
            push_literal(segments, &render_camera_html(props, context));
        }
        ViewNode::Microphone { props } => {
            push_literal(segments, &render_microphone_html(props, context));
        }
        ViewNode::Image { props } => {
            push_literal(segments, &render_image_html(props, context));
        }
        ViewNode::Candlestick { props } => {
            push_literal(segments, &render_candlestick_html(props, context));
        }
        ViewNode::ArcChart { props } => {
            push_literal(segments, &render_arc_chart_html(props, context));
        }
        ViewNode::AreaChart { props } => {
            push_literal(segments, &render_area_chart_html(props, context));
        }
        ViewNode::BarChart { props } => {
            push_literal(segments, &render_bar_chart_html(props, context));
        }
        ViewNode::LineChart { props } => {
            push_literal(segments, &render_line_chart_html(props, context));
        }
        ViewNode::PieChart { props } => {
            push_literal(segments, &render_pie_chart_html(props, context));
        }
        ViewNode::Table { props } => {
            push_literal(segments, &render_table_html(props, context));
        }
        ViewNode::Tree { props } => {
            push_literal(segments, &render_tree_html(props, context));
        }
        _ => return false,
    }
    true
}
