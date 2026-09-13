async fn validate_dowe_project(
    store: &HarnessStore,
    host: &mut impl HarnessHost,
    scope: &str,
    strict_ui: bool,
) -> AgentResult<Value> {
    if scope != "project" && scope != "views" {
        return Err(AgentError::new(
            "validate_dowe_project scope must be project or views",
        ));
    }
    let quality = if strict_ui {
        super::quality::audit_dowe_project_for_ui(store.root())?
    } else {
        super::quality::audit_dowe_project(store.root())?
    };
    let quality_failed = super::quality::quality_failed(&quality);
    let quality_passed = quality["status"] == "passed";
    if scope == "views" {
        let status = if quality_failed {
            "failed"
        } else if quality_passed {
            "passed"
        } else {
            "not_run"
        };
        return Ok(json!({
            "status": status,
            "scope": scope,
            "quality": quality,
            "compiler": {"status": "not_run", "reason": "views scope requested"},
        }));
    }
    let compiler = host.validate_dowe_project(store.root()).await?;
    let compiler_failed = compiler["status"] == "failed";
    let compiler_passed = compiler["status"] == "passed";
    let status = if quality_failed || compiler_failed {
        "failed"
    } else if compiler_passed && quality_passed {
        "passed"
    } else {
        "not_run"
    };
    Ok(json!({
        "status": status,
        "scope": scope,
        "quality": quality,
        "compiler": compiler,
    }))
}
