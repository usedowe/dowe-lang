struct ComposeReactiveRoute {
    constants: String,
    initial: String,
    signals: String,
    actions: String,
    forms: String,
    init: Vec<String>,
    autoload: Vec<String>,
}

#[derive(Clone, Default)]
struct ComposeReactiveContext {
    constants: Vec<(String, String)>,
    signals: Vec<(String, String)>,
    actions: Vec<(String, String)>,
    items: Vec<(String, String)>,
    consumed_props: std::rc::Rc<std::cell::RefCell<dowe_components::PropConsumptionRegistry>>,
}

impl ComposeReactiveContext {
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

fn compose_reactive_context_for_node(
    root: &ViewNode,
    target: &ViewNode,
) -> Option<ComposeReactiveContext> {
    fn find(
        node: &ViewNode,
        target: &ViewNode,
        context: &ComposeReactiveContext,
    ) -> Option<ComposeReactiveContext> {
        if std::ptr::eq(node, target) {
            return Some(context.clone());
        }
        let next = match node {
            ViewNode::Scope {
                constants,
                signals,
                actions,
                ..
            } => context.with_scope(constants, signals, actions),
            ViewNode::Each { item, .. } => context.with_item(item, "row.value".to_string()),
            _ => context.clone(),
        };
        node_child_groups(node)
            .into_iter()
            .flat_map(|group| group.iter())
            .find_map(|child| find(child, target, &next))
    }

    find(root, target, &ComposeReactiveContext::default())
}

fn compose_reactive_route(tree: &ViewNode) -> ComposeReactiveRoute {
    let mut constants = Vec::new();
    let mut signals = Vec::new();
    let mut metadata = Vec::new();
    let mut actions = Vec::new();
    let mut init = Vec::new();
    let mut autoload = Vec::new();
    collect_compose_reactive(
        tree,
        &ComposeReactiveContext::default(),
        &mut constants,
        &mut signals,
        &mut metadata,
        &mut actions,
        &mut init,
        &mut autoload,
    );
    let form_ids = compose_form_signal_ids(tree);
    let forms = collect_view_forms(tree)
        .iter()
        .map(|form| compose_form_value(form, &form_ids))
        .collect::<Vec<_>>();
    ComposeReactiveRoute {
        constants: format!("mapOf<String, Any?>({})", constants.join(", ")),
        initial: format!("mapOf<String, Any?>({})", signals.join(", ")),
        signals: format!("mapOf({})", metadata.join(", ")),
        actions: format!("mapOf({})", actions.join(", ")),
        forms: format!("mapOf({})", forms.join(", ")),
        init,
        autoload,
    }
}

fn compose_form_signal_ids(tree: &ViewNode) -> BTreeMap<String, String> {
    fn collect(node: &ViewNode, output: &mut BTreeMap<String, String>) {
        match node {
            ViewNode::Scope { signals, children, .. } => {
                output.extend(signals.iter().map(|signal| (signal.name.clone(), signal.id.clone())));
                for child in children { collect(child, output); }
            }
            _ => for child in node_child_groups(node).into_iter().flat_map(|group| group.iter()) { collect(child, output); },
        }
    }
    let mut output = BTreeMap::new();
    collect(tree, &mut output);
    output
}

fn compose_form_value(form: &ViewForm, signal_ids: &BTreeMap<String, String>) -> String {
    let signal = signal_ids.get(&form.signal).cloned().unwrap_or_else(|| form.signal.clone());
    let fields = form.fields.iter().map(|field| {
        let rules = field.rules.iter().map(|rule| {
            let argument = match &rule.kind {
                FormValidationRuleKind::Matches(path) => {
                    let (root, suffix) = path.split_once('.').unwrap_or((path.as_str(), ""));
                    let resolved = signal_ids.get(root).cloned().unwrap_or_else(|| root.to_string());
                    let resolved = if suffix.is_empty() { resolved } else { format!("{resolved}.{suffix}") };
                    compose_optional_string(Some(resolved.as_str()))
                }
                _ => compose_optional_string(rule.kind.argument().as_deref()),
            };
            format!("DoweValidationRule(\"{}\", {}, \"{}\")", escape_kotlin(rule.kind.name()), argument, escape_kotlin(&rule.message))
        }).collect::<Vec<_>>().join(", ");
        format!("DoweFormFieldMetadata(\"{}\", \"{}\", listOf({}))", escape_kotlin(&field.path), match field.kind { ViewFormFieldKind::Boolean => "boolean", ViewFormFieldKind::String => "string" }, rules)
    }).collect::<Vec<_>>().join(", ");
    format!("\"{}\" to listOf({})", escape_kotlin(&signal), fields)
}

