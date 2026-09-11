#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ReadArgs {
    path: String,
    #[serde(default = "first_line")]
    offset: usize,
    #[serde(default = "page_size")]
    limit: usize,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ListArgs {
    path: String,
    #[serde(default)]
    offset: usize,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SearchArgs {
    path: String,
    query: String,
    #[serde(default = "first_line")]
    offset: usize,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SvgConversionArgs {
    path: String,
    #[serde(default = "original_svg_colors")]
    colors: String,
    #[serde(default = "source_svg_format")]
    format: String,
}
#[derive(Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum SkillSource {
    #[default]
    Embedded,
    Project,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SkillArgs {
    id: String,
    #[serde(default)]
    resource: Option<String>,
    #[serde(default = "first_line")]
    offset: usize,
    #[serde(default)]
    source: SkillSource,
    #[serde(default)]
    hash: Option<String>,
}
fn first_line() -> usize {
    1
}
fn page_size() -> usize {
    200
}
fn original_svg_colors() -> String {
    "original".to_string()
}
fn source_svg_format() -> String {
    "source".to_string()
}

// Skills are reference material, so a single page should leave room for the
// agent to act on it. The general tool-output bound remains configurable, but
// skill pages use this smaller cap to avoid flooding the next request.
const MAX_SKILL_PAGE_BYTES: usize = 8192;

fn normalize_embedded_skill_request(
    id: &str,
    resource: Option<&str>,
) -> AgentResult<(String, Option<String>)> {
    let id = id.trim();
    if id.starts_with("references/") {
        let matches = crate::public_skills()
            .into_iter()
            .filter(|skill| skill.resources.iter().any(|path| path == id))
            .collect::<Vec<_>>();
        return match (matches.as_slice(), resource) {
            ([skill], None) => Ok((skill.id.clone(), Some(id.to_string()))),
            ([skill], Some(bundle)) => {
                let normalized_bundle =
                    bundle.trim().strip_prefix("dowe-").unwrap_or(bundle.trim());
                if normalized_bundle != skill.id {
                    return Err(AgentError::new(format!(
                        "embedded skill resource `{id}` does not match bundle `{bundle}`; expected `{}`",
                        skill.id
                    )));
                }
                Ok((skill.id.clone(), Some(id.to_string())))
            }
            ([], _) => Err(AgentError::new(format!(
                "unknown embedded skill resource `{id}`"
            ))),
            _ => Err(AgentError::new(format!(
                "embedded skill resource `{id}` is ambiguous; specify the skill id"
            ))),
        };
    }
    let normalized_id = id.strip_prefix("dowe-").unwrap_or(id).to_string();
    let Some(resource) = resource else {
        return Ok((normalized_id, None));
    };
    let resource = resource.trim();
    if resource.starts_with("references/")
        && resource
            .strip_prefix("references/")
            .and_then(|name| name.strip_suffix(".md"))
            .is_some()
    {
        let matches = crate::public_skills()
            .into_iter()
            .filter(|skill| skill.resources.iter().any(|path| path == resource))
            .collect::<Vec<_>>();
        let skill = match matches.as_slice() {
            [skill] => skill,
            [] => {
                return Ok((normalized_id, Some(resource.to_string())));
            }
            _ => {
                return Err(AgentError::new(format!(
                    "embedded skill resource `{resource}` is ambiguous; specify the skill id"
                )));
            }
        };
        let basename = resource
            .strip_prefix("references/")
            .and_then(|name| name.strip_suffix(".md"))
            .expect("declared reference resources have a Markdown basename");
        let normalized_basename = basename.replace('_', "-");
        let basename_is_unambiguous = crate::public_skills()
            .into_iter()
            .flat_map(|skill| skill.resources)
            .filter_map(|path| {
                path.strip_prefix("references/")
                    .and_then(|name| name.strip_suffix(".md"))
                    .map(|name| name.replace('_', "-"))
            })
            .filter(|name| name == &normalized_basename)
            .count()
            == 1;
        if normalized_id != skill.id
            && normalized_id != basename
            && (!basename_is_unambiguous || normalized_id.replace('_', "-") != normalized_basename)
        {
            return Err(AgentError::new(format!(
                "embedded skill resource `{resource}` does not match skill id `{id}`; expected `{}` or `{basename}`",
                skill.id
            )));
        }
        return Ok((skill.id.clone(), Some(resource.to_string())));
    }
    if let Some(alias_id) = resource.strip_prefix("bundles/") {
        let is_exact_alias =
            alias_id == normalized_id || alias_id.replace('/', "-") == normalized_id;
        let is_parent_alias = normalized_id
            .strip_prefix(alias_id)
            .is_some_and(|suffix| suffix.starts_with('/'));
        if is_exact_alias || is_parent_alias {
            return Ok((normalized_id, None));
        }
        return Err(AgentError::new(format!(
            "legacy skill bundle alias `{resource}` does not match logical skill id `{normalized_id}`"
        )));
    }
    if resource.contains('/') || resource != normalized_id {
        return Ok((normalized_id, Some(resource.to_string())));
    }
    let shorthand = match normalized_id.as_str() {
        "core" => "references/main.md".to_string(),
        bundle => {
            let skill = crate::public_skills()
                .into_iter()
                .find(|skill| skill.id == bundle)
                .ok_or_else(|| unknown_embedded_skill(id))?;
            let matches = skill
                .resources
                .into_iter()
                .filter(|path| {
                    path.rsplit('/')
                        .next()
                        .is_some_and(|name| name.strip_suffix(".md") == Some(bundle))
                })
                .collect::<Vec<_>>();
            if matches.len() != 1 {
                return Err(AgentError::new(format!(
                    "resource shorthand `{resource}` is not unambiguous for `{id}`"
                )));
            }
            matches.into_iter().next().unwrap()
        }
    };
    Ok((normalized_id, Some(shorthand)))
}

fn unknown_embedded_skill(id: &str) -> AgentError {
    AgentError::new(format!(
        "unknown public Dowe skill `{id}`; use an exact logical id such as core, theme, views, server, domain-modeling, native-ipc, or one of the suggested scoped units"
    ))
}

