struct SwiftReactiveRoute {
    constants: String,
    initial: String,
    signals: String,
    actions: String,
    forms: String,
    init: Vec<String>,
    autoload: Vec<String>,
}

#[derive(Clone, Default)]
struct SwiftReactiveContext {
    constants: Vec<(String, String)>,
    signals: Vec<(String, String)>,
    actions: Vec<(String, String)>,
    items: Vec<(String, String)>,
    children_expression: Option<String>,
    node_expressions: BTreeMap<usize, String>,
    consumed_props: std::rc::Rc<std::cell::RefCell<dowe_components::PropConsumptionRegistry>>,
}

impl SwiftReactiveContext {
    fn register_consumed_prop(
        &self,
        component: dowe_components::BuiltinComponent,
        prop: &'static str,
        ir_field: &'static str,
    ) {
        dowe_components::register_consumed_prop(
            &mut self.consumed_props.borrow_mut(),
            component,
            prop,
            ir_field,
        );
    }

    fn with_scope(
        &self,
        constants: &[ViewConstant],
        signals: &[ViewSignal],
        actions: &[ViewAction],
    ) -> Self {
        let mut next = self.clone();
        next.constants.extend(
            constants
                .iter()
                .map(|constant| (constant.name.clone(), constant.id.clone())),
        );
        next.signals.extend(
            signals
                .iter()
                .map(|signal| (signal.name.clone(), signal.id.clone())),
        );
        next.actions.extend(
            actions
                .iter()
                .map(|action| (action.name.clone(), action.id.clone())),
        );
        next
    }

    fn with_item(&self, name: &str, value: String) -> Self {
        let mut next = self.clone();
        next.items.push((name.to_string(), value));
        next
    }

    fn with_children_expression(&self, expression: &str) -> Self {
        let mut next = self.clone();
        next.children_expression = Some(expression.to_string());
        next
    }

    fn with_node_expressions(&self, expressions: BTreeMap<usize, String>) -> Self {
        let mut next = self.clone();
        next.node_expressions = expressions;
        next
    }

    fn without_node_expression(&self, node: &ViewNode) -> Self {
        let mut next = self.clone();
        next.node_expressions.remove(&swift_node_key(node));
        next
    }

    fn node_expression(&self, node: &ViewNode) -> Option<&str> {
        self.node_expressions
            .get(&swift_node_key(node))
            .map(String::as_str)
    }

    fn signal_path(&self, path: &str) -> String {
        if let Some(path) = path.strip_prefix('!') {
            return format!("!{}", self.signal_path(path));
        }
        let (root, suffix) = path
            .split_once('.')
            .map(|(root, suffix)| (root, format!(".{suffix}")))
            .unwrap_or((path, String::new()));
        self.signals
            .iter()
            .rev()
            .chain(self.constants.iter().rev())
            .find(|(name, _)| name == root)
            .map(|(_, id)| format!("{id}{suffix}"))
            .unwrap_or_else(|| path.to_string())
    }

    fn action_id(&self, name: &str) -> Option<&str> {
        self.actions
            .iter()
            .rev()
            .find(|(action_name, _)| action_name == name)
            .map(|(_, id)| id.as_str())
    }

    fn item_value(&self, path: &str) -> Option<&str> {
        let root = path.split('.').next().unwrap_or(path);
        self.items
            .iter()
            .rev()
            .find(|(name, _)| name == root)
            .map(|(_, value)| value.as_str())
    }

    fn item_path(&self, path: &str) -> Option<String> {
        let (root, suffix) = path
            .split_once('.')
            .map(|(root, suffix)| (root, Some(suffix)))
            .unwrap_or((path, None));
        self.items
            .iter()
            .rev()
            .find(|(name, _)| name == root)
            .map(|_| {
                suffix
                    .map(|suffix| format!("item.{suffix}"))
                    .unwrap_or_else(|| "item".to_string())
            })
    }

    fn active_item(&self) -> Option<&str> {
        self.items.last().map(|(_, value)| value.as_str())
    }

    fn dynamic_path(&self, path: &str) -> Option<String> {
        if let Some(path) = self.item_path(path) {
            return Some(path);
        }
        let resolved = self.signal_path(path);
        (resolved != path).then_some(resolved)
    }
}

