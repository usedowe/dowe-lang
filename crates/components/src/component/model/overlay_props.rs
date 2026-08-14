#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AvatarProps {
    pub style: VariantProps,
    pub src: Option<String>,
    pub name: Option<String>,
    pub alt: String,
    pub size: ButtonSize,
    pub status: Option<AvatarStatus>,
    pub bordered: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BadgeProps {
    pub style: VariantProps,
    pub text: String,
    pub position: OverlayCornerPosition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChipProps {
    pub style: VariantProps,
    pub on_close: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkeletonProps {
    pub style: StyleProps,
    pub variant: SkeletonVariant,
    pub animation: SkeletonAnimation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModalProps {
    pub style: VariantProps,
    pub open: String,
    pub on_close: Option<String>,
    pub disable_overlay_close: bool,
    pub hide_close_button: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlertDialogProps {
    pub style: VariantProps,
    pub open: String,
    pub title: String,
    pub description: String,
    pub confirm_text: String,
    pub cancel_text: String,
    pub on_confirm: Option<String>,
    pub on_cancel: Option<String>,
    pub loading: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TooltipProps {
    pub style: VariantProps,
    pub label: String,
    pub position: OverlayPosition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToastProps {
    pub style: VariantProps,
    pub source: Option<String>,
    pub kind: ToastKind,
    pub title: Option<String>,
    pub description: String,
    pub position: OverlayCornerPosition,
    pub show_icon: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DropdownProps {
    pub style: VariantProps,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandProps {
    pub style: VariantProps,
    pub open: Option<String>,
    pub placeholder: String,
    pub empty_text: String,
    pub close_text: String,
    pub navigate_text: String,
    pub select_text: String,
    pub toggle_text: String,
    pub shortcut: String,
    pub disable_global_shortcut: bool,
    pub show_footer: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AvatarGroupProps {
    pub style: VariantProps,
    pub items: Option<String>,
    pub size: ButtonSize,
    pub max: Option<u16>,
    pub auto_fit: bool,
    pub inline: bool,
    pub bordered: bool,
}

impl AvatarGroupProps {
    pub fn visible_item_count(&self, item_count: usize) -> usize {
        self.max
            .map(|max| usize::from(max).min(item_count))
            .unwrap_or(item_count)
    }

    pub fn overflow_count(&self, item_count: usize) -> usize {
        item_count.saturating_sub(self.visible_item_count(item_count))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AvatarGroupItem {
    pub src: Option<String>,
    pub name: Option<String>,
    pub alt: Option<String>,
    pub on_click: Option<String>,
    pub navigation: Option<NavigationAction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatBoxProps {
    pub style: VariantProps,
    pub messages: String,
    pub mode: ChatBoxMode,
    pub current_user_id: String,
    pub user_name: String,
    pub user_avatar: Option<String>,
    pub user_status: String,
    pub assistant_name: String,
    pub assistant_avatar: Option<String>,
    pub show_header: bool,
    pub placeholder: String,
    pub show_attachments: bool,
    pub show_voice_note: bool,
    pub show_camera: bool,
    pub loading: Option<String>,
    pub sending: Option<String>,
    pub streaming: Option<String>,
    pub has_more: Option<String>,
    pub on_send: Option<String>,
    pub on_load_more: Option<String>,
    pub on_stop: Option<String>,
    pub on_voice_note: Option<String>,
    pub on_file_attach: Option<String>,
    pub on_camera_capture: Option<String>,
}
