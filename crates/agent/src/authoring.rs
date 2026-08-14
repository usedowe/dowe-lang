use crate::error::{AgentError, AgentResult};
use dowe_agent_harness::{ManagedAgentSkill, ManagedAgentSkillFile};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicSkill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub scope: String,
    pub path: String,
    pub resources: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicSkillDocument {
    pub id: String,
    pub name: String,
    pub full: bool,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicSkillResourceDocument {
    pub id: String,
    pub name: String,
    pub path: String,
    pub content: String,
}

pub fn public_skills() -> Vec<PublicSkill> {
    skill_records()
        .iter()
        .map(|record| PublicSkill {
            id: record.id.to_string(),
            name: record.name.to_string(),
            description: record.description.to_string(),
            scope: "dowe-authoring".to_string(),
            path: format!("skill-data/{}/SKILL.md", record.name),
            resources: record
                .resources
                .iter()
                .map(|resource| resource.path.to_string())
                .collect(),
        })
        .collect()
}

pub fn get_public_skill(id: &str, full: bool) -> AgentResult<PublicSkillDocument> {
    let normalized = normalize_skill_id(id)?;
    let record = skill_records()
        .iter()
        .find(|record| record.id == normalized)
        .ok_or_else(|| AgentError::new(format!("unknown public Dowe skill `{id}`")))?;
    let mut content = record.content.trim_end().to_string();
    if full {
        for resource in record.resources {
            content.push_str("\n\n## Resource: ");
            content.push_str(resource.path);
            content.push_str("\n\n");
            content.push_str(resource.content.trim());
        }
        content.push('\n');
    }
    Ok(PublicSkillDocument {
        id: record.id.to_string(),
        name: record.name.to_string(),
        full,
        content,
    })
}

pub fn get_public_skill_resource(id: &str, path: &str) -> AgentResult<PublicSkillResourceDocument> {
    let normalized = normalize_skill_id(id)?;
    let record = skill_records()
        .iter()
        .find(|record| record.id == normalized)
        .ok_or_else(|| AgentError::new(format!("unknown public Dowe skill `{id}`")))?;
    let resource = record
        .resources
        .iter()
        .find(|resource| resource.path == path)
        .ok_or_else(|| {
            AgentError::new(format!(
                "unknown public Dowe skill resource `{path}` for `{}`",
                record.name
            ))
        })?;
    Ok(PublicSkillResourceDocument {
        id: record.id.to_string(),
        name: record.name.to_string(),
        path: resource.path.to_string(),
        content: resource.content.trim_end().to_string(),
    })
}

pub(crate) fn managed_agent_skills() -> Vec<ManagedAgentSkill> {
    skill_records()
        .iter()
        .map(|record| {
            let mut files = vec![ManagedAgentSkillFile {
                path: "SKILL.md".to_string(),
                content: record.content.to_string(),
            }];
            files.extend(
                record
                    .resources
                    .iter()
                    .map(|resource| ManagedAgentSkillFile {
                        path: resource.path.to_string(),
                        content: resource.content.to_string(),
                    }),
            );
            ManagedAgentSkill {
                name: record.name.to_string(),
                files,
            }
        })
        .collect()
}

fn normalize_skill_id(id: &str) -> AgentResult<&str> {
    let id = id.trim();
    if id.is_empty() || id.len() > 64 {
        return Err(AgentError::new(
            "public Dowe skill id must be 1 to 64 bytes",
        ));
    }
    Ok(id.strip_prefix("dowe-").unwrap_or(id))
}

struct SkillRecord {
    id: &'static str,
    name: &'static str,
    description: &'static str,
    content: &'static str,
    resources: &'static [SkillResource],
}

struct SkillResource {
    path: &'static str,
    content: &'static str,
}

fn skill_records() -> &'static [SkillRecord] {
    &[
        SkillRecord {
            id: "core",
            name: "dowe-core",
            description: "Author Dowe project structure, root configuration, and workflows.",
            content: include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../skill-data/dowe-core/SKILL.md"
            )),
            resources: &[
                SkillResource {
                    path: "references/main.md",
                    content: include_str!(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/../../skill-data/dowe-core/references/main.md"
                    )),
                },
                SkillResource {
                    path: "references/workflow.md",
                    content: include_str!(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/../../skill-data/dowe-core/references/workflow.md"
                    )),
                },
                SkillResource {
                    path: "references/standard-library.md",
                    content: include_str!(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/../../skill-data/dowe-core/references/standard-library.md"
                    )),
                },
            ],
        },
        SkillRecord {
            id: "server",
            name: "dowe-server",
            description: "Author Dowe APIs, handlers, middleware, persistence, and server runtime behavior.",
            content: include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../skill-data/dowe-server/SKILL.md"
            )),
            resources: &[
                SkillResource {
                    path: "references/server.md",
                    content: include_str!(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/../../skill-data/dowe-server/references/server.md"
                    )),
                },
                SkillResource {
                    path: "references/data.md",
                    content: include_str!(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/../../skill-data/dowe-server/references/data.md"
                    )),
                },
                SkillResource {
                    path: "references/runtime.md",
                    content: include_str!(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/../../skill-data/dowe-server/references/runtime.md"
                    )),
                },
            ],
        },
        SkillRecord {
            id: "domain-modeling",
            name: "dowe-domain-modeling",
            description: "Turn business descriptions into Dowe modules, entities, workflows, permissions, APIs, seeders, and views.",
            content: include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../skill-data/dowe-domain-modeling/SKILL.md"
            )),
            resources: &[
                SkillResource {
                    path: "references/workflow.md",
                    content: include_str!(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/../../skill-data/dowe-domain-modeling/references/workflow.md"
                    )),
                },
                SkillResource {
                    path: "references/pos.md",
                    content: include_str!(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/../../skill-data/dowe-domain-modeling/references/pos.md"
                    )),
                },
                SkillResource {
                    path: "references/crm.md",
                    content: include_str!(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/../../skill-data/dowe-domain-modeling/references/crm.md"
                    )),
                },
                SkillResource {
                    path: "references/ecommerce.md",
                    content: include_str!(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/../../skill-data/dowe-domain-modeling/references/ecommerce.md"
                    )),
                },
                SkillResource {
                    path: "references/reservations.md",
                    content: include_str!(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/../../skill-data/dowe-domain-modeling/references/reservations.md"
                    )),
                },
            ],
        },
        SkillRecord {
            id: "theme",
            name: "dowe-theme",
            description: "Author semantic Dowe themes and cross-platform visual tokens.",
            content: include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../skill-data/dowe-theme/SKILL.md"
            )),
            resources: &[SkillResource {
                path: "references/theme.md",
                content: include_str!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/../../skill-data/dowe-theme/references/theme.md"
                )),
            }],
        },
        SkillRecord {
            id: "views",
            name: "dowe-views",
            description: "Author Dowe views, routes, layouts, pages, Dowe-native reference-driven UI, visual fidelity without screenshot crops, state, requests, and components.",
            content: include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../skill-data/dowe-views/SKILL.md"
            )),
            resources: &[
                SkillResource {
                    path: "references/views.md",
                    content: include_str!(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/../../skill-data/dowe-views/references/views.md"
                    )),
                },
                SkillResource {
                    path: "references/composition.md",
                    content: include_str!(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/../../skill-data/dowe-views/references/composition.md"
                    )),
                },
                SkillResource {
                    path: "references/blocks/index.json",
                    content: include_str!(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/../../skill-data/dowe-views/references/blocks/index.json"
                    )),
                },
                SkillResource {
                    path: "references/reference-ui.md",
                    content: include_str!(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/../../skill-data/dowe-views/references/reference-ui.md"
                    )),
                },
                SkillResource {
                    path: "references/components.md",
                    content: include_str!(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/../../skill-data/dowe-views/references/components.md"
                    )),
                },
                SkillResource {
                    path: "references/styles.md",
                    content: include_str!(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/../../skill-data/dowe-views/references/styles.md"
                    )),
                },
                SkillResource {
                    path: "references/canvas.md",
                    content: include_str!(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/../../skill-data/dowe-views/references/canvas.md"
                    )),
                },
                SkillResource {
                    path: "scripts/visual_qa.py",
                    content: include_str!(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/../../skill-data/dowe-views/scripts/visual_qa.py"
                    )),
                },
                SkillResource {
                    path: "scripts/visual_qa_blueprint.py",
                    content: include_str!(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/../../skill-data/dowe-views/scripts/visual_qa_blueprint.py"
                    )),
                },
                SkillResource {
                    path: "scripts/visual_qa_png.py",
                    content: include_str!(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/../../skill-data/dowe-views/scripts/visual_qa_png.py"
                    )),
                },
            ],
        },
    ]
}
