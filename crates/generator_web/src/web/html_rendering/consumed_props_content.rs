fn register_content_consumed_props(node: &ViewNode, context: &ReactiveRenderContext) {
    match node {
        ViewNode::Avatar { .. } => register_structural_consumed_props(
            BuiltinComponent::Avatar,
            context,
            &[
                ("name", "AvatarProps.name"),
                ("alt", "AvatarProps.alt"),
                ("icon", "SideNavIcon.props"),
            ],
        ),
        ViewNode::AvatarGroup { .. } => register_structural_consumed_props(
            BuiltinComponent::AvatarGroup,
            context,
            &[
                ("items", "AvatarGroupProps.items"),
                ("size", "AvatarGroupProps.size"),
            ],
        ),
        ViewNode::Badge { .. } => register_structural_consumed_props(
            BuiltinComponent::Badge,
            context,
            &[
                ("variant", "VariantProps.variant"),
                ("scheme", "VariantProps.color"),
            ],
        ),
        ViewNode::Chip { .. } => register_structural_consumed_props(
            BuiltinComponent::Chip,
            context,
            &[
                ("variant", "VariantProps.variant"),
                ("scheme", "VariantProps.color"),
                ("size", "VariantProps.size"),
                ("rounded", "VariantProps.style.rounded"),
            ],
        ),
        ViewNode::ChatBox { .. } => register_structural_consumed_props(
            BuiltinComponent::ChatBox,
            context,
            &[
                ("messages", "ChatBoxProps.messages"),
                ("actionLabel", "ChatBoxProps.action_label"),
                ("actionVisible", "ChatBoxProps.action_visible"),
                ("loading", "ChatBoxProps.loading"),
                ("sending", "ChatBoxProps.sending"),
                ("streaming", "ChatBoxProps.streaming"),
                ("hasMore", "ChatBoxProps.has_more"),
            ],
        ),
        ViewNode::Empty { .. } => register_structural_consumed_props(
            BuiltinComponent::Empty,
            context,
            &[
                ("title", "EmptyProps.title"),
                ("description", "EmptyProps.description"),
                ("actionLabel", "EmptyProps.action_label"),
            ],
        ),
        ViewNode::Marquee { .. } => register_structural_consumed_props(
            BuiltinComponent::Marquee,
            context,
            &[
                ("speed", "MarqueeProps.speed"),
                ("pauseOnHover", "MarqueeProps.pause_on_hover"),
                ("reverse", "MarqueeProps.reverse"),
                ("orientation", "MarqueeProps.orientation"),
                ("fade", "MarqueeProps.fade"),
                ("fadeColor", "MarqueeProps.fade_color"),
                ("gap", "MarqueeProps.gap"),
            ],
        ),
        ViewNode::TypeWriter { .. } => register_structural_consumed_props(
            BuiltinComponent::TypeWriter,
            context,
            &[
                ("typeSpeed", "TypeWriterProps.type_speed"),
                ("deleteSpeed", "TypeWriterProps.delete_speed"),
                ("afterTyped", "TypeWriterProps.after_typed"),
                ("afterDeleted", "TypeWriterProps.after_deleted"),
                ("repeat", "TypeWriterProps.repeat"),
            ],
        ),
        ViewNode::RichText { .. } => register_structural_consumed_props(
            BuiltinComponent::RichText,
            context,
            &[
                ("text", "RichTextMark.text"),
                ("style", "RichTextMark.style"),
                ("color", "RichTextMark.color"),
            ],
        ),
        ViewNode::Record { .. } => register_structural_consumed_props(
            BuiltinComponent::Record,
            context,
            &[
                ("name", "RecordProps.name"),
                ("url", "RecordProps.url"),
                ("disabled", "RecordProps.disabled"),
                ("maxDuration", "RecordProps.max_duration"),
            ],
        ),
        ViewNode::ToggleGroup { .. } => register_structural_consumed_props(
            BuiltinComponent::ToggleGroup,
            context,
            &[
                ("value", "ToggleGroupProps.value"),
                ("selected", "ToggleGroupProps.selected"),
                ("multiple", "ToggleGroupProps.multiple"),
                ("wide", "ToggleGroupProps.wide"),
                ("vertical", "ToggleGroupProps.vertical"),
                ("disabled", "ToggleGroupProps.disabled"),
                ("ariaLabel", "ToggleGroupProps.aria_label"),
            ],
        ),
        ViewNode::Collapsible { .. } => register_structural_consumed_props(
            BuiltinComponent::Collapsible,
            context,
            &[
                ("label", "CollapsibleProps.label"),
                ("defaultOpen", "CollapsibleProps.default_open"),
                ("disabled", "CollapsibleProps.disabled"),
            ],
        ),
        ViewNode::Countdown { .. } => register_structural_consumed_props(
            BuiltinComponent::Countdown,
            context,
            &[
                ("target", "CountdownProps.target"),
                ("showDays", "CountdownProps.show_days"),
                ("showHours", "CountdownProps.show_hours"),
                ("showMinutes", "CountdownProps.show_minutes"),
                ("showSeconds", "CountdownProps.show_seconds"),
                ("size", "CountdownProps.size"),
                ("onComplete", "CountdownProps.on_complete"),
            ],
        ),
        ViewNode::Map { .. } => register_structural_consumed_props(
            BuiltinComponent::Map,
            context,
            &[
                ("centerLat", "MapProps.center_lat"),
                ("centerLng", "MapProps.center_lng"),
                ("zoom", "MapProps.zoom"),
                ("height", "MapProps.height"),
                ("width", "MapProps.width"),
                ("showControls", "MapProps.show_controls"),
                ("showScale", "MapProps.show_scale"),
                ("interactive", "MapProps.interactive"),
                ("onLocation", "MapProps.on_location"),
                ("onLocationError", "MapProps.on_location_error"),
                ("onRoute", "MapProps.on_route"),
            ],
        ),
        ViewNode::Image { props: _ } => {
            context.register_consumed_prop(BuiltinComponent::Image, "src", "ImageProps.src");
            context.register_consumed_prop(BuiltinComponent::Image, "alt", "ImageProps.alt");
            context.register_consumed_prop(
                BuiltinComponent::Image,
                "objectFit",
                "ImageProps.object_fit",
            );
            context.register_consumed_prop(
                BuiltinComponent::Image,
                "loading",
                "ImageProps.loading",
            );
        }
        ViewNode::Text { props, .. } | ViewNode::Title { props, .. } => {
            if props.size.is_some() || props.size_binding.is_some() {
                context.register_consumed_prop(BuiltinComponent::Text, "size", "TextProps.size");
            }
            if props.weight.is_some() || props.weight_binding.is_some() {
                context.register_consumed_prop(
                    BuiltinComponent::Text,
                    "weight",
                    "TextProps.weight",
                );
            }
            if props.letter_spacing.is_some() || props.letter_spacing_binding.is_some() {
                context.register_consumed_prop(
                    BuiltinComponent::Text,
                    "spacing",
                    "TextProps.letter_spacing",
                );
            }
        }
        ViewNode::Input { props } | ViewNode::Select { props, .. } => {
            let component = if matches!(node, ViewNode::Input { .. }) {
                BuiltinComponent::Input
            } else {
                BuiltinComponent::Select
            };
            if props.color.is_some() || props.color_binding.is_some() {
                context.register_consumed_prop(component, "scheme", "VariantProps.color");
            }
            if props.variant.is_some() || props.variant_binding.is_some() {
                context.register_consumed_prop(component, "variant", "VariantProps.variant");
            }
            if props.size.is_some() || props.size_binding.is_some() {
                context.register_consumed_prop(component, "size", "VariantProps.size");
            }
            if props.element.bind.is_some() {
                context.register_consumed_prop(component, "bind", "ElementProps.bind");
            }
            if props.label.is_some() {
                context.register_consumed_prop(component, "label", "VariantProps.label");
            }
            if props.placeholder.is_some() {
                context.register_consumed_prop(
                    component,
                    "placeholder",
                    "VariantProps.placeholder",
                );
            }
            if props.i18n.is_some() {
                context.register_consumed_prop(component, "i18n", "VariantProps.i18n");
            }
        }
        ViewNode::Button { props, .. } => {
            if props.color.is_some() || props.color_binding.is_some() {
                context.register_consumed_prop(
                    BuiltinComponent::Button,
                    "scheme",
                    "VariantProps.color",
                );
            }
            if props.variant.is_some() || props.variant_binding.is_some() {
                context.register_consumed_prop(
                    BuiltinComponent::Button,
                    "variant",
                    "VariantProps.variant",
                );
            }
            if props.size.is_some() || props.size_binding.is_some() {
                context.register_consumed_prop(
                    BuiltinComponent::Button,
                    "size",
                    "VariantProps.size",
                );
            }
            if props.style.rounded.is_some() || props.style.rounded_binding.is_some() {
                context.register_consumed_prop(
                    BuiltinComponent::Button,
                    "rounded",
                    "VariantProps.style.rounded",
                );
            }
            if props.reactive.loading.is_some() {
                context.register_consumed_prop(
                    BuiltinComponent::Button,
                    "loading",
                    "VariantProps.reactive.loading",
                );
            }
            if props.reactive.disabled.is_some() {
                context.register_consumed_prop(
                    BuiltinComponent::Button,
                    "disabled",
                    "VariantProps.reactive.disabled",
                );
            }
        }
        _ => {}
    }
}
