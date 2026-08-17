const DEV_DYNAMIC_ICON_SHARD_LIMIT: usize = 24 * 1024;
const DEV_DYNAMIC_ICON_LITERAL_LIMIT: usize = 8 * 1024;

fn dev_activity_dynamic_icon_runtime() -> &'static str {
    r#"    private String doweDynamicIconPayload(String name, String fallback) {
        String payload = DoweDevDynamicIcons.payload(name);
        return payload == null ? DoweDevDynamicIcons.payload(fallback) : payload;
    }

"#
}

fn dev_dynamic_icon_shards(app_bundle: &str) -> Vec<DevActivityShard> {
    let entries =
        dowe_components::runtime_icon_catalog_shared().expect("validated runtime icon catalog");
    let mut groups = Vec::new();
    let mut group = Vec::new();
    let mut group_size = 0;
    for entry in entries.iter() {
        let payload = dev_java_payload_expression(&entry.1);
        let line = format!(
            "        values.put(\"{}\", {payload});\n",
            escape_java(&entry.0),
        );
        if !group.is_empty() && group_size + line.len() > DEV_DYNAMIC_ICON_SHARD_LIMIT {
            groups.push(group);
            group = Vec::new();
            group_size = 0;
        }
        group_size += line.len();
        group.push(line);
    }
    if !group.is_empty() {
        groups.push(group);
    }

    let mut dispatcher = dev_shard_header(app_bundle);
    dispatcher.push_str(
        "@SuppressWarnings({\"unchecked\", \"deprecation\"})\nfinal class DoweDevDynamicIcons {\n    private static final HashMap<String, String> VALUES = new HashMap<>();\n\n    static {\n",
    );
    for index in 0..groups.len() {
        dispatcher.push_str(&format!(
            "        DoweDevDynamicIcons{index}.add(VALUES);\n"
        ));
    }
    dispatcher.push_str(
        "    }\n\n    private DoweDevDynamicIcons() {}\n\n    static String payload(String name) {\n        return name == null ? null : VALUES.get(name);\n    }\n}\n",
    );
    let mut shards = vec![DevActivityShard {
        file_name: "DoweDevDynamicIcons.java".to_string(),
        content: dispatcher,
    }];
    for (index, lines) in groups.into_iter().enumerate() {
        let class_name = format!("DoweDevDynamicIcons{index}");
        let mut output = dev_shard_header(app_bundle);
        output.push_str(&format!(
            "@SuppressWarnings({{\"unchecked\", \"deprecation\"}})\nfinal class {class_name} {{\n    private {class_name}() {{}}\n\n    private static String joinPayload(String... parts) {{\n        StringBuilder output = new StringBuilder();\n        for (String part : parts) output.append(part);\n        return output.toString();\n    }}\n\n    static void add(HashMap<String, String> values) {{\n"
        ));
        for line in lines {
            output.push_str(&line);
        }
        output.push_str("    }\n}\n");
        shards.push(DevActivityShard {
            file_name: format!("{class_name}.java"),
            content: output,
        });
    }
    shards
}

fn dev_java_payload_expression(payload: &str) -> String {
    let mut chunks = Vec::new();
    let mut current = String::new();
    let mut current_size = 0;
    let escaped_payload = escape_java(payload);
    let mut characters = escaped_payload.chars().peekable();
    while let Some(character) = characters.next() {
        let escaped_pair = character == '\\';
        let pair_tail = escaped_pair.then(|| characters.next()).flatten();
        let character_size = character.len_utf8() + pair_tail.map_or(0, char::len_utf8);
        if !current.is_empty() && current_size + character_size > DEV_DYNAMIC_ICON_LITERAL_LIMIT {
            chunks.push(current);
            current = String::new();
            current_size = 0;
        }
        current.push(character);
        if let Some(pair_tail) = pair_tail {
            current.push(pair_tail);
        }
        current_size += character_size;
    }
    if !current.is_empty() {
        chunks.push(current);
    }

    if chunks.len() == 1 {
        format!("\"{}\"", chunks.pop().expect("payload chunk"))
    } else {
        let mut expression = String::from("joinPayload(");
        for (index, chunk) in chunks.iter().enumerate() {
            if index > 0 {
                expression.push_str(", ");
            }
            expression.push('"');
            expression.push_str(chunk);
            expression.push('"');
        }
        expression.push(')');
        expression
    }
}
