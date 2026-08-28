use std::collections::BTreeSet;
use std::fmt::{Display, Formatter};

pub type ComponentResult<T> = Result<T, ComponentError>;

include!("model/view_nodes.rs");
include!("model/state_and_actions.rs");
include!("model/routing.rs");
include!("model/style_props.rs");
include!("model/text_and_bars.rs");
include!("model/navigation_props.rs");
include!("model/overlay_props.rs");
include!("model/rich_content_props.rs");
include!("model/form_props.rs");
include!("model/navigation_entries.rs");
include!("model/component_variants.rs");
include!("model/navigation_items.rs");
include!("model/select_options.rs");
include!("model/code_props.rs");
include!("model/media_props.rs");
include!("model/chart_props.rs");
include!("model/diagram_props.rs");

include!("model/table_props.rs");
include!("model/media_variants.rs");
include!("model/code_model.rs");
include!("model/visibility_and_svg.rs");
include!("model/feedback_variants.rs");
include!("model/responsive_props.rs");
include!("model/style_bindings.rs");
include!("model/variant_bindings.rs");
include!("model/builtin_component.rs");
include!("prop_contracts.rs");
include!("ir_field.rs");
include!("view_contract.rs");
include!("prop_consumption.rs");
include!("render_report.rs");
