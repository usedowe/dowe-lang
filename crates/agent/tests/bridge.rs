use dowe_agent::{
    get_public_skill, get_public_skill_resource, handle_mcp_message, project_context,
    public_skills, search_public_examples, summarize_codegraph_for,
};
use dowe_agent_harness::{InitOptions, init_project_harness};
use dowe_components::BuiltinComponent;
use serde_json::Value;
use std::fs;
use tempfile::TempDir;

const VIEW_COMPONENT_REFERENCE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/embedded/dowe-views/references/components.md"
));

const VIEW_BLOCK_INDEX: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/embedded/dowe-views/references/blocks/index.json"
));

include!("support/bridge_skill_inventory.rs");
include!("support/bridge_view_contracts.rs");
include!("support/bridge_skill_resources.rs");
include!("support/bridge_mcp.rs");
