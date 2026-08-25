#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FormControlValidation {
    pub help_text: Option<String>,
    pub error_text: Option<String>,
    pub rules: Vec<FormValidationRule>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormValidationRule {
    pub kind: FormValidationRuleKind,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewForm {
    pub signal: String,
    pub fields: Vec<ViewFormField>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewFormField {
    pub path: String,
    pub kind: ViewFormFieldKind,
    pub rules: Vec<FormValidationRule>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewFormFieldKind {
    String,
    Boolean,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormValidationRuleKind {
    Required,
    Email,
    Min(u16),
    Max(u16),
    Url,
    Phone,
    Pattern(String),
    Alphanumeric,
    Numeric,
    Alpha,
    Matches(String),
    StrongPassword,
    CreditCard,
    Date,
    MinWords(u16),
    MaxWords(u16),
}

impl FormValidationRuleKind {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Required => "required",
            Self::Email => "email",
            Self::Min(_) => "min",
            Self::Max(_) => "max",
            Self::Url => "url",
            Self::Phone => "phone",
            Self::Pattern(_) => "pattern",
            Self::Alphanumeric => "alphanumeric",
            Self::Numeric => "numeric",
            Self::Alpha => "alpha",
            Self::Matches(_) => "matches",
            Self::StrongPassword => "strongPassword",
            Self::CreditCard => "creditCard",
            Self::Date => "date",
            Self::MinWords(_) => "minWords",
            Self::MaxWords(_) => "maxWords",
        }
    }

    pub fn argument(&self) -> Option<String> {
        match self {
            Self::Min(value) | Self::Max(value) | Self::MinWords(value) | Self::MaxWords(value) => {
                Some(value.to_string())
            }
            Self::Pattern(value) | Self::Matches(value) => Some(value.clone()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckboxProps {
    pub style: VariantProps,
    pub checked: bool,
    pub disabled: bool,
    pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColorProps {
    pub style: VariantProps,
    pub value: String,
    pub size: ButtonSize,
    pub name: Option<String>,
    pub help_text: Option<String>,
    pub error_text: Option<String>,
    pub show_hex: bool,
    pub show_rgb: bool,
    pub show_cmyk: bool,
    pub show_oklch: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DateProps {
    pub style: VariantProps,
    pub value: Option<String>,
    pub size: ButtonSize,
    pub name: Option<String>,
    pub help_text: Option<String>,
    pub error_text: Option<String>,
    pub min: Option<String>,
    pub max: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DateRangeProps {
    pub style: VariantProps,
    pub start: Option<String>,
    pub end: Option<String>,
    pub start_value: Option<String>,
    pub end_value: Option<String>,
    pub size: ButtonSize,
    pub name: Option<String>,
    pub help_text: Option<String>,
    pub error_text: Option<String>,
    pub min: Option<String>,
    pub max: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RadioGroupProps {
    pub style: VariantProps,
    pub size: ButtonSize,
    pub orientation: RadioGroupOrientation,
    pub name: Option<String>,
    pub info: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RadioOption {
    pub value: String,
    pub label: String,
    pub disabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RadioGroupOrientation {
    Vertical,
    Horizontal,
}

impl RadioGroupOrientation {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "vertical" => Some(Self::Vertical),
            "horizontal" => Some(Self::Horizontal),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Vertical => "vertical",
            Self::Horizontal => "horizontal",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToggleProps {
    pub style: VariantProps,
    pub checked: bool,
    pub disabled: bool,
    pub name: Option<String>,
    pub label_left: Option<String>,
    pub label_right: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThemeToggleProps {
    pub style: VariantProps,
    pub light_label: String,
    pub dark_label: String,
    pub light_icon: SideNavIcon,
    pub dark_icon: SideNavIcon,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThemeSelectProps {
    pub style: VariantProps,
    pub label: String,
    pub placeholder: String,
    pub themes: Vec<String>,
    pub default_theme: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FabProps {
    pub style: VariantProps,
    pub position: OverlayCornerPosition,
    pub fixed: bool,
    pub offset_x: ScaleValue,
    pub offset_y: ScaleValue,
    pub icon: ViewIcon,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FabAction {
    pub label: String,
    pub icon: ViewIcon,
    pub color: ColorFamily,
    pub on_click: Option<String>,
    pub navigation: Option<NavigationAction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SliderProps {
    pub style: VariantProps,
    pub value: String,
    pub min: String,
    pub max: String,
    pub step: Option<String>,
    pub size: ButtonSize,
    pub name: Option<String>,
    pub hide_label: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DropzoneProps {
    pub style: VariantProps,
    pub accept: Option<String>,
    pub multiple: bool,
    pub max_size: Option<u64>,
    pub size: ButtonSize,
    pub name: Option<String>,
    pub help_text: Option<String>,
    pub error_text: Option<String>,
    pub disabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComboBoxProps {
    pub style: VariantProps,
    pub value: Option<String>,
    pub search_placeholder: String,
    pub empty_text: String,
    pub loading_text: String,
    pub loading_more_text: String,
    pub clearable: bool,
    pub disabled: bool,
    pub name: Option<String>,
    pub help_text: Option<String>,
    pub error_text: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComboOption {
    pub value: String,
    pub label: String,
    pub description: Option<String>,
    pub src: Option<String>,
    pub icon: Option<ViewIcon>,
    pub disabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CsvFieldProps {
    pub style: VariantProps,
    pub button_text: String,
    pub modal_title: String,
    pub instructions: String,
    pub cancel_text: String,
    pub confirm_text: String,
    pub clear_text: String,
    pub preview_title: String,
    pub multiple: bool,
    pub show_preview: bool,
    pub preview_rows: u16,
    pub preview_page_size: u16,
    pub error_text: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CsvColumn {
    pub name: String,
    pub label: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DragDropProps {
    pub style: VariantProps,
    pub empty_text: String,
    pub direction: DragDropDirection,
    pub allow_group_transfer: bool,
    pub disabled: bool,
    pub size: ButtonSize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DragItem {
    pub id: String,
    pub label: Option<String>,
    pub description: Option<String>,
    pub disabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DragGroup {
    pub id: String,
    pub title: Option<String>,
    pub items: Vec<DragItem>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DragDropDirection {
    Horizontal,
    Vertical,
}

impl DragDropDirection {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "horizontal" => Some(Self::Horizontal),
            "vertical" => Some(Self::Vertical),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Horizontal => "horizontal",
            Self::Vertical => "vertical",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorProps {
    pub style: VariantProps,
    pub value: Option<String>,
    pub min_height: u16,
    pub hide_toolbar: bool,
    pub disabled: bool,
    pub readonly: bool,
    pub name: Option<String>,
    pub help_text: Option<String>,
    pub error_text: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageCropperProps {
    pub style: VariantProps,
    pub src: Option<String>,
    pub alt: String,
    pub accept: String,
    pub aspect_ratio: Option<String>,
    pub min_width: u16,
    pub min_height: u16,
    pub max_width: Option<u16>,
    pub max_height: Option<u16>,
    pub shape: ImageCropperShape,
    pub disabled: bool,
    pub name: Option<String>,
    pub help_text: Option<String>,
    pub error_text: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageCropperShape {
    Circle,
    Square,
}

impl ImageCropperShape {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "circle" => Some(Self::Circle),
            "square" => Some(Self::Square),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Circle => "circle",
            Self::Square => "square",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PasswordProps {
    pub style: VariantProps,
    pub value: Option<String>,
    pub hide_strength: bool,
    pub weak_label: String,
    pub medium_label: String,
    pub strong_label: String,
    pub disabled: bool,
    pub readonly: bool,
    pub name: Option<String>,
    pub help_text: Option<String>,
    pub error_text: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhoneProps {
    pub style: VariantProps,
    pub value: Option<String>,
    pub country: Option<String>,
    pub dial_code_name: String,
    pub search_placeholder: String,
    pub empty_text: String,
    pub loading_text: String,
    pub priority_countries: Vec<String>,
    pub disabled: bool,
    pub name: Option<String>,
    pub help_text: Option<String>,
    pub error_text: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PinProps {
    pub style: VariantProps,
    pub value: Option<String>,
    pub length: u8,
    pub kind: PinKind,
    pub name: Option<String>,
    pub help_text: Option<String>,
    pub error_text: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PinKind {
    Text,
    Password,
    Number,
}

impl PinKind {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "text" => Some(Self::Text),
            "password" => Some(Self::Password),
            "number" => Some(Self::Number),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Password => "password",
            Self::Number => "number",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextareaProps {
    pub style: VariantProps,
    pub value: Option<String>,
    pub rows: u16,
    pub cols: Option<u16>,
    pub max_length: Option<u16>,
    pub resize: bool,
    pub disabled: bool,
    pub readonly: bool,
    pub name: Option<String>,
    pub help_text: Option<String>,
    pub error_text: Option<String>,
}
