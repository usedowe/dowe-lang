use crate::error::{DoweError, DoweResult};
use crate::model::{
    AppConfig, CompileEnvironment, CorsConfig, EnvironmentConfig, ProjectServerConfig,
    normalize_cors_method, normalize_cors_origin, normalize_http_header_name,
};
use crate::parser::source_ast::{SourceFile, SourceNode, SourceProp, SourceValue};
use crate::parser::source_environment::parse_environment_files_for;
use crate::parser::source_parser::parse_source_file;
use dowe_components::{
    BorderWidth, ButtonSize, ColorFamily, ColorToken, ComponentVariant, DesignComponentSlot,
    DesignConfig, DesignDefaults, DesignTheme, FontConfig, FontFamily, RoundedSize, ShadowSize,
    TabsVariant, integrated_design_theme,
};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fs;
use std::path::Path;

pub struct ParsedConfig {
    pub app_config: AppConfig,
    pub font_config: FontConfig,
    pub design_config: DesignConfig,
    pub environment_config: EnvironmentConfig,
}

#[derive(Clone)]
struct RawCors {
    node: SourceNode,
    target: CorsTarget,
    config: CorsConfig,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CorsTarget {
    Server,
    Desktop,
    All,
}

#[derive(Clone)]
struct RawTheme {
    node: SourceNode,
    name: String,
    extends: Option<String>,
    colors: BTreeMap<ColorToken, String>,
}

pub(crate) fn parse_project_config_for(
    root: &Path,
    environment: CompileEnvironment,
    compile_views: bool,
) -> DoweResult<ParsedConfig> {
    let environment_config = parse_environment_files_for(root, environment)?;
    if !compile_views {
        return Ok(ParsedConfig {
            app_config: AppConfig::default(),
            font_config: FontConfig::default(),
            design_config: DesignConfig::default(),
            environment_config,
        });
    }

    let json_path = root.join("dowe.json");
    if json_path.exists() {
        return Err(DoweError::at_path(
            &json_path,
            "`dowe.json` is no longer supported; use root `theme.dowe`, `.env.example`, `.env`, and `main.dowe`",
        ));
    }

    let config_path = root.join("src/config.dowe");
    if config_path.exists() {
        return Err(DoweError::at_path(
            &config_path,
            "`src/config.dowe` has been replaced by root `theme.dowe`, dotenv files, and server CORS in `main.dowe`",
        ));
    }

    reject_legacy_root_file(root, "theme.dowe")?;

    let theme_path = root.join("theme.dowe");
    let (app_config, font_config, design_config) = if theme_path.exists() {
        let source = fs::read_to_string(&theme_path)
            .map_err(|error| DoweError::at_path(&theme_path, error.to_string()))?;
        let file = parse_source_file(root, &theme_path, source)?;
        parse_theme_file(&file)?
    } else {
        (
            AppConfig::default(),
            FontConfig::default(),
            DesignConfig::default(),
        )
    };

    Ok(ParsedConfig {
        app_config,
        font_config,
        design_config,
        environment_config,
    })
}

include!("source_config_server_and_cors.rs");
include!("source_config_design.rs");
include!("source_config_themes.rs");
include!("source_config_validation.rs");
