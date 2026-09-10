fn register_swift_display_consumed_props(node: &ViewNode, context: &SwiftReactiveContext) {
    match node {
        ViewNode::Avatar { .. } => register_swift_structural_consumed_props(
            dowe_components::BuiltinComponent::Avatar,
            context,
            &[
                ("name", "AvatarProps.name"),
                ("alt", "AvatarProps.alt"),
                ("icon", "SideNavIcon.props"),
            ],
        ),
        ViewNode::AvatarGroup { .. } => register_swift_structural_consumed_props(
            dowe_components::BuiltinComponent::AvatarGroup,
            context,
            &[
                ("items", "AvatarGroupProps.items"),
                ("size", "AvatarGroupProps.size"),
            ],
        ),
        ViewNode::Badge { .. } => register_swift_structural_consumed_props(
            dowe_components::BuiltinComponent::Badge,
            context,
            &[
                ("variant", "VariantProps.variant"),
                ("scheme", "VariantProps.color"),
            ],
        ),
        ViewNode::Chip { .. } => register_swift_structural_consumed_props(
            dowe_components::BuiltinComponent::Chip,
            context,
            &[
                ("variant", "VariantProps.variant"),
                ("scheme", "VariantProps.color"),
                ("size", "VariantProps.size"),
                ("rounded", "VariantProps.style.rounded"),
            ],
        ),
        ViewNode::ChatBox { .. } => register_swift_structural_consumed_props(
            dowe_components::BuiltinComponent::ChatBox,
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
        ViewNode::Empty { .. } => register_swift_structural_consumed_props(
            dowe_components::BuiltinComponent::Empty,
            context,
            &[
                ("title", "EmptyProps.title"),
                ("description", "EmptyProps.description"),
                ("actionLabel", "EmptyProps.action_label"),
            ],
        ),
        ViewNode::Marquee { .. } => register_swift_structural_consumed_props(
            dowe_components::BuiltinComponent::Marquee,
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
        ViewNode::TypeWriter { .. } => register_swift_structural_consumed_props(
            dowe_components::BuiltinComponent::TypeWriter,
            context,
            &[
                ("typeSpeed", "TypeWriterProps.type_speed"),
                ("deleteSpeed", "TypeWriterProps.delete_speed"),
                ("afterTyped", "TypeWriterProps.after_typed"),
                ("afterDeleted", "TypeWriterProps.after_deleted"),
                ("repeat", "TypeWriterProps.repeat"),
            ],
        ),
        ViewNode::RichText { .. } => register_swift_structural_consumed_props(
            dowe_components::BuiltinComponent::RichText,
            context,
            &[
                ("text", "RichTextMark.text"),
                ("style", "RichTextMark.style"),
                ("color", "RichTextMark.color"),
            ],
        ),
        ViewNode::Record { .. } => register_swift_structural_consumed_props(
            dowe_components::BuiltinComponent::Record,
            context,
            &[
                ("name", "RecordProps.name"),
                ("url", "RecordProps.url"),
                ("disabled", "RecordProps.disabled"),
                ("maxDuration", "RecordProps.max_duration"),
            ],
        ),
        ViewNode::ToggleGroup { .. } => register_swift_structural_consumed_props(
            dowe_components::BuiltinComponent::ToggleGroup,
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
        ViewNode::Collapsible { .. } => register_swift_structural_consumed_props(
            dowe_components::BuiltinComponent::Collapsible,
            context,
            &[
                ("label", "CollapsibleProps.label"),
                ("defaultOpen", "CollapsibleProps.default_open"),
                ("disabled", "CollapsibleProps.disabled"),
            ],
        ),
        ViewNode::Countdown { .. } => register_swift_structural_consumed_props(
            dowe_components::BuiltinComponent::Countdown,
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
        ViewNode::Map { .. } => register_swift_structural_consumed_props(
            dowe_components::BuiltinComponent::Map,
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
        _ => {}
    }
}
