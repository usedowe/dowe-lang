use super::{validate_view_source, validate_view_store_source, view_declarations};
use crate::model::{
    EnvironmentConfig, EnvironmentValueSource, EnvironmentVariable, EnvironmentVisibility,
};
use crate::parser::source_parser::parse_source_file;
use dowe_components::{
    AvatarSize, AvatarStatus, Breakpoint, ButtonSize, CameraFacing, CanvasBackground, CanvasFit,
    CarouselIndicatorType, CarouselOrientation, CarouselVariant, ChartCurve, ChartLegendPosition,
    ChartPalette, ChartSize, ChatBoxMode, ColorFamily, ColorToken, CommandEntry, ComponentVariant,
    CountdownSize, DividerOrientation, EmptyKind, GapSize, GapValue, ImageAspect, ImageLoading,
    ImageObjectFit, MapMarkerIcon, MarqueeOrientation, MarqueeSpeed, NativeExternalMode,
    NavigationAction, OverlayCornerPosition, OverlayEntry, OverlayPosition, RichTextMarkStyle,
    ScaleValue, SectionBackground, SkeletonAnimation, SkeletonVariant, SvgPathFill,
    TableColumnAlign, TableSize, ToastKind, VideoAspect, ViewActionKind, ViewAnimation,
    ViewFunctionStatement, ViewGesture, ViewIcon, ViewNode, ViewRotation, ViewScale,
    ViewSignalScope, ViewSignalStorage, ViewTransition, ViewTranslation, VisibilityCondition,
    WebTarget,
};
use std::fs;
use std::path::Path;
use tempfile::TempDir;

include!("tests/routes_and_state.rs");
include!("tests/requests_and_actions.rs");
include!("tests/forms_and_reactivity.rs");
include!("tests/types_and_stores.rs");
include!("tests/media_and_charts.rs");
include!("tests/tables_and_bars.rs");
include!("tests/navigation_and_shells.rs");
include!("tests/display_and_motion.rs");
include!("tests/media_and_forms.rs");
include!("tests/invalid_props.rs");
include!("tests/functions.rs");

fn environment() -> EnvironmentConfig {
    EnvironmentConfig {
        variables: vec![EnvironmentVariable {
            name: "BACKEND_URL".to_string(),
            visibility: EnvironmentVisibility::Client,
            resolved_source: EnvironmentValueSource::DotEnv,
            resolved_value: Some(String::new()),
        }],
    }
}
