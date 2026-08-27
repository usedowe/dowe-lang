#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AvatarSize {
    Xs,
    Sm,
    Md,
    Lg,
    Xl,
    Xxl,
    Xxxl,
    Xxxxl,
    Xxxxxl,
    Xxxxxxl,
    Xxxxxxxl,
}

impl AvatarSize {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "xs" => Some(Self::Xs),
            "sm" => Some(Self::Sm),
            "md" => Some(Self::Md),
            "lg" => Some(Self::Lg),
            "xl" => Some(Self::Xl),
            "2xl" => Some(Self::Xxl),
            "3xl" => Some(Self::Xxxl),
            "4xl" => Some(Self::Xxxxl),
            "5xl" => Some(Self::Xxxxxl),
            "6xl" => Some(Self::Xxxxxxl),
            "7xl" => Some(Self::Xxxxxxxl),
            _ => None,
        }
    }

    pub fn all() -> &'static [Self] {
        &[
            Self::Xs,
            Self::Sm,
            Self::Md,
            Self::Lg,
            Self::Xl,
            Self::Xxl,
            Self::Xxxl,
            Self::Xxxxl,
            Self::Xxxxxl,
            Self::Xxxxxxl,
            Self::Xxxxxxxl,
        ]
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Xs => "xs",
            Self::Sm => "sm",
            Self::Md => "md",
            Self::Lg => "lg",
            Self::Xl => "xl",
            Self::Xxl => "2xl",
            Self::Xxxl => "3xl",
            Self::Xxxxl => "4xl",
            Self::Xxxxxl => "5xl",
            Self::Xxxxxxl => "6xl",
            Self::Xxxxxxxl => "7xl",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AvatarProps {
    pub style: VariantProps,
    pub src: Option<String>,
    pub name: Option<String>,
    pub name_binding: Option<PropBinding>,
    pub alt: String,
    pub alt_binding: Option<PropBinding>,
    pub size: AvatarSize,
    pub size_binding: Option<PropBinding>,
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
