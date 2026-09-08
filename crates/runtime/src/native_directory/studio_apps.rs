use super::project_files::*;
use super::*;

pub(super) async fn list_studio_apps(root: &Path) -> RuntimeResult<Value> {
    let database = open_database(root, "dowe-studio-apps")
        .map_err(|error| RuntimeError::new(error.to_string()))?;
    let records = database
        .records("workspace_apps")
        .map_err(|error| RuntimeError::new(error.to_string()))?;
    Ok(Value::Array(
        records
            .into_iter()
            .map(|record| {
                let mut object = serde_json::Map::new();
                for (key, value) in record {
                    object.insert(key, value.to_json());
                }
                Value::Object(object)
            })
            .collect(),
    ))
}

pub(super) async fn save_studio_app(root: &Path, args: &Value) -> RuntimeResult<Value> {
    let path = required_string(args, "path")?;
    let name = required_string(args, "name")?;
    register_studio_app_record(root, path, name, "local").await
}

pub(super) async fn register_studio_app(root: &Path, args: &Value) -> RuntimeResult<Value> {
    let path = required_string(args, "path")?;
    let name = required_string(args, "name")?;
    let source = required_string(args, "source")?;
    register_studio_app_record(root, path, name, &source).await
}

pub(super) async fn register_studio_app_record(
    root: &Path,
    path: String,
    name: String,
    source: &str,
) -> RuntimeResult<Value> {
    if !matches!(source, "local" | "github" | "dowe") {
        return Err(RuntimeError::new(
            "application source must be local, github, or dowe",
        ));
    }
    let path = validated_dowe_project_root(&path).await?;
    let database = open_database(root, "dowe-studio-apps")
        .map_err(|error| RuntimeError::new(error.to_string()))?;
    let mut record = StoreRecord::new();
    record.insert("id".to_string(), StoreValue::String(generate_ulid()));
    record.insert(
        "path".to_string(),
        StoreValue::String(path.to_string_lossy().into_owned()),
    );
    record.insert("name".to_string(), StoreValue::String(name));
    record.insert("source".to_string(), StoreValue::String(source.to_string()));
    let icon_path = path.join("icons/web/favicon-32x32.png");
    let has_icon = tokio::fs::metadata(&icon_path)
        .await
        .map(|metadata| metadata.is_file())
        .unwrap_or(false);
    record.insert(
        "icon".to_string(),
        StoreValue::String(icon_path.to_string_lossy().into_owned()),
    );
    record.insert("hasIcon".to_string(), StoreValue::Bool(has_icon));
    let now = StoreValue::Timestamp(format!(
        "{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| RuntimeError::new("system clock is invalid"))?
            .as_secs()
    ));
    record.insert("createdAt".to_string(), now.clone());
    record.insert("updatedAt".to_string(), now);
    let inserted = database
        .insert("workspace_apps", record)
        .map_err(|error| RuntimeError::new(error.to_string()))?;
    let mut object = serde_json::Map::new();
    for (key, value) in inserted {
        object.insert(key, value.to_json());
    }
    Ok(Value::Object(object))
}

pub(super) async fn delete_studio_app(root: &Path, args: &Value) -> RuntimeResult<Value> {
    let id = required_string(args, "id")?;
    let database = open_database(root, "dowe-studio-apps")
        .map_err(|error| RuntimeError::new(error.to_string()))?;
    let changed = database
        .delete("workspace_apps", "id", &StoreValue::String(id))
        .map_err(|error| RuntimeError::new(error.to_string()))?;
    Ok(json!({ "changed": changed }))
}
