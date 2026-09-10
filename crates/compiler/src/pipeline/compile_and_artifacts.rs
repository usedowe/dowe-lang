use crate::error::{DoweError, DoweResult};
use crate::model::{
    AppConfig, AppOutput, CompileEnvironment, CompiledProject, EnvironmentConfig, GeneratedFile,
    ViewPlatform, ViewTargetRoutes,
};
use crate::parser::parse_project_for;
use crate::typecheck_artifacts::{obsolete_typecheck_artifacts, typecheck_artifacts};
use dowe_components::{
    DesignConfig, FontConfig, FontFamily, collect_route_font_families, font_catalog,
};
use dowe_generator_android::generate_android_with_app_translations_and_icons;
use dowe_generator_desktop::{
    generate_desktop_with_app, generate_desktop_with_app_for_development,
};
use dowe_generator_ios::generate_ios_with_app_translations_and_icons;
use dowe_generator_web::{
    WebOutput, inspector_manifest, prepare_design_asset, prepare_dev_design_asset,
    prepare_incremental_dev_design_asset, web_artifact_update, web_artifacts_for_target,
};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path, PathBuf};

include!("compile_entrypoints.rs");
include!("output_directories_and_fonts.rs");
include!("app_outputs.rs");
include!("generated_paths_and_manifests.rs");
include!("font_asset_tests.rs");
