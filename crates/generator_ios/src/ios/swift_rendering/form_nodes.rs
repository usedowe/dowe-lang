fn render_swift_form_node(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    flow: NativeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
    match node {
        ViewNode::Button { props, children } => {
            let current_font = props.style.font.as_ref().or(inherited_font);
            let reactive_text = |path: &str, fallback: &str| {
                context
                    .item_value(path)
                    .map(|item| {
                        format!(
                            "state.text(\"{}\", item: {item})",
                            escape_swift(&context.item_path(path).expect("item path"))
                        )
                    })
                    .unwrap_or_else(|| {
                        format!(
                            "state.text(\"{}\", fallback: \"{fallback}\")",
                            escape_swift(&context.signal_path(path))
                        )
                    })
            };
            let reactive_bool = |path: &str| {
                context
                    .item_value(path)
                    .map(|item| {
                        format!(
                            "state.bool(\"{}\", item: {item})",
                            escape_swift(&context.item_path(path).expect("item path"))
                        )
                    })
                    .unwrap_or_else(|| {
                        format!(
                            "state.bool(\"{}\", fallback: true)",
                            escape_swift(&context.signal_path(path))
                        )
                    })
            };
            let icon_condition = |path: &str,
                                  comparison: Option<&dowe_components::ReactiveNumberComparison>| {
                comparison
                    .map(|comparison| {
                        format!(
                            "(Double({}) ?? 0) {} {}",
                            reactive_text(path, "0"),
                            comparison.operator.as_str(),
                            comparison.value
                        )
                    })
                    .unwrap_or_else(|| reactive_bool(path))
            };
            let action = props
                .element
                .on_click
                .as_deref()
                .and_then(|name| context.action_id(name))
                .map(|id| {
                    let item = context
                        .active_item()
                        .map(|value| format!(", item: {value}"))
                        .unwrap_or_default();
                    format!("{{ state.run(\"{}\"{item}) }}", escape_swift(id))
                })
                .unwrap_or_else(|| swift_navigation_action(props.navigation.as_ref()));
            let loading = props.reactive.loading.as_ref().map(|path| reactive_bool(path));
            let variant = props.reactive.variant.as_ref().map(|path| reactive_text(path, "solid"));
            let scheme = props.reactive.scheme.as_ref().map(|path| reactive_text(path, "primary"));
            let variant_value = variant.clone().unwrap_or_else(|| {
                format!(
                    "\"{}\"",
                    props.variant.unwrap_or(ComponentVariant::Solid).as_str()
                )
            });
            let scheme_value = scheme.clone().unwrap_or_else(|| {
                format!(
                    "\"{}\"",
                    props.color.unwrap_or(ColorFamily::Primary).as_str()
                )
            });
            let reactive_visual = variant.is_some() || scheme.is_some();
            let content = if reactive_visual {
                format!("doweButtonContent({variant_value}, {scheme_value})")
            } else {
                variant_content(props).to_string()
            };
            output.push_str(&format!("{pad}Button(action: {action}) {{\n"));
            let render_contents = |content_indent: usize,
                                   opacity: Option<&str>,
                                   output: &mut String| {
                let content_pad = " ".repeat(content_indent);
                output.push_str(&format!("{content_pad}HStack(spacing: 8) {{\n"));
                if let Some(icon) = props.icon_start.as_ref() {
                    if let Some(path) = props.reactive.icon_start_when.as_ref() {
                        output.push_str(&format!("{content_pad}    if {} {{\n", icon_condition(path, props.reactive.icon_start_comparison.as_ref())));
                        render_swift_button_icon(icon, &content, content_indent + 8, output);
                        output.push_str(&format!("{content_pad}    }}\n"));
                    } else {
                        render_swift_button_icon(icon, &content, content_indent + 4, output);
                    }
                }
                for child in children {
                    render_swift_node_in_flow(
                        child,
                        content_indent + 4,
                        output,
                        NativeFlow::Inline,
                        current_font,
                        default_family,
                        context,
                    );
                }
                if let Some(icon) = props.icon_end.as_ref() {
                    if let Some(path) = props.reactive.icon_end_when.as_ref() {
                        output.push_str(&format!("{content_pad}    if {} {{\n", icon_condition(path, props.reactive.icon_end_comparison.as_ref())));
                        render_swift_button_icon(icon, &content, content_indent + 8, output);
                        output.push_str(&format!("{content_pad}    }}\n"));
                    } else {
                        render_swift_button_icon(icon, &content, content_indent + 4, output);
                    }
                }
                let modifier = opacity.map(|value| format!(".opacity({value})")).unwrap_or_default();
                output.push_str(&format!("{content_pad}}}{modifier}\n"));
            };
            if let Some(loading) = loading.as_ref() {
                output.push_str(&format!("{pad}    ZStack {{\n"));
                let opacity = format!("{loading} ? 0 : 1");
                render_contents(indent + 8, Some(&opacity), output);
                output.push_str(&format!("{pad}        if {loading} {{\n"));
                if let Some(icon) = props.loading_icon.as_ref() {
                    render_swift_button_spinner(icon, &content, indent + 12, output);
                }
                output.push_str(&format!("{pad}        }}\n"));
                output.push_str(&format!("{pad}    }}\n"));
            } else {
                render_contents(indent + 4, None, output);
            }
            output.push_str(&format!("{pad}}}\n"));
            let mut button_style = props.style.clone();
            button_style.shadow = None;
            button_style.shadow_color = None;
            button_style.animation = None;
            let mut modifiers = swift_modifiers_for_style(&button_style);
            if let Some(size) = props.reactive.size.as_ref().map(|path| reactive_text(path, "md")) {
                modifiers.push(format!(".padding(.horizontal, doweButtonHorizontalPadding({size}))"));
                modifiers.push(format!(".padding(.vertical, doweButtonVerticalPadding({size}))"));
                modifiers.push(format!(".frame(minHeight: doweButtonMinHeight({size}))"));
            }
            if flow.is_grid_item() && props.style.sizing.w.is_none() {
                modifiers.push(".frame(maxWidth: .infinity, alignment: .center)".to_string());
            }
            modifiers.push(".contentShape(Rectangle())".to_string());
            let container = if reactive_visual {
                format!("doweButtonContainer({variant_value}, {scheme_value})")
            } else {
                variant_container(props).to_string()
            };
            modifiers.push(format!(".background({container})"));
            modifiers.push(format!(".foregroundStyle({content})"));
            let radius = props.reactive.rounded.as_ref().map(|path| format!("doweButtonRadius({})", reactive_text(path, "md"))).unwrap_or_else(|| swift_control_radius(&props.style));
            modifiers.push(format!(
                ".clipShape(RoundedRectangle(cornerRadius: {radius}))"
            ));
            if reactive_visual {
                modifiers.push(format!(".overlay(RoundedRectangle(cornerRadius: {radius}).stroke({content}, lineWidth: {variant_value} == \"outlined\" ? CGFloat(1) : CGFloat(0)))"));
            } else if props.variant.unwrap_or(ComponentVariant::Solid) == ComponentVariant::Outlined {
                modifiers.push(format!(
                    ".overlay(RoundedRectangle(cornerRadius: {radius}).stroke({}, lineWidth: CGFloat(1)))",
                    variant_content(props)
                ));
            }
            modifiers.push(".buttonStyle(.plain)".to_string());
            if props.icon_only {
                modifiers.push(".accessibilityElement(children: .ignore)".to_string());
                modifiers.push(format!(
                    ".accessibilityLabel(Text(\"{}\"))",
                    escape_swift(props.label.as_deref().unwrap_or_default())
                ));
            }
            if let Some(loading) = loading.as_ref() {
                modifiers.push(format!(".disabled({loading})"));
            }
            if let Some(modifier) = swift_shadow_modifier_with_radius(&props.style, &radius) {
                modifiers.push(modifier);
            }
            if let Some(animation) = props.style.animation {
                modifiers.push(format!(
                    ".modifier(DoweAnimationModifier(preset: {}))",
                    swift_animation_preset(animation)
                ));
            }
            append_swift_modifiers(output, indent, &modifiers);
        }
        ViewNode::ToggleTheme { props } => render_swift_theme_toggle(props, indent, output),
        ViewNode::SelectTheme { props } => render_swift_theme_select(
            props,
            indent,
            output,
            inherited_font,
            default_family,
        ),
        ViewNode::Fab { props, actions } => {
            render_swift_fab(props, actions, indent, output, context, None)
        }
        ViewNode::Input { props } => {
            let binding = props
                .element
                .bind
                .as_deref()
                .map(|path| {
                    format!(
                        "state.binding(\"{}\")",
                        escape_swift(&context.signal_path(path))
                    )
                })
                .unwrap_or_else(|| "nil".to_string());
            let size = swift_text_size_expr(false, INPUT_TEXT_SIZE);
            let base_border =
                if props.variant.unwrap_or(ComponentVariant::Solid) == ComponentVariant::Outlined {
                    (
                        format!("Optional({})", color_ref(ColorToken::Muted)),
                        "CGFloat(1)".to_string(),
                    )
                } else {
                    ("nil".to_string(), "CGFloat(0)".to_string())
                };
            let (border, border_width) =
                swift_style_border(&props.style, &base_border.0, &base_border.1);
            let shadow = swift_shadow_spec(&props.style)
                .map(|value| format!("Optional({value})"))
                .unwrap_or_else(|| "nil".to_string());
            output.push_str(&format!(
                "{pad}DoweInputField(value: {binding}, label: {}, placeholder: {}, floating: {}, font: {}, fontSize: {size}, lineHeight: CGFloat({}), minHeight: CGFloat({}), horizontalPadding: CGFloat({}), backgroundColor: {}, contentColor: {}, borderColor: {border}, borderWidth: {border_width}, radius: {}, shadow: {shadow}, startIcon: {}, endIcon: {})\n",
                swift_optional_literal(props.label.as_deref()),
                swift_string_literal(props.placeholder.as_deref().unwrap_or_default()),
                props.label_floating,
                swift_font_value(
                    props.style.font.as_ref().or(inherited_font),
                    &size,
                    default_family,
                ),
                text_typography(false, INPUT_TEXT_SIZE).line_height,
                INPUT_MIN_HEIGHT.native_units(),
                INPUT_HORIZONTAL_PADDING.native_units(),
                variant_container(props),
                variant_content(props),
                swift_control_radius(&props.style),
                swift_control_icon(props.icon_start.as_ref()),
                swift_control_icon(props.icon_end.as_ref())
            ));
            let mut input_style = props.style.clone();
            input_style.shadow = None;
            input_style.shadow_color = None;
            input_style.rounded = None;
            input_style.border = None;
            input_style.border_color = None;
            append_swift_modifiers(output, indent, &swift_modifiers_for_style(&input_style));
        }
        ViewNode::Slider { props } => render_swift_slider(props, indent, output, context),
        ViewNode::Dropzone { props } => render_swift_dropzone(props, indent, output),
        ViewNode::Select {
            props,
            options,
            option_each,
        } => {
            let binding = props
                .element
                .bind
                .as_deref()
                .map(|path| {
                    format!(
                        "state.binding(\"{}\")",
                        escape_swift(&context.signal_path(path))
                    )
                })
                .unwrap_or_else(|| "nil".to_string());
            let size = swift_text_size_expr(false, INPUT_TEXT_SIZE);
            let border =
                if props.variant.unwrap_or(ComponentVariant::Solid) == ComponentVariant::Outlined {
                    format!("Optional({})", color_ref(ColorToken::Muted))
                } else {
                    "nil".to_string()
                };
            output.push_str(&format!(
                "{pad}DoweSelectField(value: {binding}, label: {}, placeholder: {}, floating: {}, options: {}, font: {}, fontSize: {size}, lineHeight: CGFloat({}), minHeight: CGFloat({}), horizontalPadding: CGFloat({}), backgroundColor: {}, contentColor: {}, borderColor: {border}, radius: {})\n",
                swift_optional_literal(props.label.as_deref()),
                swift_string_literal(props.placeholder.as_deref().unwrap_or("Select an option")),
                props.label_floating,
                swift_select_options(options, option_each.as_ref(), context),
                swift_font_value(props.style.font.as_ref().or(inherited_font), &size, default_family),
                text_typography(false, INPUT_TEXT_SIZE).line_height,
                INPUT_MIN_HEIGHT.native_units(),
                INPUT_HORIZONTAL_PADDING.native_units(),
                variant_container(props),
                variant_content(props),
                swift_control_radius(&props.style)
            ));
            append_swift_modifiers(output, indent, &swift_modifiers_for_style(&props.style));
        }
        ViewNode::ComboBox { props, options } => {
            render_swift_combo_box(props, options, indent, output, inherited_font, default_family, context);
        }
        ViewNode::CsvField { props, columns } => {
            output.push_str(&format!(
                "{pad}DoweCsvField(label: {}, buttonText: {}, modalTitle: {}, instructions: {}, columns: {}, backgroundColor: {}, contentColor: {})\n",
                swift_optional_literal(props.style.label.as_deref()),
                swift_string_literal(&props.button_text),
                swift_string_literal(&props.modal_title),
                swift_string_literal(&props.instructions),
                swift_csv_columns(columns),
                variant_container(&props.style),
                variant_content(&props.style)
            ));
            append_swift_modifiers(output, indent, &swift_modifiers_for_style(&props.style.style));
        }
        ViewNode::DragDrop {
            props,
            items,
            groups,
        } => {
            output.push_str(&format!(
                "{pad}DoweDragDrop(label: {}, emptyText: {}, direction: {}, items: {}, groups: {}, backgroundColor: {}, contentColor: {})\n",
                swift_optional_literal(props.style.label.as_deref()),
                swift_string_literal(&props.empty_text),
                swift_string_literal(props.direction.as_str()),
                swift_drag_items(items),
                swift_drag_groups(groups),
                variant_container(&props.style),
                variant_content(&props.style)
            ));
            append_swift_modifiers(output, indent, &swift_modifiers_for_style(&props.style.style));
        }
        ViewNode::Editor { props } => {
            output.push_str(&format!(
                "{pad}DoweEditorField(value: {}, initialValue: {}, label: {}, placeholder: {}, minHeight: CGFloat({}), hideToolbar: {}, readOnly: {}, backgroundColor: {}, contentColor: {})\n",
                swift_text_binding(props.style.element.bind.as_deref(), context),
                swift_string_literal(props.value.as_deref().unwrap_or_default()),
                swift_optional_literal(props.style.label.as_deref()),
                swift_string_literal(props.style.placeholder.as_deref().unwrap_or_default()),
                props.min_height,
                props.hide_toolbar,
                props.readonly || props.disabled,
                variant_container(&props.style),
                variant_content(&props.style)
            ));
            append_swift_modifiers(output, indent, &swift_modifiers_for_style(&props.style.style));
        }
        ViewNode::ImageCropper { props } => {
            output.push_str(&format!(
                "{pad}DoweImageCropper(value: {}, initialValue: {}, label: {}, placeholder: {}, shape: {}, backgroundColor: {}, contentColor: {})\n",
                swift_text_binding(props.style.element.bind.as_deref(), context),
                swift_string_literal(props.src.as_deref().unwrap_or_default()),
                swift_optional_literal(props.style.label.as_deref()),
                swift_string_literal(props.style.placeholder.as_deref().unwrap_or("Upload")),
                swift_string_literal(props.shape.as_str()),
                variant_container(&props.style),
                variant_content(&props.style)
            ));
            append_swift_modifiers(output, indent, &swift_modifiers_for_style(&props.style.style));
        }
        ViewNode::PasswordField { props } => {
            output.push_str(&format!(
                "{pad}DowePasswordField(value: {}, initialValue: {}, label: {}, placeholder: {}, floating: {}, hideStrength: {}, weakLabel: {}, mediumLabel: {}, strongLabel: {}, readOnly: {}, backgroundColor: {}, contentColor: {})\n",
                swift_text_binding(props.style.element.bind.as_deref(), context),
                swift_string_literal(props.value.as_deref().unwrap_or_default()),
                swift_optional_literal(props.style.label.as_deref()),
                swift_string_literal(props.style.placeholder.as_deref().unwrap_or_default()),
                props.style.label_floating,
                props.hide_strength,
                swift_string_literal(&props.weak_label),
                swift_string_literal(&props.medium_label),
                swift_string_literal(&props.strong_label),
                props.readonly || props.disabled,
                variant_container(&props.style),
                variant_content(&props.style)
            ));
            append_swift_modifiers(output, indent, &swift_modifiers_for_style(&props.style.style));
        }
        ViewNode::PhoneField { props } => {
            output.push_str(&format!(
                "{pad}DowePhoneField(value: {}, initialValue: {}, label: {}, placeholder: {}, country: {}, countries: {}, priorityCountries: {}, dialCodeName: {}, searchPlaceholder: {}, emptyText: {}, loadingText: {}, floating: {}, disabled: {}, backgroundColor: {}, contentColor: {})\n",
                swift_text_binding(props.style.element.bind.as_deref(), context),
                swift_string_literal(props.value.as_deref().unwrap_or_default()),
                swift_optional_literal(props.style.label.as_deref()),
                swift_string_literal(props.style.placeholder.as_deref().unwrap_or("Enter phone number")),
                swift_string_literal(props.country.as_deref().unwrap_or("US")),
                "DowePhoneCatalog.countries",
                swift_string_array(&props.priority_countries),
                swift_string_literal(&props.dial_code_name),
                swift_string_literal(&props.search_placeholder),
                swift_string_literal(&props.empty_text),
                swift_string_literal(&props.loading_text),
                props.style.label_floating,
                props.disabled,
                variant_container(&props.style),
                variant_content(&props.style)
            ));
            append_swift_modifiers(output, indent, &swift_modifiers_for_style(&props.style.style));
        }
        ViewNode::PinField { props } => {
            let size = props.style.size.unwrap_or(ButtonSize::Md);
            let base_border = if props.style.variant.unwrap_or(ComponentVariant::Solid)
                == ComponentVariant::Outlined
            {
                (
                    format!("Optional({})", color_ref(ColorToken::Muted)),
                    "CGFloat(1)".to_string(),
                )
            } else {
                ("nil".to_string(), "CGFloat(0)".to_string())
            };
            let (border, border_width) =
                swift_style_border(&props.style.style, &base_border.0, &base_border.1);
            output.push_str(&format!(
                "{pad}DowePinField(value: {}, initialValue: {}, label: {}, length: {}, kind: {}, size: {}, variant: {}, helpText: {}, errorText: {}, backgroundColor: {}, contentColor: {}, borderColor: {}, borderWidth: {}, radius: {})\n",
                swift_text_binding(props.style.element.bind.as_deref(), context),
                swift_string_literal(props.value.as_deref().unwrap_or_default()),
                swift_optional_literal(props.style.label.as_deref()),
                props.length,
                swift_string_literal(props.kind.as_str()),
                swift_string_literal(size.as_str()),
                swift_string_literal(props.style.variant.unwrap_or(ComponentVariant::Solid).as_str()),
                swift_optional_literal(props.help_text.as_deref()),
                swift_optional_literal(props.error_text.as_deref()),
                variant_container(&props.style),
                variant_content(&props.style),
                border,
                border_width,
                swift_control_radius(&props.style.style)
            ));
            append_swift_modifiers(output, indent, &swift_modifiers_for_style(&props.style.style));
        }
        ViewNode::Textarea { props } => {
            output.push_str(&format!(
                "{pad}DoweTextarea(value: {}, initialValue: {}, label: {}, placeholder: {}, floating: {}, rows: {}, maxLength: {}, readOnly: {}, backgroundColor: {}, contentColor: {})\n",
                swift_text_binding(props.style.element.bind.as_deref(), context),
                swift_string_literal(props.value.as_deref().unwrap_or_default()),
                swift_optional_literal(props.style.label.as_deref()),
                swift_string_literal(props.style.placeholder.as_deref().unwrap_or_default()),
                props.style.label_floating,
                props.rows,
                props.max_length
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "nil".to_string()),
                props.readonly || props.disabled,
                variant_container(&props.style),
                variant_content(&props.style)
            ));
            append_swift_modifiers(output, indent, &swift_modifiers_for_style(&props.style.style));
        }
        ViewNode::Checkbox { props } => render_swift_checkbox(props, indent, output, context),
        ViewNode::Color { props } => render_swift_color(props, indent, output, context),
        ViewNode::Date { props } => render_swift_date(props, indent, output, context),
        ViewNode::DateRange { props } => render_swift_date_range(props, indent, output, context),
        ViewNode::RadioGroup { props, options } => {
            render_swift_radio_group(props, options, indent, output, context)
        }
        ViewNode::Toggle { props } => render_swift_toggle(props, indent, output, context),
        _ => unreachable!(),
    }
}

fn swift_control_icon(icon: Option<&SideNavIcon>) -> String {
    icon.map(|icon| format!(
        "DoweControlIcon(viewBox: {}, paths: {})",
        swift_svg_view_box(&icon.props.view_box),
        swift_svg_paths(&icon.paths)
    ))
    .unwrap_or_else(|| "nil".to_string())
}

fn swift_string_array(values: &[String]) -> String {
    format!(
        "[{}]",
        values
            .iter()
            .map(|value| swift_string_literal(value))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn render_swift_combo_box(
    props: &ComboBoxProps,
    options: &[ComboOption],
    indent: usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
    let size = swift_text_size_expr(false, INPUT_TEXT_SIZE);
    let border = if props.style.variant.unwrap_or(ComponentVariant::Solid) == ComponentVariant::Outlined {
        format!("Optional({})", color_ref(ColorToken::Muted))
    } else {
        "nil".to_string()
    };
    output.push_str(&format!(
        "{pad}DoweComboBox(value: {}, initialValue: {}, label: {}, placeholder: {}, floating: {}, searchPlaceholder: {}, emptyText: {}, clearable: {}, options: {}, font: {}, fontSize: {size}, lineHeight: CGFloat({}), minHeight: CGFloat({}), horizontalPadding: CGFloat({}), backgroundColor: {}, contentColor: {}, borderColor: {border}, radius: {})\n",
        swift_text_binding(props.style.element.bind.as_deref(), context),
        swift_string_literal(props.value.as_deref().unwrap_or_default()),
        swift_optional_literal(props.style.label.as_deref()),
        swift_string_literal(props.style.placeholder.as_deref().unwrap_or("Select an option")),
        props.style.label_floating,
        swift_string_literal(&props.search_placeholder),
        swift_string_literal(&props.empty_text),
        props.clearable,
        swift_combo_options(options),
        swift_font_value(props.style.style.font.as_ref().or(inherited_font), &size, default_family),
        text_typography(false, INPUT_TEXT_SIZE).line_height,
        INPUT_MIN_HEIGHT.native_units(),
        INPUT_HORIZONTAL_PADDING.native_units(),
        variant_container(&props.style),
        variant_content(&props.style),
        swift_control_radius(&props.style.style)
    ));
    append_swift_modifiers(output, indent, &swift_modifiers_for_style(&props.style.style));
}

fn swift_text_binding(bind: Option<&str>, context: &SwiftReactiveContext) -> String {
    bind.map(|path| {
        format!(
            "state.binding(\"{}\")",
            escape_swift(&context.signal_path(path))
        )
    })
    .unwrap_or_else(|| "nil".to_string())
}

fn swift_combo_options(options: &[ComboOption]) -> String {
    format!(
        "[{}]",
        options
            .iter()
            .map(|option| format!(
                "DoweSelectOption(value: {}, label: {}, description: {})",
                swift_string_literal(&option.value),
                swift_string_literal(&option.label),
                swift_optional_literal(option.description.as_deref())
            ))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn swift_csv_columns(columns: &[CsvColumn]) -> String {
    format!(
        "[{}]",
        columns
            .iter()
            .map(|column| format!(
                "DoweCsvColumn(name: {}, label: {})",
                swift_string_literal(&column.name),
                swift_optional_literal(column.label.as_deref())
            ))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn swift_drag_items(items: &[DragItem]) -> String {
    format!(
        "[{}]",
        items
            .iter()
            .map(swift_drag_item)
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn swift_drag_groups(groups: &[DragGroup]) -> String {
    format!(
        "[{}]",
        groups
            .iter()
            .map(|group| format!(
                "DoweDragGroup(id: {}, title: {}, items: {})",
                swift_string_literal(&group.id),
                swift_optional_literal(group.title.as_deref()),
                swift_drag_items(&group.items)
            ))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn swift_drag_item(item: &DragItem) -> String {
    format!(
        "DoweDragItem(id: {}, label: {}, description: {}, disabled: {})",
        swift_string_literal(&item.id),
        swift_optional_literal(item.label.as_deref()),
        swift_optional_literal(item.description.as_deref()),
        item.disabled
    )
}
