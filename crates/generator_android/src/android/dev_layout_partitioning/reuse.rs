fn reusable_dev_layouts(routes: &[ViewRoute]) -> (Vec<&ViewNode>, Vec<Option<usize>>) {
    let mut layouts = Vec::new();
    let mut route_layouts = Vec::new();
    for route in routes {
        if !dev_layout_can_split(&route.layout_tree)
            || dev_page_references_layout_bindings(&route.layout_tree, &route.page_tree)
        {
            route_layouts.push(None);
            continue;
        }
        let index = layouts
            .iter()
            .position(|layout| *layout == &route.layout_tree)
            .unwrap_or_else(|| {
                layouts.push(&route.layout_tree);
                layouts.len() - 1
            });
        route_layouts.push(Some(index));
    }
    (layouts, route_layouts)
}

fn dev_page_references_layout_bindings(layout: &ViewNode, page: &ViewNode) -> bool {
    let mut bindings = DevLayoutBindings::default();
    dev_collect_scope_bindings(layout, &mut bindings);
    if bindings.is_empty() {
        return false;
    }
    dev_node_references_layout_bindings(page, &bindings)
}

#[derive(Clone, Default)]
struct DevLayoutBindings {
    signal_names: std::collections::BTreeSet<String>,
    signal_ids: std::collections::BTreeSet<String>,
    action_names: std::collections::BTreeSet<String>,
    action_ids: std::collections::BTreeSet<String>,
}

impl DevLayoutBindings {
    fn is_empty(&self) -> bool {
        self.signal_names.is_empty()
            && self.signal_ids.is_empty()
            && self.action_names.is_empty()
            && self.action_ids.is_empty()
    }

    fn with_scope_shadowed(
        &self,
        constants: &[ViewConstant],
        signals: &[ViewSignal],
        actions: &[ViewAction],
    ) -> Self {
        let mut next = self.clone();
        for constant in constants {
            next.signal_names.remove(&constant.name);
            next.signal_ids.remove(&constant.id);
        }
        for signal in signals {
            next.signal_names.remove(&signal.name);
            next.signal_ids.remove(&signal.id);
        }
        for action in actions {
            next.action_names.remove(&action.name);
            next.action_ids.remove(&action.id);
        }
        next
    }

    fn references_signal(&self, path: &str) -> bool {
        let root = path.split_once('.').map(|(root, _)| root).unwrap_or(path);
        self.signal_names.contains(root) || self.signal_ids.contains(root)
    }

    fn references_action(&self, name: &str) -> bool {
        self.action_names.contains(name) || self.action_ids.contains(name)
    }
}

