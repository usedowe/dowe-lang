use std::{fs, path::Path};

#[test]
fn avatar_icon_prop_has_a_shared_contract() {
    let contract = super::component_prop_contract(super::BuiltinComponent::Avatar, "icon")
        .expect("Avatar icon prop contract");
    assert_eq!(contract.kind, super::PropValueKind::String);
    assert_eq!(contract.validator, super::PropValidator::IconName);
}

use super::{
    AvatarGroupProps, BarPosition, BoxPosition, BrandProps, Breakpoint, BuiltinComponent,
    ButtonSize, COMPONENT_REGISTRY, COUNTRY_FLAGS, CanvasBackground, CanvasFit, CarouselVariant,
    ChartCurve, ChartLegendPosition, ChartPalette, ChartSize, CodeLanguage, CodeTemplateSegment,
    CodeTokenKind, ColorFamily, ColorToken, ComponentError, ComponentProp, ComponentVariant,
    ContainerSize, DeviceProfile, DividerOrientation, EmptyKind, FabProps, FlexDirection, FlexItem,
    FontFamily, GapSize, GapValue, GridAlignment, GridTracks, IframeLoading, NativeExternalMode,
    NavigationAction, OverlayCornerPosition, OverlayPaint, PropValue, RadioGroupOrientation,
    ResponsivePropEntry, ResponsiveValue, RichTextMark, RichTextMarkStyle, RoundedSize,
    SVG_LOGOS, SVG_SPINNERS,
    ScaleValue, SectionBackground, SizeValue, SpacingProps, SvgLineCap, SvgLineJoin, SvgPathFill,
    SvgTransform, TableColumnAlign, TableSize, TabsPosition, TabsVariant, TextAlign, TextSize,
    TextSpacing, TextWeight, VideoAspect, ViewAnimation, ViewGesture, ViewIcon, ViewNode,
    ViewRotation, ViewScale, ViewTransition, ViewTranslation, VisibilityCondition, WebTarget,
    all_icon_names, arc_chart_component_node, area_chart_component_node, bar_chart_component_node,
    bar_component_node, box_node, candlestick_node, canvas_component_node, carousel_component_node,
    carousel_slide_component, children_node, code_node, compose_tree, container_component_node,
    country_flag_icon, device_node, diagram_component_node, divider_node, empty_icon,
    first_text, fixed_box_nodes,
    fixed_fab_nodes, font_catalog, form_control_min_height, form_control_text_size,
    icon_component_node, iframe_node, input_node, integrated_design_theme,
    line_chart_component_node, phone_countries, pie_chart_component_node,
    radio_group_component_node, radio_option_component, rich_text_component_node,
    runtime_icon_catalog, runtime_icon_catalog_for_names, runtime_icon_catalog_shared,
    section_content_spacing, select_node, select_option_component, stepper_component_node,
    stepper_step_component, svg_component_node, svg_path_component, table_column_component,
    table_node, tabs_component_node, tabs_tab_component, text_binding_path, text_component_node,
    text_node, text_spacing_em, text_typography, text_weight_number, validate_solar_icon_catalog,
    validate_svg_logo_catalog, validate_svg_spinner_catalog, validate_view_tree, video_node,
};

include!("tests_registry_catalog.rs");
include!("tests_basic_components.rs");
include!("tests_data_components.rs");
include!("tests_navigation_and_forms.rs");
include!("tests_layout_and_styles.rs");
include!("tests_action_components.rs");
