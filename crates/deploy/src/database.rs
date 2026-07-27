use crate::files::write_file;
use crate::{DeployResult, DeploySurface};
use dowe_compiler::{CompiledProject, DatabaseProvider, database_migration_plan};
use serde_json::json;
use std::path::Path;

pub(crate) fn write_database_artifacts(
    project: &CompiledProject,
    output: &Path,
    surface: DeploySurface,
) -> DeployResult<()> {
    if surface != DeploySurface::Server || project.databases.is_empty() {
        return Ok(());
    }
    let mut entries = Vec::new();
    for binding in &project.databases {
        let plan = database_migration_plan(&binding.connection);
        if let Some(sql) = &plan.sql {
            write_file(
                &output
                    .join("database")
                    .join(&binding.binding)
                    .join("00001_schema.sql"),
                sql,
            )?;
        }
        entries.push(json!({
            "binding": binding.binding,
            "provider": provider_name(binding.connection.provider),
            "name": binding.connection.database,
            "schemaMode": plan.schema_mode,
            "fingerprint": plan.fingerprint,
            "entities": binding
                .connection
                .entities
                .iter()
                .map(|entity| entity.binding.as_str())
                .collect::<Vec<_>>(),
            "seeders": binding
                .connection
                .seeders
                .iter()
                .map(|seeder| json!({
                    "binding": seeder.binding,
                    "fingerprint": seeder.fingerprint,
                }))
                .collect::<Vec<_>>(),
        }));
    }
    let mut manifest = serde_json::to_string_pretty(&json!({
        "version": 1,
        "databases": entries,
    }))?;
    manifest.push('\n');
    write_file(&output.join("database/manifest.json"), manifest)
}

fn provider_name(provider: DatabaseProvider) -> &'static str {
    match provider {
        DatabaseProvider::Postgres => "postgres",
        DatabaseProvider::D1 => "d1",
        DatabaseProvider::Dowe => "dowe",
    }
}
