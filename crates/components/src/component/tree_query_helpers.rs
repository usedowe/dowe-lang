pub fn fixed_fab_nodes(node: &ViewNode) -> Vec<&ViewNode> {
    fn collect<'a>(node: &'a ViewNode, fabs: &mut Vec<&'a ViewNode>) {
        if matches!(node, ViewNode::Fab { props, .. } if props.fixed) {
            fabs.push(node);
            return;
        }
        for group in node_child_groups(node) {
            for child in group {
                collect(child, fabs);
            }
        }
    }

    let mut fabs = Vec::new();
    collect(node, &mut fabs);
    fabs
}

pub fn fixed_box_nodes(node: &ViewNode) -> Vec<&ViewNode> {
    fn collect<'a>(node: &'a ViewNode, boxes: &mut Vec<&'a ViewNode>) {
        if matches!(node, ViewNode::Box { props, .. } if props.position().mode == BoxPosition::Fixed)
        {
            boxes.push(node);
            return;
        }
        for group in node_child_groups(node) {
            for child in group {
                collect(child, boxes);
            }
        }
    }

    let mut boxes = Vec::new();
    collect(node, &mut boxes);
    boxes
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StylePropMode {
    Box,
    Banner,
    Section,
    Layout,
    Grid,
    Card,
    Variant,
    Text,
}
