use crate::language::analysis::{
    document_workspace_root, environment_config, normalize_path, reference_fields, signal_fields,
};
use crate::language::documentation::{
    VIEW_COMPONENTS, component_documentation, component_prop_documentation, server_documentation,
    server_names, server_owner_prop_documentation, server_props, stdlib_documentation,
};
use crate::language::model::{LanguageCompletion, LanguageCompletionKind, LanguageDocument};
use crate::parser::{SourceNode, SourceValue, parse_source_file};
use dowe_components::{
    AlertKind, Align, AvatarSize, AvatarStatus, BarPosition, BoxPosition, BuiltinComponent,
    ButtonSize, CameraFacing, CarouselIndicatorType, CarouselOrientation, CarouselVariant,
    ChartCurve, ChartLegendPosition, ChartPalette, ChartSize, ChatBoxMode, CodeLanguage,
    ColorFamily, ColorToken, ComponentVariant, ContainerSize, CountdownSize, DividerOrientation,
    DrawerPosition, EmptyKind, FlexDirection, FlexItem, FontFamily, GridAlignment, ImageAspect,
    ImageLoading, ImageObjectFit, Justify, MarqueeOrientation, MarqueeSpeed, NativeExternalMode,
    NavigationOperation, OverlayCornerPosition, OverlayPosition, RoundedSize, SectionBackground,
    ShadowSize, SideNavSize, SkeletonAnimation, SkeletonVariant, TableColumnAlign, TableSize,
    TabsPosition, TabsVariant, TextAlign, TextSize, TextSpacing, TextWeight, ToastKind,
    VIEW_META_NAMES, VideoAspect, ViewAnimation, ViewGesture, ViewIcon, ViewTransition, WebTarget,
};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};


include!("completion_document.rs");
include!("completion_context.rs");
include!("completion_sources.rs");
include!("completion_component_values.rs");
include!("completion_visual_values.rs");
include!("completion_common_values.rs");
include!("completion_project_values.rs");
include!("completion_props.rs");
include!("completion_layout_props.rs");
include!("completion_display_props.rs");
include!("completion_media_props.rs");
include!("completion_action_props.rs");
include!("completion_editor_props.rs");
include!("completion_data_props.rs");
include!("completion_text_props.rs");
