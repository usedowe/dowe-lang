use crate::model::{InitOptions, InitReport, ManagedAgentSkill};
use crate::paths::{
    create_agent_dir, validate_agent_file_path, validate_root_instruction_files,
    write_root_instruction_file,
};
use crate::templates::{
    project_agents_markdown, root_agents_markdown, root_claude_markdown, tdd_harness_markdown,
};
use std::collections::BTreeSet;

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
    init_project_harness_with_skills(root.as_ref(), options, &[])
}

fn init_project_harness_with_skills(
    root: &Path,
    options: InitOptions,
    managed_skills: &[String],
) -> HarnessResult<InitReport> {
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
            &json(&default_manifest(managed_skills.to_vec()))?,
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

pub fn init_agent_project(
    root: impl AsRef<Path>,
    options: InitOptions,
) -> HarnessResult<InitReport> {
    init_agent_project_with_skills(root, options, &[])
}

pub fn init_agent_project_with_skills(
    root: impl AsRef<Path>,
    options: InitOptions,
    managed_skills: &[ManagedAgentSkill],
) -> HarnessResult<InitReport> {
    let root = root.as_ref();
    validate_root_instruction_files(root)?;
    validate_managed_skills(root, managed_skills)?;
    let mut skill_names = managed_skills
        .iter()
        .map(|skill| skill.name.clone())
        .collect::<Vec<_>>();
    skill_names.sort();
    if options.update_existing {
        reset_managed_skill_directories(root, &skill_names)?;
    }
    let mut report = init_project_harness_with_skills(root, options, &skill_names)?;
    for skill in managed_skills {
        for file in &skill.files {
            let path = Path::new("skills").join(&skill.name).join(&file.path);
            record_outcome(
                &mut report,
                write_agent_file(root, &path, &file.content, write_mode(options))?,
            );
        }
    }
    record_outcome(
        &mut report,
        write_root_instruction_file(
            root,
            "AGENTS.md",
            &root_agents_markdown(),
            write_mode(options),
        )?,
    );
    record_outcome(
        &mut report,
        write_root_instruction_file(
            root,
            "CLAUDE.md",
            &root_claude_markdown(),
            write_mode(options),
        )?,
    );
    Ok(report)
}

fn reset_managed_skill_directories(root: &Path, current: &[String]) -> HarnessResult<()> {
    let mut names = current.iter().cloned().collect::<BTreeSet<_>>();
    let manifest_path = root.join(".agents/manifest.json");
    if manifest_path.is_file() {
        names.extend(read_manifest(root)?.managed_skills);
    }
    for name in names {
        if !valid_managed_skill_name(&name) {
            return Err(HarnessError::new(format!(
                "invalid managed skill name `{name}` in manifest"
            )));
        }
        let relative = Path::new("skills").join(&name);
        validate_agent_file_path(root, &relative)?;
        let path = root.join(".agents").join(relative);
        if path.exists() {
            fs::remove_dir_all(&path)
                .map_err(|error| HarnessError::at_path(&path, error.to_string()))?;
        }
    }
    Ok(())
}

fn validate_managed_skills(root: &Path, managed_skills: &[ManagedAgentSkill]) -> HarnessResult<()> {
    let mut names = BTreeSet::new();
    for skill in managed_skills {
        if !valid_managed_skill_name(&skill.name) || !names.insert(skill.name.as_str()) {
            return Err(HarnessError::new(format!(
                "invalid or duplicate managed skill name `{}`",
                skill.name
            )));
        }
        let mut paths = BTreeSet::new();
        for file in &skill.files {
            if file.path.is_empty() || !paths.insert(file.path.as_str()) {
                return Err(HarnessError::new(format!(
                    "invalid or duplicate managed skill path `{}`",
                    file.path
                )));
            }
            let path = Path::new("skills").join(&skill.name).join(&file.path);
            validate_agent_file_path(root, &path).map_err(|error| {
                HarnessError::new(format!("invalid managed skill path `{}`: {error}", file.path))
            })?;
        }
        if !paths.contains("SKILL.md") {
            return Err(HarnessError::new(format!(
                "managed skill `{}` is missing SKILL.md",
                skill.name
            )));
        }
    }
    Ok(())
}

fn valid_managed_skill_name(name: &str) -> bool {
    name.starts_with("dowe-")
        && name.len() > 5
        && name
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}
