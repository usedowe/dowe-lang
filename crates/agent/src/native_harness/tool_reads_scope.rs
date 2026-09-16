fn parse_codegraph_scope(value: &str) -> AgentResult<Option<dowe_codegraph::clean::Namespace>> {
    match value.trim().to_ascii_lowercase().as_str() {
        "" | "all" => Ok(None),
        "frontend" => Ok(Some(dowe_codegraph::clean::Namespace::Frontend)),
        "backend" => Ok(Some(dowe_codegraph::clean::Namespace::Backend)),
        "shared" => Ok(Some(dowe_codegraph::clean::Namespace::Shared)),
        "product" => Ok(Some(dowe_codegraph::clean::Namespace::Product)),
        "design" => Ok(Some(dowe_codegraph::clean::Namespace::Design)),
        "asset" => Ok(Some(dowe_codegraph::clean::Namespace::Asset)),
        "i18n" => Ok(Some(dowe_codegraph::clean::Namespace::I18n)),
        "contract" => Ok(Some(dowe_codegraph::clean::Namespace::Contract)),
        "verification" => Ok(Some(dowe_codegraph::clean::Namespace::Verification)),
        other => Err(AgentError::new(format!("unknown CodeGraph scope `{other}`"))),
    }
}
