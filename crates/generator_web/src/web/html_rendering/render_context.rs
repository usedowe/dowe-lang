use dowe_components::{BuiltinComponent, RadioGroupPresentation};

fn text_line_css(value: TextSize) -> &'static str {
    text_typography(false, value).line_height
}

fn title_text_size_css(value: TextSize) -> String {
    fluid_text_size_css(text_typography(true, value).font_size)
}

fn title_text_line_css(value: TextSize) -> &'static str {
    text_typography(true, value).line_height
}

fn title_text_weight_css(value: TextSize) -> &'static str {
    text_weight_number(text_typography(true, value).weight)
}

fn title_text_spacing_css(value: TextSize) -> String {
    format!("{}em", text_typography(true, value).letter_spacing_em)
}

fn text_weight_css(value: TextWeight) -> &'static str {
    text_weight_number(value)
}

fn text_spacing_css(value: TextSpacing) -> String {
    format!("{}em", text_spacing_em(value))
}

fn text_token(family: ColorFamily) -> &'static str {
    family.text_token().as_str()
}

fn title_token(family: ColorFamily) -> &'static str {
    family.title_token().as_str()
}

fn family_color_token(family: ColorFamily) -> &'static str {
    family.color_token().as_str()
}

fn family_text_token(family: ColorFamily) -> &'static str {
    family.text_token().as_str()
}

fn append_responsive_rule(css: &mut String, breakpoint: Breakpoint, class_name: &str, body: &str) {
    css.push_str(&format!(
        ".{}\\:{}{{{body}}}",
        breakpoint.as_str(),
        css_class_name(class_name)
    ));
}

fn append_rule(css: &mut String, class_name: &str, body: &str) {
    css.push_str(&format!(".{}{{{body}}}", css_class_name(class_name)));
}

fn css_class_name(value: &str) -> String {
    value.replace(':', "\\:").replace('.', "\\.")
}

fn page_file_name(page: &ViewPage) -> String {
    let file_name = page.route_path.trim_matches('/').replace('/', "-");
    if file_name.is_empty() {
        "index".to_string()
    } else {
        file_name
    }
}

#[derive(Clone, Default)]
struct ReactiveRenderContext {
    constants: Vec<(String, String)>,
    constant_values: Vec<(String, ViewSignalValue)>,
    scope_values: Vec<(String, ViewSignalValue)>,
    signals: Vec<(String, String)>,
    signal_initial_values: Vec<(String, ViewSignalValue)>,
    actions: Vec<(String, String)>,
    consumed_props: std::rc::Rc<std::cell::RefCell<dowe_components::PropConsumptionRegistry>>,
}

impl ReactiveRenderContext {
    fn register_consumed_prop(
        &self,
        component: BuiltinComponent,
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
}

impl ReactiveRenderContext {
    fn with_scope(
        &self,
        constants: &[ViewConstant],
        signals: &[ViewSignal],
        actions: &[ViewAction],
    ) -> Self {
        let mut context = self.clone();
        context.constants.extend(
            constants
                .iter()
                .map(|constant| (constant.name.clone(), constant.id.clone())),
        );
        context.constant_values.extend(
            constants
                .iter()
                .map(|constant| (constant.name.clone(), constant.value.clone())),
        );
        context.signals.extend(
            signals
                .iter()
                .map(|signal| (signal.name.clone(), signal.id.clone())),
        );
        context.signal_initial_values.extend(
            signals
                .iter()
                .map(|signal| (signal.name.clone(), signal.initial.clone())),
        );
        context.actions.extend(
            actions
                .iter()
                .map(|action| (action.name.clone(), action.id.clone())),
        );
        context
    }

    fn signal_path(&self, value: &str) -> String {
        if let Some(value) = value.strip_prefix('!') {
            return format!("!{}", self.signal_path(value));
        }
        let (root, suffix) = value
            .split_once('.')
            .map(|(root, suffix)| (root, Some(suffix)))
            .unwrap_or((value, None));
        let resolved = self
            .signals
            .iter()
            .rev()
            .chain(self.constants.iter().rev())
            .find(|(name, _)| name == root);
        let Some((_, id)) = resolved else {
            return value.to_string();
        };
        suffix
            .map(|suffix| format!("{id}.{suffix}"))
            .unwrap_or_else(|| id.clone())
    }

    fn action_id(&self, value: &str) -> String {
        self.actions
            .iter()
            .rev()
            .find(|(name, _)| name == value)
            .map(|(_, id)| id.clone())
            .unwrap_or_else(|| value.to_string())
    }

    fn constant_array_values(&self, value: &str) -> Option<&Vec<ViewSignalValue>> {
        if value.contains('.') {
            return None;
        }
        self.constant_values
            .iter()
            .rev()
            .find(|(name, _)| name == value)
            .and_then(|(_, value)| match value {
                ViewSignalValue::Array(values) => Some(values),
                _ => None,
            })
    }

    fn with_scope_value(&self, name: &str, value: &ViewSignalValue) -> Self {
        let mut context = self.clone();
        context.scope_values.push((name.to_string(), value.clone()));
        context
    }

    fn initial_value(&self, value: &str) -> Option<&ViewSignalValue> {
        let (root, suffix) = value
            .split_once('.')
            .map(|(root, suffix)| (root, Some(suffix)))
            .unwrap_or((value, None));
        let initial = self
            .scope_values
            .iter()
            .rev()
            .find(|(name, _)| name == root)
            .or_else(|| {
                self.signal_initial_values
                    .iter()
                    .rev()
                    .find(|(name, _)| name == root)
            })
            .or_else(|| {
                self.constant_values
                    .iter()
                    .rev()
                    .find(|(name, _)| name == root)
            })
            .map(|(_, initial)| initial)?;
        let Some(suffix) = suffix else {
            return Some(initial);
        };
        let mut current = initial;
        for segment in suffix.split('.') {
            let ViewSignalValue::Object(entries) = current else {
                return None;
            };
            let entry = entries.iter().find(|(name, _)| name == segment)?;
            current = &entry.1;
        }
        Some(current)
    }

    fn initial_text(&self, value: &str) -> Option<String> {
        let mut output = String::new();
        for (literal, binding) in text_template_segments(value) {
            output.push_str(&literal);
            if let Some(binding) = binding {
                let value = self.initial_value(&binding)?;
                output.push_str(match value {
                    ViewSignalValue::Null => "null",
                    ViewSignalValue::Bool(value) => {
                        if *value { "true" } else { "false" }
                    }
                    ViewSignalValue::Number(value) | ViewSignalValue::String(value) => value,
                    ViewSignalValue::Array(_) | ViewSignalValue::Object(_) => return None,
                });
            }
        }
        Some(output)
    }
}

