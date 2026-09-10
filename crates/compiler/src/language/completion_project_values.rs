fn project_component_value_completions(
    root: &Path,
    component: BuiltinComponent,
    prop: &str,
) -> Option<Vec<LanguageCompletion>> {
    let mut completions = component_value_completions(component, prop)?;
    if prop == "scheme" {
        completions.extend(custom_color_family_completions(root));
    }
    Some(completions)
}

fn custom_color_family_completions(root: &Path) -> Vec<LanguageCompletion> {
    let path = root.join("theme.dowe");
    let Ok(source) = fs::read_to_string(&path) else {
        return Vec::new();
    };
    let Ok(file) = parse_source_file(root, &path, source) else {
        return Vec::new();
    };
    let mut families = BTreeSet::new();
    for node in &file.nodes {
        collect_custom_color_families(node, &mut families);
    }
    quoted_values(families)
}

fn collect_custom_color_families(node: &SourceNode, families: &mut BTreeSet<String>) {
    if node.name == "colors" {
        for child in &node.children {
            if let Some((family, false)) = ColorFamily::from_theme_name(&child.name)
                && !family.is_builtin()
            {
                families.insert(family.as_str().to_string());
            }
        }
    }
    for child in &node.children {
        collect_custom_color_families(child, families);
    }
}

fn solid_values() -> Vec<LanguageCompletion> {
    quoted_values(["solid"])
}

fn control_size_values() -> Vec<LanguageCompletion> {
    quoted_values(["sm", "md", "lg"])
}

fn column_value_completions(prop: &str) -> Option<Vec<LanguageCompletion>> {
    if !COLUMN_PROPS.contains(&prop) {
        return None;
    }
    match prop {
        "align" => Some(quoted_values(
            TableColumnAlign::all().iter().map(|value| value.as_str()),
        )),
        _ => None,
    }
}

fn quoted_values<I>(values: I) -> Vec<LanguageCompletion>
where
    I: IntoIterator,
    I::Item: AsRef<str>,
{
    values
        .into_iter()
        .map(|value| {
            let value = value.as_ref();
            completion(
                &format!("\"{value}\""),
                LanguageCompletionKind::Value,
                "quoted static value",
            )
        })
        .collect()
}

fn boolean_values() -> Vec<LanguageCompletion> {
    ["true", "false"]
        .into_iter()
        .map(|value| completion(value, LanguageCompletionKind::Value, "static boolean value"))
        .collect()
}

