use crate::error::{AgentError, AgentResult};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicExampleSearch {
    pub query: String,
    pub terms: Vec<String>,
    pub results: Vec<PublicExampleResult>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicExampleResult {
    pub id: String,
    pub title: String,
    pub description: String,
    pub source_path: String,
    pub skill: String,
    pub tags: Vec<String>,
    pub content: String,
    pub score: usize,
}

pub fn search_public_examples(query: &str, limit: usize) -> AgentResult<PublicExampleSearch> {
    if query.trim().is_empty() || query.len() > 512 {
        return Err(AgentError::new("example query must be 1 to 512 bytes"));
    }
    if !(1..=5).contains(&limit) {
        return Err(AgentError::new(
            "example search limit must be between 1 and 5",
        ));
    }
    let terms = search_terms(query);
    if terms.is_empty() {
        return Err(AgentError::new(
            "example query must contain searchable terms",
        ));
    }
    let mut results = example_records()
        .iter()
        .filter_map(|record| {
            let score = score_example(record, &terms);
            (score > 0).then(|| PublicExampleResult {
                id: record.id.to_string(),
                title: record.title.to_string(),
                description: record.description.to_string(),
                source_path: record.source_path.to_string(),
                skill: record.skill.to_string(),
                tags: record.tags.iter().map(|tag| tag.to_string()).collect(),
                content: record.content.trim().to_string(),
                score,
            })
        })
        .collect::<Vec<_>>();
    results.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.id.cmp(&right.id))
    });
    let mut source_paths = BTreeSet::new();
    results.retain(|result| source_paths.insert(result.source_path.clone()));
    results.truncate(limit);
    Ok(PublicExampleSearch {
        query: query.trim().to_string(),
        terms,
        results,
    })
}

fn search_terms(query: &str) -> Vec<String> {
    query
        .to_ascii_lowercase()
        .split(|character: char| !character.is_ascii_alphanumeric() && character != '-')
        .filter(|term| !term.is_empty())
        .map(str::to_string)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn score_example(record: &ExampleRecord, terms: &[String]) -> usize {
    let id = record.id.to_ascii_lowercase();
    let title = record.title.to_ascii_lowercase();
    let description = record.description.to_ascii_lowercase();
    let source_path = record.source_path.to_ascii_lowercase();
    let content = record.content.to_ascii_lowercase();
    terms
        .iter()
        .map(|term| {
            usize::from(id.contains(term)) * 5
                + usize::from(title.contains(term)) * 5
                + usize::from(description.contains(term)) * 3
                + record
                    .tags
                    .iter()
                    .filter(|tag| tag.contains(term.as_str()))
                    .count()
                    * 8
                + usize::from(source_path.contains(term)) * 2
                + usize::from(content.contains(term))
        })
        .sum()
}

struct ExampleRecord {
    id: &'static str,
    title: &'static str,
    description: &'static str,
    source_path: &'static str,
    skill: &'static str,
    tags: &'static [&'static str],
    content: &'static str,
}

fn example_records() -> &'static [ExampleRecord] {
    &[
        ExampleRecord {
            id: "dashboard-layout",
            title: "Application sidebar layout",
            description: "A Scaffold with AppBar, Sidebar, SideNav, and routed content insertion.",
            source_path: "skill-data/examples/fullstack/views/layouts/app-layout.dowe",
            skill: "views",
            tags: &["dashboard", "layout", "navigation", "sidebar"],
            content: include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../skill-data/examples/fullstack/views/layouts/app-layout.dowe"
            )),
        },
        ExampleRecord {
            id: "dashboard-page",
            title: "Section-based responsive page",
            description: "Ordered Sections with a responsive form and record Grid.",
            source_path: "skill-data/examples/fullstack/views/pages/blogs-page.dowe",
            skill: "views",
            tags: &["card", "dashboard", "grid", "responsive", "section"],
            content: include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../skill-data/examples/fullstack/views/pages/blogs-page.dowe"
            )),
        },
        ExampleRecord {
            id: "form-controls",
            title: "Form and catalog page",
            description: "Signals, form controls, requests, Sections, Cards, and a responsive Grid.",
            source_path: "skill-data/examples/fullstack/views/pages/blogs-page.dowe",
            skill: "views",
            tags: &["button", "form", "input", "request", "textarea"],
            content: include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../skill-data/examples/fullstack/views/pages/blogs-page.dowe"
            )),
        },
        ExampleRecord {
            id: "fullstack-actions",
            title: "Sequential fullstack view functions",
            description: "Signals, request result bindings, state updates, reset, and toast feedback.",
            source_path: "skill-data/examples/fullstack/views/pages/blogs-page.dowe",
            skill: "views",
            tags: &["form", "fullstack", "request", "signal", "toast"],
            content: include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../skill-data/examples/fullstack/views/pages/blogs-page.dowe"
            )),
        },
        ExampleRecord {
            id: "server-routes",
            title: "Grouped server routes",
            description: "Imported handlers and middleware inside an endpoint path group.",
            source_path: "skill-data/examples/fullstack/server/endpoints.dowe",
            skill: "server",
            tags: &["api", "endpoint", "handler", "middleware", "server"],
            content: include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../skill-data/examples/fullstack/server/endpoints.dowe"
            )),
        },
    ]
}
