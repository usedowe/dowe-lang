use crate::error::{CodeGraphError, CodeGraphResult};
use crate::model::Waiver;
use crate::paths::{discover_files, slash_path};
use std::fs;
use std::path::Path;

pub(crate) fn collect_waivers(root: &Path) -> CodeGraphResult<Vec<Waiver>> {
    let files = discover_files(root, crate::model::CodeGraphMode::Project)?;
    let mut waivers = Vec::new();

    for file in files {
        let relative = file.strip_prefix(root).unwrap_or(&file);
        let relative = slash_path(relative);
        if !relative.starts_with("specs/") || !relative.ends_with(".md") {
            continue;
        }
        let content = fs::read_to_string(&file)
            .map_err(|error| CodeGraphError::at_path(&file, error.to_string()))?;
        waivers.extend(parse_waivers(&relative, &content));
    }

    Ok(waivers)
}

pub(crate) fn waiver_for<'a>(
    waivers: &'a [Waiver],
    path: &str,
    limit: usize,
) -> Option<&'a Waiver> {
    waivers
        .iter()
        .find(|waiver| waiver.path == path && waiver.limit <= limit && is_complete(waiver))
}

fn parse_waivers(spec_path: &str, content: &str) -> Vec<Waiver> {
    content
        .lines()
        .filter(|line| line.trim_start().starts_with('|'))
        .filter_map(|line| parse_table_row(spec_path, line))
        .collect()
}

fn parse_table_row(spec_path: &str, line: &str) -> Option<Waiver> {
    let cells = line
        .trim()
        .trim_matches('|')
        .split('|')
        .map(str::trim)
        .collect::<Vec<_>>();

    if cells.len() < 6 || cells[0] == "path" || cells[0].starts_with("---") {
        return None;
    }
    if !cells[0].ends_with(".rs") {
        return None;
    }

    Some(Waiver {
        path: cells[0].to_string(),
        limit: cells[1].parse().unwrap_or(0),
        reason: cells[2].to_string(),
        spec: if cells[3].is_empty() {
            spec_path.to_string()
        } else {
            cells[3].to_string()
        },
        expires: cells[4].to_string(),
        owner: cells[5].to_string(),
    })
}

fn is_complete(waiver: &Waiver) -> bool {
    waiver.limit > 0
        && !waiver.reason.is_empty()
        && !waiver.spec.is_empty()
        && !waiver.expires.is_empty()
        && !waiver.owner.is_empty()
}
