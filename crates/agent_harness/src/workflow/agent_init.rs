use crate::model::{InitOptions, InitReport};
use crate::paths::create_agent_dir;
use crate::templates::{default_manifest, project_agents_markdown, tdd_harness_markdown};

pub fn detect_mode(root: impl AsRef<Path>) -> HarnessResult<DetectedMode> {
    let root = root.as_ref();
    let dowe_mode = root.join("agents/README.md").exists() && root.join("AGENTS.md").exists();
    let project_mode = root.join(".agents/manifest.json").exists();

    match (dowe_mode, project_mode) {
        (true, true) => Err(HarnessError::new(
            "both Dowe and project harness markers exist; select a mode explicitly",
        )),
        (true, false) => Ok(DetectedMode::Dowe),
        (false, true) => Ok(DetectedMode::Project),
        (false, false) => Ok(DetectedMode::Unknown),
    }
}

pub fn init_project_harness(
    root: impl AsRef<Path>,
    options: InitOptions,
) -> HarnessResult<InitReport> {
    let root = root.as_ref();
    if detect_mode(root)? == DetectedMode::Dowe {
        return Err(HarnessError::new(
            "Dowe mode uses /agents; project harness init writes only .agents",
        ));
    }

    let mut report = InitReport::new();
    record_outcome(
        &mut report,
        write_agent_file(
            root,
            Path::new("AGENTS.md"),
            &project_agents_markdown(),
            write_mode(options),
        )?,
    );
    record_outcome(
        &mut report,
        write_agent_file(
            root,
            Path::new("manifest.json"),
            &json(&default_manifest())?,
            write_mode(options),
        )?,
    );
    record_outcome(
        &mut report,
        write_agent_file(
            root,
            Path::new("harnesses/tdd.md"),
            &tdd_harness_markdown(),
            write_mode(options),
        )?,
    );
    record_outcome(&mut report, create_agent_dir(root, Path::new("plans"))?);

    Ok(report)
}
