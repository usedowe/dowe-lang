use crate::error::DoweResult;
use crate::parser::source_ast::SourceNode;

#[derive(Clone)]
pub(super) struct FlatNode {
    pub(super) level: usize,
    pub(super) node: SourceNode,
}

pub(super) fn parse_block(
    flat_nodes: &[FlatNode],
    index: &mut usize,
    level: usize,
) -> DoweResult<Vec<SourceNode>> {
    let mut nodes = Vec::new();

    while *index < flat_nodes.len() {
        let current = &flat_nodes[*index];
        if current.level < level {
            break;
        }
        if current.level > level {
            return Err(crate::error::DoweError::at_path(
                &current.node.location.path,
                format!(
                    "{}:{}: block is not nested under a parent",
                    current.node.location.line, current.node.location.column
                ),
            ));
        }

        let mut node = current.node.clone();
        *index += 1;
        if *index < flat_nodes.len() {
            let next = &flat_nodes[*index];
            if next.level > level + 1 {
                return Err(crate::error::DoweError::at_path(
                    &next.node.location.path,
                    format!(
                        "{}:{}: indentation can only increase one level at a time",
                        next.node.location.line, next.node.location.column
                    ),
                ));
            }
            if next.level == level + 1 {
                node.children = parse_block(flat_nodes, index, level + 1)?;
            }
        }
        nodes.push(node);
    }

    Ok(nodes)
}
