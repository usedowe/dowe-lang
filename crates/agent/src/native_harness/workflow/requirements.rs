use super::*;
use std::fs;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Resolution {
    schema: u32,
    project: String,
    plan: String,
    answers: BTreeMap<String, String>,
}

pub(super) async fn resolve(
    store: &HarnessStore,
    draft: &WorkflowPlan,
    redactor: &Redactor,
    host: &mut impl HarnessHost,
) -> AgentResult<Option<WorkflowPlan>> {
    let path = store
        .workflow_path(&draft.id)?
        .with_extension("requirements.json");
    for ancestor in path.ancestors() {
        if fs::symlink_metadata(ancestor).is_ok_and(|m| m.file_type().is_symlink()) {
            return Err(AgentError::new(
                "requirements path must not traverse symlinks",
            ));
        }
    }
    let project = digest(store.root().as_os_str().as_encoded_bytes());
    let fingerprint = digest(&serde_json::to_vec(draft)?);
    let mut record = match fs::symlink_metadata(&path) {
        Ok(metadata) => {
            if !metadata.is_file() || metadata.len() > 2 * 1024 * 1024 {
                return Err(AgentError::new(
                    "requirements record must be a bounded regular file",
                ));
            }
            use std::io::Read;
            let mut bytes = Vec::new();
            fs::File::open(&path)?
                .take(2 * 1024 * 1024 + 1)
                .read_to_end(&mut bytes)?;
            if bytes.len() > 2 * 1024 * 1024 {
                return Err(AgentError::new("requirements record grew beyond its limit"));
            }
            let record: Resolution = serde_json::from_slice(&bytes)?;
            if record.schema != 1 || record.project != project || record.plan != fingerprint {
                return Err(AgentError::new(
                    "requirements draft changed or belongs to another project; use a new workflow ID",
                ));
            }
            record
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Resolution {
            schema: 1,
            project,
            plan: fingerprint,
            answers: BTreeMap::new(),
        },
        Err(error) => return Err(error.into()),
    };
    let mut plan = draft.clone();
    let mut validation =
        Coordinator::new(&draft.id, Intent::Build, &draft.objective).map_err(AgentError::new)?;
    for requirement in &draft.requirements {
        validation
            .add_requirement(requirement.clone())
            .map_err(AgentError::new)?;
    }
    for (id, answer) in &record.answers {
        let requirement = plan
            .requirements
            .iter_mut()
            .find(|r| &r.id == id && r.blocking && r.answer.is_none())
            .ok_or_else(|| {
                AgentError::new("saved answer does not match an unanswered blocking requirement")
            })?;
        validate_answer(&mut validation, id, answer, redactor)?;
        requirement.answer = Some(answer.clone());
    }
    let questions = plan
        .requirements
        .iter()
        .filter(|r| r.blocking && r.answer.is_none())
        .map(|r| {
            let question = crate::ClarificationQuestion {
                id: digest(r.id.as_bytes()),
                text: r.question.clone(),
                options: vec![],
            };
            question.validate()?;
            Ok((r.id.clone(), question))
        })
        .collect::<AgentResult<Vec<_>>>()?;
    if !questions.is_empty() {
        save(&path, &record)?;
    }
    for (id, question) in questions {
        host.event(
            &json!({"event":"workflow_requirement_pending","id":draft.id,"requirement":id}),
        )?;
        let Some(answer) = host.ask_clarification(&question).await? else {
            return Ok(None);
        };
        validate_answer(&mut validation, &id, &answer, redactor)?;
        record.answers.insert(id.clone(), answer.clone());
        save(&path, &record)?;
        plan.requirements
            .iter_mut()
            .find(|r| r.id == id)
            .expect("validated requirement")
            .answer = Some(answer);
        host.event(
            &json!({"event":"workflow_requirement_resolved","id":draft.id,"requirement":id}),
        )?;
    }
    Ok(Some(plan))
}

fn validate_answer(
    validation: &mut Coordinator,
    id: &str,
    answer: &str,
    redactor: &Redactor,
) -> AgentResult<()> {
    if redactor.text(answer) != answer {
        return Err(AgentError::new(
            "requirements must not contain secret values; reference a configuration name instead",
        ));
    }
    validation.answer(id, answer).map_err(AgentError::new)
}

fn save(path: &std::path::Path, record: &Resolution) -> AgentResult<()> {
    for ancestor in path.ancestors() {
        if fs::symlink_metadata(ancestor).is_ok_and(|m| m.file_type().is_symlink()) {
            return Err(AgentError::new("requirements path changed to a symlink"));
        }
    }
    if serde_json::to_vec(record)?.len() > 2 * 1024 * 1024 {
        return Err(AgentError::new("requirements record exceeds 2 MiB"));
    }
    crate::auth::write_private_json(path, record)
}
