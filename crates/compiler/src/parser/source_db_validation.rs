fn validate_database_name(node: &SourceNode, value: &str, label: &str) -> DoweResult<()> {
    if value.is_empty()
        || matches!(value, "." | "..")
        || value.contains('/')
        || value.contains('\\')
        || value.chars().any(char::is_control)
        || !value
            .chars()
            .all(|value| value.is_ascii_alphanumeric() || matches!(value, '_' | '-'))
    {
        return Err(node_error(node, format!("invalid {label} name `{value}`")));
    }
    Ok(())
}

fn required_database_provider(node: &SourceNode) -> DoweResult<DatabaseProvider> {
    let prop = node
        .prop("provider")
        .ok_or_else(|| node_error(node, "database handle must declare `provider`"))?;
    match &prop.value {
        SourceValue::String(value) if value == "postgres" => Ok(DatabaseProvider::Postgres),
        SourceValue::String(value) if value == "d1" => Ok(DatabaseProvider::D1),
        SourceValue::String(value) if value == "dowe" => Ok(DatabaseProvider::Dowe),
        _ => Err(node_error(
            node,
            "database `provider` must be \"postgres\", \"d1\", or \"dowe\"",
        )),
    }
}

fn validate_provider_props(
    node: &SourceNode,
    provider: DatabaseProvider,
    host: Option<&StoreConnectionValue>,
    port: Option<&StoreConnectionValue>,
    account: Option<&StoreConnectionValue>,
    secret: Option<&StoreConnectionValue>,
) -> DoweResult<()> {
    if provider == DatabaseProvider::D1 && (host.is_some() || port.is_some()) {
        return Err(node_error(
            node,
            "D1 database handles use `account`, `secret`, and `name`; `host` and `port` are not supported",
        ));
    }
    let mut missing = Vec::new();
    if provider != DatabaseProvider::D1 && host.is_none() {
        missing.push("host");
    }
    if provider != DatabaseProvider::D1 && port.is_none() {
        missing.push("port");
    }
    if account.is_none() {
        missing.push("account");
    }
    if secret.is_none() {
        missing.push("secret");
    }
    if !missing.is_empty() {
        return Err(node_error(
            node,
            format!(
                "database provider requires {} for production",
                missing
                    .iter()
                    .map(|name| format!("`{name}`"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        ));
    }
    Ok(())
}

fn optional_connection_value_prop(
    node: &SourceNode,
    name: &str,
    environment: Option<&EnvironmentConfig>,
) -> DoweResult<Option<StoreConnectionValue>> {
    let Some(prop) = node.prop(name) else {
        return Ok(None);
    };
    match &prop.value {
        SourceValue::String(value) if !value.is_empty() => {
            if name == "account" {
                validate_database_name(node, value, "account")?;
            }
            Ok(Some(StoreConnectionValue::Static(value.clone())))
        }
        SourceValue::Bareword(value) => {
            let Some(env_name) = value.strip_prefix("env.") else {
                return Err(node_error(
                    node,
                    format!("`{name}` must be a quoted string or server env reference"),
                ));
            };
            if let Some(environment) = environment {
                let variable = environment.variable(env_name).ok_or_else(|| {
                    node_error(node, format!("unknown environment variable `{env_name}`"))
                })?;
                if variable.visibility != EnvironmentVisibility::Server {
                    return Err(node_error(
                        node,
                        format!("environment variable `{env_name}` must be server-only"),
                    ));
                }
            }
            Ok(Some(StoreConnectionValue::Environment(
                env_name.to_string(),
            )))
        }
        _ => Err(node_error(
            node,
            format!("`{name}` must be a quoted string or server env reference"),
        )),
    }
}

fn required_database_name_prop(
    node: &SourceNode,
    environment: Option<&EnvironmentConfig>,
) -> DoweResult<String> {
    let prop = node
        .prop("name")
        .ok_or_else(|| node_error(node, "database must declare `name`"))?;
    let value = match &prop.value {
        SourceValue::String(value) => value.clone(),
        SourceValue::Bareword(value) => {
            let env_name = value.strip_prefix("env.").ok_or_else(|| {
                node_error(
                    node,
                    "`name` must be a quoted string or server env reference",
                )
            })?;
            validate_server_environment(node, environment, env_name)?;
            environment
                .and_then(|environment| environment.variable(env_name))
                .and_then(|variable| variable.resolved_value.clone())
                .ok_or_else(|| {
                    node_error(
                        node,
                        format!(
                            "database name environment variable `{env_name}` must resolve during compilation"
                        ),
                    )
                })?
        }
        _ => {
            return Err(node_error(
                node,
                "`name` must be a quoted string or server env reference",
            ));
        }
    };
    if value.is_empty() {
        return Err(node_error(node, "database `name` must not be empty"));
    }
    Ok(value)
}

fn optional_port_value_prop(
    node: &SourceNode,
    environment: Option<&EnvironmentConfig>,
) -> DoweResult<Option<StoreConnectionValue>> {
    let Some(prop) = node.prop("port") else {
        return Ok(None);
    };
    match &prop.value {
        SourceValue::Number(value) if value.parse::<u16>().ok().is_some_and(|port| port > 0) => {
            Ok(Some(StoreConnectionValue::Static(value.clone())))
        }
        SourceValue::Bareword(value) => {
            let Some(env_name) = value.strip_prefix("env.") else {
                return Err(node_error(
                    node,
                    "`port` must be an integer or server env reference",
                ));
            };
            validate_server_environment(node, environment, env_name)?;
            Ok(Some(StoreConnectionValue::Environment(
                env_name.to_string(),
            )))
        }
        _ => Err(node_error(
            node,
            "`port` must be an integer or server env reference",
        )),
    }
}

fn validate_server_environment(
    node: &SourceNode,
    environment: Option<&EnvironmentConfig>,
    env_name: &str,
) -> DoweResult<()> {
    if let Some(environment) = environment {
        let variable = environment.variable(env_name).ok_or_else(|| {
            node_error(node, format!("unknown environment variable `{env_name}`"))
        })?;
        if variable.visibility != EnvironmentVisibility::Server {
            return Err(node_error(
                node,
                format!("environment variable `{env_name}` must be server-only"),
            ));
        }
    }
    Ok(())
}

pub(crate) fn binding_array_prop(node: &SourceNode, name: &str) -> DoweResult<Vec<String>> {
    let Some(prop) = node.prop(name) else {
        return Ok(Vec::new());
    };
    let SourceValue::Array(values) = &prop.value else {
        return Err(node_error(
            node,
            format!("`{name}` must be an array of bindings"),
        ));
    };
    let mut output = Vec::new();
    let mut seen = HashSet::new();
    for value in values {
        let SourceValue::Bareword(value) = value else {
            return Err(node_error(
                node,
                format!("`{name}` values must be binding names"),
            ));
        };
        validate_binding_identifier(node, value, name)?;
        if !seen.insert(value.clone()) {
            return Err(node_error(
                node,
                format!("duplicate `{name}` binding `{value}`"),
            ));
        }
        output.push(value.clone());
    }
    Ok(output)
}

fn validate_unique_entity_tables(node: &SourceNode, entities: &[DatabaseEntity]) -> DoweResult<()> {
    let mut tables = HashSet::new();
    for entity in entities {
        if !tables.insert(entity.table.clone()) {
            return Err(node_error(
                node,
                format!("duplicate entity table `{}`", entity.table),
            ));
        }
    }
    Ok(())
}

fn validate_seeder_entities(
    node: &SourceNode,
    entities: &[DatabaseEntity],
    seeders: &[DatabaseSeeder],
) -> DoweResult<()> {
    let included = entities
        .iter()
        .map(|entity| entity.binding.as_str())
        .collect::<HashSet<_>>();
    for seeder in seeders {
        for insert in &seeder.inserts {
            if !included.contains(insert.entity.as_str()) {
                return Err(node_error(
                    node,
                    format!(
                        "seeder `{}` uses entity `{}` that is not included in `entities`",
                        seeder.binding, insert.entity
                    ),
                ));
            }
        }
    }
    Ok(())
}

fn parse_database_field_type(node: &SourceNode, value: &str) -> DoweResult<DatabaseFieldType> {
    match value {
        "string" => Ok(DatabaseFieldType::String),
        "bool" => Ok(DatabaseFieldType::Bool),
        "int" => Ok(DatabaseFieldType::Int),
        "number" => Ok(DatabaseFieldType::Number),
        "decimal" => Ok(DatabaseFieldType::Decimal),
        "timestamp" => Ok(DatabaseFieldType::Timestamp),
        "json" => Ok(DatabaseFieldType::Json),
        _ => Err(node_error(
            node,
            format!("unknown database field type `{value}`"),
        )),
    }
}

fn validate_binding_identifier(node: &SourceNode, value: &str, label: &str) -> DoweResult<()> {
    if value.is_empty()
        || !value.chars().enumerate().all(|(index, character)| {
            character == '_'
                || character.is_ascii_alphanumeric() && (index > 0 || !character.is_ascii_digit())
        })
    {
        return Err(node_error(node, format!("invalid {label} name `{value}`")));
    }
    Ok(())
}

fn lower_snake_case(value: &str) -> String {
    let mut output = String::new();
    let mut previous_lowercase = false;
    for character in value.chars() {
        if character.is_ascii_uppercase() {
            if previous_lowercase {
                output.push('_');
            }
            output.push(character.to_ascii_lowercase());
            previous_lowercase = false;
        } else {
            output.push(character);
            previous_lowercase = character.is_ascii_lowercase() || character.is_ascii_digit();
        }
    }
    output
}

fn validate_static_seed_value(node: &SourceNode, value: &StoreLiteral) -> DoweResult<()> {
    match value {
        StoreLiteral::Reference(reference) => Err(node_error(
            node,
            format!("seeder values must be static; found `{reference}`"),
        )),
        StoreLiteral::Array(values) => {
            for value in values {
                validate_static_seed_value(node, value)?;
            }
            Ok(())
        }
        StoreLiteral::Object(entries) => {
            for (_, value) in entries {
                validate_static_seed_value(node, value)?;
            }
            Ok(())
        }
        StoreLiteral::Null
        | StoreLiteral::Bool(_)
        | StoreLiteral::Number(_)
        | StoreLiteral::String(_) => Ok(()),
    }
}

fn validate_seed_fields(
    node: &SourceNode,
    entity: &DatabaseEntity,
    value: &StoreLiteral,
) -> DoweResult<()> {
    let StoreLiteral::Object(entries) = value else {
        return Err(node_error(node, "seeder insert `value` must be an object"));
    };
    let fields = entity
        .fields
        .iter()
        .map(|field| field.name.as_str())
        .collect::<HashSet<_>>();
    for (name, _) in entries {
        if !fields.contains(name.as_str()) {
            return Err(node_error(
                node,
                format!("unknown field `{name}` for entity `{}`", entity.binding),
            ));
        }
    }
    for field in entity.fields.iter().filter(|field| field.required) {
        if !entries.iter().any(|(name, _)| name == &field.name) {
            return Err(node_error(
                node,
                format!(
                    "seeder insert for `{}` is missing required field `{}`",
                    entity.binding, field.name
                ),
            ));
        }
    }
    Ok(())
}

fn seeder_fingerprint(binding: &str, inserts: &[DatabaseSeedInsert]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(binding.as_bytes());
    for insert in inserts {
        hasher.update([0]);
        hasher.update(insert.entity.as_bytes());
        hasher.update([0]);
        hasher.update(insert.table.as_bytes());
        fingerprint_literal(&mut hasher, &insert.value);
    }
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn fingerprint_literal(hasher: &mut Sha256, value: &StoreLiteral) {
    match value {
        StoreLiteral::Null => hasher.update(b"null"),
        StoreLiteral::Bool(value) => {
            hasher.update(if *value { &b"true"[..] } else { &b"false"[..] })
        }
        StoreLiteral::Number(value) => {
            hasher.update(b"number:");
            hasher.update(value.as_bytes());
        }
        StoreLiteral::String(value) => {
            hasher.update(b"string:");
            hasher.update(value.as_bytes());
        }
        StoreLiteral::Reference(value) => {
            hasher.update(b"reference:");
            hasher.update(value.as_bytes());
        }
        StoreLiteral::Array(values) => {
            hasher.update(b"[");
            for value in values {
                fingerprint_literal(hasher, value);
                hasher.update([0]);
            }
            hasher.update(b"]");
        }
        StoreLiteral::Object(entries) => {
            hasher.update(b"{");
            for (name, value) in entries {
                hasher.update(name.as_bytes());
                hasher.update([0]);
                fingerprint_literal(hasher, value);
                hasher.update([0]);
            }
            hasher.update(b"}");
        }
    }
}

fn reject_unknown_props(node: &SourceNode, allowed: &[&str]) -> DoweResult<()> {
    for prop in &node.props {
        if !allowed.iter().any(|allowed| *allowed == prop.name) {
            return Err(node_error(
                node,
                format!("database declaration does not support `{}`", prop.name),
            ));
        }
    }
    Ok(())
}

fn reject_unknown_transaction_props(node: &SourceNode) -> DoweResult<()> {
    for prop in &node.props {
        if prop.name != "conn" {
            return Err(node_error(
                node,
                format!("store tx does not support `{}`", prop.name),
            ));
        }
    }
    Ok(())
}

fn reject_unknown_transaction_insert_props(node: &SourceNode) -> DoweResult<()> {
    for prop in &node.props {
        if !matches!(prop.name.as_str(), "conn" | "table" | "value") {
            return Err(node_error(
                node,
                format!("store tx insert does not support `{}`", prop.name),
            ));
        }
    }
    Ok(())
}

fn node_error(node: &SourceNode, message: impl AsRef<str>) -> DoweError {
    DoweError::at_path(
        &node.location.path,
        format!(
            "{}:{}: {}",
            node.location.line,
            node.location.column,
            message.as_ref()
        ),
    )
}

