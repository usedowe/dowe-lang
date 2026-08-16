use crate::error::{DoweError, DoweResult};
use crate::model::{
    DoweType, DoweTypeField, EnvironmentConfig, TranslationCatalog, ViewNode, ViewPage,
    ViewPlatform, ViewRoute, ViewTargetRoutes, WebOutput,
};
use crate::parser::source_ast::{
    SourceFile, SourceNode, SourceObjectEntry, SourceProp, SourceValue,
};
use crate::parser::source_i18n::validate_view_i18n_keys;
use crate::parser::source_imports::resolve_import;
use crate::parser::source_parser::parse_source_file;
use crate::parser::source_stdlib::parse_stdlib_call;
use crate::parser::source_types::{TypeRegistry, is_shared_type_path, validate_source_value_type};
use crate::parser::source_values::parse_value;
use dowe_components::{
    BuiltinComponent, COMPONENT_REGISTRY, CodeTemplateSegment, ColorFamily, ColorToken,
    ComponentError, ComponentProp, ComponentVariant, DesignComponentSlot, DesignConfig,
    NavigationAction, NavigationOperation, OverlayCornerPosition, PropScalar, PropValue,
    ResponsivePropEntry, StdlibArgument, StdlibCall, StdlibValue, TabsVariant, ToggleGroupKind,
    VIEW_META_NAMES, ViewAction, ViewActionKind, ViewAssignAction, ViewConstant,
    ViewFunctionParameter, ViewFunctionReturn, ViewFunctionStatement, ViewMetadata,
    ViewNavigationAction, ViewRequestAction, ViewRequestHeader, ViewRequestHeaderValue,
    ViewRequestMethod, ViewResetAction, ViewSection, ViewSignal, ViewSignalScope,
    ViewSignalStorage, ViewSignalValue, ViewToastAction, VisibilityCondition,
    accordion_component_node, accordion_item_component, alert_dialog_component_node,
    apply_design_defaults_to_tree, apply_theme_catalog_to_tree, arc_chart_component_node,
    area_chart_component_node, attach_form_validation, audio_component_node, avatar_component_node,
    avatar_group_component_node, avatar_group_item_component, badge_component_node,
    camera_component_node,
    bar_chart_component_node, bar_component_node, bottom_bar_component_node,
    bottom_bar_tab_component, candlestick_node, canvas_component_node, carousel_component_node,
    carousel_slide_component, chat_box_component_node, checkbox_component_node, children_node,
    chip_component_node, code_node, collapsible_component_node, color_component_node,
    combo_box_component_node, combo_option_component, command_component_node,
    command_group_component, compose_tree, container_component_node, countdown_component_node,
    csv_column_component, csv_field_component_node, date_component_node, date_range_component_node,
    device_node, divider_node, drag_drop_component_node, drag_group_component, drag_item_component,
    drawer_component_node, dropdown_component_node, dropzone_component_node, editor_component_node,
    empty_component_node, fab_action_component, fab_component_node, first_text,
    form_validation_rule, icon_component_node, iframe_node, image_component_node,
    image_cropper_component_node, input_node, line_chart_component_node, map_component_node,
    map_marker_component, map_waypoint_component, marquee_component_node, microphone_component_node,
    modal_component_node,
    nav_menu_component_node, nav_menu_item_component, nav_menu_megamenu_component,
    nav_menu_submenu_component, navigation_action, node_child_groups, node_element_props,
    overlay_icon_component, overlay_item_component, password_component_node, phone_component_node,
    pie_chart_component_node, pin_component_node, radio_group_component_node,
    radio_option_component, rail_nav_component_node, rail_nav_item_component,
    record_component_node, rich_text_component_node, rich_text_mark_component,
    scaffold_component_node, select_node_with_each, select_option_component,
    side_nav_component_node, side_nav_header_component, side_nav_icon_component,
    side_nav_item_component, side_nav_submenu_component, sidebar_component_node,
    skeleton_component_node, slider_component_node, stepper_component_node, stepper_step_component,
    svg_component_node, svg_path_component, table_column_component, table_node,
    tabs_component_node, tabs_tab_component, text_binding_path, text_component_node, text_node,
    textarea_component_node, theme_select_component_node, theme_toggle_component_node,
    toast_component_node, toggle_component_node, toggle_group_component_node,
    toggle_group_item_component, tooltip_component_node, type_writer_component_node,
    type_writer_item_component, validate_view_tree, video_node,
};
use dowe_generator_web::{
    build_translation_chunks, render_page_document, router_js,
};
use dowe_stdlib::StdlibSurface;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;

include!("source_views/environment.rs");
include!("source_views/entry.rs");
include!("source_views/model.rs");
include!("source_views/routes.rs");
include!("source_views/imports.rs");
include!("source_views/tree.rs");
include!("source_views/metadata.rs");
include!("source_views/inline_actions.rs");
include!("source_views/functions.rs");
include!("source_views/legacy_requests.rs");
include!("source_views/function_statements.rs");
include!("source_views/lowering.rs");
include!("source_views/special_components.rs");
include!("source_views/component_entries.rs");
include!("source_views/menus.rs");
include!("source_views/navigation_components.rs");
include!("source_views/shell_components.rs");
include!("source_views/component_props.rs");
include!("source_views/known_component_props.rs");
include!("source_views/component_props/layout_and_data.rs");
include!("source_views/component_props/forms_and_actions.rs");
include!("source_views/component_props/shells_and_feedback.rs");
include!("source_views/component_props/display.rs");
include!("source_views/component_props/selection_and_text.rs");
include!("source_views/prop_values.rs");
include!("source_views/navigation_collect.rs");
include!("source_views/reactive_validation.rs");
include!("source_views/node_validation.rs");
include!("source_views/node_variant_validation.rs");
include!("source_views/theme_validation.rs");
include!("source_views/action_validation.rs");
include!("source_views/data_validation.rs");
include!("source_views/path_validation.rs");
include!("source_views/final_helpers.rs");
include!("source_views/inspector.rs");

#[cfg(test)]
mod tests;
