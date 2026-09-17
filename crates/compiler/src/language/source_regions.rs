use super::LanguageDocument;
use crate::parser::{SourceNode, parse_source_file};
use std::path::Path;

/// Parsed source boundaries for graph retrieval, independent of the editor's
/// deliberately small symbol/selection ranges. Line numbers are one-based.
#[derive(Clone, Debug)]
pub struct SourceRegion {
    pub kind: String,
    pub name: String,
    pub start_line: u32,
    pub end_line: u32,
    pub children: Vec<SourceRegion>,
}

pub fn document_source_regions(root: &Path, document: &LanguageDocument) -> Vec<SourceRegion> {
    parse_source_file(root, &document.path, document.source.clone())
        .map(|file| regions(&file.nodes, document.source.lines().count()))
        .unwrap_or_default()
}

fn regions(nodes: &[SourceNode], parent_end: usize) -> Vec<SourceRegion> {
    nodes
        .iter()
        .enumerate()
        .filter_map(|(index, node)| {
            // Quoted literal children are part of the owning region, not searchable
            // symbols. The parser already consumed multiline strings and headers.
            if !node
                .name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_')
            {
                return None;
            }
            let end = nodes
                .get(index + 1)
                .map_or(parent_end, |next| next.location.line.saturating_sub(1));
            let label = node
                .prop("id")
                .or_else(|| node.prop("name"))
                .or_else(|| node.prop("path"))
                .and_then(|p| p.value.as_string_like())
                .filter(|s| s.len() <= 128 && !s.chars().any(char::is_control));
            Some(SourceRegion {
                kind: node.name.clone(),
                name: label.map_or_else(
                    || format!("{}@{}", node.name, node.location.line),
                    |label| format!("{} {label}", node.name),
                ),
                start_line: node.location.line as u32,
                end_line: end.max(node.location.line) as u32,
                children: regions(&node.children, end),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn regions_include_multiline_values_without_inventing_literal_nodes() {
        let root = Path::new("/project");
        let doc = LanguageDocument {
            path: root.join("page.dowe"),
            source: concat!(
                "page Home\n",
                "  Section id:\"hero\"\n",
                "    Code content:\"\"\"\n",
                "Section id:\"fake\"\n",
                "    \"\"\"\n",
                "  Section id:\"footer\"\n",
                "    Text\n",
                "      \"Footer\"\n"
            )
            .into(),
        };
        let regions = document_source_regions(root, &doc);
        assert_eq!(regions.len(), 1);
        assert_eq!(regions[0].end_line, 8);
        let sections = &regions[0].children;
        assert_eq!(sections.len(), 2);
        assert_eq!((sections[0].start_line, sections[0].end_line), (2, 5));
        assert_eq!(sections[0].children[0].end_line, 5);
        assert!(sections[0].children[0].children.is_empty());
        assert_eq!((sections[1].start_line, sections[1].end_line), (6, 8));
    }
}
